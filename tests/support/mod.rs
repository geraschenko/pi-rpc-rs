use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::ops::{Deref, DerefMut};
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use pi_rpc_rs::session::{PiSession, PiSessionConfig, PiVersionCheck, SessionPersistence};
use serde_json::{Value, json};
use tempfile::TempDir;

pub struct TestSession {
  session: PiSession,
  environment: TestEnvironment,
}

impl TestSession {
  pub async fn spawn() -> Self {
    let source_dir = env::var_os("PI_CODING_AGENT_DIR")
      .map(PathBuf::from)
      .unwrap_or_else(|| {
        PathBuf::from(env::var_os("HOME").expect("HOME is required")).join(".pi/agent")
      });
    let auth = fs::read(source_dir.join("auth.json")).expect("Cannot read pi auth.json");
    let now = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap()
      .as_secs_f64();
    let snapshot =
      codex_auth_snapshot(&auth, now).expect("Cannot prepare isolated test credentials");
    let environment = TestEnvironment::new(snapshot, &find_pi());
    let session = PiSession::spawn(PiSessionConfig {
      pi_binary: environment
        .launcher
        .to_str()
        .expect("launcher path must be UTF-8")
        .into(),
      working_dir: Some(environment.working_dir.clone()),
      session_persistence: SessionPersistence::Disabled,
      version_check: PiVersionCheck::Error,
      provider: Some("openai-codex".into()),
      model: Some("gpt-5.5".into()),
      ..Default::default()
    })
    .await
    .expect("Failed to spawn isolated pi session");
    Self {
      session,
      environment,
    }
  }

  /// Reap pi before auditing credentials and removing its private directories.
  pub async fn finish(mut self) {
    self
      .session
      .kill()
      .await
      .expect("Failed to stop isolated pi");
    tokio::time::timeout(Duration::from_secs(5), self.session.wait_closed())
      .await
      .expect("Isolated pi did not exit");
    // Do not use assert_eq!: its diagnostics would print credentials.
    assert!(
      fs::read(&self.environment.auth_path).unwrap() == self.environment.auth_snapshot,
      "Test modified its auth copy; refresh-token isolation may have been violated"
    );
  }
}

impl Deref for TestSession {
  type Target = PiSession;
  fn deref(&self) -> &Self::Target {
    &self.session
  }
}

impl DerefMut for TestSession {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.session
  }
}

struct TestEnvironment {
  _root: TempDir,
  launcher: PathBuf,
  working_dir: PathBuf,
  auth_path: PathBuf,
  auth_snapshot: Vec<u8>,
}

impl TestEnvironment {
  fn new(auth_snapshot: Vec<u8>, pi_binary: &Path) -> Self {
    let root = tempfile::Builder::new()
      .prefix("pi-rpc-test-")
      .permissions(fs::Permissions::from_mode(0o700))
      .tempdir()
      .unwrap();
    let working_dir = root.path().join("work");
    fs::create_dir(&working_dir).unwrap();
    let mut variables = Vec::new();
    for key in ["PATH", "LANG", "LC_ALL", "SSL_CERT_FILE", "SSL_CERT_DIR"] {
      if let Ok(value) = env::var(key) {
        variables.push((key, value));
      }
    }
    for (key, directory) in [
      ("HOME", "home"),
      ("TMPDIR", "tmp"),
      ("XDG_CONFIG_HOME", "config"),
      ("XDG_CACHE_HOME", "cache"),
      ("XDG_DATA_HOME", "data"),
      ("XDG_STATE_HOME", "state"),
      ("PI_CODING_AGENT_DIR", "agent"),
    ] {
      let path = root.path().join(directory);
      fs::create_dir(&path).unwrap();
      variables.push((key, path.to_str().unwrap().into()));
    }
    variables.extend([
      ("PI_SKIP_VERSION_CHECK", "1".into()),
      ("PI_TELEMETRY", "0".into()),
    ]);
    let auth_path = root.path().join("agent/auth.json");
    write_private_file(&auth_path, &auth_snapshot, 0o600);
    let mut script = String::from("#!/bin/sh\nexec /usr/bin/env -i");
    for (key, value) in variables {
      script.push(' ');
      script.push_str(&shell_quote(&format!("{key}={value}")));
    }
    script.push(' ');
    script.push_str(&shell_quote(
      pi_binary.to_str().expect("pi path must be UTF-8"),
    ));
    script.push_str(" \"$@\"\n");
    let launcher = root.path().join("pi");
    write_private_file(&launcher, script.as_bytes(), 0o700);
    Self {
      _root: root,
      launcher,
      working_dir,
      auth_path,
      auth_snapshot,
    }
  }
}

fn write_private_file(path: &Path, content: &[u8], mode: u32) {
  OpenOptions::new()
    .write(true)
    .create_new(true)
    .mode(mode)
    .open(path)
    .unwrap()
    .write_all(content)
    .unwrap();
}

fn shell_quote(value: &str) -> String {
  format!("'{}'", value.replace('\'', "'\\''"))
}

fn find_pi() -> PathBuf {
  env::split_paths(&env::var_os("PATH").expect("PATH is required"))
    .map(|directory| directory.join("pi"))
    .find(|path| {
      fs::metadata(path)
        .is_ok_and(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
    })
    .map(|path| fs::canonicalize(path).unwrap())
    .expect("pi must be installed on PATH")
}

fn codex_auth_snapshot(auth: &[u8], now_seconds: f64) -> Result<Vec<u8>, &'static str> {
  let auth: Value = serde_json::from_slice(auth).map_err(|_| "Invalid auth.json")?;
  let credential = auth
    .get("openai-codex")
    .ok_or("Log in to openai-codex before running integration tests")?;
  if credential.get("type").and_then(Value::as_str) != Some("oauth") {
    return Err("Integration tests require an openai-codex OAuth credential");
  }
  let expires = credential
    .get("expires")
    .and_then(Value::as_f64)
    .ok_or("Codex credential has no numeric expiry")?;
  if expires / 1000.0 < now_seconds + 1800.0 {
    return Err(
      "Refresh your Codex login before testing: at least 30 minutes of token validity is required",
    );
  }
  serde_json::to_vec(&json!({"openai-codex": credential}))
    .map_err(|_| "Cannot serialize test credential")
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn auth_snapshot_excludes_other_providers_and_rejects_expiring_tokens() {
    let auth = json!({"openai-codex": {"type": "oauth", "access": "fake", "expires": 3_000_000},
      "other": {"type": "oauth", "access": "unrelated", "expires": 0}});
    let encoded = serde_json::to_vec(&auth).unwrap();
    let snapshot: Value =
      serde_json::from_slice(&codex_auth_snapshot(&encoded, 100.0).unwrap()).unwrap();
    assert_eq!(snapshot.as_object().unwrap().len(), 1);
    assert_eq!(snapshot["openai-codex"], auth["openai-codex"]);
    assert!(codex_auth_snapshot(&encoded, 2000.0).is_err());
    assert!(codex_auth_snapshot(b"{}", 0.0).is_err());
    assert!(codex_auth_snapshot(br#"{"openai-codex":{"type":"oauth"}}"#, 0.0).is_err());
  }

  #[test]
  fn launcher_isolates_directories_and_preserves_arguments() {
    let fake_binary_dir = tempfile::tempdir().unwrap();
    let fake_pi = fake_binary_dir.path().join("pi with ' quotes");
    write_private_file(&fake_pi, b"#!/bin/sh\nprintf '%s\\n' \"$HOME\" \"$PI_CODING_AGENT_DIR\" \"$TMPDIR\" \"$XDG_CONFIG_HOME\" \"$XDG_CACHE_HOME\" \"$XDG_DATA_HOME\" \"$XDG_STATE_HOME\" \"$PWD\" \"$1\"\n", 0o700);
    let first = TestEnvironment::new(b"{}".to_vec(), &fake_pi);
    let second = TestEnvironment::new(b"{}".to_vec(), &fake_pi);
    assert_ne!(first.working_dir, second.working_dir);
    let output = std::process::Command::new(&first.launcher)
      .current_dir(&first.working_dir)
      .arg("argument with ' quotes")
      .env("HOME", "/must-not-inherit")
      .env("PI_CODING_AGENT_DIR", "/must-not-inherit")
      .output()
      .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<_> = stdout.lines().collect();
    assert_eq!(lines.len(), 9);
    for path in &lines[..8] {
      assert!(Path::new(path).starts_with(first._root.path()));
    }
    assert_eq!(lines[8], "argument with ' quotes");
    assert_eq!(
      fs::metadata(&first.auth_path).unwrap().permissions().mode() & 0o777,
      0o600
    );
    assert_eq!(
      fs::metadata(first._root.path())
        .unwrap()
        .permissions()
        .mode()
        & 0o777,
      0o700
    );
    let root_path = first._root.path().to_owned();
    drop(first);
    assert!(!root_path.exists());
  }
}
