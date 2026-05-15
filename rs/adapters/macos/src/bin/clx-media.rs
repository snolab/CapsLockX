//! clx-media — Voice Note Media Browser
//!
//! Starts a local HTTP server on 127.0.0.1:7842 and opens the default browser.
//! Files are served from ~/.capslockx/voice/
//!
//! Usage:
//!   clx media          (via main clx binary)
//!   clx-media          (standalone)

use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::thread;

const PORT: u16 = 7842;

const HTML: &str = r#"<!DOCTYPE html>
<html><head><meta charset="utf-8"><title>CLX Voice Notes</title>
<style>
* { box-sizing: border-box; margin: 0; padding: 0; }
body { background: #1e1e2e; color: #cdd6f4; font-family: system-ui; display: flex; height: 100vh; overflow: hidden; }
#sidebar { width: 260px; border-right: 1px solid #313244; display: flex; flex-direction: column; flex-shrink: 0; }
#sidebar-header { padding: 14px 16px; color: #89b4fa; font-size: 13px; font-weight: 600; border-bottom: 1px solid #313244; flex-shrink: 0; }
#list { overflow-y: auto; flex: 1; }
.session { padding: 10px 14px; cursor: pointer; border-bottom: 1px solid #313244; }
.session:hover { background: #313244; }
.session.active { background: #45475a; }
.session .ts { font-size: 11px; color: #6c7086; }
.session .label { font-size: 12px; margin-top: 3px; }
#main { flex: 1; display: flex; flex-direction: column; overflow: hidden; }
#player-header { padding: 14px 20px; border-bottom: 1px solid #313244; font-size: 12px; color: #6c7086; flex-shrink: 0; min-height: 44px; }
#player-wrap { padding: 16px 20px; border-bottom: 1px solid #313244; flex-shrink: 0; }
video { width: 100%; height: 72px; background: #181825; border-radius: 8px; display: block; }
#transcript-wrap { flex: 1; overflow: hidden; display: flex; flex-direction: column; padding: 0 20px 20px; }
#transcript-label { font-size: 11px; color: #6c7086; padding: 12px 0 6px; flex-shrink: 0; }
#transcript { flex: 1; overflow-y: auto; background: #181825; border-radius: 8px; padding: 14px; font-size: 13px; line-height: 1.65; white-space: pre-wrap; color: #a6adc8; }
#empty { margin: auto; color: #6c7086; font-size: 13px; }
::cue { background: rgba(0,0,0,0.75); color: #fff; font-size: 13px; }
</style></head>
<body>
<div id="sidebar">
  <div id="sidebar-header">🎤 Voice Notes</div>
  <div id="list"><div style="padding:14px;color:#6c7086;font-size:12px">Loading…</div></div>
</div>
<div id="main">
  <div id="player-header"><span id="session-label">← Select a voice note</span></div>
  <div id="player-wrap" style="display:none"><video id="player" controls></video></div>
  <div id="transcript-wrap">
    <div id="transcript-label" style="display:none">Transcript</div>
    <div id="transcript" style="display:none"></div>
    <div id="empty">No session selected</div>
  </div>
</div>
<script>
const player = document.getElementById('player');
let sessions = [];

async function load() {
  const r = await fetch('/api/sessions');
  sessions = await r.json();
  const list = document.getElementById('list');
  if (!sessions.length) {
    list.innerHTML = '<div style="padding:14px;color:#6c7086;font-size:12px">No voice notes yet.<br>Hold Space+V to record.</div>';
    return;
  }
  list.innerHTML = sessions.map((s, i) => `
    <div class="session" onclick="play(${i})" id="s${i}">
      <div class="ts">${s.date}</div>
      <div class="label">${s.has_srt ? '📝' : '🎵'} ${s.name}</div>
    </div>`).join('');
}

async function play(i) {
  document.querySelectorAll('.session').forEach(e => e.classList.remove('active'));
  document.getElementById('s' + i).classList.add('active');
  const s = sessions[i];

  document.getElementById('session-label').textContent = s.date + ' — ' + s.name;
  document.getElementById('player-wrap').style.display = '';
  document.getElementById('empty').style.display = 'none';
  document.getElementById('transcript-label').style.display = '';
  document.getElementById('transcript').style.display = '';

  // Reset player tracks
  while (player.firstChild) player.removeChild(player.firstChild);
  player.src = '/files/' + encodeURIComponent(s.webm);

  if (s.has_srt) {
    const r = await fetch('/files/' + encodeURIComponent(s.srt));
    const srt = await r.text();
    // Embed as VTT subtitle track
    const vtt = 'WEBVTT\n\n' + srt.replace(/(\d\d:\d\d:\d\d),(\d\d\d)/g, '$1.$2');
    const blob = new Blob([vtt], { type: 'text/vtt' });
    const track = document.createElement('track');
    track.kind = 'subtitles'; track.label = 'Subtitles';
    track.srclang = 'en'; track.default = true;
    track.src = URL.createObjectURL(blob);
    player.appendChild(track);
    player.addEventListener('loadedmetadata', () => {
      if (player.textTracks[0]) player.textTracks[0].mode = 'showing';
    }, { once: true });
    // Show as transcript text (strip cue numbers and timestamps)
    document.getElementById('transcript').textContent = srt
      .replace(/^\d+\s*$/mg, '')
      .replace(/^\d\d:\d\d:\d\d,\d+ --> .+$/mg, '')
      .replace(/\n{3,}/g, '\n\n')
      .trim();
  } else {
    document.getElementById('transcript').textContent = '(no transcript)';
  }
  player.play().catch(() => {});
}

load();
</script>
</body></html>
"#;

fn voice_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/"))
        .join(".capslockx")
        .join("voice")
}

pub fn main() {
    let addr = format!("127.0.0.1:{PORT}");
    let listener = TcpListener::bind(&addr).unwrap_or_else(|e| {
        eprintln!("[clx-media] bind failed on {addr}: {e}");
        std::process::exit(1);
    });
    println!("[clx-media] Voice note browser at http://localhost:{PORT}");

    // Open browser
    let _ = std::process::Command::new("open")
        .arg(format!("http://localhost:{PORT}"))
        .spawn();

    for stream in listener.incoming() {
        if let Ok(s) = stream {
            thread::spawn(move || handle_connection(s));
        }
    }
}

fn handle_connection(stream: TcpStream) {
    let mut reader = BufReader::new(&stream);

    let mut req_line = String::new();
    if reader.read_line(&mut req_line).is_err() { return; }

    // Read headers, capture Range if present
    let mut range_header: Option<String> = None;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).is_err() { break; }
        let trimmed = line.trim();
        if trimmed.is_empty() { break; }
        if let Some(v) = trimmed.strip_prefix("Range: ") {
            range_header = Some(v.trim().to_string());
        }
    }

    let parts: Vec<&str> = req_line.trim().splitn(3, ' ').collect();
    if parts.len() < 2 { return; }
    let raw_path = parts[1];
    let path = url_decode(raw_path.split('?').next().unwrap_or(raw_path));

    let mut w = &stream;
    match path.as_str() {
        "/" => {
            let body = HTML.as_bytes();
            let _ = write!(
                w,
                "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\n\r\n",
                body.len()
            );
            let _ = w.write_all(body);
        }
        "/api/sessions" => serve_sessions(w),
        p if p.starts_with("/files/") => {
            serve_file(w, &p[7..], range_header.as_deref());
        }
        _ => {
            let _ = w.write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n");
        }
    }
}

fn serve_sessions(mut w: &TcpStream) {
    let dir = voice_dir();
    let mut webms: Vec<String> = Vec::new();

    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".webm") {
                webms.push(name);
            }
        }
    }

    webms.sort_by(|a, b| b.cmp(a)); // newest first

    let items: Vec<String> = webms
        .iter()
        .map(|webm| {
            let ts = webm.trim_end_matches(".webm");
            let srt = format!("{ts}.srt");
            let has_srt = dir.join(&srt).exists();
            // Format for display: "20260402_153000" → "2026-04-02 15:30:00" (best-effort)
            let date = ts.replacen('T', " ", 1);
            format!(
                r#"{{"name":"{ts}","webm":"{webm}","srt":"{srt}","has_srt":{has_srt},"date":"{date}"}}"#
            )
        })
        .collect();

    let body = format!("[{}]", items.join(","));
    let body = body.as_bytes();
    let _ = write!(
        w,
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n",
        body.len()
    );
    let _ = w.write_all(body);
}

fn serve_file(mut w: &TcpStream, name: &str, range: Option<&str>) {
    // Prevent path traversal
    if name.contains("..") || name.contains('/') || name.contains('\\') {
        let _ = w.write_all(b"HTTP/1.1 403 Forbidden\r\nContent-Length: 0\r\n\r\n");
        return;
    }

    let path = voice_dir().join(name);
    let mime = if name.ends_with(".webm") {
        "video/webm"
    } else if name.ends_with(".srt") {
        "text/plain; charset=utf-8"
    } else {
        "application/octet-stream"
    };

    let mut file = match File::open(&path) {
        Ok(f) => f,
        Err(_) => {
            let _ = w.write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n");
            return;
        }
    };

    let file_size = path.metadata().map(|m| m.len()).unwrap_or(0);

    if let Some(range_str) = range {
        if let Some(range_val) = range_str.strip_prefix("bytes=") {
            let mut parts = range_val.splitn(2, '-');
            let start: u64 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
            let end: u64 = parts
                .next()
                .and_then(|s| s.parse().ok())
                .unwrap_or(file_size.saturating_sub(1));
            let end = end.min(file_size.saturating_sub(1));
            let length = end.saturating_sub(start) + 1;

            if file.seek(SeekFrom::Start(start)).is_ok() {
                let mut buf = vec![0u8; length as usize];
                let read = file.read(&mut buf).unwrap_or(0);
                buf.truncate(read);
                let _ = write!(
                    w,
                    "HTTP/1.1 206 Partial Content\r\nContent-Type: {mime}\r\nContent-Range: bytes {start}-{end}/{file_size}\r\nContent-Length: {}\r\nAccept-Ranges: bytes\r\n\r\n",
                    buf.len()
                );
                let _ = w.write_all(&buf);
            }
            return;
        }
    }

    // Full response
    let mut buf = Vec::with_capacity(file_size as usize);
    let _ = file.read_to_end(&mut buf);
    let _ = write!(
        w,
        "HTTP/1.1 200 OK\r\nContent-Type: {mime}\r\nContent-Length: {}\r\nAccept-Ranges: bytes\r\n\r\n",
        buf.len()
    );
    let _ = w.write_all(&buf);
}

fn url_decode(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(hex) = std::str::from_utf8(&bytes[i + 1..i + 3]) {
                if let Ok(b) = u8::from_str_radix(hex, 16) {
                    result.push(b as char);
                    i += 3;
                    continue;
                }
            }
        } else if bytes[i] == b'+' {
            result.push(' ');
            i += 1;
            continue;
        }
        result.push(bytes[i] as char);
        i += 1;
    }
    result
}
