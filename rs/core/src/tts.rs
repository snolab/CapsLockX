/// Text-to-speech with fallback chain:
///   ElevenLabs (if key) → Gemini (if key) → OpenAI (if key) → msedge-tts → native (macOS say)
///
/// All backends output audio to the system default output device.

/// Speak text aloud. Tries each backend in order until one succeeds.
/// `lang` is a BCP-47 hint like "en", "ja", "zh", "ko".
#[cfg(not(target_arch = "wasm32"))]
pub fn speak(text: &str, lang: &str, elevenlabs_key: &str, gemini_key: &str, openai_key: &str) -> Result<(), String> {
    if text.trim().is_empty() { return Ok(()); }

    // Tier 1: ElevenLabs
    if !elevenlabs_key.is_empty() {
        match speak_elevenlabs(text, lang, elevenlabs_key) {
            Ok(()) => return Ok(()),
            Err(e) => eprintln!("[CLX] tts: ElevenLabs failed: {}, trying next", e),
        }
    }

    // Tier 2: Gemini TTS
    if !gemini_key.is_empty() {
        match speak_gemini(text, lang, gemini_key) {
            Ok(()) => return Ok(()),
            Err(e) => eprintln!("[CLX] tts: Gemini failed: {}, trying next", e),
        }
    }

    // Tier 3: OpenAI TTS
    if !openai_key.is_empty() {
        match speak_openai(text, lang, openai_key) {
            Ok(()) => return Ok(()),
            Err(e) => eprintln!("[CLX] tts: OpenAI failed: {}, trying next", e),
        }
    }

    // Tier 4: msedge-tts (free, needs internet)
    match speak_msedge(text, lang) {
        Ok(()) => return Ok(()),
        Err(e) => eprintln!("[CLX] tts: msedge-tts failed: {}, trying native", e),
    }

    // Tier 5: Native (macOS say)
    speak_native(text, lang)
}

// ── ElevenLabs ───────────────────────────────────────────────────────────────

#[cfg(not(target_arch = "wasm32"))]
fn speak_elevenlabs(text: &str, _lang: &str, api_key: &str) -> Result<(), String> {
    eprintln!("[CLX] tts: ElevenLabs ({} chars)", text.len());
    let body = serde_json::json!({
        "text": text,
        "model_id": "eleven_turbo_v2_5",
    });

    let resp = ureq::post("https://api.elevenlabs.io/v1/text-to-speech/21m00Tcm4TlvDq8ikWAM") // Rachel voice
        .set("xi-api-key", api_key)
        .set("Content-Type", "application/json")
        .set("Accept", "audio/mpeg")
        .send_string(&body.to_string())
        .map_err(|e| format!("{}", e))?;

    play_audio_response(resp)
}

// ── Gemini TTS ───────────────────────────────────────────────────────────────

#[cfg(not(target_arch = "wasm32"))]
fn speak_gemini(text: &str, lang: &str, api_key: &str) -> Result<(), String> {
    eprintln!("[CLX] tts: Gemini ({} chars, lang={})", text.len(), lang);

    let voice = match lang {
        "ja" => "Aoede",
        "zh" => "Aoede",
        "ko" => "Aoede",
        _ => "Zephyr",
    };

    let body = serde_json::json!({
        "contents": [{"parts": [{"text": text}]}],
        "generationConfig": {
            "responseModalities": ["AUDIO"],
            "speechConfig": {
                "voiceConfig": {
                    "prebuiltVoiceConfig": {"voiceName": voice}
                }
            }
        }
    });

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash-preview-tts:generateContent?key={}",
        api_key
    );

    let resp = ureq::post(&url)
        .set("Content-Type", "application/json")
        .send_string(&body.to_string())
        .map_err(|e| format!("{}", e))?;

    // Gemini returns JSON with inline_data containing base64 audio.
    let result: serde_json::Value = serde_json::from_str(
        &resp.into_string().map_err(|e| format!("read: {}", e))?
    ).map_err(|e| format!("parse: {}", e))?;

    let audio_b64 = result["candidates"][0]["content"]["parts"][0]["inlineData"]["data"]
        .as_str()
        .ok_or("no audio data in Gemini response")?;

    let audio_bytes = base64_decode(audio_b64)?;
    play_audio_bytes(&audio_bytes, "wav")
}

// ── OpenAI TTS ───────────────────────────────────────────────────────────────

#[cfg(not(target_arch = "wasm32"))]
fn speak_openai(text: &str, _lang: &str, api_key: &str) -> Result<(), String> {
    eprintln!("[CLX] tts: OpenAI ({} chars)", text.len());
    let body = serde_json::json!({
        "model": "tts-1",
        "input": text,
        "voice": "nova",
        "response_format": "mp3",
    });

    let resp = ureq::post("https://api.openai.com/v1/audio/speech")
        .set("Authorization", &format!("Bearer {}", api_key))
        .set("Content-Type", "application/json")
        .send_string(&body.to_string())
        .map_err(|e| format!("{}", e))?;

    play_audio_response(resp)
}

// ── msedge-tts (free, no API key) ────────────────────────────────────────────

#[cfg(not(target_arch = "wasm32"))]
fn speak_msedge(text: &str, lang: &str) -> Result<(), String> {
    eprintln!("[CLX] tts: msedge-tts ({} chars, lang={})", text.len(), lang);

    let voice = match lang {
        "ja" => "ja-JP-NanamiNeural",
        "zh" => "zh-CN-XiaoxiaoNeural",
        "ko" => "ko-KR-SunHiNeural",
        _ => "en-US-AriaNeural",
    };

    // Run synchronously (msedge-tts 0.3 has a sync client).
    edge_tts_sync(text, voice)
}

#[cfg(not(target_arch = "wasm32"))]
fn edge_tts_sync(text: &str, voice: &str) -> Result<(), String> {
    use msedge_tts::tts::client::connect;
    use msedge_tts::tts::SpeechConfig;

    let mut tts = connect().map_err(|e| format!("connect: {}", e))?;
    let config = SpeechConfig {
        voice_name: voice.to_string(),
        audio_format: "audio-24khz-48kbitrate-mono-mp3".to_string(),
        pitch: 0,
        rate: 0,
        volume: 0,
    };
    let audio = tts.synthesize(text, &config).map_err(|e| format!("synth: {}", e))?;

    let path = "/tmp/clx-tts-edge.mp3";
    std::fs::write(path, &audio.audio_bytes).map_err(|e| format!("write: {}", e))?;
    play_audio_file(path)
}

// ── Native TTS (macOS say) ───────────────────────────────────────────────────

#[cfg(not(target_arch = "wasm32"))]
fn speak_native(text: &str, lang: &str) -> Result<(), String> {
    eprintln!("[CLX] tts: native say ({} chars, lang={})", text.len(), lang);

    let voice = match lang {
        "ja" => "Kyoko",
        "zh" => "Ting-Ting",
        "ko" => "Yuna",
        _ => "Samantha",
    };

    let status = std::process::Command::new("say")
        .args(["-v", voice, text])
        .status()
        .map_err(|e| format!("say: {}", e))?;

    if status.success() { Ok(()) } else { Err(format!("say exit: {}", status)) }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

#[cfg(not(target_arch = "wasm32"))]
fn play_audio_response(resp: ureq::Response) -> Result<(), String> {
    let mut reader = resp.into_reader();
    let path = "/tmp/clx-tts-output.mp3";
    let mut file = std::fs::File::create(path).map_err(|e| format!("create: {}", e))?;
    std::io::copy(&mut reader, &mut file).map_err(|e| format!("write: {}", e))?;
    play_audio_file(path)
}

#[cfg(not(target_arch = "wasm32"))]
fn play_audio_bytes(data: &[u8], ext: &str) -> Result<(), String> {
    let path = format!("/tmp/clx-tts-output.{}", ext);
    std::fs::write(&path, data).map_err(|e| format!("write: {}", e))?;
    play_audio_file(&path)
}

#[cfg(not(target_arch = "wasm32"))]
fn play_audio_file(path: &str) -> Result<(), String> {
    // afplay on macOS, aplay on Linux, start on Windows.
    #[cfg(target_os = "macos")]
    let status = std::process::Command::new("afplay").arg(path).status();
    #[cfg(target_os = "linux")]
    let status = std::process::Command::new("aplay").arg(path).status();
    #[cfg(target_os = "windows")]
    let status = std::process::Command::new("cmd").args(["/c", "start", "/b", path]).status();

    match status {
        Ok(s) if s.success() => Ok(()),
        Ok(s) => Err(format!("player exit: {}", s)),
        Err(e) => Err(format!("player: {}", e)),
    }
}

fn base64_decode(input: &str) -> Result<Vec<u8>, String> {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = Vec::new();
    let bytes: Vec<u8> = input.bytes().filter(|b| *b != b'\n' && *b != b'\r' && *b != b' ').collect();
    for chunk in bytes.chunks(4) {
        if chunk.len() < 2 { break; }
        let a = CHARS.iter().position(|&c| c == chunk[0]).unwrap_or(0) as u32;
        let b = CHARS.iter().position(|&c| c == chunk[1]).unwrap_or(0) as u32;
        let c = if chunk.len() > 2 && chunk[2] != b'=' { CHARS.iter().position(|&x| x == chunk[2]).unwrap_or(0) as u32 } else { 0 };
        let d = if chunk.len() > 3 && chunk[3] != b'=' { CHARS.iter().position(|&x| x == chunk[3]).unwrap_or(0) as u32 } else { 0 };
        let triple = (a << 18) | (b << 12) | (c << 6) | d;
        out.push((triple >> 16) as u8);
        if chunk.len() > 2 && chunk[2] != b'=' { out.push((triple >> 8) as u8); }
        if chunk.len() > 3 && chunk[3] != b'=' { out.push(triple as u8); }
    }
    Ok(out)
}

// ── WASM stub ────────────────────────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
pub fn speak(_text: &str, _lang: &str, _elevenlabs_key: &str, _gemini_key: &str, _openai_key: &str) -> Result<(), String> {
    // In WASM, use Web Speech API via JS interop.
    Err("TTS on WASM: use window.speechSynthesis from JS".into())
}
