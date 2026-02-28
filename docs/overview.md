# pi-rpc-rs: Overview

## Goal

A Rust crate providing a typed, ergonomic interface to [pi](https://github.com/badlogic/pi-mono)'s RPC mode (`pi --mode rpc`). This is a faithful Rust analog of pi's `AgentSession` — exposing the full RPC protocol with type safety.

## Non-goals

- Orchestration logic (multi-agent coordination, task scheduling)
- Custom UI rendering
- Re-implementing pi's agent logic in Rust

## Architecture

```
┌──────────────────────────────────┐
│          User's Rust code        │
│                                  │
│   session.prompt("...").await    │
│   session.subscribe(|event| ...) │
└──────────┬───────────────────────┘
           │
┌──────────▼───────────────────────┐
│         pi-rpc-rs crate          │
│                                  │
│  Rust types (hand-written)       │
│  PiSession (impl, owns process)  │
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

## Crate structure

```
pi-rpc-rs/
├── docs/                           # Design docs (this directory)
├── src/
│   ├── lib.rs                      # Crate root (mod declarations only)
│   ├── session/                    # PiSession — process management + RPC
│   │   ├── mod.rs                  # Routing only: mod declarations + pub use re-exports
│   │   ├── session.rs              # PiSession struct, spawn, reader task, send_command, lifecycle
│   │   ├── error.rs                # PiError enum
│   │   └── impl_rpc_methods.rs     # Public command methods (prompt, get_state, etc.)
│   └── types/                      # Rust types mirroring pi's TypeScript
│       ├── README.md               # File mapping, version, update instructions
│       ├── ai.rs                   # packages/ai/src/types.ts
│       ├── agent.rs                # packages/agent/src/types.ts
│       ├── agent_session.rs        # packages/coding-agent/src/core/agent-session.ts
│       ├── bash_executor.rs        # packages/coding-agent/src/core/bash-executor.ts
│       ├── compaction.rs           # packages/coding-agent/src/core/compaction/compaction.ts
│       └── rpc_types.rs            # packages/coding-agent/src/modes/rpc/rpc-types.ts
├── Cargo.toml
└── README.md
```

**Convention:** `mod.rs` files are routing only (module declarations and `pub use` re-exports). All implementation code goes in named files.

## Key design decisions

1. **Hand-written Rust types** closely mirroring pi's TypeScript sources, organized file-by-file to match. See [src/types/README.md](../src/types/README.md) for the mapping and update instructions.
2. **Async-first** using tokio. Event streaming via `mpsc::UnboundedSender` fan-out.
3. **Faithful to pi's RPC protocol** — every command and event type is represented.
4. **Process lifecycle management** — spawn, kill, and detect unexpected exits.

## Source of truth

The RPC protocol is defined in [pi-mono](https://github.com/badlogic/pi-mono). The specific TypeScript files each Rust module mirrors are documented in [src/types/README.md](../src/types/README.md).

The installed pi package lives at:
```
~/.nvm/versions/node/v23.11.1/lib/node_modules/@mariozechner/pi-coding-agent/
```

## Related docs

- [codegen.md](codegen.md) — Why types are hand-written (not auto-generated)
- [types.md](types.md) — RPC protocol type inventory
- [session-api.md](session-api.md) — PiSession API design
