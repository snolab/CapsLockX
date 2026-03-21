/// Compare SenseVoice (local) vs Gemini (cloud) STT accuracy and performance.
///
/// Usage: GEMINI_API_KEY=... stt-compare <wav> <srt>
///   or:  GEMINI_API_KEY=... stt-compare --all  (runs all test files in /tmp/stt-bench/)

use capslockx_core::local_sherpa::LocalSherpa;
use capslockx_core::cloud_stt::transcribe_gemini;
use std::io::Read;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let gemini_key = std::env::var("GEMINI_API_KEY").unwrap_or_default();

    if args.get(1).map(|s| s.as_str()) == Some("--all") {
        run_all(&gemini_key);
    } else if args.len() >= 3 {
        run_single(&args[1], &args[2], &gemini_key);
    } else {
        eprintln!("Usage: stt-compare <wav> <srt>");
        eprintln!("   or: stt-compare --all");
        std::process::exit(1);
    }
}

fn run_all(gemini_key: &str) {
    let tests = vec![
        ("English", "/tmp/stt-bench/en.wav", "/tmp/stt-bench/en.en.srt"),
        ("Japanese", "/tmp/stt-bench/ja.wav", "/tmp/stt-bench/ja.ja.srt"),
        ("Korean", "/tmp/stt-bench/ko.wav", "/tmp/stt-bench/ko.ko.srt"),
    ];

    // Also check for second set of test files.
    let tests2 = vec![
        ("English2", "/tmp/stt-bench/en2.wav", "/tmp/stt-bench/en2.en.srt"),
        ("Japanese2", "/tmp/stt-bench/ja2.wav", "/tmp/stt-bench/ja2.ja.srt"),
        ("Korean2", "/tmp/stt-bench/ko2.wav", "/tmp/stt-bench/ko2.ko.srt"),
    ];

    println!("\n{}", "=".repeat(110));
    println!("{:<12} {:>8} {:>8} {:>10} {:>8} {:>8} {:>10} {:>10}",
        "Language", "SV ms", "SV RT", "SV CER%", "Gem ms", "Gem RT", "Gem CER%", "Winner");
    println!("{}", "-".repeat(110));

    let mut sherpa = LocalSherpa::new().expect("load sherpa");

    for (label, wav_path, srt_path) in tests.iter().chain(tests2.iter()) {
        if !std::path::Path::new(wav_path).exists() { continue; }
        if !std::path::Path::new(srt_path).exists() { continue; }

        let samples = load_wav_f32(wav_path);
        let total_secs = samples.len() as f64 / 16000.0;
        let srt = std::fs::read_to_string(srt_path).unwrap_or_default();
        let gt = get_ground_truth(&srt, total_secs);
        let gt_norm = normalize(&gt);
        let gt_chars: Vec<char> = gt_norm.chars().filter(|c| !c.is_whitespace()).collect();

        if gt_chars.is_empty() {
            println!("{:<12} (ground truth empty, skipping)", label);
            continue;
        }

        // SenseVoice (local) — process in 10s chunks.
        let mut sv_texts = Vec::new();
        let mut sv_ms: u128 = 0;
        let chunk_len = (10.0 * 16000.0) as usize;
        let mut off = 0;
        while off < samples.len() {
            let end = (off + chunk_len).min(samples.len());
            let mut chunk = samples[off..end].to_vec();
            if chunk.len() < 16000 { chunk.resize(16000, 0.0); }
            let t0 = std::time::Instant::now();
            sv_texts.push(sherpa.transcribe(&chunk).unwrap_or_default());
            sv_ms += t0.elapsed().as_millis();
            off = end;
        }
        let sv_full = normalize(&sv_texts.join(" "));
        let sv_chars: Vec<char> = sv_full.chars().filter(|c| !c.is_whitespace()).collect();
        let sv_cer = char_error_rate(&gt_chars, &sv_chars);
        let sv_rt = total_secs * 1000.0 / sv_ms as f64;

        // Gemini (cloud) — send full audio at once.
        let (gem_ms, gem_cer) = if !gemini_key.is_empty() {
            let t0 = std::time::Instant::now();
            match transcribe_gemini(&samples, gemini_key) {
                Ok(text) => {
                    let ms = t0.elapsed().as_millis();
                    let gem_norm = normalize(&text);
                    let gem_chars: Vec<char> = gem_norm.chars().filter(|c| !c.is_whitespace()).collect();
                    let cer = char_error_rate(&gt_chars, &gem_chars);
                    (ms, cer)
                }
                Err(e) => {
                    eprintln!("  Gemini error: {}", e);
                    (0, 1.0)
                }
            }
        } else {
            (0, 1.0)
        };
        let gem_rt = if gem_ms > 0 { total_secs * 1000.0 / gem_ms as f64 } else { 0.0 };

        let winner = if gem_cer < sv_cer { "Gemini" } else if sv_cer < gem_cer { "SenseVoice" } else { "Tie" };

        println!("{:<12} {:>6}ms {:>6.1}x {:>9.1}% {:>6}ms {:>6.1}x {:>9.1}% {:>10}",
            label, sv_ms, sv_rt, sv_cer * 100.0, gem_ms, gem_rt, gem_cer * 100.0, winner);

        // Brief delay to avoid rate limiting.
        std::thread::sleep(std::time::Duration::from_millis(500));
    }

    println!("{}", "=".repeat(110));
}

fn run_single(wav_path: &str, srt_path: &str, gemini_key: &str) {
    let samples = load_wav_f32(wav_path);
    let total_secs = samples.len() as f64 / 16000.0;
    let srt = std::fs::read_to_string(srt_path).unwrap_or_default();
    let gt = get_ground_truth(&srt, total_secs);
    let gt_norm = normalize(&gt);

    println!("Audio: {:.1}s | GT: {} chars", total_secs, gt_norm.len());
    println!("GT: {}\n", gt_norm.chars().take(100).collect::<String>());

    // SenseVoice
    let mut sherpa = LocalSherpa::new().expect("load sherpa");
    let t0 = std::time::Instant::now();
    let sv_text = sherpa.transcribe(&samples).unwrap_or_default();
    let sv_ms = t0.elapsed().as_millis();
    let sv_norm = normalize(&sv_text);
    println!("SenseVoice: {}ms | {}", sv_ms, sv_norm.chars().take(100).collect::<String>());

    // Gemini
    if !gemini_key.is_empty() {
        let t0 = std::time::Instant::now();
        match transcribe_gemini(&samples, gemini_key) {
            Ok(text) => {
                let ms = t0.elapsed().as_millis();
                let gem_norm = normalize(&text);
                println!("Gemini:     {}ms | {}", ms, gem_norm.chars().take(100).collect::<String>());
            }
            Err(e) => println!("Gemini error: {}", e),
        }
    }
}

fn get_ground_truth(srt: &str, max_secs: f64) -> String {
    let segs = parse_srt(srt);
    segs.iter()
        .filter(|s| s.0 < max_secs)
        .map(|s| s.1.as_str())
        .collect::<Vec<_>>()
        .join(" ")
}

fn normalize(text: &str) -> String {
    text.to_lowercase()
        .replace(|c: char| !c.is_alphanumeric() && !c.is_whitespace(), "")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn char_error_rate(reference: &[char], hypothesis: &[char]) -> f64 {
    if reference.is_empty() { return if hypothesis.is_empty() { 0.0 } else { 1.0 }; }
    let (m, n) = (reference.len(), hypothesis.len());
    let mut prev: Vec<usize> = (0..=n).collect();
    let mut curr = vec![0; n + 1];
    for i in 1..=m {
        curr[0] = i;
        for j in 1..=n {
            let cost = if reference[i-1] == hypothesis[j-1] { 0 } else { 1 };
            curr[j] = (prev[j]+1).min(curr[j-1]+1).min(prev[j-1]+cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[n] as f64 / reference.len() as f64
}

fn parse_srt(input: &str) -> Vec<(f64, String)> {
    let mut segs = Vec::new();
    let mut lines = input.lines().peekable();
    while lines.peek().is_some() {
        if let Some(l) = lines.next() { if l.trim().parse::<u32>().is_err() && !l.trim().is_empty() { continue; } }
        let ts = match lines.next() { Some(l) if l.contains("-->") => l, _ => continue };
        let parts: Vec<&str> = ts.split("-->").collect();
        if parts.len() != 2 { continue; }
        let start = parse_ts(parts[0].trim());
        let mut text = String::new();
        while let Some(l) = lines.peek() { if l.trim().is_empty() { lines.next(); break; } if !text.is_empty() { text.push(' '); } text.push_str(l.trim()); lines.next(); }
        if !text.is_empty() { segs.push((start, text)); }
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
