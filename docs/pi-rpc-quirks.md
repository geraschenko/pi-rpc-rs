# Pi RPC Protocol Quirks

Quirks and unexpected behaviors discovered while integration-testing
`PiSession` against `pi --mode rpc`. These may change as pi evolves.

## 1. No startup event

Pi does not emit any event when it finishes initializing in RPC mode.
`PiSession::spawn` works around this by sending a `get_state` command and
waiting for the response before returning.

## 2. Steer vs follow-up vs abort

These three mechanisms for interrupting the agent have different semantics
that matter primarily during **multi-turn tool-use** scenarios:

- **`steer()`** — Injected after the **current tool call** finishes,
  **skipping remaining tool calls** in the turn. The agent sees the steer
  message and produces a new response.

- **`follow_up()`** — Only delivered when the agent has **no more tool
  calls and no pending steering messages**. Extends the conversation
  beyond where the agent would normally stop.

- **`abort()`** — Cancels the in-flight API call or tool execution
  immediately. The agent emits `agent_end` with `stop_reason: "aborted"`.

For simple prompts with no tool use, `steer()` and `follow_up()` behave
identically — both queue a message delivered after the current assistant
response completes.

`prompt()` with `StreamingBehavior::Steer` or `StreamingBehavior::FollowUp`
is equivalent to calling `steer()` or `follow_up()` respectively.
Calling `prompt()` while streaming without specifying `streaming_behavior`
is an error.
