/// STT Engine Benchmark: SenseVoice vs Whisper (tiny/base/small) with WER/CER.
///
/// Usage: stt-bench <audio.wav> <ground-truth.srt>

use capslockx_core::local_sherpa::LocalSherpa;
use std::io::Read;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 { eprintln!("Usage: stt-bench <wav> <srt>"); std::process::exit(1); }

    let samples = load_wav_f32(&args[1]);
    let srt_text = std::fs::read_to_string(&args[2]).expect("read srt");
    let gt_segs = parse_srt(&srt_text);
    let total_secs = samples.len() as f64 / 16000.0;

    // GT filtered to audio duration
    let gt_full: String = gt_segs.iter()
        .filter(|s| s.start < total_secs)
        .map(|s| s.text.as_str()).collect::<Vec<_>>().join(" ");
    let gt_norm = normalize(&gt_full);
    let gt_chars: Vec<char> = gt_norm.chars().filter(|c| !c.is_whitespace()).collect();
    let gt_words: Vec<&str> = gt_norm.split_whitespace().collect();
    eprintln!("Audio: {:.1}s | GT: {} words, {} chars\n", total_secs, gt_words.len(), gt_chars.len());

    // 10s chunked transcription helper
    let chunk_transcribe = |transcribe_fn: &mut dyn FnMut(&[f32]) -> String| -> (String, u128) {
        let chunk_len = (10.0 * 16000.0) as usize;
        let mut texts = Vec::new();
        let mut total_ms: u128 = 0;
        let mut off = 0;
        while off < samples.len() {
            let end = (off + chunk_len).min(samples.len());
            let mut chunk = samples[off..end].to_vec();
            if chunk.len() < 16000 { chunk.resize(16000, 0.0); }
            let t0 = std::time::Instant::now();
            texts.push(transcribe_fn(&chunk));
            total_ms += t0.elapsed().as_millis();
            off = end;
        }
        (texts.join(" "), total_ms)
    };

    struct Result { name: String, text: String, ms: u128, wer: f64, cer: f64 }
    let mut results: Vec<Result> = Vec::new();

    // --- SenseVoice ---
    eprintln!("Loading SenseVoice...");
    let mut sherpa = LocalSherpa::new().expect("sherpa");
    let (text, ms) = chunk_transcribe(&mut |chunk| sherpa.transcribe(chunk).unwrap_or_default());
    let norm = normalize(&text);
    let wer = word_error_rate(&gt_words, &norm.split_whitespace().collect::<Vec<_>>());
    let cer = char_error_rate(&gt_chars, &norm.chars().filter(|c| !c.is_whitespace()).collect::<Vec<_>>());
    results.push(Result { name: "SenseVoice".into(), text: norm, ms, wer, cer });

    // --- Whisper models ---
    let cache = dirs::cache_dir().unwrap().join("capslockx");
    for (tier, filename, size) in [
        ("tiny", "ggml-tiny.bin", "75MB"),
        ("base", "ggml-base.bin", "142MB"),
        ("small", "ggml-small.bin", "466MB"),
        ("medium", "ggml-medium.bin", "1.5GB"),
        ("large-v3", "ggml-large-v3.bin", "3GB"),
    ] {
        let path = cache.join(filename);
        if !path.exists() { eprintln!("Skipping whisper-{} (not downloaded)", tier); continue; }

        eprintln!("Loading Whisper {}...", tier);
        let ctx = WhisperContext::new_with_params(
            path.to_str().unwrap(), WhisperContextParameters::default()
        ).expect("load whisper");

        let (text, ms) = chunk_transcribe(&mut |chunk| {
            let mut state = ctx.create_state().unwrap();
            let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
            params.set_language(None);
            params.set_print_special(false);
            params.set_print_progress(false);
            params.set_print_realtime(false);
            params.set_print_timestamps(false);
            params.set_single_segment(true);
            params.set_no_timestamps(true);
            params.set_suppress_blank(true);
            state.full(params, chunk).ok();
            let n = state.full_n_segments().unwrap_or(0);
            let mut t = String::new();
            for i in 0..n { if let Ok(s) = state.full_get_segment_text(i) { t.push_str(&s); } }
            t.trim().to_string()
        });

        let norm = normalize(&text);
        let wer = word_error_rate(&gt_words, &norm.split_whitespace().collect::<Vec<_>>());
        let cer = char_error_rate(&gt_chars, &norm.chars().filter(|c| !c.is_whitespace()).collect::<Vec<_>>());
        results.push(Result { name: format!("Whisper-{} ({})", tier, size), text: norm, ms, wer, cer });
    }

    // --- Print ---
    println!("\n{}", "=".repeat(100));
    println!("GT: {}", gt_norm.chars().take(120).collect::<String>());
    for r in &results {
        println!("{}: {}", r.name, r.text.chars().take(120).collect::<String>());
    }

    println!("\n{}", "=".repeat(100));
    println!("{:<25} {:>10} {:>12} {:>10} {:>12} {:>12}", "Engine", "Time(ms)", "Realtime", "WER", "CER", "Accuracy");
    println!("{}", "-".repeat(100));
    for r in &results {
        println!("{:<25} {:>10} {:>11.1}x {:>9.1}% {:>11.1}% {:>11.1}%",
            r.name, r.ms,
            total_secs * 1000.0 / r.ms as f64,
            r.wer * 100.0, r.cer * 100.0, (1.0 - r.cer) * 100.0);
    }
    println!("{}", "=".repeat(100));
}

fn normalize(text: &str) -> String {
    text.to_lowercase()
        .replace(|c: char| !c.is_alphanumeric() && !c.is_whitespace(), "")
        .split_whitespace().collect::<Vec<_>>().join(" ")
}

fn word_error_rate(r: &[&str], h: &[&str]) -> f64 {
    if r.is_empty() { return if h.is_empty() { 0.0 } else { 1.0 }; }
    levenshtein_generic(r, h) as f64 / r.len() as f64
}

fn char_error_rate(r: &[char], h: &[char]) -> f64 {
    if r.is_empty() { return if h.is_empty() { 0.0 } else { 1.0 }; }
    levenshtein_generic(r, h) as f64 / r.len() as f64
}

fn levenshtein_generic<T: PartialEq>(a: &[T], b: &[T]) -> usize {
    let (m, n) = (a.len(), b.len());
    let mut prev: Vec<usize> = (0..=n).collect();
    let mut curr = vec![0; n + 1];
    for i in 1..=m {
        curr[0] = i;
        for j in 1..=n {
            let cost = if a[i-1] == b[j-1] { 0 } else { 1 };
            curr[j] = (prev[j]+1).min(curr[j-1]+1).min(prev[j-1]+cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[n]
}

struct SrtSeg { start: f64, #[allow(dead_code)] end: f64, text: String }
fn parse_srt(input: &str) -> Vec<SrtSeg> {
    let mut segs = Vec::new();
    let mut lines = input.lines().peekable();
    while lines.peek().is_some() {
        if let Some(l) = lines.next() { if l.trim().parse::<u32>().is_err() && !l.trim().is_empty() { continue; } }
        let ts = match lines.next() { Some(l) if l.contains("-->") => l, _ => continue };
        let parts: Vec<&str> = ts.split("-->").collect();
        if parts.len() != 2 { continue; }
        let (start, end) = (parse_ts(parts[0].trim()), parse_ts(parts[1].trim()));
        let mut text = String::new();
        while let Some(l) = lines.peek() { if l.trim().is_empty() { lines.next(); break; } if !text.is_empty() { text.push(' '); } text.push_str(l.trim()); lines.next(); }
        if !text.is_empty() { segs.push(SrtSeg { start, end, text }); }
    }
    segs
}
fn parse_ts(ts: &str) -> f64 {
    let ts = ts.replace(',', "."); let p: Vec<&str> = ts.split(':').collect();
    if p.len() != 3 { return 0.0; }
    p[0].parse::<f64>().unwrap_or(0.0) * 3600.0 + p[1].parse::<f64>().unwrap_or(0.0) * 60.0 + p[2].parse::<f64>().unwrap_or(0.0)
}
fn load_wav_f32(path: &str) -> Vec<f32> {
    let mut f = std::fs::File::open(path).expect("open wav");
    let mut buf = Vec::new(); f.read_to_end(&mut buf).unwrap();
    let mut off = 12;
    while off + 8 < buf.len() {
        let id = &buf[off..off+4];
        let sz = u32::from_le_bytes([buf[off+4],buf[off+5],buf[off+6],buf[off+7]]) as usize;
        if id == b"data" {
            let s = off+8; let e = (s+sz).min(buf.len());
            return buf[s..e].chunks_exact(2).map(|c| i16::from_le_bytes([c[0],c[1]]) as f32 / 32768.0).collect();
        }
        off += 8 + sz;
    }
    panic!("no data chunk");
}
