# pi-rpc-rs

A Rust crate providing a typed, ergonomic interface to
[pi](https://github.com/earendil-works/pi)'s RPC mode (`pi --mode rpc`).
This is a faithful Rust analog of pi's `AgentSession` — exposing the full
RPC protocol with type safety.

**Compatible with pi 0.79.0.** This version is tracked in
`src/types/upstream.toml`.

## Compatibility

This crate tracks pi's RPC wire protocol for a specific upstream pi version.
Older pi versions are not supported by newer crate releases; if you need an old
pi version, use the corresponding old `pi-rpc-rs` release.

| `pi-rpc-rs` version | Compatible pi version |
| ------------------- | --------------------- |
| `0.1.2`             | `0.79.0`              |
| `0.1.1`             | `0.78.0`              |
| `0.1.0`             | `0.75.3`              |

## Versioning

Each `pi-rpc-rs` release targets exactly one upstream pi version. When upstream
pi compatibility changes, bump the patch version of this crate and add a new row
to the compatibility table. Do not overwrite previous compatibility rows.

The target pi version is exposed in code as `pi_rpc_rs::COMPATIBLE_PI_VERSION`.
By default, `PiSession::spawn` runs `pi --version` first and logs a warning if
it does not match. Configure this with `PiSessionConfig::version_check` and
`PiVersionCheck::{Disabled, Warn, Error}`.

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

## Architecture

```
┌──────────────────────────────────┐
│          Your Rust code          │
│                                  │
│   session.prompt("...").await    │
│   session.subscribe().await      │
└──────────┬───────────────────────┘
           │
┌──────────▼───────────────────────┐
│         pi-rpc-rs crate          │
│                                  │
│  Rust types                      │
│  PiSession (owns child process)  │
│  ├─ stdin writer (commands)      │
│  ├─ stdout reader (events)       │
│  ├─ command/response correlation │
│  └─ event fan-out                │
└──────────┬───────────────────────┘
           │ stdin/stdout JSON lines
┌──────────▼───────────────────────┐
│     pi --mode rpc (child proc)   │
└──────────────────────────────────┘
```

## Requirements

[pi](https://github.com/earendil-works/pi) must be installed and in `PATH`, and
you must have API keys or subscriptions set up so that you can run `pi` with the
given provider/model.

## Running tests

```bash
# Unit tests (no pi required)
cargo nextest run

# Integration tests (requires pi + API credentials)
cargo nextest run --run-ignored all
```

## Debug tool

A built-in debug binary prints raw RPC traffic, useful for understanding
pi's protocol behavior:

```bash
cargo run --bin pi-rpc-debug -- --prompt "say hello" --raw-json
```
