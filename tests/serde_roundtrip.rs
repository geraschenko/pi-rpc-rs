//! Serde round-trip unit tests for RPC types.
//!
//! These do NOT require `pi` to be installed — they test that known JSON payloads
//! deserialize correctly into Rust types and serialize back.

use pi_rpc_rs::types::*;

// ============================================================================
// RpcCommand serialization
// ============================================================================

#[test]
fn command_prompt_serialize() {
  let cmd = RpcCommand {
    id: Some("1".into()),
    kind: RpcCommandKind::Prompt {
      message: "hello".into(),
      images: None,
      streaming_behavior: None,
    },
  };
  let json = serde_json::to_value(&cmd).unwrap();
  assert_eq!(json["type"], "prompt");
  assert_eq!(json["id"], "1");
  assert_eq!(json["message"], "hello");
  assert!(json.get("images").is_none());
  assert!(json.get("streamingBehavior").is_none());
}

#[test]
fn command_prompt_with_streaming_behavior() {
  let cmd = RpcCommand {
    id: Some("2".into()),
    kind: RpcCommandKind::Prompt {
      message: "test".into(),
      images: None,
      streaming_behavior: Some(StreamingBehavior::Steer),
    },
  };
  let json = serde_json::to_value(&cmd).unwrap();
  assert_eq!(json["streamingBehavior"], "steer");
}

#[test]
fn command_steer_serialize() {
  let cmd = RpcCommand {
    id: Some("3".into()),
    kind: RpcCommandKind::Steer {
      message: "stop".into(),
      images: None,
    },
  };
  let json = serde_json::to_value(&cmd).unwrap();
  assert_eq!(json["type"], "steer");
  assert_eq!(json["message"], "stop");
}

#[test]
fn command_abort_serialize() {
  let cmd = RpcCommand {
    id: None,
    kind: RpcCommandKind::Abort,
  };
  let json = serde_json::to_value(&cmd).unwrap();
  assert_eq!(json["type"], "abort");
  assert!(json.get("id").is_none());
}

#[test]
fn command_set_model_serialize() {
  let cmd = RpcCommand {
    id: Some("5".into()),
    kind: RpcCommandKind::SetModel {
      provider: "anthropic".into(),
      model_id: "claude-sonnet-4-20250514".into(),
    },
  };
  let json = serde_json::to_value(&cmd).unwrap();
  assert_eq!(json["type"], "set_model");
  assert_eq!(json["provider"], "anthropic");
  assert_eq!(json["modelId"], "claude-sonnet-4-20250514");
}

#[test]
fn command_set_thinking_level_serialize() {
  let cmd = RpcCommand {
    id: Some("6".into()),
    kind: RpcCommandKind::SetThinkingLevel {
      level: ThinkingLevel::High,
    },
  };
  let json = serde_json::to_value(&cmd).unwrap();
  assert_eq!(json["type"], "set_thinking_level");
  assert_eq!(json["level"], "high");
}

#[test]
fn command_bash_serialize() {
  let cmd = RpcCommand {
    id: Some("7".into()),
    kind: RpcCommandKind::Bash {
      command: "echo hello".into(),
      exclude_from_context: true,
    },
  };
  let json = serde_json::to_value(&cmd).unwrap();
  assert_eq!(json["type"], "bash");
  assert_eq!(json["command"], "echo hello");
  assert_eq!(json["excludeFromContext"], true);
}

#[test]
fn command_bash_includes_no_exclude_from_context_when_false() {
  let cmd = RpcCommand {
    id: Some("7b".into()),
    kind: RpcCommandKind::Bash {
      command: "echo hello".into(),
      exclude_from_context: false,
    },
  };
  let json = serde_json::to_value(&cmd).unwrap();
  assert_eq!(json["type"], "bash");
  assert_eq!(json["command"], "echo hello");
  assert!(json.get("excludeFromContext").is_none());
}

#[test]
fn command_compact_serialize() {
  let cmd = RpcCommand {
    id: Some("8".into()),
    kind: RpcCommandKind::Compact {
      custom_instructions: Some("summarize briefly".into()),
    },
  };
  let json = serde_json::to_value(&cmd).unwrap();
  assert_eq!(json["type"], "compact");
  assert_eq!(json["customInstructions"], "summarize briefly");
}

#[test]
fn command_get_state_serialize() {
  let cmd = RpcCommand {
    id: Some("9".into()),
    kind: RpcCommandKind::GetState,
  };
  let json = serde_json::to_value(&cmd).unwrap();
  assert_eq!(json["type"], "get_state");
}

#[test]
fn command_new_session_serialize() {
  let cmd = RpcCommand {
    id: Some("10".into()),
    kind: RpcCommandKind::NewSession {
      parent_session: Some("/path/to/session".into()),
    },
  };
  let json = serde_json::to_value(&cmd).unwrap();
  assert_eq!(json["type"], "new_session");
  assert_eq!(json["parentSession"], "/path/to/session");
}

#[test]
fn command_set_session_name_serialize() {
  let cmd = RpcCommand {
    id: Some("11".into()),
    kind: RpcCommandKind::SetSessionName {
      name: "my session".into(),
    },
  };
  let json = serde_json::to_value(&cmd).unwrap();
  assert_eq!(json["type"], "set_session_name");
  assert_eq!(json["name"], "my session");
}

#[test]
fn command_switch_session_serialize() {
  let cmd = RpcCommand {
    id: Some("12".into()),
    kind: RpcCommandKind::SwitchSession {
      session_path: "/some/path".into(),
    },
  };
  let json = serde_json::to_value(&cmd).unwrap();
  assert_eq!(json["type"], "switch_session");
  assert_eq!(json["sessionPath"], "/some/path");
}

#[test]
fn command_fork_serialize() {
  let cmd = RpcCommand {
    id: Some("13".into()),
    kind: RpcCommandKind::Fork {
      entry_id: "entry-123".into(),
    },
  };
  let json = serde_json::to_value(&cmd).unwrap();
  assert_eq!(json["type"], "fork");
  assert_eq!(json["entryId"], "entry-123");
}

#[test]
fn command_follow_up_serialize() {
  let cmd = RpcCommand {
    id: Some("14".into()),
    kind: RpcCommandKind::FollowUp {
      message: "next".into(),
      images: None,
    },
  };
  let json = serde_json::to_value(&cmd).unwrap();
  assert_eq!(json["type"], "follow_up");
  assert_eq!(json["message"], "next");
}

#[test]
fn command_set_queue_modes_serialize() {
  let cmd = RpcCommand {
    id: Some("15".into()),
    kind: RpcCommandKind::SetSteeringMode {
      mode: QueueMode::OneAtATime,
    },
  };
  let json = serde_json::to_value(&cmd).unwrap();
  assert_eq!(json["type"], "set_steering_mode");
  assert_eq!(json["mode"], "one-at-a-time");

  let cmd2 = RpcCommand {
    id: Some("16".into()),
    kind: RpcCommandKind::SetFollowUpMode {
      mode: QueueMode::All,
    },
  };
  let json2 = serde_json::to_value(&cmd2).unwrap();
  assert_eq!(json2["type"], "set_follow_up_mode");
  assert_eq!(json2["mode"], "all");
}

// ============================================================================
// RpcCommand deserialization (round-trip)
// ============================================================================

#[test]
fn command_roundtrip() {
  let commands = vec![
    RpcCommandKind::GetState,
    RpcCommandKind::Abort,
    RpcCommandKind::CycleModel,
    RpcCommandKind::CycleThinkingLevel,
    RpcCommandKind::GetAvailableModels,
    RpcCommandKind::GetMessages,
    RpcCommandKind::GetCommands,
    RpcCommandKind::GetSessionStats,
    RpcCommandKind::Clone,
    RpcCommandKind::GetForkMessages,
    RpcCommandKind::GetLastAssistantText,
    RpcCommandKind::AbortBash,
    RpcCommandKind::AbortRetry,
    RpcCommandKind::SetAutoCompaction { enabled: true },
    RpcCommandKind::SetAutoRetry { enabled: false },
    RpcCommandKind::Prompt {
      message: "hi".into(),
      images: None,
      streaming_behavior: Some(StreamingBehavior::FollowUp),
    },
    RpcCommandKind::Bash {
      command: "ls".into(),
      exclude_from_context: false,
    },
  ];

  for kind in commands {
    let cmd = RpcCommand {
      id: Some("rt".into()),
      kind: kind.clone(),
    };
    let json = serde_json::to_string(&cmd).unwrap();
    let back: RpcCommand = serde_json::from_str(&json).unwrap();
    assert_eq!(cmd, back, "round-trip failed for {:?}", kind);
  }
}

#[test]
fn command_kind_wire_names() {
  let cases = [
    (RpcCommandKind::GetState, "get_state"),
    (RpcCommandKind::Abort, "abort"),
    (RpcCommandKind::CycleModel, "cycle_model"),
    (RpcCommandKind::CycleThinkingLevel, "cycle_thinking_level"),
    (RpcCommandKind::GetAvailableModels, "get_available_models"),
    (RpcCommandKind::GetMessages, "get_messages"),
    (RpcCommandKind::GetCommands, "get_commands"),
    (RpcCommandKind::GetSessionStats, "get_session_stats"),
    (RpcCommandKind::Clone, "clone"),
    (RpcCommandKind::GetForkMessages, "get_fork_messages"),
    (
      RpcCommandKind::GetLastAssistantText,
      "get_last_assistant_text",
    ),
    (RpcCommandKind::AbortBash, "abort_bash"),
    (RpcCommandKind::AbortRetry, "abort_retry"),
    (
      RpcCommandKind::SetAutoCompaction { enabled: true },
      "set_auto_compaction",
    ),
    (
      RpcCommandKind::SetAutoRetry { enabled: false },
      "set_auto_retry",
    ),
    (
      RpcCommandKind::Prompt {
        message: "hi".into(),
        images: None,
        streaming_behavior: None,
      },
      "prompt",
    ),
    (
      RpcCommandKind::Steer {
        message: "hi".into(),
        images: None,
      },
      "steer",
    ),
    (
      RpcCommandKind::FollowUp {
        message: "hi".into(),
        images: None,
      },
      "follow_up",
    ),
    (
      RpcCommandKind::NewSession {
        parent_session: None,
      },
      "new_session",
    ),
    (
      RpcCommandKind::SetModel {
        provider: "p".into(),
        model_id: "m".into(),
      },
      "set_model",
    ),
    (
      RpcCommandKind::SetThinkingLevel {
        level: ThinkingLevel::Medium,
      },
      "set_thinking_level",
    ),
    (
      RpcCommandKind::SetSteeringMode {
        mode: QueueMode::All,
      },
      "set_steering_mode",
    ),
    (
      RpcCommandKind::SetFollowUpMode {
        mode: QueueMode::OneAtATime,
      },
      "set_follow_up_mode",
    ),
    (
      RpcCommandKind::Compact {
        custom_instructions: None,
      },
      "compact",
    ),
    (
      RpcCommandKind::Bash {
        command: "ls".into(),
        exclude_from_context: false,
      },
      "bash",
    ),
    (
      RpcCommandKind::ExportHtml { output_path: None },
      "export_html",
    ),
    (
      RpcCommandKind::SwitchSession {
        session_path: "session.json".into(),
      },
      "switch_session",
    ),
    (
      RpcCommandKind::Fork {
        entry_id: "entry".into(),
      },
      "fork",
    ),
    (
      RpcCommandKind::SetSessionName {
        name: "name".into(),
      },
      "set_session_name",
    ),
  ];

  for (kind, expected) in cases {
    assert_eq!(kind.as_ref(), expected);
    assert_eq!(kind.to_string(), expected);
  }
}

#[test]
fn small_enum_wire_names() {
  let cases = [
    (StreamingBehavior::Steer.as_ref(), "steer"),
    (StreamingBehavior::FollowUp.as_ref(), "followUp"),
    (QueueMode::OneAtATime.as_ref(), "one-at-a-time"),
    (SlashCommandSource::Extension.as_ref(), "extension"),
    (NotifyType::Warning.as_ref(), "warning"),
    (WidgetPlacement::AboveEditor.as_ref(), "aboveEditor"),
    (ThinkingLevel::XHigh.as_ref(), "xhigh"),
    (CompactionReason::Threshold.as_ref(), "threshold"),
    (TextSignaturePhase::FinalAnswer.as_ref(), "final_answer"),
    (StopReason::ToolUse.as_ref(), "toolUse"),
    (SourceScope::Temporary.as_ref(), "temporary"),
    (SourceOrigin::TopLevel.as_ref(), "top-level"),
    (
      DeserializationErrorContext::RpcExtensionUIRequest.as_ref(),
      "rpc_extension_ui_request",
    ),
  ];

  for (actual, expected) in cases {
    assert_eq!(actual, expected);
  }

  assert_eq!(QueueMode::OneAtATime.to_string(), "one-at-a-time");
  assert_eq!(ThinkingLevel::XHigh.to_string(), "xhigh");
}

#[test]
fn tagged_enum_wire_names() {
  let content = ContentBlock::ToolCall {
    id: "id".into(),
    name: "tool".into(),
    arguments: Default::default(),
    thought_signature: None,
  };
  assert_eq!(content.as_ref(), "toolCall");
  assert_eq!(content.to_string(), "toolCall");

  let assistant_event = AssistantMessageEvent::ToolcallStart {
    content_index: 0.0,
    partial: Box::new(serde_json::json!({})),
  };
  assert_eq!(assistant_event.as_ref(), "toolcall_start");

  let agent_message = AgentMessage::Custom {
    custom_type: "x".into(),
    content: serde_json::json!(null),
    display: false,
    details: None,
    timestamp: 0.0,
  };
  assert_eq!(agent_message.as_ref(), "custom");

  let agent_event = AgentEvent::AutoRetryStart {
    attempt: 1.0,
    max_attempts: 3.0,
    delay_ms: 100.0,
    error_message: "err".into(),
  };
  assert_eq!(agent_event.as_ref(), "auto_retry_start");

  let session_event = SessionEvent::ProcessExited {
    code: Some(0),
    stderr: String::new(),
  };
  assert_eq!(session_event.as_ref(), "session_process_exited");

  let extension_request = RpcExtensionUIRequestKind::SetEditorText {
    text: "hello".into(),
  };
  assert_eq!(extension_request.as_ref(), "set_editor_text");
  assert_eq!(extension_request.to_string(), "set_editor_text");
}

// ============================================================================
// RpcResponse deserialization
// ============================================================================

#[test]
fn response_success_no_data() {
  let json = r#"{"type":"response","id":"1","command":"prompt","success":true}"#;
  let resp: RpcResponse = serde_json::from_str(json).unwrap();
  assert_eq!(resp.id, Some("1".into()));
  assert_eq!(resp.kind, RpcResponseKind::Prompt);
}

#[test]
fn response_kind_command_name() {
  assert_eq!(RpcResponseKind::Prompt.command_name(), "prompt");
  assert_eq!(
    RpcResponseKind::CycleThinkingLevel(None).command_name(),
    "cycle_thinking_level"
  );
  assert_eq!(
    RpcResponseKind::Error {
      command: "custom_command".into(),
      error: "err".into(),
    }
    .command_name(),
    "custom_command"
  );
}

#[test]
fn response_success_no_data_all_commands() {
  let no_data_commands = [
    ("prompt", RpcResponseKind::Prompt),
    ("steer", RpcResponseKind::Steer),
    ("follow_up", RpcResponseKind::FollowUp),
    ("abort", RpcResponseKind::Abort),
    ("set_thinking_level", RpcResponseKind::SetThinkingLevel),
    ("set_steering_mode", RpcResponseKind::SetSteeringMode),
    ("set_follow_up_mode", RpcResponseKind::SetFollowUpMode),
    ("set_auto_compaction", RpcResponseKind::SetAutoCompaction),
    ("set_auto_retry", RpcResponseKind::SetAutoRetry),
    ("abort_retry", RpcResponseKind::AbortRetry),
    ("abort_bash", RpcResponseKind::AbortBash),
    ("set_session_name", RpcResponseKind::SetSessionName),
  ];

  for (cmd_name, expected_kind) in no_data_commands {
    let json = format!(
      r#"{{"type":"response","id":"1","command":"{}","success":true}}"#,
      cmd_name
    );
    let resp: RpcResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(resp.kind, expected_kind, "failed for command {}", cmd_name);
  }
}

#[test]
fn response_error() {
  let json =
    r#"{"type":"response","id":"2","command":"prompt","success":false,"error":"not ready"}"#;
  let resp: RpcResponse = serde_json::from_str(json).unwrap();
  assert!(
    matches!(resp.kind, RpcResponseKind::Error { ref command, ref error }
        if command == "prompt" && error == "not ready")
  );
}

#[test]
fn response_get_state() {
  let json = r#"{
        "type": "response",
        "id": "3",
        "command": "get_state",
        "success": true,
        "data": {
            "model": {
                "id": "claude-sonnet-4-20250514",
                "name": "Claude Sonnet 4",
                "api": "anthropic",
                "provider": "anthropic",
                "baseUrl": "https://api.anthropic.com",
                "reasoning": true,
                "input": ["text", "image"],
                "cost": {"input": 3.0, "output": 15.0, "cacheRead": 0.3, "cacheWrite": 3.75},
                "contextWindow": 200000,
                "maxTokens": 16384
            },
            "thinkingLevel": "medium",
            "isStreaming": false,
            "isCompacting": false,
            "steeringMode": "all",
            "followUpMode": "all",
            "sessionFile": "/home/user/.pi/sessions/abc.json",
            "sessionId": "abc-123",
            "sessionName": "test session",
            "autoCompactionEnabled": true,
            "messageCount": 5,
            "pendingMessageCount": 0
        }
    }"#;
  let resp: RpcResponse = serde_json::from_str(json).unwrap();
  if let RpcResponseKind::GetState(state) = &resp.kind {
    assert_eq!(state.session_id, "abc-123");
    assert_eq!(state.thinking_level, ThinkingLevel::Medium);
    assert!(!state.is_streaming);
    assert!(state.auto_compaction_enabled);
    assert_eq!(state.message_count, 5.0);
    let model = state.model.as_ref().unwrap();
    assert_eq!(model.id, "claude-sonnet-4-20250514");
    assert!(model.reasoning);
  } else {
    panic!("Expected GetState, got {:?}", resp.kind);
  }
}

#[test]
fn response_get_state_roundtrip() {
  let json = r#"{
        "type": "response",
        "id": "3",
        "command": "get_state",
        "success": true,
        "data": {
            "model": null,
            "thinkingLevel": "off",
            "isStreaming": false,
            "isCompacting": false,
            "steeringMode": "all",
            "followUpMode": "one-at-a-time",
            "sessionId": "xyz",
            "autoCompactionEnabled": false,
            "messageCount": 0,
            "pendingMessageCount": 0
        }
    }"#;
  let resp: RpcResponse = serde_json::from_str(json).unwrap();
  let serialized = serde_json::to_value(&resp).unwrap();
  assert_eq!(serialized["command"], "get_state");
  assert_eq!(serialized["success"], true);
  assert_eq!(serialized["data"]["sessionId"], "xyz");
}

#[test]
fn response_new_session() {
  let json = r#"{"type":"response","id":"4","command":"new_session","success":true,"data":{"cancelled":false}}"#;
  let resp: RpcResponse = serde_json::from_str(json).unwrap();
  if let RpcResponseKind::NewSession(data) = &resp.kind {
    assert!(!data.cancelled);
  } else {
    panic!("Expected NewSession");
  }
}

#[test]
fn response_bash() {
  let json = r#"{
        "type": "response",
        "id": "5",
        "command": "bash",
        "success": true,
        "data": {
            "output": "hello world\n",
            "exitCode": 0,
            "cancelled": false,
            "truncated": false
        }
    }"#;
  let resp: RpcResponse = serde_json::from_str(json).unwrap();
  if let RpcResponseKind::Bash(data) = &resp.kind {
    assert_eq!(data.output, "hello world\n");
    assert_eq!(data.exit_code, Some(0.0));
    assert!(!data.cancelled);
    assert!(!data.truncated);
  } else {
    panic!("Expected Bash");
  }
}

#[test]
fn response_get_available_models() {
  let json = r#"{
        "type": "response",
        "id": "6",
        "command": "get_available_models",
        "success": true,
        "data": {
            "models": [{
                "id": "test-model",
                "name": "Test Model",
                "api": "anthropic",
                "provider": "anthropic",
                "baseUrl": "https://api.anthropic.com",
                "reasoning": false,
                "input": ["text"],
                "cost": {"input": 1.0, "output": 5.0, "cacheRead": 0.1, "cacheWrite": 1.0},
                "contextWindow": 100000,
                "maxTokens": 4096
            }]
        }
    }"#;
  let resp: RpcResponse = serde_json::from_str(json).unwrap();
  if let RpcResponseKind::GetAvailableModels(data) = &resp.kind {
    assert_eq!(data.models.len(), 1);
    assert_eq!(data.models[0].id, "test-model");
  } else {
    panic!("Expected GetAvailableModels");
  }
}

#[test]
fn response_cycle_model_null_data() {
  let json = r#"{"type":"response","id":"7","command":"cycle_model","success":true,"data":null}"#;
  let resp: RpcResponse = serde_json::from_str(json).unwrap();
  assert!(matches!(resp.kind, RpcResponseKind::CycleModel(None)));
}

#[test]
fn response_get_session_stats() {
  let json = r#"{
        "type": "response",
        "id": "8",
        "command": "get_session_stats",
        "success": true,
        "data": {
            "sessionFile": "/path/to/session.json",
            "sessionId": "sess-1",
            "userMessages": 3,
            "assistantMessages": 3,
            "toolCalls": 2,
            "toolResults": 2,
            "totalMessages": 10,
            "tokens": {
                "input": 1000,
                "output": 500,
                "cacheRead": 200,
                "cacheWrite": 100,
                "total": 1800
            },
            "cost": 0.05
        }
    }"#;
  let resp: RpcResponse = serde_json::from_str(json).unwrap();
  if let RpcResponseKind::GetSessionStats(stats) = &resp.kind {
    assert_eq!(stats.session_id, "sess-1");
    assert_eq!(stats.user_messages, 3.0);
    assert_eq!(stats.cost, 0.05);
    assert_eq!(stats.tokens.total, 1800.0);
  } else {
    panic!("Expected GetSessionStats");
  }
}

#[test]
fn response_clone() {
  let json =
    r#"{"type":"response","id":"9","command":"clone","success":true,"data":{"cancelled":false}}"#;
  let resp: RpcResponse = serde_json::from_str(json).unwrap();
  if let RpcResponseKind::Clone(data) = &resp.kind {
    assert!(!data.cancelled);
  } else {
    panic!("Expected Clone");
  }
}

#[test]
fn response_export_html() {
  let json = r#"{"type":"response","id":"9","command":"export_html","success":true,"data":{"path":"/tmp/export.html"}}"#;
  let resp: RpcResponse = serde_json::from_str(json).unwrap();
  if let RpcResponseKind::ExportHtml(data) = &resp.kind {
    assert_eq!(data.path, "/tmp/export.html");
  } else {
    panic!("Expected ExportHtml");
  }
}

#[test]
fn response_get_commands() {
  let json = r#"{
        "type": "response",
        "id": "10",
        "command": "get_commands",
        "success": true,
        "data": {
            "commands": [{
                "name": "test",
                "description": "A test command",
                "source": "extension",
                "sourceInfo": {
                    "path": "/home/user/.pi/extensions/test",
                    "source": "test-extension",
                    "scope": "user",
                    "origin": "top-level"
                }
            }]
        }
    }"#;
  let resp: RpcResponse = serde_json::from_str(json).unwrap();
  if let RpcResponseKind::GetCommands(data) = &resp.kind {
    assert_eq!(data.commands.len(), 1);
    assert_eq!(data.commands[0].name, "test");
    assert_eq!(data.commands[0].source, SlashCommandSource::Extension);
    assert_eq!(data.commands[0].source_info.scope, SourceScope::User);
    assert_eq!(
      data.commands[0].source_info.path,
      "/home/user/.pi/extensions/test"
    );
  } else {
    panic!("Expected GetCommands");
  }
}

#[test]
fn response_get_messages() {
  let json = r#"{
        "type": "response",
        "id": "11",
        "command": "get_messages",
        "success": true,
        "data": {
            "messages": [
                {"role": "user", "content": "hello", "timestamp": 1000.0},
                {"role": "assistant", "content": [{"type": "text", "text": "hi there"}], "api": "anthropic", "provider": "anthropic", "model": "claude-sonnet-4-20250514", "usage": {"input": 10, "output": 5, "cacheRead": 0, "cacheWrite": 0, "totalTokens": 15, "cost": {"input": 0.01, "output": 0.005, "cacheRead": 0, "cacheWrite": 0, "total": 0.015}}, "stopReason": "stop", "timestamp": 1001.0}
            ]
        }
    }"#;
  let resp: RpcResponse = serde_json::from_str(json).unwrap();
  if let RpcResponseKind::GetMessages(data) = &resp.kind {
    assert_eq!(data.messages.len(), 2);
    assert!(matches!(&data.messages[0], AgentMessage::User { .. }));
    assert!(matches!(&data.messages[1], AgentMessage::Assistant { .. }));
  } else {
    panic!("Expected GetMessages");
  }
}

#[test]
fn response_get_last_assistant_text() {
  let json = r#"{"type":"response","id":"12","command":"get_last_assistant_text","success":true,"data":{"text":"some response"}}"#;
  let resp: RpcResponse = serde_json::from_str(json).unwrap();
  if let RpcResponseKind::GetLastAssistantText(data) = &resp.kind {
    assert_eq!(data.text, Some("some response".into()));
  } else {
    panic!("Expected GetLastAssistantText");
  }
}

#[test]
fn response_get_last_assistant_text_null() {
  let json = r#"{"type":"response","id":"13","command":"get_last_assistant_text","success":true,"data":{"text":null}}"#;
  let resp: RpcResponse = serde_json::from_str(json).unwrap();
  if let RpcResponseKind::GetLastAssistantText(data) = &resp.kind {
    assert_eq!(data.text, None);
  } else {
    panic!("Expected GetLastAssistantText");
  }
}

#[test]
fn response_fork() {
  let json = r#"{"type":"response","id":"14","command":"fork","success":true,"data":{"text":"forked","cancelled":false}}"#;
  let resp: RpcResponse = serde_json::from_str(json).unwrap();
  if let RpcResponseKind::Fork(data) = &resp.kind {
    assert_eq!(data.text, "forked");
    assert!(!data.cancelled);
  } else {
    panic!("Expected Fork");
  }
}

#[test]
fn response_get_fork_messages() {
  let json = r#"{"type":"response","id":"15","command":"get_fork_messages","success":true,"data":{"messages":[{"entryId":"e1","text":"msg1"}]}}"#;
  let resp: RpcResponse = serde_json::from_str(json).unwrap();
  if let RpcResponseKind::GetForkMessages(data) = &resp.kind {
    assert_eq!(data.messages.len(), 1);
    assert_eq!(data.messages[0].entry_id, "e1");
  } else {
    panic!("Expected GetForkMessages");
  }
}

#[test]
fn response_switch_session() {
  let json = r#"{"type":"response","id":"16","command":"switch_session","success":true,"data":{"cancelled":true}}"#;
  let resp: RpcResponse = serde_json::from_str(json).unwrap();
  if let RpcResponseKind::SwitchSession(data) = &resp.kind {
    assert!(data.cancelled);
  } else {
    panic!("Expected SwitchSession");
  }
}

#[test]
fn response_cycle_thinking_level() {
  let json = r#"{"type":"response","id":"17","command":"cycle_thinking_level","success":true,"data":{"level":"high"}}"#;
  let resp: RpcResponse = serde_json::from_str(json).unwrap();
  if let RpcResponseKind::CycleThinkingLevel(Some(data)) = &resp.kind {
    assert_eq!(data.level, ThinkingLevel::High);
  } else {
    panic!("Expected CycleThinkingLevel with data");
  }
}

#[test]
fn response_compact() {
  let json = r#"{
        "type": "response",
        "id": "18",
        "command": "compact",
        "success": true,
        "data": {
            "summary": "summarized",
            "firstKeptEntryId": "entry-5",
            "tokensBefore": 50000
        }
    }"#;
  let resp: RpcResponse = serde_json::from_str(json).unwrap();
  if let RpcResponseKind::Compact(data) = &resp.kind {
    assert_eq!(data.summary, "summarized");
    assert_eq!(data.first_kept_entry_id, "entry-5");
    assert_eq!(data.tokens_before, 50000.0);
  } else {
    panic!("Expected Compact");
  }
}

// ============================================================================
// RpcResponse round-trip (serialize then deserialize)
// ============================================================================

#[test]
fn response_roundtrip_no_data() {
  let resp = RpcResponse {
    id: Some("rt1".into()),
    kind: RpcResponseKind::Abort,
  };
  let json = serde_json::to_string(&resp).unwrap();
  let back: RpcResponse = serde_json::from_str(&json).unwrap();
  assert_eq!(resp, back);
}

#[test]
fn response_roundtrip_error() {
  let resp = RpcResponse {
    id: Some("rt2".into()),
    kind: RpcResponseKind::Error {
      command: "bash".into(),
      error: "failed".into(),
    },
  };
  let json = serde_json::to_string(&resp).unwrap();
  let back: RpcResponse = serde_json::from_str(&json).unwrap();
  assert_eq!(resp, back);
}

#[test]
fn response_roundtrip_new_session() {
  let resp = RpcResponse {
    id: Some("rt3".into()),
    kind: RpcResponseKind::NewSession(NewSessionData { cancelled: false }),
  };
  let json = serde_json::to_string(&resp).unwrap();
  let back: RpcResponse = serde_json::from_str(&json).unwrap();
  assert_eq!(resp, back);
}

#[test]
fn response_roundtrip_bash() {
  let resp = RpcResponse {
    id: Some("rt4".into()),
    kind: RpcResponseKind::Bash(BashResult {
      output: "ok\n".into(),
      exit_code: Some(0.0),
      cancelled: false,
      truncated: false,
      full_output_path: None,
    }),
  };
  let json = serde_json::to_string(&resp).unwrap();
  let back: RpcResponse = serde_json::from_str(&json).unwrap();
  assert_eq!(resp, back);
}

// ============================================================================
// AgentEvent deserialization
// ============================================================================

#[test]
fn event_agent_start() {
  let json = r#"{"type":"agent_start"}"#;
  let event: AgentEvent = serde_json::from_str(json).unwrap();
  assert!(matches!(event, AgentEvent::AgentStart));
}

#[test]
fn event_turn_start() {
  let json = r#"{"type":"turn_start"}"#;
  let event: AgentEvent = serde_json::from_str(json).unwrap();
  assert!(matches!(event, AgentEvent::TurnStart));
}

#[test]
fn event_message_start_user() {
  let json =
    r#"{"type":"message_start","message":{"role":"user","content":"hello","timestamp":1000.0}}"#;
  let event: AgentEvent = serde_json::from_str(json).unwrap();
  if let AgentEvent::MessageStart {
    message: AgentMessage::User { content, .. },
  } = &event
  {
    assert!(matches!(content, UserContent::Text(t) if t == "hello"));
  } else {
    panic!("Expected MessageStart with User message");
  }
}

#[test]
fn event_message_update_text_delta() {
  let json = r#"{
        "type": "message_update",
        "message": {"role": "assistant", "content": [{"type": "text", "text": "h"}], "api": "anthropic", "provider": "anthropic", "model": "test", "usage": {"input": 0, "output": 0, "cacheRead": 0, "cacheWrite": 0, "totalTokens": 0, "cost": {"input": 0, "output": 0, "cacheRead": 0, "cacheWrite": 0, "total": 0}}, "stopReason": "stop", "timestamp": 1000.0},
        "assistantMessageEvent": {"type": "text_delta", "contentIndex": 0, "delta": "h", "partial": {}}
    }"#;
  let event: AgentEvent = serde_json::from_str(json).unwrap();
  if let AgentEvent::MessageUpdate {
    assistant_message_event: AssistantMessageEvent::TextDelta { delta, .. },
    ..
  } = &event
  {
    assert_eq!(delta, "h");
  } else {
    panic!("Expected MessageUpdate with text_delta");
  }
}

#[test]
fn event_agent_end() {
  let json = r#"{"type":"agent_end","messages":[]}"#;
  let event: AgentEvent = serde_json::from_str(json).unwrap();
  if let AgentEvent::AgentEnd { messages } = &event {
    assert!(messages.is_empty());
  } else {
    panic!("Expected AgentEnd");
  }
}

#[test]
fn event_tool_execution() {
  let json = r#"{"type":"tool_execution_start","toolCallId":"tc1","toolName":"bash","args":{"command":"ls"}}"#;
  let event: AgentEvent = serde_json::from_str(json).unwrap();
  if let AgentEvent::ToolExecutionStart {
    tool_call_id,
    tool_name,
    ..
  } = &event
  {
    assert_eq!(tool_call_id, "tc1");
    assert_eq!(tool_name, "bash");
  } else {
    panic!("Expected ToolExecutionStart");
  }
}

#[test]
fn event_compaction_start() {
  let json = r#"{"type":"compaction_start","reason":"threshold"}"#;
  let event: AgentEvent = serde_json::from_str(json).unwrap();
  assert!(matches!(
    event,
    AgentEvent::CompactionStart {
      reason: CompactionReason::Threshold
    }
  ));
}

// ============================================================================
// RpcEvent deserialization
// ============================================================================

#[test]
fn rpc_event_agent() {
  let json = r#"{"type":"agent_start"}"#;
  let event: RpcEvent = serde_json::from_str(json).unwrap();
  assert!(matches!(event, RpcEvent::Agent(AgentEvent::AgentStart)));
}

#[test]
fn rpc_event_extension_ui() {
  let json = r#"{"type":"extension_ui_request","id":"req1","method":"confirm","title":"Delete?","message":"Are you sure?"}"#;
  let event: RpcEvent = serde_json::from_str(json).unwrap();
  if let RpcEvent::ExtensionUI(req) = &event {
    assert_eq!(req.id, "req1");
    assert!(matches!(
        &req.kind,
        RpcExtensionUIRequestKind::Confirm { title, message, .. }
        if title == "Delete?" && message == "Are you sure?"
    ));
  } else {
    panic!("Expected ExtensionUI");
  }
}

#[test]
fn rpc_event_session_process_exited() {
  let json = r#"{"type":"session_process_exited","code":1,"stderr":"boom"}"#;
  let event: RpcEvent = serde_json::from_str(json).unwrap();
  assert!(matches!(
    event,
    RpcEvent::Session(SessionEvent::ProcessExited {
      code: Some(1),
      ref stderr,
    }) if stderr == "boom"
  ));

  let serialized = serde_json::to_value(&event).unwrap();
  let expected: serde_json::Value = serde_json::from_str(json).unwrap();
  assert_eq!(serialized, expected);
}

#[test]
fn rpc_event_session_deserialization_error() {
  let json = r#"{"type":"session_deserialization_error","context":"json_line","error":{"message":"expected value","line":1,"column":1,"category":"Syntax"},"line":"not json"}"#;
  let event: RpcEvent = serde_json::from_str(json).unwrap();
  assert!(matches!(
    event,
    RpcEvent::Session(SessionEvent::DeserializationError {
      context: DeserializationErrorContext::JsonLine,
      ref line,
      ..
    }) if line.as_deref() == Some("not json")
  ));

  let serialized = serde_json::to_value(&event).unwrap();
  let expected: serde_json::Value = serde_json::from_str(json).unwrap();
  assert_eq!(serialized, expected);
}

// ============================================================================
// Extension UI types
// ============================================================================

#[test]
fn extension_ui_request_select() {
  let json = r#"{"type":"extension_ui_request","id":"r1","method":"select","title":"Pick one","options":["a","b","c"],"timeout":30}"#;
  let req: RpcExtensionUIRequest = serde_json::from_str(json).unwrap();
  if let RpcExtensionUIRequestKind::Select {
    title,
    options,
    timeout,
  } = &req.kind
  {
    assert_eq!(title, "Pick one");
    assert_eq!(options, &["a", "b", "c"]);
    assert_eq!(*timeout, Some(30.0));
  } else {
    panic!("Expected Select");
  }
}

#[test]
fn extension_ui_request_input() {
  let json = r#"{"type":"extension_ui_request","id":"r2","method":"input","title":"Enter name","placeholder":"Name..."}"#;
  let req: RpcExtensionUIRequest = serde_json::from_str(json).unwrap();
  assert!(matches!(
      &req.kind,
      RpcExtensionUIRequestKind::Input { title, placeholder, .. }
      if title == "Enter name" && placeholder.as_deref() == Some("Name...")
  ));
}

#[test]
fn extension_ui_request_notify() {
  let json = r#"{"type":"extension_ui_request","id":"r3","method":"notify","message":"Done!","notifyType":"info"}"#;
  let req: RpcExtensionUIRequest = serde_json::from_str(json).unwrap();
  assert!(matches!(
      &req.kind,
      RpcExtensionUIRequestKind::Notify { message, notify_type }
      if message == "Done!" && *notify_type == Some(NotifyType::Info)
  ));
}

#[test]
fn extension_ui_request_set_status() {
  let json = r#"{"type":"extension_ui_request","id":"r4","method":"setStatus","statusKey":"build","statusText":"Building..."}"#;
  let req: RpcExtensionUIRequest = serde_json::from_str(json).unwrap();
  assert!(matches!(
      &req.kind,
      RpcExtensionUIRequestKind::SetStatus { status_key, status_text }
      if status_key == "build" && status_text.as_deref() == Some("Building...")
  ));
}

#[test]
fn extension_ui_response_value_roundtrip() {
  let resp = RpcExtensionUIResponse::Value {
    type_: ExtensionUIResponseType,
    id: "r1".into(),
    value: "option_a".into(),
  };
  let json = serde_json::to_string(&resp).unwrap();
  assert!(json.contains("extension_ui_response"));
  let back: RpcExtensionUIResponse = serde_json::from_str(&json).unwrap();
  assert_eq!(resp, back);
}

#[test]
fn extension_ui_response_confirmed_roundtrip() {
  let resp = RpcExtensionUIResponse::Confirmed {
    type_: ExtensionUIResponseType,
    id: "r2".into(),
    confirmed: true,
  };
  let json = serde_json::to_string(&resp).unwrap();
  let back: RpcExtensionUIResponse = serde_json::from_str(&json).unwrap();
  assert_eq!(resp, back);
}

#[test]
fn extension_ui_response_cancelled_roundtrip() {
  let resp = RpcExtensionUIResponse::Cancelled {
    type_: ExtensionUIResponseType,
    id: "r3".into(),
    cancelled: true,
  };
  let json = serde_json::to_string(&resp).unwrap();
  let back: RpcExtensionUIResponse = serde_json::from_str(&json).unwrap();
  assert_eq!(resp, back);
}

// ============================================================================
// ContentBlock types
// ============================================================================

#[test]
fn content_block_text_roundtrip() {
  let block = ContentBlock::Text {
    text: "hello".into(),
    text_signature: None,
  };
  let json = serde_json::to_string(&block).unwrap();
  let back: ContentBlock = serde_json::from_str(&json).unwrap();
  assert_eq!(block, back);
}

#[test]
fn content_block_thinking_roundtrip() {
  let block = ContentBlock::Thinking {
    thinking: "hmm".into(),
    thinking_signature: Some("sig".into()),
    redacted: Some(false),
  };
  let json = serde_json::to_string(&block).unwrap();
  let back: ContentBlock = serde_json::from_str(&json).unwrap();
  assert_eq!(block, back);
}

#[test]
fn content_block_image() {
  let json = r#"{"type":"image","data":"base64data","mimeType":"image/png"}"#;
  let block: ContentBlock = serde_json::from_str(json).unwrap();
  assert!(
    matches!(block, ContentBlock::Image { ref data, ref mime_type }
        if data == "base64data" && mime_type == "image/png")
  );
}

#[test]
fn content_block_tool_call() {
  let json = r#"{"type":"toolCall","id":"tc1","name":"bash","arguments":{"command":"ls"}}"#;
  let block: ContentBlock = serde_json::from_str(json).unwrap();
  if let ContentBlock::ToolCall {
    id,
    name,
    arguments,
    ..
  } = &block
  {
    assert_eq!(id, "tc1");
    assert_eq!(name, "bash");
    assert_eq!(arguments["command"], "ls");
  } else {
    panic!("Expected ToolCall");
  }
}

// ============================================================================
// AgentMessage types
// ============================================================================

#[test]
fn agent_message_bash_execution() {
  let json = r#"{
        "role": "bashExecution",
        "command": "echo hi",
        "output": "hi\n",
        "exitCode": 0,
        "cancelled": false,
        "truncated": false,
        "timestamp": 1000.0
    }"#;
  let msg: AgentMessage = serde_json::from_str(json).unwrap();
  if let AgentMessage::BashExecution {
    command,
    output,
    exit_code,
    ..
  } = &msg
  {
    assert_eq!(command, "echo hi");
    assert_eq!(output, "hi\n");
    assert_eq!(*exit_code, Some(0.0));
  } else {
    panic!("Expected BashExecution");
  }
}

#[test]
fn agent_message_compaction_summary() {
  let json = r#"{
        "role": "compactionSummary",
        "summary": "Context was compacted",
        "fromId": "entry-1",
        "tokensBefore": 50000,
        "timestamp": 2000.0
    }"#;
  let msg: AgentMessage = serde_json::from_str(json).unwrap();
  assert!(matches!(msg, AgentMessage::CompactionSummary { .. }));
}

#[test]
fn agent_message_tool_result() {
  let json = r#"{
        "role": "toolResult",
        "toolCallId": "tc1",
        "toolName": "bash",
        "content": [{"type": "text", "text": "output"}],
        "isError": false,
        "timestamp": 3000.0
    }"#;
  let msg: AgentMessage = serde_json::from_str(json).unwrap();
  if let AgentMessage::ToolResult {
    tool_call_id,
    tool_name,
    is_error,
    ..
  } = &msg
  {
    assert_eq!(tool_call_id, "tc1");
    assert_eq!(tool_name, "bash");
    assert!(!is_error);
  } else {
    panic!("Expected ToolResult");
  }
}

// ============================================================================
// ThinkingLevel enum
// ============================================================================

#[test]
fn thinking_level_all_variants() {
  let levels = [
    ("\"off\"", ThinkingLevel::Off),
    ("\"minimal\"", ThinkingLevel::Minimal),
    ("\"low\"", ThinkingLevel::Low),
    ("\"medium\"", ThinkingLevel::Medium),
    ("\"high\"", ThinkingLevel::High),
    ("\"xhigh\"", ThinkingLevel::XHigh),
  ];
  for (json, expected) in levels {
    let parsed: ThinkingLevel = serde_json::from_str(json).unwrap();
    assert_eq!(parsed, expected);
    let serialized = serde_json::to_string(&parsed).unwrap();
    assert_eq!(serialized, json);
  }
}
