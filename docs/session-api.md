# PiSession API Design

## Overview

`PiSession` owns a `pi --mode rpc` child process and provides:

- **Command methods** — one async method per RPC command, implemented directly on `PiSession`, with typed responses
- **Event stream** — subscribe to agent events, extension UI requests, and session lifecycle/protocol events
- **Process lifecycle** — spawn, kill, drop

All RPC types (`RpcCommand`, `RpcResponse`, `AgentEvent`, etc.) are hand-written in `src/types/`. See `src/types/README.md` for how they map to pi's TypeScript sources.

## Lifecycle

```rust
// Spawn
let session = PiSession::spawn(PiSessionConfig {
    provider: Some("anthropic".into()),
    model: Some("claude-sonnet-4-20250514".into()),
    session_persistence: SessionPersistence::Enabled, // or Disabled or CustomDir(path)
    working_dir: Some("/path/to/project".into()),
    extra_args: vec![],
    ..Default::default()
}).await?;

// Use
session.prompt("Hello", None, None).await?;

// Subscribe to events (returns an mpsc::UnboundedReceiver)
let mut events = session.subscribe().await;
tokio::spawn(async move {
    while let Some(event) = events.recv().await {
        // handle event
    }
});

// Cleanup
session.kill().await?;
// or just drop — Drop impl kills the child process
```

## Spawning options

```rust
pub struct PiSessionConfig {
    /// Path to `pi` binary. Default: "pi" (found via PATH).
    pub pi_binary: String,
    /// LLM provider (--provider flag).
    pub provider: Option<String>,
    /// Model pattern or ID (--model flag).
    pub model: Option<String>,
    /// Session persistence mode.
    pub session_persistence: SessionPersistence,
    /// Working directory for the pi process.
    pub working_dir: Option<PathBuf>,
    /// Custom session directory (--session-dir flag).
    pub session_dir: Option<PathBuf>,
    /// Additional CLI arguments passed to pi.
    pub extra_args: Vec<String>,
}

pub enum SessionPersistence {
    /// Default pi behavior (sessions saved to ~/.pi/agent/sessions/).
    Enabled,
    /// --no-session flag.
    Disabled,
    /// --session-dir flag.
    CustomDir(PathBuf),
}
```

## Command methods

Methods are implemented directly on `PiSession` (no trait). Each sends a JSON `RpcCommand` to stdin and awaits the correlated `RpcResponse`. Returns `Result<T, PiError>` where `T` is the typed response data.

The `RpcCommand` / `RpcResponse` types in `src/types/rpc_types.rs` define the wire format. Each command method constructs the appropriate `RpcCommandKind` variant, calls the internal `send_command` helper, and unpacks the `RpcResponseKind` variant.

### Prompting

```rust
session.prompt("message", None, None).await?;               // -> ()
session.prompt("msg", Some(images), None).await?;           // with images
session.prompt("msg", None, Some(StreamingBehavior::Steer)).await?;
session.steer("interrupt message", None).await?;            // -> ()
session.follow_up("after you're done", None).await?;        // -> ()
session.abort().await?;                                     // -> ()
```

### Session management

```rust
session.new_session(None).await?;                            // -> NewSessionData
session.new_session(Some("/path".into())).await?;            // with parent
session.switch_session("/path/to/session.jsonl").await?;     // -> SwitchSessionData
session.fork("entry_id").await?;                             // -> ForkData
session.clone_session().await?;                              // -> CloneData (RPC command: "clone")
session.get_fork_messages().await?;                          // -> GetForkMessagesData
session.set_session_name("my-feature").await?;               // -> ()
```

### State

```rust
session.get_state().await?;                // -> RpcSessionState
session.get_messages().await?;             // -> GetMessagesData
session.get_session_stats().await?;        // -> SessionStats
session.get_last_assistant_text().await?;  // -> GetLastAssistantTextData
session.export_html(Some("/tmp/out.html".into())).await?; // -> ExportHtmlData
```

### Model

```rust
session.set_model("anthropic", "claude-sonnet-4-20250514").await?; // -> Model
session.cycle_model().await?;              // -> Option<CycleModelData>
session.get_available_models().await?;     // -> GetAvailableModelsData
```

### Thinking

```rust
session.set_thinking_level(ThinkingLevel::High).await?; // -> ()
session.cycle_thinking_level().await?;     // -> Option<CycleThinkingLevelData>
```

### Queue modes

```rust
session.set_steering_mode(QueueMode::OneAtATime).await?; // -> ()
session.set_follow_up_mode(QueueMode::All).await?;       // -> ()
```

### Compaction

```rust
session.compact(None).await?;                           // -> CompactionResult
session.compact(Some("focus on code changes".into())).await?;
session.set_auto_compaction(true).await?;               // -> ()
```

### Retry

```rust
session.set_auto_retry(true).await?;  // -> ()
session.abort_retry().await?;         // -> ()
```

### Bash

```rust
session.bash("ls -la", false).await?;  // -> BashResult
session.bash("secret command", true).await?;  // exclude output from LLM context
session.abort_bash().await?;          // -> ()
```

### Commands

```rust
session.get_commands().await?;  // -> GetCommandsData
```

### Extension UI

```rust
session.respond_extension_ui(response).await?; // -> ()
```

Where `response` is an `RpcExtensionUIResponse` (Value, Confirmed, or Cancelled variant).

Note: `clone_session()` corresponds to pi's RPC command named `clone`; the Rust method uses a different name to avoid confusion with `Clone::clone`.

## Event stream

```rust
pub async fn subscribe(&self) -> mpsc::UnboundedReceiver<RpcEvent>;
```

`subscribe()` is async to avoid a race condition — it acquires the subscriber mutex directly rather than spawning a task, guaranteeing the subscriber is registered before returning.

`RpcEvent` is defined in `src/types/rpc_types.rs`:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum RpcEvent {
    Agent(AgentEvent),
    ExtensionUI(RpcExtensionUIRequest),
    Session(SessionEvent),
}
```

It has a custom `Deserialize` impl that checks the `type` field: `"extension_ui_request"` → `ExtensionUI`, `"session_*"` → `Session`, anything else → `Agent`.

Every valid JSON line from pi's stdout that is not a response (i.e., `type` != `"response"`) becomes an `RpcEvent`. Lines with a recognized `AgentEvent` type become `RpcEvent::Agent(...)`, lines with `type == "extension_ui_request"` become `RpcEvent::ExtensionUI(...)`.

`RpcEvent::Session(...)` is emitted by this Rust wrapper for local session lifecycle/protocol conditions, currently:

- `SessionEvent::ProcessExited { code, stderr }` when the pi process exits. `code` is populated from the child exit status when available; `stderr` contains stderr collected up to exit.
- `SessionEvent::DeserializationError { context, error, line }` when a stdout line cannot be parsed/deserialized as the expected RPC message

These session events are serializable/deserializable like the pi-originated events.

Internally, a background reader task fans out events to all subscribers. Each call to `subscribe()` creates a new `mpsc::unbounded_channel`, registers the sender, and returns the receiver. Dead subscribers (dropped receivers) are automatically cleaned up when a send fails.

- Multiple subscribers are supported (attach/detach pattern)
- Subscribers see only events from the point of subscription onward
- Events must be `Clone`

## Error handling

```rust
pub enum PiError {
    /// Pi process exited unexpectedly
    ProcessExited { code: Option<i32>, stderr: String },
    /// Command failed (success: false in response)
    CommandFailed { command: String, error: String },
    /// JSON serialization/deserialization error
    Json(serde_json::Error),
    /// IO error (pipe broken, etc.)
    Io(std::io::Error),
    /// Response timeout
    Timeout,
    /// Pi process not running
    NotRunning,
}
```

`PiError::ProcessExited` is returned to commands that were awaiting a response when the pi process exits. Subscribers also receive `RpcEvent::Session(SessionEvent::ProcessExited { ... })`. Intentional `kill()` also emits `ProcessExited`.

## File layout

```
src/session/
  mod.rs                  — routing only: mod declarations + pub use re-exports
  session.rs              — PiSession struct, PiSessionConfig, spawn, reader task, send_command, lifecycle
  error.rs                — PiError enum
  impl_rpc_methods.rs     — public command methods (separate impl PiSession block)
```

## Internal architecture

```
PiSession
├── writer: Arc<Mutex<BufWriter<ChildStdin>>>   // serializes commands to stdin
├── subscribers: Arc<Mutex<Vec<mpsc::UnboundedSender<RpcEvent>>>>
├── pending: Arc<Mutex<HashMap<String, oneshot::Sender<Result<RpcResponse, PiError>>>>>
├── next_id: AtomicU64
├── running: Arc<AtomicBool>
├── process_control_tx: mpsc::UnboundedSender<ProcessControl>
├── reader_cancel: CancellationToken
├── supervisor_cancel: CancellationToken
├── reader_task: JoinHandle<()>     // owns stdout; parses protocol messages
└── supervisor_task: JoinHandle<()> // owns Child + stderr; manages process lifecycle

reader_task loop:
  1. Read line from stdout, unless cancelled
  2. Parse as serde_json::Value; on failure, emit RpcEvent::Session(SessionEvent::DeserializationError)
  3. Check value["type"]:
     - "response" → deserialize as RpcResponse, look up pending[id], send via oneshot
     - "extension_ui_request" → deserialize as RpcExtensionUIRequest, wrap in RpcEvent::ExtensionUI, fan out
     - anything else → deserialize as AgentEvent, wrap in RpcEvent::Agent, fan out
  4. On typed deserialization failure, emit RpcEvent::Session(SessionEvent::DeserializationError)
  5. On stdout EOF, exit; process-exit handling belongs to supervisor_task
  6. Fan out: clone and send to all subscribers, removing any whose send fails

supervisor_task loop:
  1. Own the tokio Child and stderr pipe
  2. Collect stderr while concurrently waiting for child exit, kill requests, or cancellation
  3. On child exit, drain remaining stderr
  4. Mark the session not running
  5. Fail pending commands with PiError::ProcessExited { code, stderr }
  6. Emit RpcEvent::Session(SessionEvent::ProcessExited { code, stderr })
  7. Close subscribers
```

### send_command (internal helper)

```
pub(crate) async fn send_command(&self, command: RpcCommandKind) -> Result<RpcResponse, PiError>
```

1. Generate unique `id` from `next_id` (AtomicU64)
2. Build `RpcCommand { id: Some(id), kind: command }`
3. Create oneshot channel, insert sender into `pending` keyed by `id`
4. Serialize command as JSON + newline, write to stdin via `writer`
5. Await oneshot receiver (with timeout)
6. Return the `RpcResponse`

The public command methods then match on `response.kind` to extract the typed data or convert `RpcResponseKind::Error` into `PiError::CommandFailed`.
