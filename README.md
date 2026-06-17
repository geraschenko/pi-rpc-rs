# pi-rpc-rs

Typed Rust bindings for [pi](https://github.com/earendil-works/pi)'s RPC mode.

`PiSession` runs `pi --mode rpc` as a subprocess, rebroadcasts pi's event stream
to subscribers, and correlates RPC responses to commands. Every pi RPC command
is faithfully exposed as a session method. For example, `PiSession::fork` wraps
pi's [`fork` command](https://github.com/earendil-works/pi/blob/6d5ede31c8b8584b422bd0fa2ce10a39b2a0cdce/packages/coding-agent/src/modes/rpc/rpc-types.ts#L59)
and its return type mirrors pi's [`fork` response type](https://github.com/earendil-works/pi/blob/6d5ede31c8b8584b422bd0fa2ce10a39b2a0cdce/packages/coding-agent/src/modes/rpc/rpc-types.ts#L175).

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

## Compatibility

**Compatible with pi 0.79.0.** This version is tracked in
`src/types/upstream.toml`.

| `pi-rpc-rs` version | Compatible pi version |
| ------------------- | --------------------- |
| `0.1.2`             | `0.79.0`              |
| `0.1.1`             | `0.78.0`              |
| `0.1.0`             | `0.75.3`              |

The target pi version is exposed in code as `pi_rpc_rs::COMPATIBLE_PI_VERSION`.
By default, `PiSession::spawn` runs `pi --version` first and logs a warning if
it does not match. Configure this with `PiSessionConfig::version_check` and
`PiVersionCheck::{Disabled, Warn, Error}`.
