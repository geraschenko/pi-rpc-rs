# pi RPC Type Definitions

Hand-written Rust types mirroring the TypeScript definitions from
[pi](https://github.com/badlogic/pi-mono) **v0.75.3**.

These files are **not auto-generated** — they were written by hand to closely
match the TypeScript sources. Each file has a doc comment at the top naming the
TypeScript source it corresponds to.

## File mapping

| Rust file          | TypeScript source                                                                                                       |
| ------------------ | ----------------------------------------------------------------------------------------------------------------------- |
| `ai.rs`            | `packages/ai/src/types.ts` — content blocks, messages, usage, models, streaming events                                  |
| `agent.rs`         | `packages/agent/src/types.ts` — `AgentMessage`, `AgentEvent`, `ThinkingLevel`                                           |
| `agent_session.rs` | `packages/coding-agent/src/core/agent-session.ts` — `SessionStats`                                                      |
| `bash_executor.rs` | `packages/coding-agent/src/core/bash-executor.ts` — `BashResult`                                                        |
| `compaction.rs`    | `packages/coding-agent/src/core/compaction/compaction.ts` — `CompactionResult`                                          |
| `rpc_types.rs`     | `packages/coding-agent/src/modes/rpc/rpc-types.ts` — `RpcCommand`, `RpcResponse`, `RpcSessionState`, extension UI types |
| `source_info.rs`   | `packages/coding-agent/src/core/source-info.ts` — source metadata for slash commands                                    |

## Notes on TypeScript → Rust mapping

- **Declaration merging**: TypeScript's `messages.ts` extends `AgentMessage` via
  declaration merging. In Rust, the merged variants are included directly in the
  `AgentMessage` enum in `agent.rs`, with comments marking their origin.

- **`AgentSessionEvent`**: TypeScript defines this as
  `AgentEvent | { type: "queue_update"; ... } | { type: "compaction_start"; ... } | ...`.
  In Rust, the additional variants are included directly in `AgentEvent` in
  `agent.rs`, with comments marking their origin. `agent_session.rs` contains
  `SessionStats` and related session-stat data.

## Updating for a new pi version

1. Diff the TypeScript source files listed above against the new version.
2. Update the Rust types to match.
3. Run `cargo check` to verify.
4. Update the version number in this README.
