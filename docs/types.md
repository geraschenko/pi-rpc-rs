# RPC Protocol Type Inventory

Complete inventory of types that cross the RPC boundary (stdin commands, stdout events/responses). These are what we need Rust definitions for.

## Commands (stdin → pi)

Defined in `rpc-types.ts` as `RpcCommand`. Discriminated union on `type` field. All have optional `id: string` for request/response correlation.

| Command                   | Key fields                                 | Notes                                                                         |
| ------------------------- | ------------------------------------------ | ----------------------------------------------------------------------------- |
| `prompt`                  | `message`, `images?`, `streamingBehavior?` | Main entry point. `streamingBehavior` required if agent is already streaming. |
| `steer`                   | `message`, `images?`                       | Queue interrupt message.                                                      |
| `follow_up`               | `message`, `images?`                       | Queue message for after agent finishes.                                       |
| `abort`                   | —                                          | Abort current operation.                                                      |
| `new_session`             | `parentSession?`                           | Start fresh session.                                                          |
| `get_state`               | —                                          | Returns `RpcSessionState`.                                                    |
| `get_messages`            | —                                          | Returns all `AgentMessage[]`.                                                 |
| `set_model`               | `provider`, `modelId`                      | Switch model.                                                                 |
| `cycle_model`             | —                                          | Cycle to next model.                                                          |
| `get_available_models`    | —                                          | List configured models.                                                       |
| `set_thinking_level`      | `level`                                    | `"off"` \| `"minimal"` \| `"low"` \| `"medium"` \| `"high"` \| `"xhigh"`      |
| `cycle_thinking_level`    | —                                          | Cycle through levels.                                                         |
| `set_steering_mode`       | `mode`                                     | `"all"` \| `"one-at-a-time"`                                                  |
| `set_follow_up_mode`      | `mode`                                     | `"all"` \| `"one-at-a-time"`                                                  |
| `compact`                 | `customInstructions?`                      | Manual compaction.                                                            |
| `set_auto_compaction`     | `enabled`                                  | Toggle auto-compaction.                                                       |
| `set_auto_retry`          | `enabled`                                  | Toggle auto-retry.                                                            |
| `abort_retry`             | —                                          | Cancel in-progress retry.                                                     |
| `bash`                    | `command`                                  | Execute shell command (added to context on next prompt).                      |
| `abort_bash`              | —                                          | Cancel running bash.                                                          |
| `get_session_stats`       | —                                          | Token usage and cost.                                                         |
| `export_html`             | `outputPath?`                              | Export session to HTML.                                                       |
| `switch_session`          | `sessionPath`                              | Load different session file.                                                  |
| `fork`                    | `entryId`                                  | Fork from a previous user message.                                            |
| `get_fork_messages`       | —                                          | List forkable user messages.                                                  |
| `get_last_assistant_text` | —                                          | Get last assistant response text.                                             |
| `set_session_name`        | `name`                                     | Set display name for session.                                                 |
| `get_commands`            | —                                          | List available slash commands.                                                |

Plus extension UI responses (stdin):

| Command                 | Key fields                                           |
| ----------------------- | ---------------------------------------------------- |
| `extension_ui_response` | `id`, plus one of: `value`, `confirmed`, `cancelled` |

## Responses (pi → stdout)

All have `type: "response"`, `command: string`, `success: boolean`. On failure: `error: string`. On success: optional `data` varies by command. Has optional `id` matching the command's `id`.

## Events (pi → stdout)

Defined across `AgentEvent` (agent-core) and `AgentSessionEvent` (agent-session). Discriminated union on `type` field.

| Event                   | Key fields                                          | Notes                                                            |
| ----------------------- | --------------------------------------------------- | ---------------------------------------------------------------- |
| `agent_start`           | —                                                   | Agent begins processing prompt.                                  |
| `agent_end`             | `messages: AgentMessage[]`                          | Agent done. Contains ALL new messages from this run.             |
| `turn_start`            | —                                                   | New turn (1 assistant response + tool calls).                    |
| `turn_end`              | `message`, `toolResults`                            | Turn complete.                                                   |
| `message_start`         | `message: AgentMessage`                             | Message begins. Emitted for user, assistant, toolResult, custom. |
| `message_update`        | `message`, `assistantMessageEvent`                  | Streaming delta. Only for assistant messages.                    |
| `message_end`           | `message: AgentMessage`                             | Message complete.                                                |
| `tool_execution_start`  | `toolCallId`, `toolName`, `args`                    | Tool begins.                                                     |
| `tool_execution_update` | `toolCallId`, `toolName`, `args`, `partialResult`   | Tool progress.                                                   |
| `tool_execution_end`    | `toolCallId`, `toolName`, `result`, `isError`       | Tool done.                                                       |
| `auto_compaction_start` | `reason`                                            | `"threshold"` \| `"overflow"`                                    |
| `auto_compaction_end`   | `result?`, `aborted`, `willRetry`, `errorMessage?`  |                                                                  |
| `auto_retry_start`      | `attempt`, `maxAttempts`, `delayMs`, `errorMessage` |                                                                  |
| `auto_retry_end`        | `success`, `attempt`, `finalError?`                 |                                                                  |
| `extension_error`       | `extensionPath`, `event`, `error`                   |                                                                  |

Plus extension UI requests (stdout):

| Event                  | Key fields                                  |
| ---------------------- | ------------------------------------------- |
| `extension_ui_request` | `id`, `method`, plus method-specific fields |

Methods: `select`, `confirm`, `input`, `editor` (dialog, need response), `notify`, `setStatus`, `setWidget`, `setTitle`, `set_editor_text` (fire-and-forget).

## Message types (nested in events/responses)

`AgentMessage` is a union used inside events. Discriminated on `role` field.

| Role                | Type                       | Source          |
| ------------------- | -------------------------- | --------------- |
| `user`              | `UserMessage`              | pi-ai           |
| `assistant`         | `AssistantMessage`         | pi-ai           |
| `toolResult`        | `ToolResultMessage`        | pi-ai           |
| `bashExecution`     | `BashExecutionMessage`     | pi-coding-agent |
| `custom`            | `CustomMessage`            | pi-coding-agent |
| `branchSummary`     | `BranchSummaryMessage`     | pi-coding-agent |
| `compactionSummary` | `CompactionSummaryMessage` | pi-coding-agent |

### Content blocks (nested in messages)

| Type              | Fields                                               |
| ----------------- | ---------------------------------------------------- |
| `TextContent`     | `type: "text"`, `text`                               |
| `ImageContent`    | `type: "image"`, `data` (base64), `mimeType`         |
| `ThinkingContent` | `type: "thinking"`, `thinking`, `thinkingSignature?` |
| `ToolCall`        | `type: "toolCall"`, `id`, `name`, `arguments`        |

### AssistantMessageEvent (nested in `message_update`)

Discriminated on `type`:

- `start` — generation started
- `text_start`, `text_delta`, `text_end` — text streaming
- `thinking_start`, `thinking_delta`, `thinking_end` — thinking streaming
- `toolcall_start`, `toolcall_delta`, `toolcall_end` — tool call streaming
- `done` — complete (has `reason`)
- `error` — failed (has `reason`)

All carry `partial: AssistantMessage` (the in-progress message) and `contentIndex: usize`.

### Model

```typescript
{
  id: string,
  name: string,
  api: string,           // "anthropic-messages" | "openai-chat" | etc.
  provider: string,
  baseUrl: string,
  reasoning: boolean,
  input: string[],        // ["text", "image"]
  contextWindow: number,
  maxTokens: number,
  cost: { input, output, cacheRead, cacheWrite }  // per million tokens
}
```

### Usage / Cost (nested in AssistantMessage)

```typescript
{
  input: number,
  output: number,
  cacheRead: number,
  cacheWrite: number,
  totalTokens: number,
  cost: { input, output, cacheRead, cacheWrite, total }
}
```

### StopReason

`"stop"` | `"length"` | `"toolUse"` | `"error"` | `"aborted"`

### ThinkingLevel

`"off"` | `"minimal"` | `"low"` | `"medium"` | `"high"` | `"xhigh"`
