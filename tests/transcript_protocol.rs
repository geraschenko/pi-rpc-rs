//! Assertion-based wire examples, not captured upstream fixtures.

use pi_rpc_rs::types::*;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Value, json};

fn roundtrip<T: DeserializeOwned + Serialize>(value: Value) -> T {
  let decoded: T = serde_json::from_value(value.clone()).unwrap();
  assert_eq!(serde_json::to_value(&decoded).unwrap(), value);
  decoded
}

fn usage() -> Value {
  json!({"input": 1.0, "output": 0.0, "cacheRead": 0.0, "cacheWrite": 0.0,
    "totalTokens": 1.0,
    "cost": {"input": 0.0, "output": 0.0, "cacheRead": 0.0, "cacheWrite": 0.0, "total": 0.0}})
}

fn system_message() -> Value {
  json!({"role": "system", "content": "Instructions", "timestamp": 1.0,
    "sections": {"z-first": "First", "a-second": "Second", "removed": null},
    "toolsAdded": [{"name": "bash", "description": "Run a command",
      "parameters": {"type": "object", "properties": {"command": {"type": "string"}}},
      "constrainedSampling": false}],
    "toolsRemoved": [{"name": "old_tool"}]})
}

#[test]
fn clear_queue_command_and_responses() {
  let command: RpcCommand = roundtrip(json!({"id": "q", "type": "clear_queue"}));
  assert!(matches!(command.kind, RpcCommandKind::ClearQueue));
  let response: RpcResponse = roundtrip(json!({"id": "q", "type": "response",
    "command": "clear_queue", "success": true,
    "data": {"steering": ["interrupt"], "followUp": ["later"]}}));
  assert_eq!(response.kind.command_name(), "clear_queue");
  assert!(matches!(response.kind, RpcResponseKind::ClearQueue(_)));
  roundtrip::<RpcResponse>(json!({"type": "response", "command": "clear_queue",
    "success": false, "error": "failed"}));
  assert!(
    serde_json::from_value::<RpcResponse>(json!({"type": "response",
    "command": "clear_queue", "success": true}))
    .is_err()
  );
}

#[test]
fn system_message_lifecycle_and_section_order() {
  let message: AgentMessage = roundtrip(system_message());
  let AgentMessage::System { sections, .. } = &message else {
    panic!("expected system")
  };
  assert_eq!(
    sections
      .as_ref()
      .unwrap()
      .keys()
      .map(String::as_str)
      .collect::<Vec<_>>(),
    ["z-first", "a-second", "removed"]
  );
  let encoded = serde_json::to_string(&message).unwrap();
  assert!(encoded.find("z-first").unwrap() < encoded.find("a-second").unwrap());
  for event_type in ["message_start", "message_end"] {
    let event: RpcEvent =
      serde_json::from_value(json!({"type": event_type, "message": system_message()})).unwrap();
    assert!(matches!(event, RpcEvent::Agent(_)));
  }
  roundtrip::<AgentMessage>(
    json!({"role": "system", "content": [{"type": "text", "text": "Update"}], "timestamp": 2.0}),
  );
}

#[test]
fn tool_sampling_declarations() {
  for sampling in [
    json!(false),
    json!({"type": "json_schema", "strict": "prefer"}),
    json!({"type": "json_schema", "strict": "require"}),
    json!({"type": "grammar", "variants": {"openai_lark": "start: /x/", "openai_regex": "x"}}),
  ] {
    roundtrip::<Tool>(
      json!({"name": "tool", "description": "description", "parameters": {}, "constrainedSampling": sampling}),
    );
  }
  assert!(serde_json::from_value::<ConstrainedSampling>(json!(true)).is_err());
}

#[test]
fn session_entries_and_nested_wire_boundaries() {
  let entries = vec![
    json!({"type": "usage", "id": "usage", "parentId": null, "timestamp": "2026-01-01T00:00:00Z",
      "kind": "cache_warm", "provider": "test", "model": "test", "usage": usage(), "note": "warm"}),
    json!({"type": "compaction", "id": "compact", "parentId": null, "timestamp": "2026-01-01T00:00:00Z",
      "summary": "summary", "firstKeptEntryId": "compact", "tokensBefore": 1.0, "systemMessage": system_message()}),
  ];
  for entry in entries {
    roundtrip::<SessionEntry>(entry.clone());
    let event: RpcEvent =
      serde_json::from_value(json!({"type": "entry_appended", "entry": entry.clone()})).unwrap();
    assert!(matches!(
      event,
      RpcEvent::Agent(AgentEvent::EntryAppended { .. })
    ));
    roundtrip::<RpcResponse>(
      json!({"type": "response", "command": "get_entries", "success": true,
      "data": {"entries": [entry.clone()], "leafId": null}}),
    );
    roundtrip::<SessionTreeNode>(json!({"entry": entry, "children": []}));
  }
  for replacement in [
    Value::Null,
    json!({"content": "replacement"}),
    json!({"content": [{"type": "thinking", "thinking": "reason"}, {"type": "text", "text": "replacement"}]}),
  ] {
    let entry = json!({"type": "context_edit", "id": "edit", "parentId": "target",
      "timestamp": "2026-01-01T00:00:00Z", "targetId": "target", "replacement": replacement});
    let event: AgentEvent = roundtrip(json!({"type": "entry_appended", "entry": entry}));
    assert!(matches!(
      event,
      AgentEvent::EntryAppended {
        entry: SessionEntry::ContextEdit(_)
      }
    ));
  }
}

#[test]
fn assistant_thinking_level_and_nullable_branch_summary() {
  let message: AgentMessage = roundtrip(json!({"role": "assistant", "content": [], "api": "test",
    "provider": "test", "model": "test", "providerThinkingLevel": "high", "usage": usage(),
    "stopReason": "stop", "timestamp": 1.0}));
  assert!(matches!(
    message,
    AgentMessage::Assistant {
      provider_thinking_level: Some(_),
      ..
    }
  ));
  roundtrip::<AgentMessage>(
    json!({"role": "branchSummary", "summary": "summary", "fromId": null, "timestamp": 1.0}),
  );
}

#[test]
fn model_input_limits_and_prompt_cache() {
  roundtrip::<Model>(
    json!({"id": "test", "name": "Test", "api": "test", "provider": "test",
    "baseUrl": "https://example.com", "reasoning": false, "input": ["text", "image"],
    "cost": {"input": 0.0, "output": 0.0, "cacheRead": 0.0, "cacheWrite": 0.0},
    "contextWindow": 1000.0, "maxTokens": 100.0,
    "inputLimits": {"maxRequestBytes": 10000.0, "images": {"maxPerMessage": 2.0, "maxPerRequest": 4.0,
      "resize": {"maxWidth": 100.0, "maxHeight": 100.0, "maxBytes": 1000.0, "jpegQuality": 80.0}}},
    "promptCache": {"short": 300.0, "long": 3600.0}}),
  );
  roundtrip::<ModelPromptCache>(json!({}));
}
