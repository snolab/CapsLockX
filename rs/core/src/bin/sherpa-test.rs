/// Standalone sherpa SenseVoice test binary.
///
/// Captures mic audio, runs SenseVoice transcription, prints results.
/// Press Ctrl+C to stop.
///
/// Usage:
///   cargo run -p capslockx-core --release --bin sherpa-test

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use capslockx_core::audio_capture::AudioCapture;
use capslockx_core::local_sherpa::LocalSherpa;

#[allow(dead_code)]
const STREAMING_CHUNK_SAMPLES: usize = 1_600; // 100ms at 16kHz

fn main() {
    eprintln!("=== Sherpa SenseVoice Test ===");
    eprintln!("Speak into your mic. Ctrl+C to stop.\n");

    // Load SenseVoice model.
    let mut sherpa = match LocalSherpa::new() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("ERROR: Failed to load SenseVoice: {e}");
            std::process::exit(1);
        }
    };

    // Start audio capture.
    let audio = AudioCapture::new().expect("Failed to create audio capture");
    audio.start().expect("Failed to start audio capture");
    let sample_rate = audio.sample_rate();
    eprintln!("Mic sample rate: {sample_rate}Hz");
    eprintln!("Model: {}\n", sherpa.tier_name());

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc_handler(r);

    let mut pending_buf: Vec<f32> = Vec::new();
    let mut _pending_since: usize = 0;
    let mut committed = String::new();

    while running.load(Ordering::Relaxed) {
        let raw = audio.take_samples();
        if raw.is_empty() {
            std::thread::sleep(std::time::Duration::from_millis(10));
            continue;
        }

        // Resample to 16kHz.
        let samples_16k = resample(&raw, sample_rate, 16000);
        pending_buf.extend_from_slice(&samples_16k);
        _pending_since += samples_16k.len();

        // Transcribe every 1s of accumulated audio (SenseVoice works best on longer chunks).
        if pending_buf.len() >= 16000 {
            let buf = pending_buf.clone();
            let rms: f32 = (buf.iter().map(|s| s * s).sum::<f32>() / buf.len() as f32).sqrt();
            eprintln!("[test] buf={} samples, rms={:.4}", buf.len(), rms);

            match sherpa.transcribe(&buf) {
                Ok(text) if !text.is_empty() => {
                    println!("STREAMING: {text}");
                }
                Ok(_) => eprintln!("[test] (silence/filtered)"),
                Err(e) => eprintln!("ERROR: {e}"),
            }
            // Keep last 0.5s for context overlap.
            let keep = 8000.min(pending_buf.len());
            pending_buf = pending_buf[pending_buf.len() - keep..].to_vec();
            _pending_since = 0;
        }

        // Commit after 5s of accumulated audio.
        if pending_buf.len() > 16000 * 5 {
            let buf = pending_buf.clone();
            match sherpa.transcribe(&buf) {
                Ok(text) if !text.is_empty() => {
                    committed.push_str(&text);
                    println!("COMMITTED: {text}");
                }
                _ => {}
            }
            pending_buf.clear();
            _pending_since = 0;
        }
    }

    if !committed.is_empty() {
        println!("\n=== Full transcript ===\n{committed}");
    }
}

fn resample(samples: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    if from_rate == to_rate { return samples.to_vec(); }
    let ratio = to_rate as f64 / from_rate as f64;
    let out_len = (samples.len() as f64 * ratio) as usize;
    (0..out_len).map(|i| {
        let src = i as f64 / ratio;
        let idx = src as usize;
        let frac = src - idx as f64;
        let a = samples.get(idx).copied().unwrap_or(0.0);
        let b = samples.get(idx + 1).copied().unwrap_or(a);
        a + (b - a) * frac as f32
    }).collect()
}

fn ctrlc_handler(running: Arc<AtomicBool>) {
    std::thread::spawn(move || {
        // Simple signal handling via polling.
        // In practice, Ctrl+C kills the process on macOS.
        loop {
            std::thread::sleep(std::time::Duration::from_secs(3600));
            if !running.load(Ordering::Relaxed) { break; }
        }
    });

    // Register actual signal handler.
    unsafe {
        libc::signal(libc::SIGINT, {
            extern "C" fn handler(_: libc::c_int) {
                eprintln!("\nStopping...");
                std::process::exit(0);
            }
            handler as libc::sighandler_t
        });
    }
}
