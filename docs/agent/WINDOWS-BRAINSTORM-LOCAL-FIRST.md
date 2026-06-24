# Windows clx+B — Local-First Brainstorm Design

Port the brainstorm feature (CLX+B) to Windows with a **local-first** LLM
stack: prefer a locally-running Ollama model over any cloud API, and guide
the user through a one-time Ollama bootstrap on first use.

Status: **design only — not started.** Held until the in-flight merge lands.

Goal: make CLX+B on Windows feel like the AHK version (capture → ask →
streamed answer → copied to clipboard), but with **no online API required**.
Cloud keys become an optional fallback, not the default.

---

## TL;DR

| Decision | Choice |
|---|---|
| Provider priority | **Always local-first** — Ollama whenever `Ready`, even if cloud keys exist; cloud is fallback only |
| Windows UI tech | **Tauri / WebView2**, all windows out-of-process (prompt, overlay, setup) |
| Default local model | **Qwen2.5** (strong zh/ja/en, good translate/summary), sized by hardware |
| Onboarding | First-run / broken-state **setup wizard** that installs Ollama, picks a model, pulls it |

---

## Current state

### Three implementations of clx+B

| Layer | Behavior |
|---|---|
| **AHK** (`Modules/CLX-Brainstorm.ahk`) | Online only. POSTs clipboard text + screenshot + optional mic WAV to `brainstorm.snomiao.com/ai/chat?ret=polling`, long-polls `/ai/{id}` token-by-token into a `ToolTip`, copies running answer to clipboard. ESC stops. |
| **Rust core** (`rs/core/src/modules/brainstorm.rs`) | Platform-agnostic agent loop with persistent chat history (`brainstorm_history.json`), "keep history" checkbox, streaming via `agent_chat` → `llm_client`, clipboard copy, TTS. Drives everything through 6 `Platform` trait methods. |
| **macOS adapter** | Implements all 6. Overlay = in-process non-activating click-through `NSPanel`. Prompt = **`clx-prompt` subprocess** (so the CGEventTap hook doesn't eat keystrokes meant for the field). Selected text via AX, clipboard via `NSPasteboard`. |
| **Windows adapter** | **None implemented.** `WinPlatform` inherits the 6 no-op trait defaults. CLX+B fires but `get_selected_text()`→`""`, `show_prompt_input()`→`None`, then exits silently. **clx+B is dead on Windows.** |

The 6 `Platform` methods (defaults in `rs/core/src/platform.rs`):
`get_selected_text`, `get_clipboard_text`, `set_clipboard_text`,
`show_brainstorm_overlay`, `hide_brainstorm_overlay`, `show_prompt_input`.

### What already supports local-first

- **`llm_client.rs` already speaks Ollama** — `LlmProvider::Ollama`, OpenAI-compatible
  SSE at `localhost:11434`, also probes MLX at `:8321`. `discover_ollama()` picks
  the largest installed model.
- **`lib/otoji/src/main.rs:1659-1812`** is a working reference for `ollama serve`
  (detached spawn + 30s reachability poll) and `ensure_ollama_model` (`ollama pull`
  if absent). Async/reqwest there; core will reimplement sync with `ureq` +
  `std::process::Command`.
- **`state.rs:147 best_llm_key_and_model`** picks provider priority
  **Gemini > OpenAI > Anthropic > Ollama** — i.e. *online-first*. This ordering is
  what we invert.

---

## Design

### 1. Provider order — always local-first

Invert priority in `state.rs best_llm_key_and_model` and
`llm_client.rs fallback_chain`: **Ollama (local) first**, cloud keys only as
fallback when local is unavailable.

Add to config (`ClxConfig` + Windows `config_store.rs FullConfig`):

```rust
prefer_local: bool,   // default true
local_model: String,  // e.g. "qwen2.5:7b"; empty = auto-recommend
```

When `prefer_local` and `local_status() == Ready`, brainstorm uses the local
model regardless of cloud keys.

### 2. New `rs/core/src/local_llm.rs` (platform-agnostic, sync/ureq)

```rust
enum LocalLlmStatus { Ready, OllamaDownNotRunning, RunningNoModel, NotInstalled }

fn local_status() -> LocalLlmStatus;
fn probe_hardware() -> Hardware;        // { ram_gb, vram_gb, cpu_cores }
fn recommend_model(hw: &Hardware) -> String;
fn ensure_running() -> Result<(), String>;   // `ollama serve` detached + poll /api/tags
fn ensure_model(model: &str) -> Result<(), String>;  // `ollama pull` if absent
fn install_ollama() -> Result<(), String>;   // platform-specific
```

Hardware probing: RAM via `GlobalMemoryStatusEx` (Win) / `sysinfo`; VRAM via
DXGI adapter desc or `nvidia-smi`.

`install_ollama`:
- `#[cfg(windows)]` → `winget install Ollama.Ollama`, fallback download
  `https://ollama.com/download/OllamaSetup.exe` and run silently.
- `#[cfg(target_os = "macos")]` → `brew install ollama`.

Model recommendation — **Qwen2.5**, "smartest that still runs," tiered by
RAM (or VRAM, whichever is the binding constraint, with headroom):

| RAM / VRAM | model |
|---|---|
| < 8 GB | `qwen2.5:1.5b` |
| 8–16 GB | `qwen2.5:3b` |
| 16–32 GB | `qwen2.5:7b` |
| 32–64 GB | `qwen2.5:14b` |
| 64 GB+ | `qwen2.5:32b` |

Qwen2.5 chosen for strong zh/ja/en multilingual + translation/summary quality,
and a full ladder of small tiers. (Vision/agent models are out of scope here —
see `LOCAL-AGENT-MODELS.md` for the multimodal agent stack.)

### 3. Windows `Platform` implementation (`rs/adapters/windows/src/output.rs`)

| Method | Implementation |
|---|---|
| `get_clipboard_text` / `set_clipboard_text` | Win32 `CF_UNICODETEXT` (or `arboard`). |
| `get_selected_text` | Return `""` — the module's existing Ctrl+C-with-clipboard-restore fallback handles it. (UI Automation `TextPattern` is a possible later upgrade.) |
| `show_brainstorm_overlay` / `hide_brainstorm_overlay` | Drive the **overlay subprocess** (see §4). |
| `show_prompt_input` | Spawn the **prompt subprocess** (see §4). |

### 4. UI — Tauri/WebView2, all out-of-process

A focused WebView2 window hosted **in the hook process** stops Windows from
delivering `WH_KEYBOARD_LL` while focused (this is why prefs is already a
separate `clx prefs-window` process — see
`memory: windows-prefs-out-of-process`). So **every** brainstorm window is a
subprocess, mirroring how macOS puts the prompt in `clx-prompt`.

- **Prompt** = `clx.exe prompt-window <title> <msg> <prefill>` → prints
  `[KEEP]\n<text>` to stdout (the `[KEEP]` prefix carries the checkbox state,
  same contract as macOS `clx-prompt`). Exit 1 = cancelled.
- **Overlay** = non-activating Tauri window
  (`WS_EX_NOACTIVATE`, topmost, click-through).
  **New IPC requirement:** because the overlay is a subprocess, the hook
  process can't update it in-process the way macOS updates its NSPanel. Stream
  tokens through the windows adapter's existing **`shm.rs` shared memory** —
  the hook process writes the running answer, the overlay polls and renders.
  (macOS gets this for free via an in-process panel; Windows needs the channel.)
- **Setup wizard** = the prefs window launched in setup mode:
  `clx.exe prefs-window --setup=brainstorm`. Adds an "AI / Brainstorm" section
  to `rs/adapters/windows/src/prefs/index.html`.

### 5. First-run / recovery setup wizard

`BrainstormModule::start_turn` currently errors with "No LLM API key configured"
when `llm_config` is `None`. Change: when **`prefer_local` is on and
`local_status() != Ready` and no cloud key is set**, launch the setup wizard
instead of erroring.

Wizard flow (Tauri commands call the `local_llm.rs` functions):

```
explain local + private
  → show detected hardware + recommended Qwen2.5 model
  → [Agree & Install]
       install_ollama()  (winget / OllamaSetup.exe, progress)
       ensure_running()  (ollama serve + poll)
       ensure_model()    (ollama pull, progress)
       write config (local_model = recommended)
       signal CapsLockX_ConfigChanged
  → hook process hot-reloads BrainstormModule::update_llm_config
  → [Use my own API key instead]  → key entry, skip Ollama
  → [Cancel]
```

The `CapsLockX_ConfigChanged` named-event + config.json reload path already
exists for prefs; reuse it verbatim.

---

## Implementation order

1. **`local_llm.rs` + provider-order/config change** — pure core, unit-testable
   in isolation (status enum, hardware→model table, fallback ordering).
2. **Windows `Platform`: clipboard + prompt subprocess** — makes CLX+B reach the
   LLM and show a prompt.
3. **Overlay over `shm`** — streamed token display.
4. **Setup wizard** — Ollama install/serve/pull UI in the prefs window.

Each step builds + relaunches per `CLAUDE.md`; do not commit/push without
explicit approval.

---

## Open considerations

- **Prompt latency**: a WebView2 cold start per CLX+B may feel sluggish vs the
  macOS native `clx-prompt`. If it's noticeable, keep a warm prompt subprocess
  alive or fall back to a tiny native Win32 input. Revisit after step 2.
- **System load**: `recommend_model` sizes by total RAM/VRAM; could also factor
  *current* free memory to avoid thrashing on a busy machine. Minor; size-by-
  hardware is the primary signal.
- **Model refresh**: `discover_ollama()` already picks the largest installed
  model, so a user who pulls a bigger one later gets it automatically.
