/// Incremental LLM-based STT error correction.
///
/// Maintains a conversation context with the LLM. Each new STT segment is
/// appended as a user message. The LLM returns the corrected version.
/// Because the conversation grows incrementally, the LLM's KV cache is reused
/// for all prior context — only the new segment costs inference tokens.
///
/// Usage:
///   let mut corrector = SttCorrector::new(llm_config);
///   let corrected = corrector.correct("今日は私の事故紹介をします");
///   // → "今日は私の自己紹介をします"

use crate::llm_client::{LlmConfig, Message, stream_chat};

const SYSTEM_PROMPT: &str = "\
You are a speech-to-text error corrector. You receive raw STT output that may contain \
recognition errors. Fix obvious STT mistakes while preserving the speaker's intended meaning. \
Rules:
- Only fix clear recognition errors (wrong homophones, missing particles, garbled words)
- Do NOT change grammar, style, or add punctuation unless the original had it
- Do NOT translate between languages — keep the original language
- If the input looks correct, return it unchanged
- Return ONLY the corrected text, no explanations
- Keep the same language as input (Japanese stays Japanese, etc.)";

pub struct SttCorrector {
    config: LlmConfig,
    /// Accumulated conversation for KV cache reuse.
    history: Vec<Message>,
    /// Whether correction is enabled.
    enabled: bool,
}

impl SttCorrector {
    pub fn new(config: LlmConfig) -> Self {
        let history = vec![
            Message { role: "system".into(), content: SYSTEM_PROMPT.into() },
        ];
        Self { config, history, enabled: true }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled && !self.config.api_key.is_empty()
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Correct a new STT segment. Appends to conversation context.
    /// Returns the corrected text, or the original on error.
    pub fn correct(&mut self, raw_text: &str) -> String {
        if !self.is_enabled() || raw_text.trim().is_empty() {
            return raw_text.to_string();
        }

        // Append user message with the raw STT output.
        self.history.push(Message {
            role: "user".into(),
            content: raw_text.to_string(),
        });

        // Call LLM.
        let mut corrected = String::new();
        match stream_chat(&self.config, &self.history, &mut |token| {
            corrected.push_str(token);
        }) {
            Ok(_) => {
                let corrected = corrected.trim().to_string();
                eprintln!("[CLX] stt-correct: {:?} → {:?}", raw_text, corrected);

                // Append assistant response to history for context.
                self.history.push(Message {
                    role: "assistant".into(),
                    content: corrected.clone(),
                });

                // Trim history if it gets too long (keep system + last 40 messages).
                if self.history.len() > 42 {
                    let system = self.history[0].clone();
                    let tail: Vec<Message> = self.history[self.history.len()-40..].to_vec();
                    self.history = vec![system];
                    self.history.extend(tail);
                }

                corrected
            }
            Err(e) => {
                eprintln!("[CLX] stt-correct error: {}", e);
                // Remove the failed user message.
                self.history.pop();
                raw_text.to_string()
            }
        }
    }

    /// Reset conversation context (e.g., when starting a new voice session).
    pub fn reset(&mut self) {
        self.history.truncate(1); // keep system prompt
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm_client::{LlmConfig, LlmProvider};

    fn cfg_with_key(key: &str) -> LlmConfig {
        LlmConfig {
            provider: LlmProvider::Gemini,
            api_key: key.to_string(),
            model: "gemini-2.0-flash".to_string(),
            base_url: None,
        }
    }

    #[test]
    fn new_initializes_with_system_prompt() {
        let c = SttCorrector::new(cfg_with_key("k"));
        assert_eq!(c.history.len(), 1);
        assert_eq!(c.history[0].role, "system");
        assert!(c.history[0].content.contains("speech-to-text"));
    }

    #[test]
    fn is_enabled_requires_api_key() {
        assert!(!SttCorrector::new(cfg_with_key("")).is_enabled());
        assert!(SttCorrector::new(cfg_with_key("k")).is_enabled());
    }

    #[test]
    fn set_enabled_disables_correction() {
        let mut c = SttCorrector::new(cfg_with_key("k"));
        assert!(c.is_enabled());
        c.set_enabled(false);
        assert!(!c.is_enabled());
        c.set_enabled(true);
        assert!(c.is_enabled());
    }

    #[test]
    fn correct_returns_input_when_disabled() {
        let mut c = SttCorrector::new(cfg_with_key("k"));
        c.set_enabled(false);
        assert_eq!(c.correct("hello"), "hello");
        assert_eq!(c.history.len(), 1);
    }

    #[test]
    fn correct_returns_input_when_no_api_key() {
        let mut c = SttCorrector::new(cfg_with_key(""));
        assert_eq!(c.correct("hello"), "hello");
        assert_eq!(c.history.len(), 1);
    }

    #[test]
    fn correct_returns_input_when_blank_text() {
        let mut c = SttCorrector::new(cfg_with_key("k"));
        assert_eq!(c.correct("   "), "   ");
        assert_eq!(c.correct(""), "");
        assert_eq!(c.history.len(), 1);
    }

    #[test]
    fn reset_keeps_only_system_prompt() {
        let mut c = SttCorrector::new(cfg_with_key("k"));
        c.history.push(Message { role: "user".into(), content: "a".into() });
        c.history.push(Message { role: "assistant".into(), content: "b".into() });
        assert_eq!(c.history.len(), 3);
        c.reset();
        assert_eq!(c.history.len(), 1);
        assert_eq!(c.history[0].role, "system");
    }
}
