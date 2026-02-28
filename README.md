# pi-rpc-rs

A Rust crate providing a typed, ergonomic interface to
[pi](https://github.com/badlogic/pi-mono)'s RPC mode (`pi --mode rpc`).
This is a faithful Rust analog of pi's `AgentSession` — exposing the full
RPC protocol with type safety.

## Quick start

```rust
use pi_rpc_rs::session::{PiSession, PiSessionConfig};
use pi_rpc_rs::types::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  // Spawn a pi session using Sonnet
  let config = PiSessionConfig {
    provider: Some("anthropic".to_string()),
    model: Some("claude-sonnet-4-6".into()),
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
│  Rust types (hand-written)       │
│  PiSession (owns child process)  │
│  ├─ stdin writer (commands)      │
│  ├─ stdout reader (events)      │
│  ├─ command/response correlation │
│  └─ event fan-out                │
└──────────┬───────────────────────┘
           │ stdin/stdout JSON lines
┌──────────▼───────────────────────┐
│     pi --mode rpc (child proc)   │
└──────────────────────────────────┘
```

## Requirements

[pi](https://github.com/badlogic/pi-mono) must be installed and in `PATH`, and
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
