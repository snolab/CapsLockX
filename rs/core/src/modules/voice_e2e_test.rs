//! E2E test for the voice input pipeline.
//!
//! Flow: TTS WAV → MockAudioStream → VoiceModule (VAD + STT) → MockPlatform → assert typed text.
//! No real mic or keyboard involved.

use super::VoiceModule;
use crate::key_code::KeyCode;
use crate::platform::{Platform, SystemAudioStream, MouseButton};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

// ── Mock Audio Stream ────────────────────────────────────────────────────────

struct MockAudioStream {
    samples: Arc<Mutex<VecDeque<f32>>>,
    done: Arc<AtomicBool>,
}

impl MockAudioStream {
    fn new(samples: Vec<f32>) -> Self {
        Self {
            samples: Arc::new(Mutex::new(VecDeque::from(samples))),
            done: Arc::new(AtomicBool::new(false)),
        }
    }

    fn is_done(&self) -> bool {
        self.done.load(Ordering::Relaxed)
    }
}

impl Clone for MockAudioStream {
    fn clone(&self) -> Self {
        Self {
            samples: Arc::clone(&self.samples),
            done: Arc::clone(&self.done),
        }
    }
}

impl SystemAudioStream for MockAudioStream {
    fn take_samples(&self) -> Vec<f32> {
        let mut buf = self.samples.lock().unwrap();
        // Return ~50ms of audio per call (16kHz → 800 samples)
        let chunk_size = 800;
        let n = chunk_size.min(buf.len());
        if n == 0 {
            self.done.store(true, Ordering::Relaxed);
            return Vec::new();
        }
        buf.drain(..n).collect()
    }

    fn stop(&self) {}

    fn sample_rate(&self) -> u32 {
        16000
    }
}

// ── Mock Platform ────────────────────────────────────────────────────────────

struct MockPlatform {
    typed: Arc<Mutex<String>>,
    mock_audio: MockAudioStream,
    subtitles: Arc<Mutex<Vec<String>>>,
}

impl MockPlatform {
    fn new(mock_audio: MockAudioStream) -> Self {
        Self {
            typed: Arc::new(Mutex::new(String::new())),
            mock_audio,
            subtitles: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn typed_text(&self) -> String {
        self.typed.lock().unwrap().clone()
    }

    fn last_subtitle(&self) -> String {
        self.subtitles.lock().unwrap().last().cloned().unwrap_or_default()
    }
}

impl Platform for MockPlatform {
    fn key_down(&self, key: KeyCode) {
        if key == KeyCode::Backspace {
            let mut s = self.typed.lock().unwrap();
            s.pop();
        }
    }
    fn key_up(&self, _key: KeyCode) {}
    fn mouse_move(&self, _dx: i32, _dy: i32) {}
    fn scroll_v(&self, _delta: i32) {}
    fn scroll_h(&self, _delta: i32) {}
    fn mouse_button(&self, _button: MouseButton, _pressed: bool) {}

    fn type_text(&self, text: &str) {
        self.typed.lock().unwrap().push_str(text);
    }

    fn start_aec_mic(&self) -> Option<Box<dyn SystemAudioStream>> {
        Some(Box::new(self.mock_audio.clone()))
    }

    fn update_voice_subtitle(&self, text: &str) {
        self.subtitles.lock().unwrap().push(text.to_string());
    }
}

// ── WAV Loader ───────────────────────────────────────────────────────────────

/// Load a 16-bit PCM WAV file as f32 samples.
fn load_wav(path: &str) -> Vec<f32> {
    let data = std::fs::read(path).expect(&format!("cannot read {}", path));
    assert!(data.len() > 44, "WAV too small");
    assert!(&data[0..4] == b"RIFF", "not a WAV file");

    // Parse header
    let channels = u16::from_le_bytes([data[22], data[23]]) as usize;
    let sample_rate = u32::from_le_bytes([data[24], data[25], data[26], data[27]]);
    let bits = u16::from_le_bytes([data[34], data[35]]);
    eprintln!("[test] WAV: {}Hz {}ch {}bit, {} bytes", sample_rate, channels, bits, data.len());

    assert_eq!(bits, 16, "only 16-bit WAV supported");

    // Find data chunk
    let mut pos = 12;
    while pos + 8 < data.len() {
        let chunk_id = &data[pos..pos+4];
        let chunk_size = u32::from_le_bytes([data[pos+4], data[pos+5], data[pos+6], data[pos+7]]) as usize;
        if chunk_id == b"data" {
            let audio_data = &data[pos+8..pos+8+chunk_size.min(data.len()-pos-8)];
            let samples: Vec<f32> = audio_data.chunks(2)
                .map(|c| i16::from_le_bytes([c[0], c.get(1).copied().unwrap_or(0)]) as f32 / 32768.0)
                .collect();
            // Mix to mono if stereo
            if channels > 1 {
                return samples.chunks(channels)
                    .map(|ch| ch.iter().sum::<f32>() / channels as f32)
                    .collect();
            }
            return samples;
        }
        pos += 8 + chunk_size;
        if chunk_size % 2 != 0 { pos += 1; } // padding
    }
    panic!("no data chunk in WAV");
}

// ── E2E Test ─────────────────────────────────────────────────────────────────

#[test]
#[ignore = "depends on real otoji subprocess; see CLX_DISABLE_OTOJI_SPAWN gate"]
fn test_voice_input_e2e() {
    // 1. Load test audio
    let wav_path = concat!(env!("CARGO_MANIFEST_DIR"), "/src/test_data/test_speech.wav");
    let samples = load_wav(wav_path);
    let duration_secs = samples.len() as f64 / 16000.0;
    eprintln!("[test] loaded {} samples ({:.1}s)", samples.len(), duration_secs);

    // 2. Create mocks
    let mock_audio = MockAudioStream::new(samples);
    let platform = Arc::new(MockPlatform::new(mock_audio.clone()));

    // 3. Create voice module
    let voice = VoiceModule::new(Arc::clone(&platform) as Arc<dyn Platform>);

    // 4. Simulate Space+V hold (>300ms = input mode)
    eprintln!("[test] >>> on_key_down(V)");
    voice.on_key_down(KeyCode::V);
    std::thread::sleep(Duration::from_millis(400)); // enter input mode
    eprintln!("[test] input mode should be active");

    // 5. Wait for audio to be consumed by the voice pipeline
    let deadline = std::time::Instant::now() + Duration::from_secs(15);
    while !mock_audio.is_done() {
        if std::time::Instant::now() > deadline {
            eprintln!("[test] TIMEOUT waiting for audio consumption");
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    eprintln!("[test] audio consumed, waiting for STT...");
    std::thread::sleep(Duration::from_secs(3)); // let STT process remaining chunks

    // 6. Release V (triggers final polish + stop)
    eprintln!("[test] >>> on_key_up(V)");
    voice.on_key_up(KeyCode::V);
    std::thread::sleep(Duration::from_secs(3)); // wait for polish

    // 7. Collect results
    let typed = platform.typed_text();
    let subtitle = platform.last_subtitle();
    eprintln!("[test] ══════════════════════════════════════");
    eprintln!("[test] Typed text: {:?}", typed);
    eprintln!("[test] Last subtitle: {:?}", subtitle);
    eprintln!("[test] ══════════════════════════════════════");

    // 8. Assert — fuzzy match (STT won't be perfect)
    assert!(!typed.is_empty(), "Voice pipeline should have typed something");
    let lower = typed.to_lowercase();
    let keywords = ["quick", "brown", "fox", "jump", "lazy", "dog"];
    let matches: Vec<&str> = keywords.iter().filter(|&&k| lower.contains(k)).copied().collect();
    eprintln!("[test] Matched keywords: {:?} / {:?}", matches, keywords);
    assert!(
        matches.len() >= 2,
        "Expected at least 2 keywords from 'The quick brown fox jumps over the lazy dog', \
         got {} matches ({:?}) in: {:?}",
        matches.len(), matches, typed
    );
}

// ── Matrix Test: 3 languages × 7 sentences ──────────────────────────────────

/// Run voice input E2E for a single test case. Returns (typed_text, matched_count, total_keywords).
fn run_voice_e2e(wav_file: &str, keywords: &[&str]) -> (String, usize, usize) {
    let wav_path = format!("{}/src/test_data/{}", env!("CARGO_MANIFEST_DIR"), wav_file);
    let samples = load_wav(&wav_path);
    let duration_secs = samples.len() as f64 / 16000.0;

    let mock_audio = MockAudioStream::new(samples);
    let platform = Arc::new(MockPlatform::new(mock_audio.clone()));
    let voice = VoiceModule::new(Arc::clone(&platform) as Arc<dyn Platform>);

    // Hold V (input mode)
    voice.on_key_down(KeyCode::V);
    std::thread::sleep(Duration::from_millis(400));

    // Wait for audio consumption
    let deadline = std::time::Instant::now() + Duration::from_secs(15);
    while !mock_audio.is_done() {
        if std::time::Instant::now() > deadline { break; }
        std::thread::sleep(Duration::from_millis(100));
    }
    std::thread::sleep(Duration::from_secs(3));

    // Release V
    voice.on_key_up(KeyCode::V);
    std::thread::sleep(Duration::from_secs(3));

    let typed = platform.typed_text();
    let lower = typed.to_lowercase();
    let matches: Vec<&str> = keywords.iter().filter(|&&k| lower.contains(k)).copied().collect();

    eprintln!("[matrix] {} ({:.1}s) → {:?} | matched {}/{}: {:?}",
        wav_file, duration_secs, typed, matches.len(), keywords.len(), matches);

    (typed, matches.len(), keywords.len())
}

#[test]
#[ignore = "depends on real otoji subprocess; see CLX_DISABLE_OTOJI_SPAWN gate"]
fn test_voice_matrix() {
    let cases: Vec<(&str, &str, Vec<&str>)> = vec![
        // English
        ("en1.wav", "The quick brown fox jumps over the lazy dog",
         vec!["quick", "brown", "fox", "jump", "lazy", "dog"]),
        ("en2.wav", "How are you doing today my friend",
         vec!["how", "doing", "today", "friend"]),
        ("en3.wav", "Please remember to save your work before closing",
         vec!["remember", "save", "work", "closing"]),
        ("en4.wav", "The weather is beautiful outside this morning",
         vec!["weather", "beautiful", "outside", "morning"]),
        ("en5.wav", "I would like to order a cup of coffee please",
         vec!["order", "cup", "coffee", "please"]),
        ("en6.wav", "Can you help me find the nearest train station",
         vec!["help", "find", "nearest", "train", "station"]),
        ("en7.wav", "Technology is changing the world every single day",
         vec!["technology", "changing", "world", "every", "day"]),
        // Chinese
        ("zh1.wav", "今天天气真的很不错",
         vec!["今天", "天气", "不错"]),
        ("zh2.wav", "请问最近的地铁站在哪里",
         vec!["请问", "地铁", "哪里"]),
        ("zh3.wav", "我想要一杯热咖啡谢谢",
         vec!["咖啡", "谢谢"]),
        ("zh4.wav", "这个周末我们一起去爬山吧",
         vec!["周末", "一起", "爬山"]),
        ("zh5.wav", "人工智能正在改变我们的生活",
         vec!["人工智能", "改变", "生活"]),
        ("zh6.wav", "学习新的编程语言很有意思",
         vec!["学习", "编程", "语言"]),
        ("zh7.wav", "早上好希望你今天过得愉快",
         vec!["早上", "今天", "愉快"]),
        // Japanese
        ("ja1.wav", "今日はとても良い天気ですね",
         vec!["今日", "天気"]),
        ("ja2.wav", "すみません駅はどこですか",
         vec!["すみません", "駅"]),
        ("ja3.wav", "コーヒーを一杯お願いします",
         vec!["コーヒー", "お願い"]),
        ("ja4.wav", "週末に一緒に山に登りましょう",
         vec!["週末", "一緒", "山"]),
        ("ja5.wav", "技術は毎日世界を変えています",
         vec!["技術", "世界", "変え"]),
        ("ja6.wav", "新しいプログラミング言語を学ぶのは楽しい",
         vec!["プログラミング", "言語", "楽し"]),
        ("ja7.wav", "おはようございます良い一日を",
         vec!["おはよう", "一日"]),
    ];

    let mut results: Vec<(String, String, usize, usize, bool)> = Vec::new();

    for (wav, original, keywords) in &cases {
        let (typed, matched, total) = run_voice_e2e(wav, keywords);
        let pass = !typed.is_empty() && matched >= 1;
        results.push((wav.to_string(), typed, matched, total, pass));
    }

    // Print matrix summary
    eprintln!("\n╔══════════════════════════════════════════════════════════════╗");
    eprintln!("║              Voice E2E Test Matrix Results                  ║");
    eprintln!("╠══════════╦═══════════════════════════════════╦══════╦═══════╣");
    eprintln!("║ File     ║ Typed                             ║ Match║ Pass  ║");
    eprintln!("╠══════════╬═══════════════════════════════════╬══════╬═══════╣");
    for (wav, typed, matched, total, pass) in &results {
        let typed_short: String = typed.chars().take(30).collect();
        let status = if *pass { "✓" } else { "✗" };
        eprintln!("║ {:<8} ║ {:<33} ║ {}/{:<2} ║   {}   ║",
            wav, typed_short, matched, total, status);
    }
    eprintln!("╚══════════╩═══════════════════════════════════╩══════╩═══════╝");

    let passed = results.iter().filter(|r| r.4).count();
    let total = results.len();
    eprintln!("\nPassed: {}/{}", passed, total);

    // At least 70% should pass
    assert!(
        passed * 100 / total >= 70,
        "Expected >=70% pass rate, got {}/{} ({}%)",
        passed, total, passed * 100 / total
    );
}

#[test]
fn test_voice_click_toggle() {
    // Test that V click (<300ms) enters note mode, not input mode.
    // No text should be typed (note mode = record-only, no cursor input).
    let mock_audio = MockAudioStream::new(vec![0.0; 16000]); // 1s silence
    let platform = Arc::new(MockPlatform::new(mock_audio));
    let voice = VoiceModule::new(Arc::clone(&platform) as Arc<dyn Platform>);

    // Quick click: down + up within 100ms
    voice.on_key_down(KeyCode::V);
    std::thread::sleep(Duration::from_millis(100));
    voice.on_key_up(KeyCode::V);
    std::thread::sleep(Duration::from_secs(2));

    let typed = platform.typed_text();
    assert!(typed.is_empty(), "Click mode should not type anything, got: {:?}", typed);
}
