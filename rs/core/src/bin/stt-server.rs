/// Persistent STT server: reads WAV file paths from stdin, outputs JSON per line.
/// Usage: echo "/tmp/chunk.wav" | stt-server
use capslockx_core::local_sherpa::LocalSherpa;
use std::io::{BufRead, Read};

fn main() {
    eprintln!("[stt-server] Loading SenseVoice...");
    let mut sherpa = LocalSherpa::new().expect("load sherpa");
    eprintln!("[stt-server] Ready. Send WAV paths on stdin.");

    let stdin = std::io::stdin();
    for line in stdin.lock().lines() {
        let path = match line { Ok(l) => l, Err(_) => break };
        let path = path.trim();
        if path.is_empty() { continue; }
        let t0 = std::time::Instant::now();
        let samples = load_wav(path);
        if samples.is_empty() { println!("{{\"text\":\"\",\"ms\":0}}"); continue; }
        let text = sherpa.transcribe(&samples).unwrap_or_default();
        let ms = t0.elapsed().as_millis();
        let dur = samples.len() as f64 / 16000.0;
        eprintln!("[stt-server] {:.1}s → {}ms: {:?}", dur, ms, text.trim());
        // JSON output
        let escaped = text.trim().replace('\\', "\\\\").replace('"', "\\\"");
        println!("{{\"text\":\"{}\",\"ms\":{}}}", escaped, ms);
    }
}

fn load_wav(path: &str) -> Vec<f32> {
    let mut f = match std::fs::File::open(path) { Ok(f) => f, Err(_) => return Vec::new() };
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).unwrap();
    let mut off = 12;
    while off + 8 < buf.len() {
        let id = &buf[off..off+4];
        let sz = u32::from_le_bytes([buf[off+4],buf[off+5],buf[off+6],buf[off+7]]) as usize;
        if id == b"data" {
            let s = off+8; let e = (s+sz).min(buf.len());
            return buf[s..e].chunks_exact(2)
                .map(|c| i16::from_le_bytes([c[0],c[1]]) as f32 / 32768.0).collect();
        }
        off += 8 + sz;
    }
    Vec::new()
}
