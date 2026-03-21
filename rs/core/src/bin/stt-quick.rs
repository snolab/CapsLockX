/// Quick single-file transcription: reads WAV, outputs text to stdout.
/// Usage: stt-quick <file.wav>
use capslockx_core::local_sherpa::LocalSherpa;
use std::io::Read;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 { std::process::exit(1); }
    let samples = load_wav(&args[1]);
    let mut sherpa = LocalSherpa::new().expect("load sherpa");
    let text = sherpa.transcribe(&samples).unwrap_or_default();
    print!("{}", text.trim());
}

fn load_wav(path: &str) -> Vec<f32> {
    let mut f = std::fs::File::open(path).expect("open");
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).unwrap();
    let mut off = 12;
    while off + 8 < buf.len() {
        let id = &buf[off..off+4];
        let sz = u32::from_le_bytes([buf[off+4],buf[off+5],buf[off+6],buf[off+7]]) as usize;
        if id == b"data" {
            let s = off+8;
            let e = (s+sz).min(buf.len());
            return buf[s..e].chunks_exact(2)
                .map(|c| i16::from_le_bytes([c[0],c[1]]) as f32 / 32768.0)
                .collect();
        }
        off += 8 + sz;
    }
    Vec::new()
}
