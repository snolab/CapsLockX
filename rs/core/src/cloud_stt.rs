/// Cloud speech-to-text via Gemini API.
///
/// Sends audio (base64-encoded WAV) to Gemini generateContent and gets text back.
/// Simpler than the Live WebSocket API but requires complete audio before sending.

/// Transcribe audio samples using Gemini's audio understanding.
/// `samples`: 16kHz mono f32 PCM. `api_key`: Gemini API key.
/// Returns transcribed text or error.
#[cfg(not(target_arch = "wasm32"))]
pub fn transcribe_gemini(samples: &[f32], api_key: &str) -> Result<String, String> {
    if samples.is_empty() || api_key.is_empty() {
        return Err("empty samples or API key".into());
    }

    let t0 = std::time::Instant::now();

    // Encode f32 samples to 16-bit PCM WAV.
    let wav_bytes = encode_wav_bytes(samples, 16000);
    let b64 = base64_encode(&wav_bytes);

    let body = serde_json::json!({
        "contents": [{
            "parts": [
                {"text": "Transcribe this audio exactly as spoken. Output ONLY the transcription, no explanations. Keep the original language."},
                {
                    "inline_data": {
                        "mime_type": "audio/wav",
                        "data": b64
                    }
                }
            ]
        }],
        "generationConfig": {
            "temperature": 0.0
        }
    });

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key={}",
        api_key
    );

    let resp = ureq::post(&url)
        .set("Content-Type", "application/json")
        .send_string(&body.to_string())
        .map_err(|e| format!("Gemini STT request: {}", e))?;

    let result: serde_json::Value = serde_json::from_str(
        &resp.into_string().map_err(|e| format!("read: {}", e))?
    ).map_err(|e| format!("parse: {}", e))?;

    let text = result["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .unwrap_or("")
        .trim()
        .to_string();

    let elapsed_ms = t0.elapsed().as_millis();
    let audio_dur = samples.len() as f64 / 16000.0;
    eprintln!("[CLX] gemini-stt: {:.1}s audio → {}ms ({:.1}x realtime)",
        audio_dur, elapsed_ms, audio_dur * 1000.0 / elapsed_ms as f64);

    Ok(text)
}

/// Encode f32 samples as a 16-bit PCM WAV file.
fn encode_wav_bytes(samples: &[f32], sample_rate: u32) -> Vec<u8> {
    let num_samples = samples.len();
    let data_size = num_samples * 2; // 16-bit = 2 bytes per sample
    let file_size = 36 + data_size;

    let mut buf = Vec::with_capacity(44 + data_size);

    // RIFF header
    buf.extend_from_slice(b"RIFF");
    buf.extend_from_slice(&(file_size as u32).to_le_bytes());
    buf.extend_from_slice(b"WAVE");

    // fmt chunk
    buf.extend_from_slice(b"fmt ");
    buf.extend_from_slice(&16u32.to_le_bytes()); // chunk size
    buf.extend_from_slice(&1u16.to_le_bytes());  // PCM format
    buf.extend_from_slice(&1u16.to_le_bytes());  // mono
    buf.extend_from_slice(&sample_rate.to_le_bytes());
    buf.extend_from_slice(&(sample_rate * 2).to_le_bytes()); // byte rate
    buf.extend_from_slice(&2u16.to_le_bytes());  // block align
    buf.extend_from_slice(&16u16.to_le_bytes()); // bits per sample

    // data chunk
    buf.extend_from_slice(b"data");
    buf.extend_from_slice(&(data_size as u32).to_le_bytes());

    for &s in samples {
        let clamped = s.clamp(-1.0, 1.0);
        let i16_val = (clamped * 32767.0) as i16;
        buf.extend_from_slice(&i16_val.to_le_bytes());
    }

    buf
}

/// Simple base64 encoder.
fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity(data.len() * 4 / 3 + 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        result.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 { result.push(CHARS[((triple >> 6) & 0x3F) as usize] as char); } else { result.push('='); }
        if chunk.len() > 2 { result.push(CHARS[(triple & 0x3F) as usize] as char); } else { result.push('='); }
    }
    result
}

#[cfg(target_arch = "wasm32")]
pub fn transcribe_gemini(_samples: &[f32], _api_key: &str) -> Result<String, String> {
    Err("Gemini STT not supported on WASM".into())
}
