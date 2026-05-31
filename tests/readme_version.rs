#[test]
fn top_level_readme_mentions_upstream_pi_version() {
  let root = env!("CARGO_MANIFEST_DIR");
  let upstream = read_toml(&format!("{root}/src/types/upstream.toml"));
  let cargo = read_toml(&format!("{root}/Cargo.toml"));
  let readme = std::fs::read_to_string(format!("{root}/README.md")).expect("read README.md");

  let upstream_version = upstream
    .get("version")
    .and_then(toml::Value::as_str)
    .expect("src/types/upstream.toml has a string version entry");
  let crate_version = cargo
    .get("package")
    .and_then(|package| package.get("version"))
    .and_then(toml::Value::as_str)
    .expect("Cargo.toml has a package.version string entry");

  assert!(
    readme.contains(&format!("Compatible with pi {upstream_version}")),
    "README.md should mention the pi compatibility version from src/types/upstream.toml ({upstream_version}) near the top"
  );

  assert!(
    readme.contains(&format!("| `{crate_version}`"))
      && readme.contains(&format!("| `{upstream_version}`")),
    "README.md compatibility table should include crate version {crate_version} and pi version {upstream_version}"
  );
}

fn read_toml(path: &str) -> toml::Value {
  let contents = std::fs::read_to_string(path).unwrap_or_else(|err| panic!("read {path}: {err}"));
  toml::from_str(&contents).unwrap_or_else(|err| panic!("parse {path}: {err}"))
}
