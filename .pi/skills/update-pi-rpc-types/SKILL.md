---
name: update-pi-rpc-types
description: Update this crate to match a new upstream pi RPC protocol version. Use when bumping pi-rpc-rs compatibility to a newer pi tag.
---

# Update pi RPC Types

Use this skill when updating `pi-rpc-rs` to support a new upstream pi version.

## Required first steps

1. Read `src/types/README.md` completely before inspecting code.
2. Read `src/types/upstream.toml` to identify the currently mirrored pi version.
3. Identify the target upstream pi version/tag. If the user did not provide one, use the latest release tag reachable from `upstream/main` in `/home/anton/git/earendil-works/pi`:

   ```bash
   git -C /home/anton/git/earendil-works/pi fetch upstream main --tags
   git -C /home/anton/git/earendil-works/pi tag --merged upstream/main --sort=-v:refname 'v[0-9]*' | head -n1
   ```

4. Confirm the target new pi version/tag exists in `/home/anton/git/earendil-works/pi`.
5. Inspect upstream diffs before editing.
6. Present a concise update plan to the user and wait for approval before making changes.

## Upstream diff workflow

From this crate, use commands like:

```bash
git -C /home/anton/git/earendil-works/pi diff --stat vOLD..vNEW -- \
  packages/ai/src/types.ts \
  packages/agent/src/types.ts \
  packages/coding-agent/src/core/agent-session.ts \
  packages/coding-agent/src/core/bash-executor.ts \
  packages/coding-agent/src/core/compaction/compaction.ts \
  packages/coding-agent/src/core/messages.ts \
  packages/coding-agent/src/core/source-info.ts \
  packages/coding-agent/src/modes/rpc/rpc-types.ts
```

Then inspect changed files with enough context, especially `rpc-types.ts` and any new imports it introduces.

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
