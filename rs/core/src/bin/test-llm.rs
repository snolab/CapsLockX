/// Quick LLM client test.
/// Usage: test-llm
use capslockx_core::llm_client::{LlmConfig, Message, stream_chat};

fn main() {
    // Read key from .env.local or env
    let key = std::env::var("GEMINI_API_KEY")
        .or_else(|_| {
            // Try .env.local
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap().join(".env.local");
            let content = std::fs::read_to_string(path).unwrap_or_default();
            content.lines()
                .find(|l| l.starts_with("GEMINI_API_KEY="))
                .map(|l| l.trim_start_matches("GEMINI_API_KEY=").to_string())
                .ok_or(std::env::VarError::NotPresent)
        })
        .expect("Set GEMINI_API_KEY or add to .env.local");

    let config = LlmConfig::from_key_and_model(&key, ""); // auto-detect
    println!("Provider: {:?}, Model: {}", config.provider, config.model);

    let messages = vec![
        Message { role: "system".into(), content: "You correct speech-to-text errors. Return ONLY corrected text.".into() },
        Message { role: "user".into(), content: "今日は私の事故紹介をします".into() },
    ];

    print!("Response: ");
    match stream_chat(&config, &messages, &mut |token| {
        print!("{}", token);
    }) {
        Ok(full) => println!("\n\nFull: {}", full),
        Err(e) => println!("\nError: {}", e),
    }
}
