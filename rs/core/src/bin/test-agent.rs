/// Agent tool test runner — runs test scenarios from the test matrix.
///
/// Usage: GEMINI_API_KEY=... test-agent

use capslockx_core::llm_client::{LlmConfig, Message};
use capslockx_core::agent::agent_chat;
use std::io::Write;

const SYSTEM_PROMPT: &str = "You are a helpful assistant with tools. Use them when needed. Be concise.";

fn main() {
    let key = std::env::var("GEMINI_API_KEY")
        .or_else(|_| std::env::var("OPENAI_API_KEY"))
        .expect("Set GEMINI_API_KEY or OPENAI_API_KEY");

    let model = std::env::var("CLX_LLM_MODEL").unwrap_or_default();
    let config = LlmConfig::from_key_and_model(&key, &model);
    eprintln!("Agent Test Runner ({:?} / {})\n", config.provider, config.model);

    let tests: Vec<(&str, &str, &str)> = vec![
        // Web Search
        ("web_search",   "What is the current price of Bitcoin in USD?", "web_search"),
        ("web_search",   "Find the GitHub repo URL for sherpa-onnx", "web_search"),
        // URL Fetch
        ("fetch_url",    "Summarize https://news.ycombinator.com in 3 bullets", "fetch_url"),
        ("fetch_url",    "What does https://api.github.com return? First 3 fields.", "fetch_url"),
        // JS Eval — calculations and data processing
        ("js_eval",      "Calculate the factorial of 20 precisely", "js_eval"),
        ("js_eval",      "Generate the first 10 Fibonacci numbers", "js_eval"),
        ("js_eval",      "Convert 'hello world' to base64", "js_eval"),
        ("js_eval",      "What is Math.PI * Math.E?", "js_eval"),
        // Math Eval — symbolic math
        ("math_eval",    "What is the integral of x^3 with respect to x?", "math_eval"),
        ("math_eval",    "Factor the polynomial x^4 - 1", "math_eval"),
        ("math_eval",    "Solve the equation x^2 - 5x + 6 = 0", "math_eval"),
        ("math_eval",    "What is the 100th prime number?", "math_eval"),
        // Read Screen
        ("read_screen",  "What window is currently active on my computer?", "read_screen"),
        // No tools
        ("no_tool",      "What is 1 divided by 0?", "none"),
        ("no_tool",      "Reply in exactly 3 words.", "none"),
    ];

    let mut pass = 0;
    let mut fail = 0;
    let total = tests.len();

    println!("{:<18} {:<6} {:<15} {:<50}", "Test", "Status", "Tools Used", "Response (first 50 chars)");
    println!("{}", "-".repeat(100));

    for (id, prompt, expected_tool) in &tests {
        let mut messages = vec![
            Message { role: "system".into(), content: SYSTEM_PROMPT.into(), image_base64: None },
            Message { role: "user".into(), content: prompt.to_string(), image_base64: None },
        ];

        let mut tools_used = Vec::new();
        let mut response = String::new();

        let result = agent_chat(&config, &mut messages, &mut |token| {
            response.push_str(token);
        }, &mut |status| {
            // Extract tool name from status like "Running web_search(...)..."
            if let Some(name) = status.strip_prefix("Running ") {
                if let Some(paren) = name.find('(') {
                    tools_used.push(name[..paren].to_string());
                }
            }
        });

        match result {
            Ok(text) => {
                if response.is_empty() { response = text; }
                let tools_str = if tools_used.is_empty() { "none".to_string() } else { tools_used.join(",") };
                let ok = if *expected_tool == "none" {
                    tools_used.is_empty()
                } else {
                    tools_used.iter().any(|t| t == expected_tool)
                };

                let status = if ok && !response.is_empty() { pass += 1; "PASS" } else { fail += 1; "FAIL" };
                let short: String = response.chars().take(50).collect();
                println!("{:<18} {:<6} {:<15} {}", id, status, tools_str, short.replace('\n', " "));
            }
            Err(e) => {
                fail += 1;
                println!("{:<18} {:<6} {:<15} Error: {}", id, "ERROR", "", e.chars().take(50).collect::<String>());
            }
        }

        std::io::stdout().flush().ok();
        // Small delay between tests to avoid rate limiting.
        std::thread::sleep(std::time::Duration::from_millis(500));
    }

    println!("{}", "-".repeat(100));
    println!("Results: {}/{} passed, {} failed", pass, total, fail);
}
