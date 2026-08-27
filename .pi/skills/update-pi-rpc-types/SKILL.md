---
name: update-pi-rpc-types
description: Update this crate to match a new upstream pi RPC protocol version. Use when bumping pi-rpc-rs compatibility to a newer pi tag.
---

# Update pi RPC Types

Use this skill when updating `pi-rpc-rs` to support a new upstream pi version.

## Required first steps

1. Read `src/types/README.md` completely before inspecting code.
2. Read `src/types/upstream.toml`. Treat it as the baseline inventory for the currently mirrored RPC types, not as proof that no new wire source exists. Derive the old tag by prefixing its `version` with `v`.
3. Identify the target upstream pi version/tag. If the user did not provide one, use the latest release tag reachable from `upstream/main` in `/home/anton/git/earendil-works/pi`:

   ```bash
   git -C /home/anton/git/earendil-works/pi fetch upstream main --tags
   git -C /home/anton/git/earendil-works/pi tag --merged upstream/main --sort=-v:refname 'v[0-9]*' | head -n1
   ```

4. Confirm both the old and new tags exist in `/home/anton/git/earendil-works/pi`:

   ```bash
   git -C /home/anton/git/earendil-works/pi rev-parse --verify "refs/tags/vOLD^{commit}"
   git -C /home/anton/git/earendil-works/pi rev-parse --verify "refs/tags/vNEW^{commit}"
   ```

5. Inspect upstream diffs before editing.
6. Present the pre-edit report described below and wait for approval before making changes.

## Upstream diff workflow

Compare the old supported tag directly with the target tag. Start with a summary across both the known type sources and the complete RPC implementation directory:

```bash
git -C /home/anton/git/earendil-works/pi diff --stat vOLD..vNEW -- \
  packages/ai/src/types.ts \
  packages/agent/src/types.ts \
  packages/coding-agent/src/core/agent-session.ts \
  packages/coding-agent/src/core/bash-executor.ts \
  packages/coding-agent/src/core/compaction/compaction.ts \
  packages/coding-agent/src/core/messages.ts \
  packages/coding-agent/src/core/session-manager.ts \
  packages/coding-agent/src/core/source-info.ts \
  packages/coding-agent/src/modes/json-event.ts \
  packages/coding-agent/src/modes/rpc
```

Then inspect the actual patch, not only the stat, for every changed source in that scope. The directory-level RPC diff is required because a newly introduced serializer or import cannot appear in the old source inventory.

Trace the target tag's complete stdout producer and typed consumer chain. In particular, inspect:

- `rpc-mode.ts`: the value passed to the stdout/output function
- serializers or adapters applied before output, such as `json-event.ts`
- `rpc-client.ts`: the event type expected by the upstream client
- `rpc-types.ts`: commands, responses, state, and extension UI records
- imports introduced by any of those files

Do not infer that events are unchanged merely because `rpc-types.ts` or the internal `AgentSessionEvent` declaration is unchanged. Runtime serialization can replace the wire shape without changing either declaration.

Determine which changes affect wire types or wire semantics and which are internal implementation changes. If a changed mapped export or serializer adds or changes an imported type used in its wire representation, follow that import. Add a new Rust mapping only when that type crosses the RPC boundary. If the source inventory changes, update both `src/types/upstream.toml` and `src/types/README.md`.

## Pre-edit report

Before editing, report:

```text
Old tag:
New tag:
Changed mapped upstream files:
Wire-visible changes:
Non-wire-visible changes:
Affected Rust files/API:
Uncertainties:
Proposed implementation and validation:
```

Keep the report concise, but account for every changed mapped source and RPC boundary file. Wait for user approval before implementation.

## Before/after RPC smoke capture

When npm access and working credentials are available, capture a broad tool-using session before and after changing the Rust decoder. The before capture must use the old pi release and the after capture the new release. Use exact npm package versions, and verify them rather than relying on whichever `pi` is on `PATH`.

The smoke script creates version-pinned `npx` wrappers, verifies their `--version` output, runs `pi-rpc-debug`, retains the full JSONL and stderr captures, and normalizes structural event information. npm versions omit the git tag's `v` prefix.

Before editing Rust code, capture the baseline:

```bash
.pi/skills/update-pi-rpc-types/rpc-smoke.sh before vOLD vNEW
```

After implementation, capture the target and then compare:

```bash
.pi/skills/update-pi-rpc-types/rpc-smoke.sh after vOLD vNEW
.pi/skills/update-pi-rpc-types/rpc-smoke.sh compare vOLD vNEW
```

`before` and `after` run setup automatically when wrappers are missing, then verify existing wrappers without rewriting them. `compare` never invokes pi or an LLM, so normalization and diff logic can be changed and rerun against the retained raw captures.

Artifacts are written under `/tmp/pi-rpc-update-OLD-NEW`. The default comparison is a pretty-printed JSON diff at `normalized.diff`; inspect the full `before.raw.jsonl`, `after.raw.jsonl`, and stderr files whenever more context is needed.

Normalization preserves discriminators, errors, decoded classifications, and scalar field paths while ignoring unstable values. It collapses only adjacent identical normalized records so nondeterministic token chunk counts do not dominate the diff while event order remains visible.

Review the normalized diff for wire changes, then inspect the corresponding raw records whenever context is needed. Confirm prompt, assistant, tool-call, tool-execution, and tool-result lifecycle coverage. An expected upstream event appearing as `unknown`, any session deserialization-error record, or any final assistant error requires investigation. If a capture is impossible, state why in the pre-edit report; do not fabricate a fixture.

The smoke capture complements assertion-based serde tests and the user-run release integration suite. It explores a broad real session; tests make expected decoding and ordering deterministic.

## Scope and compatibility policy

This crate tracks one upstream pi RPC protocol version per crate release. Do not add backward-compatibility shims or preserve old public method signatures unless the user explicitly asks for that.

When an upstream RPC command changes, update the corresponding Rust API shape. For example, if an upstream command gains a required or semantically important field, update the relevant `PiSession` method signature instead of adding a second compatibility method.

## Update checklist

- Update Rust wire types under `src/types/`, starting with `src/types/rpc_types.rs`.
- Update nested mirrored types only when they cross the RPC boundary.
- If commands change, update public methods in `src/session/impl_rpc_methods.rs`.
- Update serde tests and integration test call sites.
- Update protocol docs:
  - `docs/types.md`
  - `docs/session-api.md` when public methods change
  - `src/types/README.md`
  - `src/types/upstream.toml`
- Bump the crate patch version in `Cargo.toml`.
- Ensure `Cargo.lock` reflects the new crate version.
- Update `README.md`:
  - top compatible pi version
  - add a new compatibility table row for the new crate version and pi version
  - preserve old compatibility rows; do not overwrite history

## Validation

Run:

```bash
treefmt
scripts/presubmit.sh
```

Review the after-update smoke capture and confirm that expected records are typed rather than `unknown` before handing the update to the user for release integration testing.
