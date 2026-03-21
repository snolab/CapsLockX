# CapsLockX Roadmap

## CapsLockX 2.0 — Rust Rewrite (In Progress)

CapsLockX 2.0 is a ground-up rewrite in Rust, targeting cross-platform support with a focus on macOS first. The Rust version adds AI-powered features (voice, brainstorm agent, TTS) while maintaining all the keyboard/mouse productivity features of the AHK original.

### Architecture

```
capslockx-core (Rust library, platform-agnostic)
├── Engine (key event → module dispatch)
├── Modules: Edit, Mouse, Media, VirtualDesktop, Voice, Brainstorm, WindowManager
├── LLM Client (Gemini/OpenAI/Anthropic/Ollama/MLX)
├── Agent (tool-use loop with web search, JS eval, TTS, etc.)
├── STT (SenseVoice local + Gemini cloud)
├── TTS (ElevenLabs → Gemini → OpenAI → msedge-tts → native)
└── Platform trait (adapters implement per-OS)

Adapters:
├── capslockx-macos  (CGEventTap + Core Graphics + AppKit FFI)
├── capslockx-windows (WH_KEYBOARD_LL + SendInput) [stub]
├── capslockx-linux  (evdev + uinput) [stub]
└── capslockx-browser (WASM + Web APIs)
```

### Status: What's Done (v2.0-beta)

| Category | Feature | Status |
|---|---|---|
| **Core** | Trigger keys (CapsLock/Space/Insert/RAlt) | ✅ |
| | AccModel physics (smooth acceleration) | ✅ |
| | Preferences UI (WKWebView) | ✅ |
| | Launch at login (LaunchAgent) | ✅ |
| | Tray icon (NSStatusBar) | ✅ |
| | Config persistence (JSON) | ✅ |
| **Editing** | Cursor HJKL, Page YUIO, Tab PN | ✅ |
| | Shift/Ctrl/Alt passthrough modifiers | ✅ |
| | Enter without break | ✅ |
| **Mouse** | WASD movement, QE click | ✅ |
| | RF vertical scroll | ✅ |
| | Shift+RF horizontal scroll | ✅ |
| | Mouse clamp to screen bounds | ✅ |
| **Window** | Cycle windows Z (CGWindowID-based) | ✅ |
| | Arrange/cascade C | ✅ |
| | Close tab/window X | ✅ |
| | Topmost toggle | ✅ |
| **Virtual Desktop** | Switch desktop 1-9 | ✅ |
| **Media** | Volume, play/pause, next/prev | ✅ |
| **Voice (NEW)** | SenseVoice local STT (95% accuracy) | ✅ |
| | Gemini cloud STT polish (100% JA accuracy) | ✅ |
| | MLX local LLM STT correction | ✅ |
| | Dual-track recording (mic + system audio) | ✅ |
| | Echo cancellation (VPIO + NLMS) | ✅ |
| | Voice note (WebM + SRT) | ✅ |
| | Voice overlay (waveform + subtitle) | ✅ |
| | Overlay drag/resize/close on hover | ✅ |
| | Hidden from screen sharing | ✅ |
| **Brainstorm (NEW)** | Agent with chat history (Space+B) | ✅ |
| | Direct LLM streaming (SSE) | ✅ |
| | 4 providers (Gemini/OpenAI/Anthropic/Ollama+MLX) | ✅ |
| | Auto model discovery from provider APIs | ✅ |
| | Tools: web_search, fetch_url, js_eval, math_eval | ✅ |
| | Tools: speak, read_screen, read_clipboard, screenshot | ✅ |
| | Task manager (background tasks with timeout) | ✅ |
| | Context compaction (auto-summarize long conversations) | ✅ |
| | Large output → file paging (read_file_range) | ✅ |
| **TTS (NEW)** | Fallback: ElevenLabs→Gemini→OpenAI→msedge→say | ✅ |
| | Speech queue (serial, no overlap) | ✅ |
| | Auto-speak on translations | ✅ |
| **JS Engine (NEW)** | rquickjs (native, 126 tok/s) | ✅ |
| | boa_engine (WASM fallback) | ✅ |
| **Math Engine (NEW)** | Woxi (Wolfram Language subset) | ✅ |
| **Browser (NEW)** | WASM adapter with keyboard hooks | ✅ |
| | SenseVoice WASM voice input | ✅ |
| | Server-mode STT streaming | ✅ |

### What's Missing vs AHK

| Feature | Priority | Notes |
|---|---|---|
| Screenshot input for brainstorm | Medium | AHK has GDIp capture |
| Image input (multipart) | Medium | |
| Move window to desktop | Low | macOS Spaces API is limited |
| PinchZoom | Low | macOS-specific |
| Userscripts | Low | Could use js_eval |
| LaptopKeyboardFix | Low | Platform-specific |
| Windows adapter | High | Stub exists, needs implementation |
| Linux adapter | High | Stub exists, needs implementation |

### Roadmap: Next Steps

#### v2.0 Stable
- [ ] Windows adapter (WH_KEYBOARD_LL + SendInput)
- [ ] Linux adapter (evdev + uinput)
- [ ] Fix Ollama on M5 Max (Metal shader compat)
- [ ] Resize grip via main-thread dispatch
- [ ] Auto-resize overlay to content

#### v2.1
- [ ] Screenshot input for brainstorm (Gemini vision)
- [ ] Gemini Live API streaming STT (WebSocket)
- [ ] Plugin system (user scripts in JS via rquickjs)
- [ ] Multi-model brainstorm (switch models mid-conversation)

#### v2.2
- [ ] Android adapter (AccessibilityService)
- [ ] iOS adapter (if possible)
- [ ] Cross-device sync (settings, history)
- [ ] Voice commands ("open browser", "switch to terminal")

### Performance Benchmarks

| Metric | Value |
|---|---|
| Speech → first text | ~180ms |
| SenseVoice inference (1s audio) | ~30ms |
| SenseVoice accuracy (English) | 95.6% |
| SenseVoice accuracy (Japanese) | 94.7% |
| Gemini cloud STT accuracy (Japanese) | 100% |
| MLX local LLM (Qwen2.5 3B) | 126 tok/s |
| JS eval (rquickjs, factorial 20) | <1ms |
| Binary size (macOS) | ~25MB |
| RAM usage (idle) | ~50MB |
| RAM usage (voice active) | ~500MB (SenseVoice model) |

### Build

```bash
# macOS
cd rs && cargo build -p capslockx-macos --release

# Browser (WASM)
wasm-pack build --target web rs/adapters/browser

# Standalone agent CLI
cargo build -p capslockx-core --release --bin clx-agent
```

### Links
- [Window Cycle Stability](dev/window-cycle-stability.md)
- [Dual-Track STT Architecture](dev/dual-track-stt.md)
- [Agent Tool Test Matrix](dev/agent-test-matrix.md)
