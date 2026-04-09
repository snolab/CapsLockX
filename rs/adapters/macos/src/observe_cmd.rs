//! `clx observe` — capture screen region + optional Gemini vision description.
//!
//! Usage:
//!   clx observe                                          # full screen, 3s, Gemini describe
//!   clx observe --region 450,40,900,300 --duration 5s   # specific region
//!   clx observe --window voice-standalone --duration 3s  # capture by window name
//!   clx observe --prompt "what is on screen?"            # custom prompt
//!   clx observe --output ./tmp/frames/                   # save frames only (no LLM)

use std::ffi::c_void;

extern "C" {
    fn objc_getClass(name: *const std::ffi::c_char) -> *mut c_void;
    fn sel_registerName(name: *const std::ffi::c_char) -> *mut c_void;
    fn objc_msgSend(receiver: *mut c_void, sel: *mut c_void, ...) -> *mut c_void;
}

pub fn main(args: &[String]) {
    let mut region: Option<(i32, i32, i32, i32)> = None;
    let mut duration_secs: f64 = 3.0;
    let mut fps: f64 = 2.0;
    let mut prompt = "Describe what you see in these screenshots. Note any text, UI elements, and changes between frames.".to_string();
    let mut output_dir: Option<String> = None;
    let mut window_name: Option<String> = None;

    // Parse args.
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--region" | "-r" if i + 1 < args.len() => {
                i += 1;
                let parts: Vec<i32> = args[i].split(',').filter_map(|s| s.trim().parse().ok()).collect();
                if parts.len() == 4 {
                    region = Some((parts[0], parts[1], parts[2], parts[3]));
                } else {
                    eprintln!("Error: --region expects x,y,w,h (e.g., 450,40,900,300)");
                    return;
                }
            }
            "--window" | "-w" if i + 1 < args.len() => {
                i += 1;
                window_name = Some(args[i].clone());
            }
            "--duration" | "-d" if i + 1 < args.len() => {
                i += 1;
                duration_secs = parse_duration(&args[i]);
            }
            "--fps" if i + 1 < args.len() => {
                i += 1;
                fps = args[i].parse().unwrap_or(2.0);
            }
            "--prompt" | "-p" if i + 1 < args.len() => {
                i += 1;
                prompt = args[i].clone();
            }
            "--output" | "-o" if i + 1 < args.len() => {
                i += 1;
                output_dir = Some(args[i].clone());
            }
            "--help" | "-h" => {
                print_usage();
                return;
            }
            _ => {
                // Treat as prompt if not a flag.
                if !args[i].starts_with('-') {
                    prompt = args[i..].join(" ");
                    break;
                }
            }
        }
        i += 1;
    }

    // Resolve window name to region.
    if let Some(ref name) = window_name {
        match find_window_region(name) {
            Some(r) => {
                eprintln!("[observe] found window {:?}: region={},{},{},{}", name, r.0, r.1, r.2, r.3);
                region = Some(r);
            }
            None => {
                eprintln!("[observe] window {:?} not found, using full screen", name);
            }
        }
    }

    let num_frames = (duration_secs * fps).ceil() as usize;
    let frame_interval = std::time::Duration::from_secs_f64(1.0 / fps);
    eprintln!("[observe] capturing {} frames over {:.1}s ({:.0} fps), region={:?}",
        num_frames, duration_secs, fps, region);

    // Capture frames.
    let tmp_dir = std::path::PathBuf::from("/tmp/clx-observe");
    let _ = std::fs::create_dir_all(&tmp_dir);

    let mut frame_paths = Vec::new();
    for frame_i in 0..num_frames {
        let path = tmp_dir.join(format!("frame-{:04}.jpg", frame_i));
        let path_str = path.to_str().unwrap_or("/tmp/clx-observe/frame.jpg");

        let status = if let Some((x, y, w, h)) = region {
            std::process::Command::new("screencapture")
                .args(["-x", "-t", "jpg", "-R", &format!("{},{},{},{}", x, y, w, h), path_str])
                .status()
        } else {
            std::process::Command::new("screencapture")
                .args(["-x", "-t", "jpg", path_str])
                .status()
        };

        if status.map(|s| s.success()).unwrap_or(false) {
            // Resize for token efficiency.
            let _ = std::process::Command::new("sips")
                .args(["--resampleWidth", "512", path_str, "--out", path_str])
                .stderr(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .output();
            frame_paths.push(path);
        }

        if frame_i + 1 < num_frames {
            std::thread::sleep(frame_interval);
        }
    }

    eprintln!("[observe] captured {} frames", frame_paths.len());

    // If --output, copy frames and exit.
    if let Some(ref out_dir) = output_dir {
        let _ = std::fs::create_dir_all(out_dir);
        for (i, path) in frame_paths.iter().enumerate() {
            let dest = format!("{}/frame-{:04}.jpg", out_dir, i);
            let _ = std::fs::copy(path, &dest);
        }
        eprintln!("[observe] saved {} frames to {}", frame_paths.len(), out_dir);
        cleanup(&tmp_dir);
        return;
    }

    // Load LLM config.
    let config = match load_llm_config() {
        Some(c) => c,
        None => {
            eprintln!("[observe] no LLM API key found (set GEMINI_API_KEY or configure in CapsLockX prefs)");
            // Still show frame paths.
            for p in &frame_paths { eprintln!("  {}", p.display()); }
            cleanup(&tmp_dir);
            return;
        }
    };

    eprintln!("[observe] sending {} frames to {} for analysis...", frame_paths.len(), config.model);

    // Build Gemini vision message.
    let mut parts: Vec<serde_json::Value> = Vec::new();
    parts.push(serde_json::json!({"text": format!(
        "{}\n\n({} frames captured over {:.1}s at {:.0} fps)",
        prompt, frame_paths.len(), duration_secs, fps
    )}));
    for path in &frame_paths {
        if let Ok(data) = std::fs::read(path) {
            parts.push(serde_json::json!({
                "inlineData": {
                    "mimeType": "image/jpeg",
                    "data": base64_encode(&data)
                }
            }));
        }
    }

    let conversation = vec![serde_json::json!({"role": "user", "parts": parts})];

    let system_prompt = "You are a screen observer. Describe what you see accurately and concisely. \
        If there are multiple frames, note any changes between them. Focus on text content, UI state, \
        and any notable visual elements.";

    match stream_gemini_vision(&config, system_prompt, &conversation, &mut |token| {
        print!("{}", token);
        use std::io::Write;
        let _ = std::io::stdout().flush();
    }) {
        Ok(_) => println!(),
        Err(e) => eprintln!("\n[observe] LLM error: {}", e),
    }

    cleanup(&tmp_dir);
}

fn print_usage() {
    eprintln!("clx observe — capture screen region + Gemini vision description");
    eprintln!();
    eprintln!("Usage: clx observe [options] [prompt]");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  --region x,y,w,h    Capture specific screen region");
    eprintln!("  --window NAME       Capture a window by process name");
    eprintln!("  --duration Ns       Capture duration (default: 3s)");
    eprintln!("  --fps N             Frames per second (default: 2)");
    eprintln!("  --prompt TEXT       Custom prompt for Gemini");
    eprintln!("  --output DIR        Save frames to directory (no LLM)");
    eprintln!("  --help              Show this help");
}

fn parse_duration(s: &str) -> f64 {
    let s = s.trim();
    if let Some(n) = s.strip_suffix('s') { return n.parse().unwrap_or(3.0); }
    if let Some(n) = s.strip_suffix("ms") { return n.parse::<f64>().unwrap_or(3000.0) / 1000.0; }
    s.parse().unwrap_or(3.0)
}

fn cleanup(dir: &std::path::Path) {
    let _ = std::fs::remove_dir_all(dir);
}

// ── Window lookup ───────────────────────────────────────────────────────────

fn find_window_region(name: &str) -> Option<(i32, i32, i32, i32)> {
    // Use Python + Quartz to find window by owner name (most reliable).
    let script = format!(
        r#"
import Quartz, json
windows = Quartz.CGWindowListCopyWindowInfo(Quartz.kCGWindowListOptionAll, Quartz.kCGNullWindowID)
for w in windows:
    owner = w.get('kCGWindowOwnerName', '')
    if '{}' in owner.lower():
        b = w['kCGWindowBounds']
        print(json.dumps([int(b['X']), int(b['Y']), int(b['Width']), int(b['Height'])]))
        break
"#, name.to_lowercase()
    );

    let output = std::process::Command::new("python3")
        .arg("-c").arg(&script)
        .output().ok()?;

    let s = String::from_utf8_lossy(&output.stdout);
    let arr: Vec<i32> = serde_json::from_str(s.trim()).ok()?;
    if arr.len() == 4 { Some((arr[0], arr[1], arr[2], arr[3])) } else { None }
}

// ── LLM config ──────────────────────────────────────────────────────────────

fn load_llm_config() -> Option<capslockx_core::llm_client::LlmConfig> {
    // Try config.json.
    let cfg_path = dirs::config_dir()?.join("CapsLockX").join("config.json");
    let data = std::fs::read_to_string(&cfg_path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&data).ok()?;

    let mut api_key = String::new();
    // Check per-provider keys first.
    for key in ["gemini_api_key", "openai_api_key", "anthropic_api_key", "llm_api_key"] {
        if let Some(k) = v.get(key).and_then(|k| k.as_str()) {
            if !k.is_empty() { api_key = k.to_string(); break; }
        }
    }
    // Fall back to env vars.
    if api_key.is_empty() {
        api_key = std::env::var("GEMINI_API_KEY")
            .or_else(|_| std::env::var("OPENAI_API_KEY"))
            .unwrap_or_default();
    }
    if api_key.is_empty() { return None; }

    let model = v.get("llm_model").and_then(|k| k.as_str()).unwrap_or("").to_string();
    Some(capslockx_core::llm_client::LlmConfig::from_key_and_model(&api_key, &model))
}

// ── Gemini vision streaming ─────────────────────────────────────────────────

fn stream_gemini_vision(
    config: &capslockx_core::llm_client::LlmConfig,
    system_prompt: &str,
    conversation: &[serde_json::Value],
    on_token: &mut dyn FnMut(&str),
) -> Result<String, String> {
    use std::io::{BufRead, BufReader};

    let mut body = serde_json::json!({ "contents": conversation });
    body["systemInstruction"] = serde_json::json!({"parts": [{"text": system_prompt}]});

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:streamGenerateContent?alt=sse&key={}",
        config.model, config.api_key
    );

    let resp = ureq::post(&url)
        .set("Content-Type", "application/json")
        .send_string(&body.to_string())
        .map_err(|e| format!("Gemini request: {}", e))?;

    let reader = BufReader::new(resp.into_reader());
    let mut full = String::new();

    for line in reader.lines() {
        let line = line.map_err(|e| format!("read: {}", e))?;
        if let Some(data) = line.strip_prefix("data: ") {
            if let Ok(chunk) = serde_json::from_str::<serde_json::Value>(data) {
                if let Some(text) = chunk["candidates"][0]["content"]["parts"][0]["text"].as_str() {
                    full.push_str(text);
                    on_token(text);
                }
            }
        }
    }
    Ok(full)
}

// ── Base64 encoding ─────────────────────────────────────────────────────────

fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(data.len() * 4 / 3 + 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(CHARS[((n >> 18) & 0x3F) as usize] as char);
        out.push(CHARS[((n >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 { out.push(CHARS[((n >> 6) & 0x3F) as usize] as char); } else { out.push('='); }
        if chunk.len() > 2 { out.push(CHARS[(n & 0x3F) as usize] as char); } else { out.push('='); }
    }
    out
}
