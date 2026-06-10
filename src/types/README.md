# pi RPC Type Definitions

Hand-written Rust types mirroring the TypeScript definitions from
[pi](https://github.com/earendil-works/pi) **0.79.0**.

These files are **not auto-generated** — they were written by hand to closely
match the TypeScript sources. Each file has a doc comment at the top naming the
TypeScript source it corresponds to.

The goal is to accurately model the JSON wire protocol used by `pi --mode rpc`.
When upstream TypeScript types include data that cannot cross the RPC boundary,
prefer documenting the omission over expanding this crate into a full mirror of
all pi internals.

## File mapping

The human-readable mapping is below. The same mapping is also captured in
`src/types/upstream.toml`.

| Rust file          | TypeScript source                                                                                                       |
| ------------------ | ----------------------------------------------------------------------------------------------------------------------- |
| `ai.rs`            | `packages/ai/src/types.ts` — content blocks, messages, usage, models, streaming events                                  |
| `agent.rs`         | `packages/agent/src/types.ts` — `AgentMessage`, `AgentEvent`, `ThinkingLevel`                                           |
| `agent.rs`         | `packages/coding-agent/src/core/messages.ts` — declaration-merged custom message variants                               |
| `agent.rs`         | `packages/coding-agent/src/core/agent-session.ts` — `AgentSessionEvent` variants merged into `AgentEvent`               |
| `agent_session.rs` | `packages/coding-agent/src/core/agent-session.ts` — `SessionStats`                                                      |
| `bash_executor.rs` | `packages/coding-agent/src/core/bash-executor.ts` — `BashResult`                                                        |
| `compaction.rs`    | `packages/coding-agent/src/core/compaction/compaction.ts` — `CompactionResult`                                          |
| `rpc_types.rs`     | `packages/coding-agent/src/modes/rpc/rpc-types.ts` — `RpcCommand`, `RpcResponse`, `RpcSessionState`, extension UI types |
| `source_info.rs`   | `packages/coding-agent/src/core/source-info.ts` — source metadata for slash commands                                    |

When upstream `rpc-types.ts` imports a type from a new file, add a Rust file for
that source if the imported type crosses the wire. Do not treat this table as an
exclusive list of all future files.

## Notes on TypeScript → Rust mapping

- **Declaration merging**: TypeScript's `messages.ts` extends `AgentMessage` via
  declaration merging. In Rust, the merged variants are included directly in the
  `AgentMessage` enum in `agent.rs`, with comments marking their origin. This
  keeps event/message deserialization ergonomic: callers only match one
  `AgentMessage` enum.

- **`AgentSessionEvent`**: TypeScript defines this as
  `AgentEvent | { type: "queue_update"; ... } | { type: "compaction_start"; ... } | ...`.
  In Rust, the additional variants are included directly in `AgentEvent` in
  `agent.rs`, with comments marking their origin. This matches the actual event
  stream, where subscribers receive one untagged-by-source JSON union.

## Intentional deviations and rationale

These choices are intentional. Revisit them when upstream changes make the
reasons stale.

- **RPC-focused scope**: We mirror types that cross the RPC boundary. For
  example, v0.75.3 added image-generation types in `packages/ai/src/types.ts`,
  but they do not appear in `packages/coding-agent/src/modes/rpc/rpc-types.ts`,
  so they are omitted. This keeps the crate small and reduces maintenance while
  preserving correctness for its purpose: a Rust client for pi RPC mode.

- **`PiSession::clone_session()` vs RPC `"clone"`**: The wire command is named
  `clone`, but the Rust public method is `clone_session()` to avoid confusion
  with `Clone::clone`. The command enum still serializes as `"clone"`.

- **`Model.compat` is `serde_json::Value`**: Upstream `Model<TApi>` has an
  optional `compat` object for provider/API-specific behavior flags, for example
  OpenAI-compatible max-token field names, prompt-cache support, OpenRouter
  routing preferences, Anthropic-compatible cache-control/tool-streaming flags,
  etc. Its TypeScript type is conditional on `TApi`, but RPC exposes models as
  `Model<any>`, so Rust cannot know a single static shape. Keeping it as
  `serde_json::Value` preserves the wire data for callers that care while
  avoiding a large provider-compatibility enum that would need frequent updates.

- **Some streaming partials are `serde_json::Value`**: `AssistantMessageEvent`
  carries partial/final assistant messages. Some fields are represented as
  `serde_json::Value` where strict recursive Rust typing would add complexity
  without much benefit to RPC command handling. Prefer tightening these only
  when tests or real client needs show value.

- **Manual updates, not Rust codegen**: The wire surface is small and several
  mappings require judgment (declaration merging, event unions, ergonomic Rust
  method names). Codegen would still need review and patching, so it is not worth
  the complexity unless the update cadence changes substantially.

## Updating for a new pi version

Use this as a checklist, not a substitute for reading the diffs carefully.
Upstream may add new imported types or move definitions.

1. **Identify the old and new upstream tags.**

   The currently mirrored tag is listed at the top of this file and in
   `src/types/upstream.toml`.

2. **Inspect changed mapped files in `pi-mono`.**

   From the `pi-mono` checkout:

   ```bash
   git diff --stat vOLD..vNEW -- \
     packages/ai/src/types.ts \
     packages/agent/src/types.ts \
     packages/coding-agent/src/core/agent-session.ts \
     packages/coding-agent/src/core/bash-executor.ts \
     packages/coding-agent/src/core/compaction/compaction.ts \
     packages/coding-agent/src/core/messages.ts \
     packages/coding-agent/src/core/source-info.ts \
     packages/coding-agent/src/modes/rpc/rpc-types.ts
   ```

   Also inspect any **new imports** from `rpc-types.ts` or from types already
   mirrored here. A new import may require adding a new Rust source file and
   updating `src/types/mod.rs`, this README, and `src/types/upstream.toml`.

3. **Use a semantic diff if available.**

   `sem diff` can reduce noise in large TypeScript files and help orient the
   manual review:

   ```bash
   sem diff --format plain vOLD vNEW -- \
     packages/ai/src/types.ts \
     packages/agent/src/types.ts \
     packages/coding-agent/src/core/agent-session.ts \
     packages/coding-agent/src/core/bash-executor.ts \
     packages/coding-agent/src/core/compaction/compaction.ts \
     packages/coding-agent/src/core/messages.ts \
     packages/coding-agent/src/core/source-info.ts \
     packages/coding-agent/src/modes/rpc/rpc-types.ts
   ```

   Treat this as a navigation aid. The source of truth remains the upstream
   TypeScript files.

4. **Optionally record real RPC output for fixtures.**

   Prefer fixtures captured from a real `pi --mode rpc` session over hand-written
   JSON examples. `src/bin/pi-rpc-debug.rs` is the current starting point: it
   runs a real session and can print raw JSON. For now, fixture recording is
   manual and optional. If fixtures are added, record the pi version/git
   revision, command sequence, and any required model or API-key setup next to
   the captured JSON.

5. **Update Rust types.**

   Start with `rpc_types.rs`: commands, responses, state, slash commands, and
   extension UI are the primary boundary. Then update nested types used by those
   shapes (`AgentMessage`, `AgentEvent`, `Model`, `SessionStats`, etc.).

6. **Update public session methods when commands change.**

   Add one `PiSession` method per new command, preserving ergonomic Rust naming
   where necessary. Document any name that intentionally differs from the wire
   command.

7. **Update tests and docs.**

   - Add or adjust serde tests for new/changed wire shapes.
   - Update `docs/session-api.md` for public method changes.
   - Update `docs/types.md` for protocol inventory changes.
   - Update this README and `src/types/upstream.toml` to the new version.

8. **Validate.**

   Run:

   ```bash
   treefmt
   cargo nextest run
   ```

## About fixtures

Do not treat hand-written JSON as upstream evidence. Useful fixtures should come
from real RPC output, or from authoritative upstream tooling if such tooling is
added later. TypeScript interfaces are erased at runtime, and generating complete
schemas from the upstream source would risk becoming another codegen system to
maintain. For this crate, recorded `pi --mode rpc` sessions are the preferred
path when we want fixture-based regression tests.
