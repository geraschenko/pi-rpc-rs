//! Debug tool: spawns a pi RPC session and prints everything we see.
//!
//! Usage:
//!   cargo run --bin pi-rpc-debug
//!   cargo run --bin pi-rpc-debug -- --prompt "say hello"
//!   cargo run --bin pi-rpc-debug -- --raw-json --prompt "say hello"

use std::time::Duration;

use clap::Parser;
use pi_rpc_rs::session::{PiSession, PiSessionConfig, SessionPersistence};
use pi_rpc_rs::types::*;
use tokio::time::timeout;

#[derive(Parser)]
#[command(name = "pi-rpc-debug", about = "Debug tool for pi RPC sessions")]
struct Args {
  /// Send this prompt after startup
  #[arg(long)]
  prompt: Option<String>,

  /// Print raw JSON instead of formatted output
  #[arg(long)]
  raw_json: bool,

  /// How long to wait after prompt completes or after startup (seconds)
  #[arg(long, default_value = "3")]
  wait: u64,
}

#[tokio::main]
async fn main() {
  let args = Args::parse();

  eprintln!("--- Spawning pi --mode rpc --no-session ---");

  let config = PiSessionConfig {
    session_persistence: SessionPersistence::Disabled,
    provider: Some("openai-codex".to_string()),
    model: Some("gpt-5.1".to_string()),
    ..Default::default()
  };

  let session = PiSession::spawn(config)
    .await
    .expect("Failed to spawn pi session");

  eprintln!("--- Session ready ---");

  let mut rx = session.subscribe().await;

  let raw_json = args.raw_json;

  // Spawn a task to print all events as they arrive
  let event_task = tokio::spawn(async move {
    let mut count = 0u64;
    while let Some(event) = rx.recv().await {
      count += 1;
      if raw_json {
        match serde_json::to_string(&event) {
          Ok(json) => println!("[event #{count}] {json}"),
          Err(e) => eprintln!("[event #{count}] <serialize error: {e}>"),
        }
      } else {
        match &event {
          RpcEvent::Agent(agent_event) => {
            eprintln!("[event #{count}] {}", format_agent_event(agent_event));
          }
          RpcEvent::ExtensionUI(req) => {
            eprintln!("[event #{count}] ExtensionUI id={} {:?}", req.id, req.kind);
          }
        }
      }
    }
    eprintln!("--- Event stream closed ({count} events total) ---");
  });

  eprintln!("\n--- Calling get_state() ---");
  match timeout(Duration::from_secs(1), session.get_state()).await {
    Ok(Ok(state)) => {
      if raw_json {
        println!("[get_state] {}", serde_json::to_string(&state).unwrap());
      } else {
        eprintln!("  session_id: {}", state.session_id);
        eprintln!("  model: {:?}", state.model.as_ref().map(|m| &m.name));
        eprintln!("  thinking_level: {:?}", state.thinking_level);
        eprintln!("  is_streaming: {}", state.is_streaming);
        eprintln!("  message_count: {}", state.message_count);
        eprintln!("  steering_mode: {:?}", state.steering_mode);
        eprintln!("  follow_up_mode: {:?}", state.follow_up_mode);
        eprintln!("  auto_compaction: {}", state.auto_compaction_enabled);
      }
    }
    Ok(Err(e)) => eprintln!("  ERROR: {e}"),
    Err(_) => eprintln!("  TIMEOUT"),
  }

  eprintln!("\n--- Calling get_commands() ---");
  match timeout(Duration::from_secs(1), session.get_commands()).await {
    Ok(Ok(GetCommandsData { commands })) => {
      if raw_json {
        println!(
          "[get_commands] {}",
          serde_json::to_string(&commands).unwrap()
        );
      } else {
        commands
          .iter()
          .for_each(|cmd| eprintln!("  {}: {:?}", cmd.name, cmd.description));
      }
    }
    Ok(Err(e)) => eprintln!("  ERROR: {e}"),
    Err(_) => eprintln!("  TIMEOUT"),
  }

  if let Some(prompt_text) = &args.prompt {
    eprintln!("\n--- Sending prompt: {prompt_text:?} ---");
    match session.prompt(prompt_text, None, None).await {
      Ok(()) => eprintln!("  prompt accepted"),
      Err(e) => eprintln!("  prompt ERROR: {e}"),
    }

    // Wait for agent_end
    eprintln!("--- Waiting for agent_end ---");
    // The event_task is printing events; we just wait here
    tokio::time::sleep(Duration::from_secs(args.wait)).await;
  } else {
    eprintln!("\n--- No --prompt given, observing for {}s ---", args.wait);
    tokio::time::sleep(Duration::from_secs(args.wait)).await;
  }

  eprintln!("\n--- Done, dropping session ---");
  drop(session);
  let _ = timeout(Duration::from_secs(2), event_task).await;
}

fn format_agent_event(event: &AgentEvent) -> String {
  match event {
    AgentEvent::AgentStart => "agent_start".into(),
    AgentEvent::AgentEnd { messages } => {
      format!("agent_end ({} messages)", messages.len())
    }
    AgentEvent::TurnStart => "turn_start".into(),
    AgentEvent::TurnEnd {
      message,
      tool_results,
    } => {
      format!(
        "turn_end (message={}, {} tool_results)",
        format_message_role(message),
        tool_results.len()
      )
    }
    AgentEvent::MessageStart { message } => {
      format!(
        "message_start [{}] {}",
        format_message_role(message),
        format_message_preview(message)
      )
    }
    AgentEvent::MessageUpdate {
      assistant_message_event,
      ..
    } => {
      format!(
        "message_update {}",
        format_assistant_event(assistant_message_event)
      )
    }
    AgentEvent::MessageEnd { message } => {
      format!(
        "message_end [{}] {}",
        format_message_role(message),
        format_message_preview(message)
      )
    }
    AgentEvent::ToolExecutionStart {
      tool_call_id,
      tool_name,
      args,
    } => {
      format!("tool_execution_start {tool_name} id={tool_call_id} args={args}")
    }
    AgentEvent::ToolExecutionUpdate {
      tool_name,
      partial_result,
      ..
    } => {
      let preview = partial_result.to_string();
      let preview = if preview.len() > 100 {
        format!("{}...", &preview[..100])
      } else {
        preview
      };
      format!("tool_execution_update {tool_name} partial={preview}")
    }
    AgentEvent::ToolExecutionEnd {
      tool_name,
      is_error,
      ..
    } => {
      format!("tool_execution_end {tool_name} is_error={is_error}")
    }
    AgentEvent::QueueUpdate {
      steering,
      follow_up,
    } => {
      format!(
        "queue_update steering={} follow_up={}",
        steering.len(),
        follow_up.len()
      )
    }
    AgentEvent::CompactionStart { reason } => {
      format!("compaction_start reason={reason:?}")
    }
    AgentEvent::SessionInfoChanged { name } => {
      format!("session_info_changed name={name:?}")
    }
    AgentEvent::ThinkingLevelChanged { level } => {
      format!("thinking_level_changed level={level:?}")
    }
    AgentEvent::CompactionEnd {
      reason,
      result,
      aborted,
      will_retry,
      error_message,
    } => {
      format!(
        "compaction_end reason={reason:?} aborted={aborted} will_retry={will_retry} error={error_message:?} result={:?}",
        result.as_ref().map(|r| &r.summary)
      )
    }
    AgentEvent::AutoRetryStart {
      attempt,
      max_attempts,
      delay_ms,
      error_message,
    } => {
      format!(
        "auto_retry_start attempt={attempt}/{max_attempts} delay={delay_ms}ms error={error_message}"
      )
    }
    AgentEvent::AutoRetryEnd {
      success,
      attempt,
      final_error,
    } => {
      format!("auto_retry_end success={success} attempt={attempt} error={final_error:?}")
    }
    AgentEvent::ExtensionError {
      extension_path,
      event,
      error,
    } => {
      format!("extension_error path={extension_path} event={event} error={error}")
    }
  }
}

fn format_message_role(msg: &AgentMessage) -> &'static str {
  match msg {
    AgentMessage::User { .. } => "user",
    AgentMessage::Assistant { .. } => "assistant",
    AgentMessage::ToolResult { .. } => "toolResult",
    AgentMessage::BashExecution { .. } => "bashExecution",
    AgentMessage::Custom { .. } => "custom",
    AgentMessage::BranchSummary { .. } => "branchSummary",
    AgentMessage::CompactionSummary { .. } => "compactionSummary",
  }
}

fn format_message_preview(msg: &AgentMessage) -> String {
  let text = match msg {
    AgentMessage::User { content, .. } => match content {
      UserContent::Text(t) => t.clone(),
      UserContent::Blocks(blocks) => blocks
        .iter()
        .filter_map(|b| match b {
          ContentBlock::Text { text, .. } => Some(text.as_str()),
          _ => None,
        })
        .collect::<Vec<_>>()
        .join(" "),
    },
    AgentMessage::Assistant { content, .. } => content
      .iter()
      .filter_map(|b| match b {
        ContentBlock::Text { text, .. } => Some(text.as_str()),
        ContentBlock::ToolCall { name, .. } => Some(name.as_str()),
        _ => None,
      })
      .collect::<Vec<_>>()
      .join(" | "),
    AgentMessage::ToolResult {
      tool_name,
      is_error,
      ..
    } => {
      format!("{tool_name} is_error={is_error}")
    }
    AgentMessage::BashExecution {
      command, exit_code, ..
    } => {
      format!("bash: {command} exit={exit_code:?}")
    }
    AgentMessage::Custom { custom_type, .. } => format!("custom:{custom_type}"),
    AgentMessage::BranchSummary { summary, .. } => summary.clone(),
    AgentMessage::CompactionSummary { summary, .. } => summary.clone(),
  };

  if text.len() > 120 {
    format!("{:.120}...", text)
  } else {
    text
  }
}

fn format_assistant_event(event: &AssistantMessageEvent) -> String {
  match event {
    AssistantMessageEvent::Start { .. } => "start".into(),
    AssistantMessageEvent::TextStart { content_index, .. } => {
      format!("text_start[{content_index}]")
    }
    AssistantMessageEvent::TextDelta {
      content_index,
      delta,
      ..
    } => {
      let preview = if delta.len() > 60 {
        format!("{:.60}...", delta)
      } else {
        delta.clone()
      };
      format!("text_delta[{content_index}] {preview:?}")
    }
    AssistantMessageEvent::TextEnd { content_index, .. } => {
      format!("text_end[{content_index}]")
    }
    AssistantMessageEvent::ThinkingStart { content_index, .. } => {
      format!("thinking_start[{content_index}]")
    }
    AssistantMessageEvent::ThinkingDelta {
      content_index,
      delta,
      ..
    } => {
      let preview = if delta.len() > 60 {
        format!("{:.60}...", delta)
      } else {
        delta.clone()
      };
      format!("thinking_delta[{content_index}] {preview:?}")
    }
    AssistantMessageEvent::ThinkingEnd { content_index, .. } => {
      format!("thinking_end[{content_index}]")
    }
    AssistantMessageEvent::ToolcallStart { content_index, .. } => {
      format!("toolcall_start[{content_index}]")
    }
    AssistantMessageEvent::ToolcallDelta {
      content_index,
      delta,
      ..
    } => {
      let preview = if delta.len() > 60 {
        format!("{:.60}...", delta)
      } else {
        delta.clone()
      };
      format!("toolcall_delta[{content_index}] {preview:?}")
    }
    AssistantMessageEvent::ToolcallEnd {
      content_index,
      tool_call,
      ..
    } => {
      let name = match tool_call {
        ContentBlock::ToolCall { name, .. } => name.as_str(),
        _ => "?",
      };
      format!("toolcall_end[{content_index}] {name}")
    }
    AssistantMessageEvent::Done { reason, .. } => format!("done reason={reason:?}"),
    AssistantMessageEvent::Error { reason, error, .. } => {
      format!("error reason={reason:?} {error}")
    }
  }
}
