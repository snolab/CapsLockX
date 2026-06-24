//! Local-first LLM bootstrap — detect, start, and provision a local Ollama
//! server so CLX+B (brainstorm) can run entirely on-device with no cloud API.
//!
//! Split into two layers:
//!   * **pure** — `Hardware` + `recommend_model()` pick the smartest model that
//!     still fits the machine. No I/O, fully unit-tested.
//!   * **impure** (native only) — probe hardware, talk to / spawn the Ollama
//!     server, pull models, and install Ollama. Gated to non-wasm targets.
//!
//! The brainstorm module uses `local_status()` to decide whether to run a
//! turn directly or launch the first-run setup wizard. See
//! `docs/agent/WINDOWS-BRAINSTORM-LOCAL-FIRST.md`.

/// Readiness of the local Ollama stack.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalLlmStatus {
    /// Server reachable AND at least one model installed → CLX+B can run.
    Ready,
    /// Server reachable but no model pulled yet.
    RunningNoModel,
    /// `ollama` binary present but the server isn't answering.
    OllamaDownNotRunning,
    /// `ollama` binary not found — needs install.
    NotInstalled,
}

/// Coarse hardware capacity used to size the local model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Hardware {
    pub ram_gb: u32,
    pub vram_gb: u32,
    pub cpu_cores: u32,
}

/// Pick the smartest Qwen2.5 model that still runs on the given hardware.
///
/// Qwen2.5 is chosen for strong zh/ja/en multilingual + translation/summary
/// quality and a full ladder of small tiers. The budget is the larger of system
/// RAM and GPU VRAM (GB): Ollama runs on CPU+RAM, but a large GPU lets us hold a
/// bigger model, so we take whichever is larger.
pub fn recommend_model(hw: &Hardware) -> &'static str {
    match hw.ram_gb.max(hw.vram_gb) {
        0..=7 => "qwen2.5:1.5b",
        8..=15 => "qwen2.5:3b",
        16..=31 => "qwen2.5:7b",
        32..=63 => "qwen2.5:14b",
        _ => "qwen2.5:32b",
    }
}

/// Base URL of the local Ollama server.
pub const OLLAMA_BASE: &str = "http://localhost:11434";

// ── Native (non-wasm) implementation ────────────────────────────────────────

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use super::{Hardware, LocalLlmStatus, OLLAMA_BASE};
    use std::process::{Command, Stdio};
    use std::time::Duration;

    /// Is the `ollama` CLI on PATH?
    pub fn ollama_installed() -> bool {
        Command::new("ollama")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    /// Is the Ollama HTTP server answering? (Connection-refused is immediate on
    /// localhost, so no explicit timeout is needed.)
    pub fn server_reachable() -> bool {
        ureq::get(&format!("{OLLAMA_BASE}/api/tags")).call().is_ok()
    }

    /// Model tags currently installed (e.g. `["qwen2.5:7b"]`). Empty on error.
    pub fn installed_models() -> Vec<String> {
        let resp = match ureq::get(&format!("{OLLAMA_BASE}/api/tags")).call() {
            Ok(r) => r,
            Err(_) => return Vec::new(),
        };
        let body: serde_json::Value = match resp.into_string() {
            Ok(s) => serde_json::from_str(&s).unwrap_or(serde_json::Value::Null),
            Err(_) => return Vec::new(),
        };
        body["models"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|m| m["name"].as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// True if `model` (exact tag, or tag-less prefix like `qwen2.5`) is present.
    fn model_present(model: &str, installed: &[String]) -> bool {
        installed
            .iter()
            .any(|m| m == model || m.starts_with(&format!("{model}:")))
    }

    pub fn local_status() -> LocalLlmStatus {
        if server_reachable() {
            if installed_models().is_empty() {
                LocalLlmStatus::RunningNoModel
            } else {
                LocalLlmStatus::Ready
            }
        } else if ollama_installed() {
            LocalLlmStatus::OllamaDownNotRunning
        } else {
            LocalLlmStatus::NotInstalled
        }
    }

    /// Ensure the server is running: spawn `ollama serve` detached and poll up
    /// to ~20s for reachability. No-op if already reachable.
    pub fn ensure_running() -> Result<(), String> {
        if server_reachable() {
            return Ok(());
        }
        if !ollama_installed() {
            return Err("ollama is not installed".into());
        }
        // Detached: we let the server outlive clx (it forks its own process).
        Command::new("ollama")
            .arg("serve")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .stdin(Stdio::null())
            .spawn()
            .map_err(|e| format!("failed to start `ollama serve`: {e}"))?;
        for _ in 0..40 {
            std::thread::sleep(Duration::from_millis(500));
            if server_reachable() {
                return Ok(());
            }
        }
        Err("ollama did not become reachable within 20s".into())
    }

    /// Ensure `model` is pulled. `ollama pull` is a no-op if already present, but
    /// we skip the subprocess entirely when we can see it in the tag list.
    pub fn ensure_model(model: &str) -> Result<(), String> {
        if model_present(model, &installed_models()) {
            return Ok(());
        }
        let status = Command::new("ollama")
            .args(["pull", model])
            .status()
            .map_err(|e| format!("failed to run `ollama pull {model}`: {e}"))?;
        if status.success() {
            Ok(())
        } else {
            Err(format!("`ollama pull {model}` failed"))
        }
    }

    // ── Install (platform-specific) ──────────────────────────────────────────

    #[cfg(target_os = "windows")]
    pub fn install_ollama() -> Result<(), String> {
        // winget ships on Win10 21H1+/Win11.
        let status = Command::new("winget")
            .args([
                "install",
                "--id",
                "Ollama.Ollama",
                "-e",
                "--accept-source-agreements",
                "--accept-package-agreements",
            ])
            .status();
        match status {
            Ok(s) if s.success() => Ok(()),
            _ => Err(
                "winget install Ollama.Ollama failed; install manually from https://ollama.com/download"
                    .into(),
            ),
        }
    }

    #[cfg(target_os = "macos")]
    pub fn install_ollama() -> Result<(), String> {
        let status = Command::new("brew").args(["install", "ollama"]).status();
        match status {
            Ok(s) if s.success() => Ok(()),
            _ => Err(
                "brew install ollama failed; install manually from https://ollama.com/download"
                    .into(),
            ),
        }
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    pub fn install_ollama() -> Result<(), String> {
        Err(
            "automatic install not supported on this platform; see https://ollama.com/download"
                .into(),
        )
    }

    // ── Hardware probing ──────────────────────────────────────────────────────

    pub fn probe_hardware() -> Hardware {
        Hardware {
            ram_gb: total_ram_gb(),
            vram_gb: vram_gb(),
            cpu_cores: std::thread::available_parallelism()
                .map(|n| n.get() as u32)
                .unwrap_or(1),
        }
    }

    #[cfg(target_os = "windows")]
    fn total_ram_gb() -> u32 {
        #[repr(C)]
        struct MemoryStatusEx {
            dw_length: u32,
            dw_memory_load: u32,
            ull_total_phys: u64,
            ull_avail_phys: u64,
            ull_total_page_file: u64,
            ull_avail_page_file: u64,
            ull_total_virtual: u64,
            ull_avail_virtual: u64,
            ull_avail_extended_virtual: u64,
        }
        extern "system" {
            fn GlobalMemoryStatusEx(buffer: *mut MemoryStatusEx) -> i32;
        }
        let mut status = MemoryStatusEx {
            dw_length: std::mem::size_of::<MemoryStatusEx>() as u32,
            dw_memory_load: 0,
            ull_total_phys: 0,
            ull_avail_phys: 0,
            ull_total_page_file: 0,
            ull_avail_page_file: 0,
            ull_total_virtual: 0,
            ull_avail_virtual: 0,
            ull_avail_extended_virtual: 0,
        };
        unsafe {
            if GlobalMemoryStatusEx(&mut status) != 0 {
                (status.ull_total_phys / (1024 * 1024 * 1024)) as u32
            } else {
                0
            }
        }
    }

    #[cfg(target_os = "macos")]
    fn total_ram_gb() -> u32 {
        let mut mem: u64 = 0;
        let mut size = std::mem::size_of::<u64>();
        let name = b"hw.memsize\0";
        let ret = unsafe {
            libc::sysctlbyname(
                name.as_ptr() as *const libc::c_char,
                &mut mem as *mut u64 as *mut libc::c_void,
                &mut size,
                std::ptr::null_mut(),
                0,
            )
        };
        if ret == 0 {
            (mem / (1024 * 1024 * 1024)) as u32
        } else {
            0
        }
    }

    #[cfg(target_os = "linux")]
    fn total_ram_gb() -> u32 {
        if let Ok(s) = std::fs::read_to_string("/proc/meminfo") {
            for line in s.lines() {
                if let Some(rest) = line.strip_prefix("MemTotal:") {
                    let kb: u64 = rest
                        .trim()
                        .trim_end_matches("kB")
                        .trim()
                        .parse()
                        .unwrap_or(0);
                    return (kb / (1024 * 1024)) as u32;
                }
            }
        }
        0
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    fn total_ram_gb() -> u32 {
        0
    }

    /// Best-effort NVIDIA VRAM via `nvidia-smi`. Returns 0 if unavailable (no
    /// GPU, AMD/Intel, or tool missing) — the caller then sizes by RAM.
    fn vram_gb() -> u32 {
        let out = Command::new("nvidia-smi")
            .args(["--query-gpu=memory.total", "--format=csv,noheader,nounits"])
            .output();
        if let Ok(o) = out {
            if o.status.success() {
                if let Ok(s) = String::from_utf8(o.stdout) {
                    if let Some(first) = s.lines().next() {
                        if let Ok(mb) = first.trim().parse::<u64>() {
                            return (mb / 1024) as u32;
                        }
                    }
                }
            }
        }
        0
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn model_present_matches_exact_and_prefix() {
            let installed = vec!["qwen2.5:7b".to_string(), "llama3.2:3b".to_string()];
            assert!(model_present("qwen2.5:7b", &installed));
            assert!(model_present("qwen2.5", &installed)); // tag-less prefix
            assert!(!model_present("qwen2.5:14b", &installed));
            assert!(!model_present("mistral", &installed));
        }

        #[test]
        fn probe_hardware_reports_at_least_one_core() {
            // available_parallelism never returns 0; RAM/VRAM are best-effort.
            assert!(probe_hardware().cpu_cores >= 1);
        }

        // Desktop OSes have a real RAM probe (FFI / procfs); assert it works.
        #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
        #[test]
        fn probe_hardware_reports_ram_on_desktop() {
            assert!(
                probe_hardware().ram_gb >= 1,
                "expected to detect at least 1 GB RAM"
            );
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use native::{
    ensure_model, ensure_running, install_ollama, installed_models, local_status, ollama_installed,
    probe_hardware, server_reachable,
};

// ── wasm stubs ────────────────────────────────────────────────────────────────
// The browser adapter never runs Ollama; expose the same surface as no-ops so
// any shared core code links.

#[cfg(target_arch = "wasm32")]
pub fn local_status() -> LocalLlmStatus {
    LocalLlmStatus::NotInstalled
}

#[cfg(target_arch = "wasm32")]
pub fn probe_hardware() -> Hardware {
    Hardware::default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recommend_model_sizes_by_largest_of_ram_or_vram() {
        let hw = |ram, vram| Hardware {
            ram_gb: ram,
            vram_gb: vram,
            cpu_cores: 8,
        };
        assert_eq!(recommend_model(&hw(4, 0)), "qwen2.5:1.5b");
        assert_eq!(recommend_model(&hw(8, 0)), "qwen2.5:3b");
        assert_eq!(recommend_model(&hw(16, 0)), "qwen2.5:7b");
        assert_eq!(recommend_model(&hw(32, 0)), "qwen2.5:14b");
        assert_eq!(recommend_model(&hw(64, 0)), "qwen2.5:32b");
        assert_eq!(recommend_model(&hw(128, 0)), "qwen2.5:32b");
    }

    #[test]
    fn recommend_model_uses_vram_when_larger_than_ram() {
        // 16 GB RAM but a 24 GB GPU → the GPU can hold a bigger model.
        let hw = Hardware {
            ram_gb: 16,
            vram_gb: 24,
            cpu_cores: 8,
        };
        assert_eq!(recommend_model(&hw), "qwen2.5:7b");
    }

    #[test]
    fn recommend_model_boundaries() {
        let hw = |ram| Hardware {
            ram_gb: ram,
            vram_gb: 0,
            cpu_cores: 1,
        };
        // exact tier edges
        assert_eq!(recommend_model(&hw(7)), "qwen2.5:1.5b");
        assert_eq!(recommend_model(&hw(15)), "qwen2.5:3b");
        assert_eq!(recommend_model(&hw(31)), "qwen2.5:7b");
        assert_eq!(recommend_model(&hw(63)), "qwen2.5:14b");
    }
}
