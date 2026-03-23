/// CapsLockX Agent — standalone CLI chat with LLM.
///
/// Reads questions from stdin, streams responses to stdout.
/// Maintains conversation history across turns.
///
/// Usage:
///   clx-agent                          # interactive mode
///   echo "explain this code" | clx-agent  # pipe mode
///
/// Environment:
///   GEMINI_API_KEY / OPENAI_API_KEY / ANTHROPIC_API_KEY
///   CLX_LLM_MODEL (optional, auto-detected from key)

use capslockx_core::llm_client::{LlmConfig, Message};
use capslockx_core::agent::agent_chat;
use std::io::{BufRead, Write};

const SYSTEM_PROMPT: &str = "\
You are CapsLockX Agent, a helpful assistant. \
Answer concisely. Use the same language as the user.";

fn main() {
    let api_key = std::env::var("GEMINI_API_KEY")
        .or_else(|_| std::env::var("OPENAI_API_KEY"))
        .or_else(|_| std::env::var("ANTHROPIC_API_KEY"))
        .or_else(|_| {
            // Try .env.local
            for dir in &[".", &format!("{}/CapsLockX", std::env::var("HOME").unwrap_or_default())] {
                let path = std::path::Path::new(dir).join(".env.local");
                if let Ok(content) = std::fs::read_to_string(path) {
                    for line in content.lines() {
                        for prefix in &["GEMINI_API_KEY=", "OPENAI_API_KEY=", "ANTHROPIC_API_KEY="] {
                            if let Some(val) = line.strip_prefix(prefix) {
                                return Ok(val.to_string());
                            }
                        }
                    }
                }
            }
            Err(std::env::VarError::NotPresent)
        })
        .expect("Set GEMINI_API_KEY, OPENAI_API_KEY, or ANTHROPIC_API_KEY");

    let model = std::env::var("CLX_LLM_MODEL").unwrap_or_default();
    let config = LlmConfig::from_key_and_model(&api_key, &model);

    eprintln!("CapsLockX Agent ({:?} / {})", config.provider, config.model);
    eprintln!("Type your message, press Enter to send. Ctrl+D to exit.\n");

    let mut history = vec![
        Message { role: "system".into(), content: SYSTEM_PROMPT.into(), image_base64: None },
    ];

    let stdin = std::io::stdin();
    let is_tty = atty_check();

    loop {
        if is_tty { eprint!("> "); std::io::stderr().flush().ok(); }

        let mut input = String::new();
        match stdin.lock().read_line(&mut input) {
            Ok(0) => break, // EOF
            Ok(_) => {}
            Err(e) => { eprintln!("read error: {}", e); break; }
        }

        let input = input.trim();
        if input.is_empty() { continue; }
        if input == "/clear" {
            history.truncate(1);
            eprintln!("(history cleared)");
            continue;
        }
        if input == "/history" {
            for (i, m) in history.iter().enumerate() {
                eprintln!("[{}] {}: {}...", i, m.role, m.content.chars().take(60).collect::<String>());
            }
            continue;
        }

        history.push(Message { role: "user".into(), content: input.to_string(), image_base64: None });

        match agent_chat(&config, &mut history, &mut |token| {
            print!("{}", token);
            std::io::stdout().flush().ok();
        }, &mut |status| {
            eprintln!("🔧 {}", status);
        }) {
            Ok(full) => {
                println!();
                // History already updated by agent_chat (includes tool calls).
                let _ = full; // final text already streamed via on_token

                // Trim if too long.
                if history.len() > 42 {
                    let system = history[0].clone();
                    let tail: Vec<Message> = history[history.len()-40..].to_vec();
                    history = vec![system];
                    history.extend(tail);
                }
            }
            Err(e) => {
                eprintln!("\nError: {}", e);
                history.pop(); // remove failed user message
            }
        }
    }
}

fn atty_check() -> bool {
    // Simple heuristic: if stdin is a terminal
    unsafe { libc::isatty(0) != 0 }
}
