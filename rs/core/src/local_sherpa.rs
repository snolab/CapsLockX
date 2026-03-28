/// Local SenseVoice transcription using sherpa-rs (sherpa-onnx bindings).
///
/// SenseVoice supports: Chinese, English, Japanese, Korean, Cantonese
/// in a single model (~228MB int8). Non-autoregressive = fast & predictable latency.
///
/// Drop-in alternative to LocalWhisper with the same API shape.

use sherpa_rs::sense_voice::{SenseVoiceConfig, SenseVoiceRecognizer};
use crate::local_whisper::is_noise_artifact;

const CACHE_SUBDIR: &str = "capslockx";
const MODEL_DIR_NAME: &str = "sherpa-onnx-sense-voice-zh-en-ja-ko-yue-2024-07-17";
const MODEL_FILENAME: &str = "model.onnx";
const TOKENS_FILENAME: &str = "tokens.txt";

/// Download URL for the model archive.
const MODEL_URL: &str = "https://github.com/k2-fsa/sherpa-onnx/releases/download/asr-models/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-2024-07-17.tar.bz2";

pub struct LocalSherpa {
    recognizer: SenseVoiceRecognizer,
}

impl LocalSherpa {
    /// Load the SenseVoice model — downloads on first use (~228MB).
    pub fn new() -> Result<Self, String> {
        let cache_dir = dirs::cache_dir()
            .ok_or("could not determine cache directory")?
            .join(CACHE_SUBDIR);
        std::fs::create_dir_all(&cache_dir).ok();

        let model_dir = cache_dir.join(MODEL_DIR_NAME);
        let model_path = model_dir.join(MODEL_FILENAME);
        let tokens_path = model_dir.join(TOKENS_FILENAME);

        // Download and extract if not present.
        if !model_path.exists() || !tokens_path.exists() {
            eprintln!("[CLX] sherpa: downloading SenseVoice model (~228MB)...");
            download_and_extract(&cache_dir)?;
        }

        let model_str = model_path.to_str().ok_or("model path not UTF-8")?.to_string();
        let tokens_str = tokens_path.to_str().ok_or("tokens path not UTF-8")?.to_string();

        eprintln!("[CLX] sherpa: loading SenseVoice model...");
        let t0 = std::time::Instant::now();

        let config = SenseVoiceConfig {
            model: model_str,
            tokens: tokens_str,
            language: "auto".into(),
            use_itn: true,
            provider: None, // default CPU
            num_threads: Some(4),
            debug: false,
        };

        let recognizer = SenseVoiceRecognizer::new(config)
            .map_err(|e| format!("failed to load SenseVoice: {e}"))?;

        eprintln!("[CLX] sherpa: loaded SenseVoice ({:.0}ms)", t0.elapsed().as_millis());

        Ok(Self { recognizer })
    }

    /// Transcribe f32 samples (must be 16 kHz mono).
    pub fn transcribe(&mut self, samples: &[f32]) -> Result<String, String> {
        let r = self.transcribe_tagged(samples)?;
        Ok(r.text)
    }

    /// Transcribe and return both cleaned text and whether music/humming was detected.
    pub fn transcribe_tagged(&mut self, samples: &[f32]) -> Result<SttOutput, String> {
        let t0 = std::time::Instant::now();
        let audio_dur = samples.len() as f64 / 16000.0;

        let result = self.recognizer.transcribe(16000, samples);
        let inference_ms = t0.elapsed().as_millis();
        let realtime_ratio = audio_dur / (inference_ms as f64 / 1000.0);

        eprintln!("[CLX] sherpa[sensevoice]: {:.1}s audio → {:.0}ms ({:.1}x realtime, lang={})",
            audio_dur, inference_ms, realtime_ratio, result.lang);

        let raw = result.text.trim().to_string();
        let raw_lower = raw.to_lowercase();
        let is_music = raw_lower.contains("[music]")
            || raw_lower.contains("[humming]")
            || raw_lower.contains("[singing]")
            || raw_lower.contains("(music)")
            || raw_lower.contains("(humming)")
            || raw_lower.contains("(singing)");

        if is_music {
            eprintln!("[CLX] sherpa: music/humming detected in raw output: {:?}", raw);
        }

        let text = if is_noise_artifact(&raw) { String::new() } else { raw };
        Ok(SttOutput { text, is_music })
    }

    pub fn tier_name(&self) -> &str {
        "sensevoice"
    }
}

/// STT output with metadata about detected audio events.
pub struct SttOutput {
    pub text: String,
    pub is_music: bool,
}

/// Download the model archive and extract it.
#[cfg(not(target_arch = "wasm32"))]
fn download_and_extract(cache_dir: &std::path::Path) -> Result<(), String> {
    let archive_path = cache_dir.join("sensevoice-model.tar.bz2");

    // Download.
    let resp = ureq::get(MODEL_URL)
        .call()
        .map_err(|e| format!("download SenseVoice: {e}"))?;

    let mut reader = resp.into_reader();
    let mut file = std::fs::File::create(&archive_path)
        .map_err(|e| format!("create archive: {e}"))?;
    std::io::copy(&mut reader, &mut file)
        .map_err(|e| format!("write archive: {e}"))?;

    eprintln!("[CLX] sherpa: downloaded, extracting...");

    // Extract using tar command (bz2).
    let status = std::process::Command::new("tar")
        .args(["xjf", archive_path.to_str().unwrap()])
        .current_dir(cache_dir)
        .status()
        .map_err(|e| format!("tar extract: {e}"))?;

    if !status.success() {
        return Err("tar extraction failed".into());
    }

    // Clean up archive.
    let _ = std::fs::remove_file(&archive_path);

    eprintln!("[CLX] sherpa: model extracted to {}", cache_dir.join(MODEL_DIR_NAME).display());
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn download_and_extract(_cache_dir: &std::path::Path) -> Result<(), String> {
    Err("auto-download not supported on wasm".to_string())
}
