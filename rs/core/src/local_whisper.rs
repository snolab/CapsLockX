/// Local Whisper transcription using whisper-rs (whisper.cpp bindings).
///
/// Auto-scales model size based on inference speed:
///   tiny (75MB) → base (142MB) → small (466MB)
/// Upgrades when inference is >1.25x realtime, downgrades if too slow.
/// Downloads models automatically from HuggingFace on first use.
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

const CACHE_SUBDIR: &str = "capslockx";
const HF_BASE_URL: &str = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main";

/// Model tiers in ascending order of quality/size.
const MODEL_TIERS: &[ModelTier] = &[
    ModelTier { name: "tiny",  filename: "ggml-tiny.bin",  size_mb: 75 },
    ModelTier { name: "base",  filename: "ggml-base.bin",  size_mb: 142 },
    ModelTier { name: "small", filename: "ggml-small.bin", size_mb: 466 },
];

/// Upgrade if inference speed > this multiple of realtime.
/// Set high to prevent flip-flopping between models during streaming.
const UPGRADE_THRESHOLD: f64 = 5.0;
/// Downgrade if inference speed < 1.0x realtime (can't keep up).
const DOWNGRADE_THRESHOLD: f64 = 1.0;
/// Number of samples to average before making scaling decision.
const SAMPLES_BEFORE_UPGRADE: usize = 10;

struct ModelTier {
    name: &'static str,
    filename: &'static str,
    size_mb: u32,
}

pub struct LocalWhisper {
    ctx: WhisperContext,
    tier_index: usize,
    /// Rolling speed ratios (wall_time_available / inference_time).
    speed_samples: Vec<f64>,
    /// When the last transcription finished — idle time counts as headroom.
    last_done: Option<std::time::Instant>,
    /// Background model loader — old model keeps working while new one loads.
    pending: Option<std::sync::mpsc::Receiver<Result<LocalWhisper, String>>>,
}

impl LocalWhisper {
    /// Load the best model — reads saved tier from last session, falls back to smallest.
    pub fn new() -> Result<Self, String> {
        let cache_dir = dirs::cache_dir()
            .ok_or("could not determine cache directory")?
            .join(CACHE_SUBDIR);
        std::fs::create_dir_all(&cache_dir).ok();

        // Read saved tier preference from last session.
        let saved_tier = Self::read_saved_tier(&cache_dir);

        // Try saved tier first, then fall back to smallest available.
        let try_order: Vec<usize> = if let Some(saved) = saved_tier {
            let mut order = vec![saved];
            // Fallback: try smaller tiers if saved one fails.
            for i in (0..saved).rev() { order.push(i); }
            order
        } else {
            (0..MODEL_TIERS.len()).collect()
        };

        for i in try_order {
            let tier = &MODEL_TIERS[i];
            let path = cache_dir.join(tier.filename);
            if path.exists() {
                return Self::load_tier(i, &path);
            }
        }

        // No model found — download the smallest.
        let tier = &MODEL_TIERS[0];
        let path = cache_dir.join(tier.filename);
        eprintln!("[CLX] whisper: downloading {} model (~{}MB)...", tier.name, tier.size_mb);
        download_model(tier.filename, &path)?;
        Self::load_tier(0, &path)
    }

    /// Save current tier preference to disk.
    fn save_tier(&self) {
        if let Some(cache_dir) = dirs::cache_dir() {
            let path = cache_dir.join(CACHE_SUBDIR).join("whisper-tier.txt");
            let _ = std::fs::write(&path, MODEL_TIERS[self.tier_index].name);
        }
    }

    /// Read saved tier preference.
    fn read_saved_tier(cache_dir: &std::path::Path) -> Option<usize> {
        let path = cache_dir.join("whisper-tier.txt");
        let name = std::fs::read_to_string(path).ok()?;
        let name = name.trim();
        MODEL_TIERS.iter().position(|t| t.name == name)
    }

    fn load_tier(tier_index: usize, path: &std::path::Path) -> Result<Self, String> {
        let tier = &MODEL_TIERS[tier_index];
        let path_str = path.to_str().ok_or("path not UTF-8")?;

        eprintln!("[CLX] whisper: loading {} model...", tier.name);
        let t0 = std::time::Instant::now();
        let ctx = WhisperContext::new_with_params(path_str, WhisperContextParameters::default())
            .map_err(|e| format!("failed to load {} model: {e}", tier.name))?;

        eprintln!("[CLX] whisper: loaded {} model ({:.0}ms)", tier.name, t0.elapsed().as_millis());

        let s = Self {
            ctx,
            tier_index,
            speed_samples: Vec::new(),
            last_done: None,
            pending: None,
        };
        s.save_tier();
        Ok(s)
    }

    /// Transcribe f32 samples (must be 16 kHz mono).
    /// Returns (text, inference_ms, wall_budget_ratio).
    /// wall_budget_ratio = time_available / inference_time, where time_available
    /// includes idle time since last transcription (user pauses = free headroom).
    pub fn transcribe(&mut self, samples: &[f32]) -> Result<String, String> {
        let mut state = self.ctx.create_state().map_err(|e| format!("create state: {e}"))?;

        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_language(None); // auto-detect language
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
        params.set_single_segment(true);
        params.set_no_timestamps(true);
        params.set_suppress_blank(true);

        let t0 = std::time::Instant::now();
        // Wall time available = time since last transcription finished.
        // Includes user pauses/silence — that's free processing headroom.
        let wall_available = self.last_done
            .map(|d| d.elapsed().as_secs_f64())
            .unwrap_or(f64::MAX);

        state.full(params, samples).map_err(|e| format!("whisper full: {e}"))?;
        let inference_secs = t0.elapsed().as_secs_f64();
        self.last_done = Some(std::time::Instant::now());

        let audio_dur = samples.len() as f64 / 16000.0;
        let realtime_ratio = audio_dur / inference_secs;
        // Budget ratio: how much wall time was available vs how much we used.
        // >1 means we had time to spare, <1 means we're falling behind.
        let budget_ratio = wall_available / inference_secs;
        let effective_ratio = budget_ratio.min(10.0); // cap at 10x to avoid outliers

        eprintln!("[CLX] whisper[{}]: {:.1}s audio → {:.0}ms ({:.1}x realtime, {:.1}x budget)",
            MODEL_TIERS[self.tier_index].name, audio_dur,
            inference_secs * 1000.0, realtime_ratio, effective_ratio);

        // Record the budget ratio for auto-scaling decisions.
        self.speed_samples.push(effective_ratio);

        let n_segments = state.full_n_segments().map_err(|e| format!("n_segments: {e}"))?;
        let mut text = String::new();
        for i in 0..n_segments {
            if let Ok(seg) = state.full_get_segment_text(i) {
                text.push_str(&seg);
            }
        }

        let text = text.trim();
        // Filter out Whisper hallucinations and noise artifacts.
        if is_noise_artifact(text) {
            return Ok(String::new());
        }
        Ok(text.to_string())
    }

    /// Check speed samples and kick off background model loading if needed.
    /// Does NOT block — the old model keeps working while the new one loads.
    pub fn maybe_rescale(&mut self) {
        if self.pending.is_some() { return; } // already loading
        if self.speed_samples.len() < SAMPLES_BEFORE_UPGRADE { return; }

        let avg: f64 = self.speed_samples.iter().sum::<f64>() / self.speed_samples.len() as f64;
        self.speed_samples.clear();

        let target_tier = if avg < DOWNGRADE_THRESHOLD && self.tier_index > 0 {
            eprintln!("[CLX] whisper: budget {:.1}x < {:.1}x, will downgrade",
                avg, DOWNGRADE_THRESHOLD);
            Some(self.tier_index - 1)
        } else if avg > UPGRADE_THRESHOLD && self.tier_index + 1 < MODEL_TIERS.len() {
            eprintln!("[CLX] whisper: budget {:.1}x > {:.1}x, will upgrade",
                avg, UPGRADE_THRESHOLD);
            Some(self.tier_index + 1)
        } else {
            None
        };

        if let Some(idx) = target_tier {
            let (tx, rx) = std::sync::mpsc::channel();
            self.pending = Some(rx);

            std::thread::Builder::new()
                .name("clx-whisper-load".into())
                .spawn(move || {
                    let tier = &MODEL_TIERS[idx];
                    let cache_dir = match dirs::cache_dir() {
                        Some(d) => d.join(CACHE_SUBDIR),
                        None => { let _ = tx.send(Err("no cache dir".into())); return; }
                    };
                    let path = cache_dir.join(tier.filename);

                    // Download if needed.
                    if !path.exists() {
                        eprintln!("[CLX] whisper: downloading {} (~{}MB)...", tier.name, tier.size_mb);
                        if let Err(e) = download_model(tier.filename, &path) {
                            let _ = tx.send(Err(e));
                            return;
                        }
                    }

                    let _ = tx.send(Self::load_tier(idx, &path));
                })
                .ok();
        }
    }

    /// Check if a background model load has completed. If so, swap it in.
    pub fn check_pending_upgrade(&mut self, slot: &mut Option<LocalWhisper>) {
        // First check rescale trigger.
        self.maybe_rescale();

        // Then check if background load is done (non-blocking).
        if let Some(ref rx) = self.pending {
            match rx.try_recv() {
                Ok(Ok(new_model)) => {
                    eprintln!("[CLX] whisper: hot-swapped to {} model", new_model.tier_name());
                    self.pending = None;
                    *slot = Some(new_model);
                }
                Ok(Err(e)) => {
                    eprintln!("[CLX] whisper: background load failed: {e}");
                    self.pending = None;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    // Still loading — keep using current model.
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.pending = None;
                }
            }
        }
    }

    pub fn tier_name(&self) -> &str {
        MODEL_TIERS[self.tier_index].name
    }
}

/// Known Whisper hallucinations and noise artifacts to filter out.
fn is_noise_artifact(text: &str) -> bool {
    let t = text.to_lowercase();
    // Bracketed annotations like (clicking), [BLANK_AUDIO], (keyboard clicking)
    if t.starts_with('(') && t.ends_with(')') { return true; }
    if t.starts_with('[') && t.ends_with(']') { return true; }
    // Common hallucination patterns
    let patterns = [
        "blank_audio", "blank audio",
        "clicking", "keyboard",
        "music", "applause", "laughter",
        "silence", "no speech",
        "thank you", "thanks for watching",
        "subscribe", "like and subscribe",
        "see you next time", "bye",
        "you", // single-word hallucination
    ];
    for p in &patterns {
        if t == *p { return true; }
    }
    // Very short outputs are usually noise
    if t.chars().count() <= 2 { return true; }
    false
}

/// Download a model file from HuggingFace.
#[cfg(not(target_arch = "wasm32"))]
fn download_model(filename: &str, dest: &std::path::Path) -> Result<(), String> {
    let url = format!("{HF_BASE_URL}/{filename}");
    let resp = ureq::get(&url)
        .call()
        .map_err(|e| format!("download {filename}: {e}"))?;

    let mut reader = resp.into_reader();
    let mut file = std::fs::File::create(dest)
        .map_err(|e| format!("create {}: {e}", dest.display()))?;
    std::io::copy(&mut reader, &mut file)
        .map_err(|e| format!("write {}: {e}", dest.display()))?;

    eprintln!("[CLX] whisper: downloaded {} to {}", filename, dest.display());
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn download_model(_filename: &str, _dest: &std::path::Path) -> Result<(), String> {
    Err("auto-download not supported on wasm".to_string())
}
