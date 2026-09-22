# pi-rpc-rs

[![GitHub](https://img.shields.io/badge/github-geraschenko%2Fpi--rpc--rs-blue?logo=github)](https://github.com/geraschenko/pi-rpc-rs)
[![Crates.io](https://img.shields.io/crates/v/pi-rpc-rs.svg)](https://crates.io/crates/pi-rpc-rs)
[![Docs.rs](https://docs.rs/pi-rpc-rs/badge.svg)](https://docs.rs/pi-rpc-rs)

Typed Rust bindings for [pi](https://github.com/earendil-works/pi)'s RPC mode.

`PiSession` runs `pi --mode rpc` as a subprocess, rebroadcasts pi's event stream
to subscribers, and correlates RPC responses to commands. Every pi RPC command
is faithfully exposed as a session method. For example, `PiSession::fork` wraps
pi's [`fork` command](https://github.com/earendil-works/pi/blob/v0.84.3/packages/coding-agent/src/modes/rpc/rpc-types.ts#L61)
and its return type mirrors pi's [`fork` response type](https://github.com/earendil-works/pi/blob/v0.84.3/packages/coding-agent/src/modes/rpc/rpc-types.ts#L186).

## Quick start

```rust
use pi_rpc_rs::session::{PiSession, PiSessionConfig};
use pi_rpc_rs::types::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  // Spawn a pi session
  let config = PiSessionConfig {
    provider: Some("openai-codex".to_string()),
    model: Some("gpt-5.1".to_string()),
    ..Default::default()
  };
  let session = PiSession::spawn(config).await?;

  // Subscribe to events before sending a prompt
  let mut rx = session.subscribe().await;

  // Send a prompt
  session.prompt("What is 2 + 2?", None, None).await?;

  // Stream events until the agent finishes
  while let Some(event) = rx.recv().await {
    match event {
      RpcEvent::Agent(AgentEvent::MessageUpdate {
        assistant_message_event: AssistantMessageEvent::TextDelta { delta, .. },
        ..
      }) => print!("{delta}"),
      RpcEvent::Agent(AgentEvent::AgentEnd { .. }) => break,
      _ => {}
    }
  }
  println!();

  Ok(())
}
```

## Requirements

Install [pi](https://github.com/earendil-works/pi) and make sure it is on
`PATH`, or set `pi_binary` in the `PiSessionConfig` used to spawn the session.
You will also need whatever API keys or subscriptions your chosen provider/model
requires.

## Integration tests

On Unix, with `pi` on `PATH` and an `openai-codex` login:

```bash
cargo nextest run --run-ignored all --no-fail-fast
```

Each integration test uses fresh working, home, agent, cache, and temporary
directories. It copies only the Codex OAuth credential from
`$PI_CODING_AGENT_DIR/auth.json` (default: `$HOME/.pi/agent/auth.json`), with
private file permissions. User settings, extensions, other provider credentials,
and ambient provider environment variables are not inherited. Tests use `gpt-5.5`
and make real LLM calls; normal parallel execution is supported.

The credential must have at least 30 minutes remaining before expiry. If the
harness rejects it, refresh your Codex login before rerunning. This avoids tests
refreshing independent copies of the same account's refresh token. Successful
tests stop and await pi, verify that their auth copy is unchanged, and remove
the temporary directories. Assertion failures also drop the session and temporary
directory guards, using the session's normal drop cleanup.

## Compatibility

**Compatible with pi 0.87.1.** This version is tracked in
`src/types/upstream.toml`.

| `pi-rpc-rs` version | Compatible pi version |
| ------------------- | --------------------- |
| `0.1.7`             | `0.87.1`              |
| `0.1.6`             | `0.84.3`              |
| `0.1.5`             | `0.83.0`              |
| `0.1.4`             | `0.80.6`              |
| `0.1.3`             | `0.80.2`              |
| `0.1.2`             | `0.79.0`              |
| `0.1.1`             | `0.78.0`              |
| `0.1.0`             | `0.75.3`              |

The target pi version is exposed in code as `pi_rpc_rs::COMPATIBLE_PI_VERSION`.
By default, `PiSession::spawn` runs `pi --version` first and logs a warning if
it does not match. Configure this with `PiSessionConfig::version_check` and
`PiVersionCheck::{Disabled, Warn, Error}`.
