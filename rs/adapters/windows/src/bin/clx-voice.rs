//! `clx-voice.exe` — the out-of-process Windows voice host.
//!
//! Rationale: the always-on `clx.exe` keyboard hook must stay a thin, stable
//! trigger (a WebView2 or a crashing otoji reader in the hook process can wedge
//! WH_KEYBOARD_LL). So all voice work — otoji subprocess, STT, PTT typing,
//! the note/listening-input mode — lives here. The core signals Space+V
//! key-down/up over named events (see `capslockx_core::platform::voice_ipc`);
//! this process runs `VoiceModule` and can be rebuilt/restarted on its own
//! without touching the running core.
//!
//! It depends only on `capslockx-core` and the `windows` crate: it implements
//! its own tiny `Platform` (SendInput typing + the shared overlay region), so
//! it does not pull in the core hook binary's modules.
//!
//! Console subsystem (no `windows_subsystem = "windows"`) so stderr is usable
//! for debugging; the core spawns it with CREATE_NO_WINDOW, so no console
//! flashes in normal use.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use windows::core::PCWSTR;
use windows::Win32::Foundation::{
    CloseHandle, ERROR_ALREADY_EXISTS, INVALID_HANDLE_VALUE, WAIT_OBJECT_0,
};
use windows::Win32::System::Memory::{
    CreateFileMappingW, MapViewOfFile, FILE_MAP_ALL_ACCESS, PAGE_READWRITE,
};
use windows::Win32::System::Threading::{
    CreateEventW, CreateMutexW, WaitForMultipleObjects, INFINITE,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, KEYEVENTF_SCANCODE,
    KEYEVENTF_UNICODE, VIRTUAL_KEY,
};

use capslockx_core::platform::voice_ipc::{ALIVE_MUTEX, KEY_DOWN_EVENT, KEY_UP_EVENT};
use capslockx_core::platform::{Platform, PttTrayState};
use capslockx_core::{modules::voice::VoiceModule, KeyCode};

/// Same tag `clx.exe`'s hook uses to recognise (and skip) self-injected input,
/// so text this host types isn't re-interpreted as hotkeys by the core.
const CLX_EXTRA_INFO: usize = 0x434C_5800;

fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

// ── Overlay region (writer) ─────────────────────────────────────────────────
// Mirrors `overlay.rs`: seq(4) · visible(4) · len(4) · UTF-8 text. The overlay
// *window* is a `clx.exe overlay-window` subprocess (the core owns that entry
// point); this host is just another writer into the shared region.

const OVERLAY_SHM_SIZE: u32 = 65536;
const OVERLAY_HEADER: usize = 12;
const OVERLAY_MAX_TEXT: usize = OVERLAY_SHM_SIZE as usize - OVERLAY_HEADER;

struct OverlayWriter {
    ptr: *mut u8,
}
unsafe impl Send for OverlayWriter {}
unsafe impl Sync for OverlayWriter {}

impl OverlayWriter {
    fn create() -> Option<Self> {
        unsafe {
            let name = to_wide("CapsLockX_OverlayText");
            let handle = CreateFileMappingW(
                INVALID_HANDLE_VALUE, // pagefile-backed named region
                None,
                PAGE_READWRITE,
                0,
                OVERLAY_SHM_SIZE,
                PCWSTR(name.as_ptr()),
            )
            .ok()?;
            let view = MapViewOfFile(handle, FILE_MAP_ALL_ACCESS, 0, 0, OVERLAY_SHM_SIZE as usize);
            if view.Value.is_null() {
                let _ = CloseHandle(handle);
                return None;
            }
            Some(Self {
                ptr: view.Value as *mut u8,
            })
        }
    }

    fn bump_seq(&self) {
        unsafe {
            let seq = std::ptr::read_volatile(self.ptr as *const u32).wrapping_add(1);
            std::ptr::write_volatile(self.ptr as *mut u32, seq);
        }
    }

    fn show(&self, text: &str) {
        let bytes = text.as_bytes();
        let len = bytes.len().min(OVERLAY_MAX_TEXT);
        unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), self.ptr.add(OVERLAY_HEADER), len);
            std::ptr::write_volatile(self.ptr.add(8) as *mut u32, len as u32);
            std::ptr::write_volatile(self.ptr.add(4) as *mut u32, 1); // visible
        }
        self.bump_seq();
    }

    fn hide(&self) {
        unsafe {
            std::ptr::write_volatile(self.ptr.add(4) as *mut u32, 0);
        }
        self.bump_seq();
    }
}

// ── HostPlatform ─────────────────────────────────────────────────────────────

struct HostPlatform {
    overlay: Option<OverlayWriter>,
    /// 0 = overlay window not yet spawned, 1 = spawned.
    overlay_spawned: AtomicUsize,
}

impl HostPlatform {
    fn new() -> Self {
        Self {
            overlay: OverlayWriter::create(),
            overlay_spawned: AtomicUsize::new(0),
        }
    }

    /// Launch the overlay *window* once — it is a `clx.exe overlay-window`
    /// subprocess (sibling binary), which reads the shared region we write.
    fn ensure_overlay_window(&self) {
        if self.overlay_spawned.swap(1, Ordering::SeqCst) == 1 {
            return;
        }
        let clx = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("clx.exe")));
        if let Some(clx) = clx {
            if clx.is_file() {
                use std::os::windows::process::CommandExt;
                let _ = std::process::Command::new(clx)
                    .arg("overlay-window")
                    .creation_flags(0x0800_0000) // CREATE_NO_WINDOW
                    .spawn();
            }
        }
    }
}

fn vk_of(key: KeyCode) -> Option<u16> {
    Some(match key {
        KeyCode::Backspace => 0x08,
        KeyCode::Enter => 0x0D,
        KeyCode::Tab => 0x09,
        _ => return None,
    })
}

fn scancode_input(vk: u16, up: bool) -> INPUT {
    let mut flags = KEYEVENTF_SCANCODE;
    if up {
        flags |= KEYEVENTF_KEYUP;
    }
    let scan = unsafe {
        windows::Win32::UI::Input::KeyboardAndMouse::MapVirtualKeyW(
            vk as u32,
            windows::Win32::UI::Input::KeyboardAndMouse::MAPVK_VK_TO_VSC,
        )
    } as u16;
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(vk),
                wScan: scan,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: CLX_EXTRA_INFO,
            },
        },
    }
}

fn unicode_input(unit: u16, up: bool) -> INPUT {
    let mut flags = KEYEVENTF_UNICODE;
    if up {
        flags |= KEYEVENTF_KEYUP;
    }
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(0),
                wScan: unit,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: CLX_EXTRA_INFO,
            },
        },
    }
}

fn send(inputs: &[INPUT]) {
    if !inputs.is_empty() {
        unsafe {
            SendInput(inputs, std::mem::size_of::<INPUT>() as i32);
        }
    }
}

impl Platform for HostPlatform {
    fn key_down(&self, key: KeyCode) {
        if let Some(vk) = vk_of(key) {
            send(&[scancode_input(vk, false)]);
        }
    }
    fn key_up(&self, key: KeyCode) {
        if let Some(vk) = vk_of(key) {
            send(&[scancode_input(vk, true)]);
        }
    }
    // Voice never drives the pointer; required by the trait, so no-op.
    fn mouse_move(&self, _dx: i32, _dy: i32) {}
    fn scroll_v(&self, _delta: i32) {}
    fn scroll_h(&self, _delta: i32) {}
    fn mouse_button(&self, _button: capslockx_core::platform::MouseButton, _pressed: bool) {}

    fn key_tap(&self, key: KeyCode) {
        if let Some(vk) = vk_of(key) {
            send(&[scancode_input(vk, false), scancode_input(vk, true)]);
        }
    }

    fn type_text(&self, text: &str) {
        let mut inputs: Vec<INPUT> = Vec::with_capacity(text.len() * 2);
        for ch in text.chars() {
            match ch {
                '\n' => {
                    inputs.push(scancode_input(0x0D, false));
                    inputs.push(scancode_input(0x0D, true));
                }
                '\r' => {}
                '\t' => {
                    inputs.push(scancode_input(0x09, false));
                    inputs.push(scancode_input(0x09, true));
                }
                _ => {
                    let mut units = [0u16; 2];
                    for u in ch.encode_utf16(&mut units) {
                        inputs.push(unicode_input(*u, false));
                        inputs.push(unicode_input(*u, true));
                    }
                }
            }
        }
        for chunk in inputs.chunks(64) {
            send(chunk);
        }
    }

    fn show_voice_overlay(&self) {
        self.ensure_overlay_window();
        if let Some(o) = &self.overlay {
            o.show("🎤 …");
        }
    }

    fn hide_voice_overlay(&self) {
        if let Some(o) = &self.overlay {
            o.hide();
        }
    }

    fn update_voice_subtitle(&self, text: &str) {
        if let Some(o) = &self.overlay {
            let line = if text.trim().is_empty() {
                "🎤 …".to_string()
            } else {
                format!("🎤 {text}")
            };
            o.show(&line);
        }
    }

    // Tray + haptics have no Windows host equivalent yet.
    fn set_ptt_tray_state(&self, _state: PttTrayState) {}
    fn haptic_feedback(&self) {}
}

// ── Config ───────────────────────────────────────────────────────────────────

/// Path to the shared `%APPDATA%\CapsLockX\config.json`.
fn config_path() -> Option<std::path::PathBuf> {
    std::env::var_os("APPDATA")
        .map(std::path::PathBuf::from)
        .map(|p| p.join("CapsLockX").join("config.json"))
}

/// Read the few voice-relevant fields out of the shared `config.json`. Any
/// missing field falls back to the same default the core uses.
fn load_config_json() -> serde_json::Value {
    if let Some(path) = config_path() {
        if let Ok(data) = std::fs::read_to_string(&path) {
            // Tolerate a UTF-8 BOM (some editors add one) before the JSON.
            let data = data.trim_start_matches('\u{feff}');
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(data) {
                return v;
            }
        }
    }
    serde_json::Value::Null
}

/// config.json's last-modified time, for cheap change polling.
fn config_mtime() -> Option<std::time::SystemTime> {
    std::fs::metadata(config_path()?).ok()?.modified().ok()
}

/// Export the configured input device as CLX_VOICE_MIC so the otoji `voice_lite`
/// spawns pick it up. Empty/unset clears it (otoji uses the system default).
fn apply_mic_env(v: &serde_json::Value) -> String {
    let mic = v
        .get("voice_mic")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    if mic.is_empty() {
        std::env::remove_var("CLX_VOICE_MIC");
    } else {
        std::env::set_var("CLX_VOICE_MIC", &mic);
    }
    mic
}

fn apply_config(voice: &VoiceModule, v: &serde_json::Value) {
    let s = |k: &str, d: &str| v.get(k).and_then(|x| x.as_str()).unwrap_or(d).to_string();
    let f = |k: &str, d: f64| v.get(k).and_then(|x| x.as_f64()).unwrap_or(d) as f32;
    let u = |k: &str, d: u64| v.get(k).and_then(|x| x.as_u64()).unwrap_or(d) as usize;
    let b = |k: &str, d: bool| v.get(k).and_then(|x| x.as_bool()).unwrap_or(d);

    voice.update_config(
        s("stt_engine", "sherpa"),
        String::new(), // llm_api_key — otoji polishes itself
        String::new(), // llm_model
        b("stt_correction", false),
        s("tts_chain", ""),
        s("stt_polish_chain", ""),
        f("aec_gain", 15.0),
        f("noise_gate", 0.003),
        f("speech_start_prob", 0.8),
        f("speech_end_prob", 0.6),
        u("speech_start_frames", 10),
        u("silence_end_frames", 20),
        s("aec_mode", "always"),
        s("whisper_model_path", ""),
        s("whisper_language", "ja"),
        v.get("ptt_vad_auto_release_ms")
            .and_then(|x| x.as_u64())
            .unwrap_or(0),
    );
}

// ── main ─────────────────────────────────────────────────────────────────────

fn main() {
    // Single-instance: hold the alive-mutex for our lifetime. If it already
    // exists another host is running — exit so we never stack otoji processes.
    let _alive = unsafe {
        let name = to_wide(ALIVE_MUTEX);
        let h = CreateMutexW(None, true, PCWSTR(name.as_ptr()));
        match h {
            Ok(handle) => {
                if windows::Win32::Foundation::GetLastError() == ERROR_ALREADY_EXISTS {
                    let _ = CloseHandle(handle);
                    eprintln!("[clx-voice] another host is already running — exiting");
                    return;
                }
                handle
            }
            Err(e) => {
                eprintln!("[clx-voice] failed to create alive mutex: {e}");
                return;
            }
        }
    };

    eprintln!(
        "[clx-voice] voice host starting (pid={})",
        std::process::id()
    );

    let platform: Arc<dyn Platform> = Arc::new(HostPlatform::new());
    let cfg = load_config_json();
    // Pin the input device for otoji (voice_otoji reads CLX_VOICE_MIC and passes
    // it as `otoji listen <device>`; the child inherits this env). Empty/unset ->
    // otoji uses the Windows system default.
    let mic = apply_mic_env(&cfg);
    if !mic.is_empty() {
        eprintln!("[clx-voice] input device from config: {mic}");
    }
    let stt_engine = cfg
        .get("stt_engine")
        .and_then(|x| x.as_str())
        .unwrap_or("sherpa")
        .to_string();
    let voice = Arc::new(VoiceModule::with_stt_engine(
        Arc::clone(&platform),
        stt_engine,
    ));
    apply_config(&voice, &cfg);
    voice.preload();

    // Watch config.json for prefs changes (mtime poll — no shared event, so the
    // core keeps its own CapsLockX_ConfigChanged untouched). On change, re-read,
    // re-export the mic device, and drop the warm otoji so the next Space+V picks
    // up the new mic — the picker in clx+, then applies without restarting this
    // host.
    {
        let voice = Arc::clone(&voice);
        std::thread::Builder::new()
            .name("clx-voice-config-watch".into())
            .spawn(move || {
                let mut last = config_mtime();
                loop {
                    std::thread::sleep(std::time::Duration::from_secs(2));
                    let now = config_mtime();
                    if now == last {
                        continue;
                    }
                    last = now;
                    let cfg = load_config_json();
                    let mic = apply_mic_env(&cfg);
                    apply_config(&voice, &cfg);
                    voice.stop_backend();
                    eprintln!("[clx-voice] config changed — mic='{mic}', backend reset");
                }
            })
            .ok();
    }

    // Named key events. CreateEventW returns the core's existing objects when
    // they exist (auto-reset, so a key-down fired during our cold start isn't
    // lost — it stays signaled until this wait consumes it).
    let (down, up) = unsafe {
        let dn = to_wide(KEY_DOWN_EVENT);
        let up_n = to_wide(KEY_UP_EVENT);
        let down =
            CreateEventW(None, false, false, PCWSTR(dn.as_ptr())).expect("create KeyDown event");
        let up =
            CreateEventW(None, false, false, PCWSTR(up_n.as_ptr())).expect("create KeyUp event");
        (down, up)
    };

    eprintln!("[clx-voice] ready — waiting for Space+V events");

    let handles = [down, up];
    loop {
        let r = unsafe { WaitForMultipleObjects(&handles, false, INFINITE) };
        match r.0 - WAIT_OBJECT_0.0 {
            0 => {
                eprintln!("[clx-voice] KeyDown(V)");
                voice.on_key_down(KeyCode::V);
            }
            1 => {
                eprintln!("[clx-voice] KeyUp(V)");
                voice.on_key_up(KeyCode::V);
            }
            _ => {
                eprintln!("[clx-voice] wait failed ({:?}) — exiting", r);
                break;
            }
        }
    }
}
