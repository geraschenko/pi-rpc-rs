# RPC Protocol Type Inventory

Complete inventory of types that cross the RPC boundary (stdin commands, stdout events/responses). These are what we need Rust definitions for.

## Commands (stdin → pi)

Defined in `rpc-types.ts` as `RpcCommand`. Discriminated union on `type` field. All have optional `id: string` for request/response correlation.

| Command                         | Key fields                                 | Notes                                                                         |
| ------------------------------- | ------------------------------------------ | ----------------------------------------------------------------------------- |
| `prompt`                        | `message`, `images?`, `streamingBehavior?` | Main entry point. `streamingBehavior` required if agent is already streaming. |
| `steer`                         | `message`, `images?`                       | Queue interrupt message.                                                      |
| `follow_up`                     | `message`, `images?`                       | Queue message for after agent finishes.                                       |
| `abort`                         | —                                          | Abort current operation.                                                      |
| `clear_queue`                   | —                                          | Returns and clears `{ steering, followUp }` queues.                           |
| `new_session`                   | `parentSession?`                           | Start fresh session.                                                          |
| `get_state`                     | —                                          | Returns `RpcSessionState`.                                                    |
| `get_messages`                  | —                                          | Returns current context `AgentMessage[]`.                                     |
| `get_entries`                   | `since?`                                   | Returns session entries and active leaf ID.                                   |
| `get_tree`                      | —                                          | Returns session entry tree and active leaf ID.                                |
| `set_model`                     | `provider`, `modelId`                      | Switch model.                                                                 |
| `cycle_model`                   | —                                          | Cycle to next model.                                                          |
| `get_available_models`          | —                                          | List configured models.                                                       |
| `set_thinking_level`            | `level`                                    | `"off"` \| `"minimal"` \| `"low"` \| `"medium"` \| `"high"` \| `"xhigh"`      |
| `cycle_thinking_level`          | —                                          | Cycle through levels.                                                         |
| `get_available_thinking_levels` | —                                          | List thinking levels supported by the current model.                          |
| `set_steering_mode`             | `mode`                                     | `"all"` \| `"one-at-a-time"`                                                  |
| `set_follow_up_mode`            | `mode`                                     | `"all"` \| `"one-at-a-time"`                                                  |
| `compact`                       | `customInstructions?`                      | Manual compaction.                                                            |
| `set_auto_compaction`           | `enabled`                                  | Toggle auto-compaction.                                                       |
| `set_auto_retry`                | `enabled`                                  | Toggle auto-retry.                                                            |
| `abort_retry`                   | —                                          | Cancel in-progress retry.                                                     |
| `bash`                          | `command`, `excludeFromContext`            | Execute shell command; optionally exclude output from LLM context.            |
| `abort_bash`                    | —                                          | Cancel running bash.                                                          |
| `get_session_stats`             | —                                          | Token usage and cost.                                                         |
| `export_html`                   | `outputPath?`                              | Export session to HTML.                                                       |
| `switch_session`                | `sessionPath`                              | Load different session file.                                                  |
| `fork`                          | `entryId`                                  | Fork from a previous user message.                                            |
| `clone`                         | —                                          | Clone the current active branch into a new session.                           |
| `get_fork_messages`             | —                                          | List forkable user messages.                                                  |
| `get_last_assistant_text`       | —                                          | Get last assistant response text.                                             |
| `set_session_name`              | `name`                                     | Set display name for session.                                                 |
| `get_commands`                  | —                                          | List available slash commands.                                                |

Plus extension UI responses (stdin):

| Command                 | Key fields                                           |
| ----------------------- | ---------------------------------------------------- |
| `extension_ui_response` | `id`, plus one of: `value`, `confirmed`, `cancelled` |

## Responses (pi → stdout)

All have `type: "response"`, `command: string`, `success: boolean`. On failure: `error: string`. On success: optional `data` varies by command. Has optional `id` matching the command's `id`.

## Events (pi → stdout)

Defined across `AgentEvent` (agent-core) and `AgentSessionEvent` (agent-session). Discriminated union on `type` field.

| Event                               | Key fields                                                   | Notes                                                                    |
| ----------------------------------- | ------------------------------------------------------------ | ------------------------------------------------------------------------ |
| `agent_start`                       | —                                                            | Agent begins processing prompt.                                          |
| `agent_end`                         | `messages: AgentMessage[]`, `willRetry`                      | Agent done. Contains ALL new messages from this run.                     |
| `turn_start`                        | —                                                            | New turn (1 assistant response + tool calls).                            |
| `turn_end`                          | `message`, `toolResults`                                     | Turn complete.                                                           |
| `message_start`                     | `message: AgentMessage`                                      | Message begins. Emitted for system, user, assistant, toolResult, custom. |
| `message_update`                    | `usage`, `assistantMessageEvent`                             | Compact streaming delta. Only for assistant messages.                    |
| `message_end`                       | `message: AgentMessage`                                      | Message complete.                                                        |
| `tool_execution_start`              | `toolCallId`, `toolName`, `args`                             | Tool begins.                                                             |
| `tool_execution_update`             | `toolCallId`, `toolName`, `args`, `partialResult`            | Tool progress.                                                           |
| `tool_execution_end`                | `toolCallId`, `toolName`, `result`, `isError`                | Tool done.                                                               |
| `queue_update`                      | `steering`, `followUp`                                       | Current pending queues.                                                  |
| `compaction_start`                  | `reason`                                                     | `"manual"` \| `"threshold"` \| `"overflow"`                              |
| `agent_settled`                     | —                                                            | Agent run and post-run continuations finished.                           |
| `entry_appended`                    | `entry`                                                      | Session entry appended.                                                  |
| `session_info_changed`              | `name`                                                       | Session name changed.                                                    |
| `thinking_level_changed`            | `level`                                                      | Thinking level changed.                                                  |
| `compaction_end`                    | `reason`, `result?`, `aborted`, `willRetry`, `errorMessage?` |                                                                          |
| `auto_retry_start`                  | `attempt`, `maxAttempts`, `delayMs`, `errorMessage`          |                                                                          |
| `auto_retry_end`                    | `success`, `attempt`, `finalError?`                          |                                                                          |
| `summarization_retry_scheduled`     | `attempt`, `maxAttempts`, `delayMs`, `errorMessage`          | Compaction or branch-summary retry scheduled.                            |
| `summarization_retry_attempt_start` | `source`, `reason?`                                          | Summarization retry started.                                             |
| `summarization_retry_finished`      | —                                                            | Summarization retry sequence finished.                                   |
| `bash_execution_update`             | `id?`, `delta`                                               | Streaming output from an RPC bash command.                               |
| `extension_error`                   | `extensionPath`, `event`, `error`                            |                                                                          |

Unrecognized non-response records deserialize as `RpcEvent::Unknown(serde_json::Value)` so clients can tolerate future or fork-specific wire records.

Plus extension UI requests (stdout):

| Event                  | Key fields                                  |
| ---------------------- | ------------------------------------------- |
| `extension_ui_request` | `id`, `method`, plus method-specific fields |

Methods: `select`, `confirm`, `input`, `editor` (dialog, need response), `notify`, `setStatus`, `setWidget`, `setTitle`, `set_editor_text` (fire-and-forget).

## Message types (nested in events/responses)

`AgentMessage` is a union used inside events. Discriminated on `role` field.

| Role                | Type                       | Source          |
| ------------------- | -------------------------- | --------------- |
| `system`            | `SystemMessage`            | pi-ai           |
| `user`              | `UserMessage`              | pi-ai           |
| `assistant`         | `AssistantMessage`         | pi-ai           |
| `toolResult`        | `ToolResultMessage`        | pi-ai           |
| `bashExecution`     | `BashExecutionMessage`     | pi-coding-agent |
| `custom`            | `CustomMessage`            | pi-coding-agent |
| `branchSummary`     | `BranchSummaryMessage`     | pi-coding-agent |
| `compactionSummary` | `CompactionSummaryMessage` | pi-coding-agent |

Assistant messages may include `rawStopReason`, preserving the provider's stop
reason alongside pi's normalized `stopReason`, and `endTurn`, preserving whether
the provider explicitly ended its turn. A deferred response includes a
`deferred` handle identifying the provider request to poll. Tool-result messages
may include execution `usage`. Assistant messages may also include the exact
`providerThinkingLevel` used for the response. Branch-summary `fromId` may be null.

System messages carry string or text-block `content`, optional ordered `sections`
(string values replace sections; null removes them), `toolsAdded`, and
`toolsRemoved`. Tool declarations include `name`, `description`, JSON Schema
`parameters`, and optional `constrainedSampling` (`false`, JSON-schema strictness,
or grammar variants). System messages replay prompt and tool changes in transcript
order.

### Content blocks (nested in messages)

| Type              | Fields                                                      |
| ----------------- | ----------------------------------------------------------- |
| `TextContent`     | `type: "text"`, `text`                                      |
| `ImageContent`    | `type: "image"`, `data` (base64), `mimeType`                |
| `ThinkingContent` | `type: "thinking"`, `thinking`, `thinkingSignature?`        |
| `ToolCall`        | `type: "toolCall"`, `id`, `name`, `arguments`, `namespace?` |

### AssistantMessageEvent (nested in `message_update`)

Discriminated on `type`:

- `start` — generation started
- `text_start`, `text_delta`, `text_end` — text streaming
- `thinking_start`, `thinking_delta`, `thinking_end` — thinking streaming
- `toolcall_start`, `toolcall_delta`, `toolcall_end` — tool call streaming
- `done` — complete (has `reason`)
- `error` — failed (has `reason`)

RPC serializes these through `json-event.ts`, which removes cumulative `partial`
assistant snapshots. Content events carry `contentIndex`; `toolcall_start` also
carries `id` and `toolName`. The enclosing `message_update` carries cumulative
`usage`. `message_start` and `message_end` provide the initial and final
assistant messages.

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
  samplingParams?: Record<string, unknown>,
  promptCache?: { short?: number, long?: number }, // lifetime in seconds
  inputLimits?: {
    maxRequestBytes?: number,
    images?: {
      maxPerMessage?: number, maxPerRequest?: number,
      resize?: { maxWidth?: number, maxHeight?: number, maxBytes?: number, jpegQuality?: number }
    }
  },
  cost: { input, output, cacheRead, cacheWrite, tiers? }  // per million tokens
}
```

### Usage / Cost (nested in AssistantMessage)

```typescript
{
  input: number,
  output: number,
  cacheRead: number,
  cacheWrite: number,
  cacheWrite1h?: number,
  reasoning?: number,
  totalTokens: number,
  cost: { input, output, cacheRead, cacheWrite, total }
}
```

### CompactionResult

```typescript
{
  summary: string,
  firstKeptEntryId: string,
  tokensBefore: number,
  estimatedTokensAfter?: number,
  usage?: Usage,
  details?: unknown
}
```

### SessionEntry / SessionTreeNode

`get_entries` returns `{ entries: SessionEntry[], leafId: string | null }`.
`get_tree` returns `{ tree: SessionTreeNode[], leafId: string | null }`.

`SessionEntry` is discriminated by `type`: `message`, `thinking_level_change`,
`model_change`, `compaction`, `branch_summary`, `custom`, `custom_message`,
`label`, `session_info`, `usage`, or `context_edit`. Each entry has `id`,
`parentId`, and `timestamp`. Usage entries carry `kind`, `provider`, `model`,
`usage`, and optional `note` (for example, cache warming). Context edits carry
`targetId` and `replacement`: null omits the target from model context; otherwise
`{ content }` replaces its content while preserving raw history. Compaction entries
may include a `systemMessage` checkpoint.
Compaction and branch-summary entries may also include the summarization call's
`usage`. `SessionTreeNode` has `entry`, `children`, and optional `label` /
`labelTimestamp`.

### DeferredHandle

Deferred assistant messages may include `{ provider, modelId, api, id,
expiresAt?, pollAfterMs?, data? }`. `data` contains provider-specific JSON needed
to retrieve the final response.

### StopReason

`"pending"` | `"stop"` | `"length"` | `"toolUse"` | `"error"` | `"aborted"` | `"deferred"`

### ThinkingLevel

`"off"` | `"minimal"` | `"low"` | `"medium"` | `"high"` | `"xhigh"` | `"max"`
