use deer_pipeline_raw_sources::{run_hermes_smoke_prompt_with_text, AcpResponseStreamEventKind};

fn prompt_from_args() -> String {
    let prompt = std::env::args().skip(1).collect::<Vec<_>>().join(" ");
    if prompt.trim().is_empty() {
        "what are your skills?".to_string()
    } else {
        prompt
    }
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let prompt = prompt_from_args();
    match run_hermes_smoke_prompt_with_text(prompt.clone()).await {
        Ok(report) => {
            println!("PROMPT: {}", report.prompt);
            println!("ACP_SESSION: {}", report.acp_session_id);
            println!("RAW_SESSION: {}", report.raw_session_id);
            println!("RUN: {}", report.run_id);
            println!();

            println!("LIVE:");
            for event in &report.live_events {
                match &event.kind {
                    AcpResponseStreamEventKind::AssistantTextFinal { text } => {
                        println!("  assistant_text_final: {}", text);
                    }
                    AcpResponseStreamEventKind::AssistantThoughtStatusChunk { text } => {
                        println!("  assistant_thought_status_chunk: {}", text);
                    }
                    AcpResponseStreamEventKind::ToolCallStarted { tool_name } => {
                        println!("  tool_call_started: {}", tool_name);
                    }
                    AcpResponseStreamEventKind::ToolCallCompleted { tool_name } => {
                        println!("  tool_call_completed: {}", tool_name);
                    }
                    AcpResponseStreamEventKind::RunStarted => {
                        println!("  run_started");
                    }
                    AcpResponseStreamEventKind::RunCompleted => {
                        println!("  run_completed");
                    }
                    AcpResponseStreamEventKind::RunCancelled => {
                        println!("  run_cancelled");
                    }
                    AcpResponseStreamEventKind::StreamError { message } => {
                        println!("  stream_error: {}", message);
                    }
                }
            }

            println!();
            println!("ASSISTANT:");
            println!(
                "  {}",
                report
                    .assistant_text
                    .as_deref()
                    .unwrap_or("(no final assistant text observed)")
            );

            println!();
            println!("RAW:");
            for (session_id, sequence, bytes) in &report.raw_events {
                let text = String::from_utf8_lossy(bytes).trim().to_string();
                println!("  {} seq={} {}", session_id, sequence, text);
            }

            println!();
            println!("REPLAY:");
            for (index, bytes) in report.replayed_events.iter().enumerate() {
                let text = String::from_utf8_lossy(bytes).trim().to_string();
                println!("  [{}] {}", index, text);
            }
        }
        Err(error) => {
            eprintln!("Hermes smoke run failed: {}", error);
            std::process::exit(1);
        }
    }
}
