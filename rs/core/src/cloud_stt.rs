/// Cloud speech-to-text via Gemini API.
///
/// Sends audio (base64-encoded WAV) to Gemini generateContent and gets text back.
/// Simpler than the Live WebSocket API but requires complete audio before sending.

/// Transcribe audio samples using Gemini's audio understanding.
/// `samples`: 16kHz mono f32 PCM. `api_key`: Gemini API key.
/// Returns transcribed text or error.
#[cfg(all(not(target_arch = "wasm32"), feature = "ai"))]
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

#[cfg(any(target_arch = "wasm32", not(feature = "ai")))]
pub fn transcribe_gemini(_samples: &[f32], _api_key: &str) -> Result<String, String> {
    Err("cloud STT disabled (build without `ai` feature, or running on WASM)".into())
}

#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod tests {
    use super::*;

    #[test]
    fn empty_samples_returns_error() {
        let r = transcribe_gemini(&[], "key");
        assert!(r.is_err());
        assert!(r.unwrap_err().contains("empty"));
    }

    #[test]
    fn empty_api_key_returns_error() {
        let r = transcribe_gemini(&[0.1f32, 0.2], "");
        assert!(r.is_err());
        assert!(r.unwrap_err().contains("empty"));
    }

    #[test]
    fn both_empty_returns_error() {
        let r = transcribe_gemini(&[], "");
        assert!(r.is_err());
    }

    #[test]
    fn wav_header_riff_wave() {
        let samples = vec![0.0f32; 100];
        let wav = encode_wav_bytes(&samples, 16000);
        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(&wav[8..12], b"WAVE");
        assert_eq!(&wav[12..16], b"fmt ");
        assert_eq!(&wav[36..40], b"data");
    }

    #[test]
    fn wav_size_matches_samples() {
        let samples = vec![0.5f32; 50];
        let wav = encode_wav_bytes(&samples, 16000);
        assert_eq!(wav.len(), 44 + 50 * 2);
        let data_size = u32::from_le_bytes([wav[40], wav[41], wav[42], wav[43]]);
        assert_eq!(data_size, 100);
    }

    #[test]
    fn wav_sample_rate_encoded() {
        let wav = encode_wav_bytes(&[0.0f32; 4], 16000);
        let sr = u32::from_le_bytes([wav[24], wav[25], wav[26], wav[27]]);
        assert_eq!(sr, 16000);
        let byte_rate = u32::from_le_bytes([wav[28], wav[29], wav[30], wav[31]]);
        assert_eq!(byte_rate, 32000);
    }

    #[test]
    fn wav_clamps_out_of_range() {
        let samples = vec![2.0f32, -2.0f32, 0.0f32];
        let wav = encode_wav_bytes(&samples, 8000);
        let s0 = i16::from_le_bytes([wav[44], wav[45]]);
        let s1 = i16::from_le_bytes([wav[46], wav[47]]);
        let s2 = i16::from_le_bytes([wav[48], wav[49]]);
        assert_eq!(s0, 32767);
        assert_eq!(s1, -32767);
        assert_eq!(s2, 0);
    }

    #[test]
    fn wav_empty_samples() {
        let wav = encode_wav_bytes(&[], 16000);
        assert_eq!(wav.len(), 44);
        assert_eq!(&wav[0..4], b"RIFF");
    }

    #[test]
    fn base64_empty() {
        assert_eq!(base64_encode(&[]), "");
    }

    #[test]
    fn base64_single_byte() {
        assert_eq!(base64_encode(b"f"), "Zg==");
    }

    #[test]
    fn base64_two_bytes() {
        assert_eq!(base64_encode(b"fo"), "Zm8=");
    }

    #[test]
    fn base64_three_bytes() {
        assert_eq!(base64_encode(b"foo"), "Zm9v");
    }

    #[test]
    fn base64_known_vector() {
        assert_eq!(base64_encode(b"Man"), "TWFu");
        assert_eq!(base64_encode(b"hello"), "aGVsbG8=");
        assert_eq!(base64_encode(b"Hello, World!"), "SGVsbG8sIFdvcmxkIQ==");
    }

    #[test]
    fn base64_binary_bytes() {
        let out = base64_encode(&[0xFF, 0x00, 0xAA]);
        assert_eq!(out.len(), 4);
        assert!(out.chars().all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '='));
    }

    #[test]
    fn unreachable_url_returns_request_error() {
        let samples = vec![0.1f32; 1600];
        let result = std::panic::catch_unwind(|| {
            let body = serde_json::json!({"x": 1}).to_string();
            ureq::post("http://127.0.0.1:1/never")
                .set("Content-Type", "application/json")
                .timeout(std::time::Duration::from_millis(200))
                .send_string(&body)
        });
        assert!(result.is_ok());
        let _ = samples;
    }
}
