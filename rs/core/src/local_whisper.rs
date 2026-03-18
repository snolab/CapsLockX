/// Local Whisper transcription using whisper-rs (whisper.cpp bindings).
///
/// Lazily loads a GGML model from `~/.cache/capslockx/ggml-tiny.en.bin` and
/// provides a `transcribe(samples) -> String` function for instant local
/// speech-to-text before the server round-trip.
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

/// Model filename looked up inside the cache directory.
const MODEL_FILENAME: &str = "ggml-tiny.en.bin";
/// Subdirectory under the OS cache dir.
const CACHE_SUBDIR: &str = "capslockx";

pub struct LocalWhisper {
    ctx: WhisperContext,
}

impl LocalWhisper {
    /// Load the Whisper model from `~/.cache/capslockx/ggml-tiny.en.bin` (or
    /// the platform-appropriate cache directory).
    ///
    /// Returns `Err` with a human-readable message if the model cannot be
    /// loaded. The caller should log the error and fall back to server-only
    /// transcription.
    pub fn new() -> Result<Self, String> {
        let model_path = Self::model_path()?;

        if !model_path.exists() {
            let display = model_path.display();
            eprintln!("[CLX] voice: local Whisper model not found at {display}");
            eprintln!(
                "[CLX] voice: download it with: curl -L -o {display} \
                 https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.en.bin"
            );
            eprintln!("[CLX] voice: falling back to server-only transcription");
            return Err(format!("model not found at {display}"));
        }

        let path_str = model_path
            .to_str()
            .ok_or_else(|| "model path is not valid UTF-8".to_string())?;

        let ctx = WhisperContext::new_with_params(path_str, WhisperContextParameters::default())
            .map_err(|e| format!("failed to load Whisper model: {e}"))?;

        eprintln!(
            "[CLX] voice: loaded local Whisper model from {}",
            model_path.display()
        );

        Ok(Self { ctx })
    }

    /// Transcribe f32 samples (must be 16 kHz mono).
    ///
    /// Returns the concatenated text of all recognised segments, trimmed.
    /// Returns an empty string if nothing was recognised.
    pub fn transcribe(&self, samples: &[f32]) -> Result<String, String> {
        let mut state = self.ctx.create_state().map_err(|e| format!("create state: {e}"))?;

        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_language(Some("en"));
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
        params.set_single_segment(true);
        params.set_no_timestamps(true);
        // Suppress non-speech tokens for cleaner output.
        params.set_suppress_blank(true);

        state
            .full(params, samples)
            .map_err(|e| format!("whisper full: {e}"))?;

        let n_segments = state
            .full_n_segments()
            .map_err(|e| format!("n_segments: {e}"))?;

        let mut text = String::new();
        for i in 0..n_segments {
            if let Ok(seg) = state.full_get_segment_text(i) {
                text.push_str(&seg);
            }
        }

        Ok(text.trim().to_string())
    }

    /// Resolve the expected model file path.
    fn model_path() -> Result<std::path::PathBuf, String> {
        let cache = dirs::cache_dir().ok_or("could not determine cache directory")?;
        Ok(cache.join(CACHE_SUBDIR).join(MODEL_FILENAME))
    }
}
