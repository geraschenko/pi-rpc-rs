# Handoff: tolerate unknown wire records in `RpcEvent`

Context for whoever works on pi-rpc-rs next (written 2026-07-11, during
design of a Rust client for [pictl](https://github.com/geraschenko/pictl),
targeting the pi 0.80.6 bump).

## Why pi-rpc-rs will see records its types don't model

pictl runs a fork of pi (`@geraschenko/pi-coding-agent`, e.g.
`0.80.6-fork.0`) that exposes the RPC protocol over a Unix socket
(`--rpc-socket`) in addition to stdio. The socket variant emits records that
upstream `pi --mode rpc` never produces:

- a connection banner: `{"type": "hello", "protocol": "pi-rpc-socket", "version": 1}`
- `{"type": "session_changed", "sessionId": ..., "sessionFile"?: ...}`
- `{"type": "tree_navigated", "oldLeafId": ..., "newLeafId": ..., ...}`
- `{"type": "ui_wait_start", "requestId": ..., "request": ...}`
- `{"type": "ui_wait_end", "requestId": ..., "request": ..., "resolution": ...}`
- `{"type": "shutdown"}`

The plan of record is for pi-rpc-rs to eventually gain a transport
abstraction (stdio subprocess vs. existing socket) and typed variants for
these records, so that pictl's Rust client (`pictl-rs`, in pictl's `rust/`
directory) can depend on pi-rpc-rs for all wire types. Until then, pictl-rs
depends on pi-rpc-rs for the upstream types only and does its own
line-level dispatch. See pictl's `docs/pi-modifications.md` for the fork's
record shapes.

## Problem 1: no fallback for unknown record types

`impl Deserialize for RpcEvent` (`src/types/rpc_types.rs`) routes by the
`type` field and, for anything unrecognized, attempts an `AgentEvent` parse,
which fails. There is no lenient path: one record of an unknown type turns
into a hard deserialization error.

For a long-lived client attached to a live agent, an unknown record must be
survivable — new pi versions or forks add record types, and dropping the
connection (or spamming `DeserializationError` events) for each one makes
version skew needlessly painful.

Suggested fix: add an `Unknown(serde_json::Value)` variant to `RpcEvent`
(or to whatever record-level enum the transport abstraction introduces) that
captures any record whose `type` is unrecognized. Subscribers that care can
inspect it; everyone else ignores it.

## Problem 2: the `session_*` prefix dispatch claims a wire namespace

The same `Deserialize` impl routes `type_str.starts_with("session_")` to
`SessionEvent`. But `SessionEvent`'s only variants are
`session_process_exited` and `session_deserialization_error` — events
synthesized by the Rust wrapper itself, never emitted by pi on the wire. The
prefix match therefore claims the entire `session_*` wire namespace for a
type that models none of it, and the fork's `session_changed` (a genuine
wire record) fails to parse as a `SessionEvent`.

Suggested fix: match the two wrapper event names exactly instead of by
prefix. Combined with the `Unknown` fallback above, `session_changed` then
degrades gracefully until it gets a typed variant.

## Suggested acceptance test

Feed the deserializer each record listed at the top of this doc plus a
made-up `{"type": "never_heard_of_it", "x": 1}`; every one should produce
`Unknown` (not an error), and `session_process_exited` /
`session_deserialization_error` should still round-trip as `SessionEvent`.
