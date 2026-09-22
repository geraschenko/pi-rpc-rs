//! Integration tests for PiSession.
//!
//! These tests require `pi` to be installed and configured with API credentials.
//! Run with: `cargo nextest run --run-ignored all`
//!
//! Each test uses private directories and a copy of the Codex OAuth credential.
//! Requires Unix, `pi` on PATH, and at least 30 minutes of token validity.

#![cfg(unix)]

mod support;

use std::time::Duration;

use pi_rpc_rs::types::*;
use support::TestSession;
use tokio::time::timeout;

/// Timeout for operations that don't involve LLM calls (state queries, settings, bash).
const FAST_TIMEOUT: Duration = Duration::from_secs(5);

/// Timeout for operations that involve a single LLM call.
const LLM_TIMEOUT: Duration = Duration::from_secs(15);

async fn spawn_test_session() -> TestSession {
  TestSession::spawn().await
}

/// Collect events until agent_end or timeout. Panics on timeout.
async fn collect_events_until_agent_end(
  rx: &mut tokio::sync::mpsc::UnboundedReceiver<RpcEvent>,
  deadline: Duration,
) -> Vec<RpcEvent> {
  let mut events = Vec::new();
  timeout(deadline, async {
    while let Some(event) = rx.recv().await {
      let is_end = matches!(&event, RpcEvent::Agent(AgentEvent::AgentEnd { .. }));
      events.push(event);
      if is_end {
        break;
      }
    }
  })
  .await
  .expect("timed out waiting for agent_end");
  events
}

/// Extract assistant text content from an AgentMessage, if any.
fn assistant_text(msg: &AgentMessage) -> Option<String> {
  if let AgentMessage::Assistant { content, .. } = msg {
    let text: String = content
      .iter()
      .filter_map(|b| match b {
        ContentBlock::Text { text, .. } => Some(text.as_str()),
        _ => None,
      })
      .collect::<Vec<_>>()
      .join("");
    if text.is_empty() { None } else { Some(text) }
  } else {
    None
  }
}

/// Check that an assistant message doesn't have an error.
fn assert_no_error<T: std::fmt::Debug + ?Sized>(msg: &AgentMessage, diagnostic: &T) {
  if let AgentMessage::Assistant {
    error_message,
    stop_reason,
    ..
  } = msg
  {
    assert!(
      error_message.is_none(),
      "assistant message has error: {:?} (stop_reason={:?})\nfull diagnostic context:\n{diagnostic:#?}",
      error_message,
      stop_reason
    );
    assert_ne!(
      *stop_reason,
      StopReason::Error,
      "assistant stop_reason is Error\nfull diagnostic context:\n{diagnostic:#?}"
    );
  }
}

// ============================================================================
// 1. Spawn and state
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_clear_queue() {
  let session = spawn_test_session().await;
  timeout(FAST_TIMEOUT, async {
    session.steer("queued steering", None).await.unwrap();
    session.follow_up("queued follow-up", None).await.unwrap();
    let cleared = session.clear_queue().await.unwrap();
    assert_eq!(cleared.steering, ["queued steering"]);
    assert_eq!(cleared.follow_up, ["queued follow-up"]);
    let empty = session.clear_queue().await.unwrap();
    assert!(empty.steering.is_empty());
    assert!(empty.follow_up.is_empty());
    assert_eq!(
      session.get_state().await.unwrap().pending_message_count,
      0.0
    );
  })
  .await
  .expect("clear_queue timed out");
  session.finish().await;
}

#[tokio::test]
#[ignore]
async fn test_spawn_and_get_state() {
  let session = spawn_test_session().await;

  let state = session.get_state().await.expect("get_state failed");

  assert!(
    !state.session_id.is_empty(),
    "session_id should not be empty"
  );
  assert!(!state.is_streaming, "should not be streaming initially");
  assert!(!state.is_compacting, "should not be compacting initially");
  assert_eq!(
    state.message_count, 0.0,
    "fresh session should have no messages"
  );

  let model = state.model.expect("model should be set");
  assert!(!model.id.is_empty());
  assert!(!model.name.is_empty());
  assert!(model.reasoning, "gpt-5.5 should support reasoning");

  eprintln!(
    "State: session_id={}, model={}, thinking_level={:?}",
    state.session_id, model.name, state.thinking_level
  );
  session.finish().await;
}

// ============================================================================
// 2. Prompt and events
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_prompt_and_events() {
  let session = spawn_test_session().await;
  let mut rx = session.subscribe().await;

  session
    .prompt("Reply with exactly: PONG", None, None)
    .await
    .expect("prompt failed");

  let events = collect_events_until_agent_end(&mut rx, LLM_TIMEOUT).await;

  for event in &events {
    if let RpcEvent::Agent(AgentEvent::MessageEnd { message }) = event {
      assert_no_error(message, &events);
    }
  }

  type EventPredicate = fn(&RpcEvent) -> bool;
  let expected_events: &[(&str, EventPredicate)] = &[
    ("agent_start", |event| {
      matches!(event, RpcEvent::Agent(AgentEvent::AgentStart))
    }),
    ("turn_start", |event| {
      matches!(event, RpcEvent::Agent(AgentEvent::TurnStart))
    }),
    ("user message_start", |event| {
      matches!(
        event,
        RpcEvent::Agent(AgentEvent::MessageStart {
          message: AgentMessage::User { .. }
        })
      )
    }),
    ("user message_end", |event| {
      matches!(
        event,
        RpcEvent::Agent(AgentEvent::MessageEnd {
          message: AgentMessage::User { .. }
        })
      )
    }),
    ("assistant message_start", |event| {
      matches!(
        event,
        RpcEvent::Agent(AgentEvent::MessageStart {
          message: AgentMessage::Assistant { .. }
        })
      )
    }),
    ("text_delta message_update", |event| {
      matches!(
        event,
        RpcEvent::Agent(AgentEvent::MessageUpdate {
          assistant_message_event: AssistantMessageEvent::TextDelta { .. },
          ..
        })
      )
    }),
    ("assistant message_end", |event| {
      matches!(
        event,
        RpcEvent::Agent(AgentEvent::MessageEnd {
          message: AgentMessage::Assistant { .. }
        })
      )
    }),
    ("turn_end", |event| {
      matches!(event, RpcEvent::Agent(AgentEvent::TurnEnd { .. }))
    }),
    ("agent_end", |event| {
      matches!(event, RpcEvent::Agent(AgentEvent::AgentEnd { .. }))
    }),
  ];

  let mut next_event_index = 0;
  for (expected_name, predicate) in expected_events {
    let relative_index = events[next_event_index..]
      .iter()
      .position(predicate)
      .unwrap_or_else(|| {
        panic!(
          "missing {expected_name} after event index {next_event_index}\nfull event stream:\n{events:#?}"
        )
      });
    next_event_index += relative_index + 1;
  }

  // Verify the final assistant message contains PONG
  let agent_end_messages = events
    .iter()
    .find_map(|e| match e {
      RpcEvent::Agent(AgentEvent::AgentEnd { messages, .. }) => Some(messages),
      _ => None,
    })
    .unwrap_or_else(|| panic!("missing agent_end\nfull event stream:\n{events:#?}"));

  let last_assistant = agent_end_messages
    .iter()
    .rev()
    .find(|m| matches!(m, AgentMessage::Assistant { .. }))
    .unwrap_or_else(|| {
      panic!("no assistant message in agent_end\nfull event stream:\n{events:#?}")
    });
  assert_no_error(last_assistant, &events);

  let text = assistant_text(last_assistant).unwrap_or_else(|| {
    panic!("assistant message had no text content\nfull event stream:\n{events:#?}")
  });
  assert!(
    text.contains("PONG"),
    "expected PONG in response, got: {text:?}\nfull event stream:\n{events:#?}"
  );
  session.finish().await;
}

// ============================================================================
// 3. Model management
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_get_available_models() {
  let session = spawn_test_session().await;

  let models = session
    .get_available_models()
    .await
    .expect("get_available_models failed");

  assert!(!models.models.is_empty(), "should have at least one model");
  eprintln!("Available models: {}", models.models.len());
  for m in &models.models {
    eprintln!("  {} (provider: {}, api: {})", m.name, m.provider, m.api);
  }
  session.finish().await;
}

#[tokio::test]
#[ignore]
async fn test_set_model() {
  let session = spawn_test_session().await;

  let models = session
    .get_available_models()
    .await
    .expect("get_available_models failed");
  assert!(!models.models.is_empty());

  let target = &models.models[0];
  let result = session
    .set_model(&target.provider, &target.id)
    .await
    .expect("set_model failed");

  assert_eq!(result.id, target.id);
  assert_eq!(result.provider, target.provider);
  eprintln!("Set model to: {} ({})", result.name, result.id);
  session.finish().await;
}

#[tokio::test]
#[ignore]
async fn test_cycle_model() {
  let session = spawn_test_session().await;

  let result = session.cycle_model().await.expect("cycle_model failed");

  if let Some(data) = result {
    eprintln!(
      "Cycled to model: {} (thinking: {:?})",
      data.model.name, data.thinking_level
    );
  } else {
    eprintln!("cycle_model returned None (possibly only one model available)");
  }
  session.finish().await;
}

// ============================================================================
// 4. Thinking level
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_set_thinking_level() {
  let session = spawn_test_session().await;

  // gpt-5.5 supports reasoning, so this should work
  session
    .set_thinking_level(ThinkingLevel::Medium)
    .await
    .expect("set_thinking_level to medium failed");

  let state = session.get_state().await.expect("get_state failed");
  assert_eq!(state.thinking_level, ThinkingLevel::Medium);

  session
    .set_thinking_level(ThinkingLevel::Off)
    .await
    .expect("set_thinking_level to off failed");

  let state = session.get_state().await.expect("get_state failed");
  assert_eq!(state.thinking_level, ThinkingLevel::Off);
  session.finish().await;
}

#[tokio::test]
#[ignore]
async fn test_cycle_thinking_level() {
  let session = spawn_test_session().await;

  let result = session
    .cycle_thinking_level()
    .await
    .expect("cycle_thinking_level failed");

  if let Some(data) = result {
    eprintln!("Cycled thinking level to: {:?}", data.level);
  } else {
    eprintln!("cycle_thinking_level returned None");
  }
  session.finish().await;
}

#[tokio::test]
#[ignore]
async fn test_get_available_thinking_levels() {
  let session = spawn_test_session().await;

  let data = session
    .get_available_thinking_levels()
    .await
    .expect("get_available_thinking_levels failed");

  assert!(!data.levels.is_empty());
  eprintln!("Available thinking levels: {:?}", data.levels);
  session.finish().await;
}

// ============================================================================
// 5. Session management
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_new_session() {
  let session = spawn_test_session().await;

  let result = session.new_session(None).await.expect("new_session failed");
  eprintln!("New session cancelled={}", result.cancelled);
  session.finish().await;
}

#[tokio::test]
#[ignore]
async fn test_set_session_name() {
  let session = spawn_test_session().await;

  session
    .set_session_name("test-session-name")
    .await
    .expect("set_session_name failed");

  let state = session.get_state().await.expect("get_state failed");
  assert_eq!(state.session_name.as_deref(), Some("test-session-name"));
  session.finish().await;
}

#[tokio::test]
#[ignore]
async fn test_get_session_stats() {
  let session = spawn_test_session().await;

  let stats = session
    .get_session_stats()
    .await
    .expect("get_session_stats failed");

  assert!(!stats.session_id.is_empty());
  assert_eq!(
    stats.user_messages, 0.0,
    "fresh session should have 0 user messages"
  );
  assert_eq!(stats.cost, 0.0, "fresh session should have 0 cost");
  eprintln!("Session stats: {:?}", stats);
  session.finish().await;
}

// ============================================================================
// 6. Bash execution
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_bash_echo() {
  let session = spawn_test_session().await;

  let result = session
    .bash("echo hello", false)
    .await
    .expect("bash failed");

  assert!(
    result.output.contains("hello"),
    "expected 'hello', got: {:?}",
    result.output
  );
  assert_eq!(result.exit_code, Some(0.0));
  assert!(!result.cancelled);
  assert!(!result.truncated);
  session.finish().await;
}

#[tokio::test]
#[ignore]
async fn test_bash_exit_code() {
  let session = spawn_test_session().await;

  let result = session.bash("exit 42", false).await.expect("bash failed");

  assert_eq!(result.exit_code, Some(42.0));
  session.finish().await;
}

// ============================================================================
// 7. Abort
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_abort() {
  let session = spawn_test_session().await;
  let mut rx = session.subscribe().await;

  session
    .prompt(
      "Write a very long essay about the history of computing, at least 5000 words.",
      None,
      None,
    )
    .await
    .expect("prompt failed");

  // Wait a moment for streaming to start, then abort
  tokio::time::sleep(Duration::from_millis(500)).await;
  session.abort().await.expect("abort failed");

  let events = collect_events_until_agent_end(&mut rx, FAST_TIMEOUT).await;

  let has_agent_end = events
    .iter()
    .any(|e| matches!(e, RpcEvent::Agent(AgentEvent::AgentEnd { .. })));
  assert!(has_agent_end, "should receive agent_end after abort");
  session.finish().await;
}

// ============================================================================
// 8. Steer
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_steer() {
  let session = spawn_test_session().await;
  let mut rx = session.subscribe().await;

  // Start a prompt that will finish quickly. Steer doesn't interrupt
  // the current API call — it queues a message for after the current
  // turn finishes. So we need the initial prompt to complete fast.
  session
    .prompt("Say exactly: FIRST", None, None)
    .await
    .expect("prompt failed");

  // Wait for the first response to start streaming
  tokio::time::sleep(Duration::from_millis(500)).await;

  // Steer with a new instruction (will be processed after FIRST completes)
  session
    .steer("Say exactly: SECOND", None)
    .await
    .expect("steer failed");

  // Both the initial response and the steer response should complete
  // within LLM_TIMEOUT (two short LLM calls)
  let events = collect_events_until_agent_end(&mut rx, LLM_TIMEOUT).await;

  let has_agent_end = events
    .iter()
    .any(|e| matches!(e, RpcEvent::Agent(AgentEvent::AgentEnd { .. })));
  assert!(has_agent_end, "should receive agent_end after steer");

  // The steer message should appear as a user message_start event
  let has_steer_message = events.iter().any(|e| {
    if let RpcEvent::Agent(AgentEvent::MessageStart {
      message: AgentMessage::User { content, .. },
    }) = e
    {
      match content {
        UserContent::Text(t) => t.contains("SECOND"),
        UserContent::Blocks(blocks) => blocks
          .iter()
          .any(|b| matches!(b, ContentBlock::Text { text, .. } if text.contains("SECOND"))),
      }
    } else {
      false
    }
  });

  assert!(
    has_steer_message,
    "steer message should appear as a user message_start event"
  );
  session.finish().await;
}

// ============================================================================
// 9. Error handling
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_set_model_invalid() {
  let session = spawn_test_session().await;

  let result = session
    .set_model("nonexistent_provider", "nonexistent_model")
    .await;

  assert!(
    result.is_err(),
    "set_model with invalid provider/model should fail"
  );
  if let Err(e) = &result {
    eprintln!("Expected error: {}", e);
    assert!(
      matches!(e, pi_rpc_rs::session::PiError::CommandFailed { .. }),
      "should be CommandFailed, got: {:?}",
      e
    );
  }
  session.finish().await;
}

// ============================================================================
// 10. Queue mode settings
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_set_queue_modes() {
  let session = spawn_test_session().await;

  session
    .set_steering_mode(QueueMode::OneAtATime)
    .await
    .expect("set_steering_mode failed");
  session
    .set_follow_up_mode(QueueMode::OneAtATime)
    .await
    .expect("set_follow_up_mode failed");

  let state = session.get_state().await.expect("get_state failed");
  assert_eq!(state.steering_mode, QueueMode::OneAtATime);
  assert_eq!(state.follow_up_mode, QueueMode::OneAtATime);

  session
    .set_steering_mode(QueueMode::All)
    .await
    .expect("set_steering_mode failed");
  session
    .set_follow_up_mode(QueueMode::All)
    .await
    .expect("set_follow_up_mode failed");

  let state = session.get_state().await.expect("get_state failed");
  assert_eq!(state.steering_mode, QueueMode::All);
  assert_eq!(state.follow_up_mode, QueueMode::All);
  session.finish().await;
}

// ============================================================================
// 11. Auto-compaction and auto-retry settings
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_auto_compaction_setting() {
  let session = spawn_test_session().await;

  session
    .set_auto_compaction(false)
    .await
    .expect("set_auto_compaction(false) failed");
  let state = session.get_state().await.expect("get_state failed");
  assert!(!state.auto_compaction_enabled);

  session
    .set_auto_compaction(true)
    .await
    .expect("set_auto_compaction(true) failed");
  let state = session.get_state().await.expect("get_state failed");
  assert!(state.auto_compaction_enabled);
  session.finish().await;
}

#[tokio::test]
#[ignore]
async fn test_auto_retry_setting() {
  let session = spawn_test_session().await;

  session
    .set_auto_retry(false)
    .await
    .expect("set_auto_retry(false) failed");
  session
    .set_auto_retry(true)
    .await
    .expect("set_auto_retry(true) failed");
  session.finish().await;
}

// ============================================================================
// 12. Get commands
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_get_commands() {
  let session = spawn_test_session().await;

  let result = session.get_commands().await.expect("get_commands failed");

  eprintln!("Available commands: {}", result.commands.len());
  for cmd in &result.commands {
    eprintln!("  /{} ({:?})", cmd.name, cmd.source);
  }
  session.finish().await;
}

// ============================================================================
// 13. Kill and cleanup
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_kill() {
  let mut session = spawn_test_session().await;

  session.kill().await.expect("kill failed");
  session.wait_closed().await;

  let result = session.get_state().await;
  assert!(result.is_err(), "get_state should fail after kill");
  session.finish().await;
}

// ============================================================================
// 14. Get messages / last assistant text
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_get_messages_empty() {
  let session = spawn_test_session().await;

  let result = session.get_messages().await.expect("get_messages failed");
  assert!(
    result.messages.is_empty(),
    "fresh session should have no messages"
  );
  session.finish().await;
}

#[tokio::test]
#[ignore]
async fn test_get_last_assistant_text_empty() {
  let session = spawn_test_session().await;

  let result = session
    .get_last_assistant_text()
    .await
    .expect("get_last_assistant_text failed");
  assert!(
    result.text.is_none(),
    "fresh session should have no last assistant text"
  );
  session.finish().await;
}

#[tokio::test]
#[ignore]
async fn test_get_messages_after_prompt() {
  let session = spawn_test_session().await;
  let mut rx = session.subscribe().await;

  session
    .prompt("Say exactly: TEST_REPLY", None, None)
    .await
    .expect("prompt failed");

  collect_events_until_agent_end(&mut rx, LLM_TIMEOUT).await;

  let messages = session.get_messages().await.expect("get_messages failed");
  assert!(
    messages.messages.len() >= 3,
    "should have system + user + assistant messages, got {}",
    messages.messages.len()
  );

  assert!(
    matches!(&messages.messages[0], AgentMessage::System { .. }),
    "first message should be system: {:#?}",
    messages.messages
  );
  assert!(
    matches!(
      messages
        .messages
        .iter()
        .find(|message| !matches!(message, AgentMessage::System { .. })),
      Some(AgentMessage::User { .. })
    ),
    "first non-system message should be user: {:#?}",
    messages.messages
  );

  // Verify assistant responded without error
  let last_assistant = messages
    .messages
    .iter()
    .rev()
    .find(|m| matches!(m, AgentMessage::Assistant { .. }))
    .expect("no assistant message found");
  assert_no_error(last_assistant, &messages.messages);
  let text = assistant_text(last_assistant).expect("assistant had no text content");
  assert!(
    text.contains("TEST_REPLY"),
    "expected TEST_REPLY in response, got: {text:?}"
  );

  let last_text = session
    .get_last_assistant_text()
    .await
    .expect("get_last_assistant_text failed");
  assert!(
    last_text.text.is_some(),
    "should have last assistant text after prompt"
  );
  eprintln!("Last assistant text: {:?}", last_text.text);
  session.finish().await;
}
