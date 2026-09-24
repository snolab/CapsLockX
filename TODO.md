## NEXT

- [x] otoji-tray: replace text title with SF Symbol mic icon
- [x] otoji listen: idle CPU gate before RNNoise (76% → ~0%)
- [x] macOS: "Launch at Login" tray toggle — `rs/adapters/macos/src/launch_at_login.rs` (writes/removes `~/Library/LaunchAgents/com.snomiao.capslockx.plist`; enable only registers for next login, disable also `launchctl bootout`s), checkbox item wired in `tray.rs`/`prefs.rs`. Runs `dist/CapsLockX-dev.app/Contents/MacOS/clx -f` (branded .app identity for the Accessibility list) with `ORT_DYLIB_PATH` set for ten-vad's dlopen; deliberately no `KeepAlive` (would fight clx's startup dedup).
- [ ] Windows: "Launch at Login" tray toggle (mirror of the macOS feature above)
  - Add a `CheckMenuItemBuilder`-based checkbox item to the tray menu built in `rs/adapters/windows/src/main.rs` (alongside the existing `prefs`/`config_dir`/`quit` items around line 234-246), wired via `on_menu_event`.
  - Backing store: a new module (e.g. `rs/adapters/windows/src/launch_at_login.rs`) that writes/removes a value under the per-user registry key `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` (name `CapsLockX`, value = path to the exe, quoted). This is the standard lightweight mechanism for per-user autostart — no admin rights, no Task Scheduler XML. Use the `windows` crate (already a dependency here) via `RegSetValueExW`/`RegDeleteValueW` on `HKEY_CURRENT_USER`, or pull in the `auto-launch` crate if it simplifies things.
  - `is_enabled()` = registry value exists and points at the current exe path (like the mac version, treat the registry as the single source of truth — no separate config flag to desync).
  - Note: unlike macOS, Windows Run-key autostart just launches the exe directly at login — there's no separate watchdog/crash-restart wrapper on this platform to route through, so `Program` = current exe path with no wrapper args.
- [ ] otoji-tray: extract `objc-helpers.rs` shared with CLX `tray.rs` (~150 lines duplicated)
- [ ] CLX↔otoji-tray state channel: drop CLX's own tray, otoji-tray reflects CLX mode/PTT via state file

## Findings parked during the CLX+Z runaway investigation (2026-09-22)

Each of these was confirmed while chasing the CLX+Z runaway but is a separate
concern. Evidence lives in `tmp/clx-z-*.md` and `tmp/clx-raw-probe-*.md`.

- [ ] **The keyboard hook never comes back on its own.** `install_hook()` runs
  exactly once (`rs/adapters/windows/src/hook.rs:131`); there is no liveness
  check and no reinstall. The hook is installed on the Tauri UI thread and
  serviced by its run loop, so when that thread is busy the hook stops being
  called — proven live: in one clx process (PID 25880, never restarted) the hook
  suppressed Space, then stopped suppressing it for ~1 minute, then resumed,
  with an *ordinary* window in the foreground the whole time. That stall is the
  previously-unconfirmed "wedge" from the memory
  `windows-clx-wedge-and-space-flood`, and it is what starves a latched physics
  model of its key-up. Needs a liveness poll + reinstall, or the hook moved off
  the UI thread.

- [ ] **Note for anyone writing a "deliberately hung window" test fixture:
  `Start-Sleep` will not do it.** PowerShell runs STA so WinForms can work, and
  an STA thread keeps pumping messages during its blocking waits — so a window
  that looks asleep is still answering, `IsHungAppWindow` stays false, and
  Windows never raises a Ghost. Poking it harder does not help; it really is
  responding. Use `[System.Threading.Thread]::Sleep()`, which blocks the thread
  outright. Second requirement: something must actually try to talk to the
  window, or Windows never notices it is dead — `scripts/make-ghost-window.ps1` spawns
  a helper that sends `WM_NULL` once a second for exactly that reason.

- [ ] **clx's shutdown path wedges the process, and the zombie keeps its keyboard
  hook forever.** Confirmed 2026-09-23 — this is the previously-unconfirmed wedge
  in the memory `windows-clx-wedge-and-space-flood`, and it is a *shutdown* bug.
  - Reproduced four times in one session. Happens on `Stop-Process -Force` **and**
    on clx's own graceful `CapsLockX_Quit` event, so it is not force-kill specific.
  - End state: process drops from ~16 threads to 1-2, `Responding=False`,
    `ExitCode` already `0xFFFFFFFF`, `WaitForSingleObject` = 258 (not signalled),
    and `TerminateProcess` returns **ERROR_ACCESS_DENIED (5)** even though
    `OpenProcess(PROCESS_TERMINATE)` succeeded — the signature of
    `STATUS_PROCESS_IS_TERMINATING`. Surviving threads are
    `state=Wait wait=UserRequest startaddr=0x0`. No user-mode call can reap it;
    only a reboot clears it.
  - **Why it matters far more than it looks.** The zombie never terminates, so
    Windows never releases its `WH_KEYBOARD_LL` registration. After a few
    restart cycles a freshly launched clx gets a hook that receives nothing:
    observed with PID 31044, which had a healthy 16 threads and a working raw
    receiver but **zero `[hook]` lines in `capslockx_hook.log`** and a dead tray
    menu. Both symptoms are the same cause — the hook lives on the Tauri UI
    thread (`hook.rs:120`), so a stuck UI thread kills hotkeys and the tray
    together while independent threads keep running.
  - Each zombie also costs ~2.5 s on every later startup (`kill_previous` waits
    1500 ms + 1000 ms per unkillable victim) and holds a file lock on the exe.
    The lock is escapable without rebooting: **rename the running exe aside**
    (Windows permits renaming a locked image, just not deleting or overwriting
    it) — the trick `self_update.rs::aside_path` already uses.
  - Suspect range is small: everything `main.rs` does after Tauri's `run()`
    returns — `SHUTDOWN.store`, `hook::uninstall_hook()`,
    `cursor_visibility::disable()`, the AHK child kill. `UnhookWindowsHookEx`
    called while the hook is mid-callback is the leading candidate. The older
    guess in memory (tray `set_icon` cross-thread) is not supported by the
    thread states seen here.
  - **This bug is what made every other investigation harder** — each debug
    restart poisoned the next one. Fix it before doing more hook work.

- [ ] **Single-`.exe` portable build is not actually single-exe.** `build.ps1`
  copies `clx-prefs-slint.exe` next to `clx.exe` because `open_prefs_window`
  shells out to it. Carrying only `clx.exe` on a USB stick leaves Preferences
  broken. Prefs must stay a *separate process* (WebView2/Slint in the hook
  process kills the hook — see `windows-prefs-out-of-process`) but a separate
  process does not require a separate file: self-spawn the same exe with a
  subcommand (`clx prefs-window`). Also audit for stray runtime DLLs
  (onnxruntime et al.) that would need to ride along.

- [ ] **`CLX_EXTRA_INFO` is trivially spoofable.** It is a plain constant
  `0x434C5800` = `"CLX\0"` (`output.rs:42`), and the hook's self-injection test
  is `injected && dwExtraInfo == CLX_EXTRA_INFO` (`hook.rs:248`). Any process
  can tag input with it and make clx blind to those keys. Windows already
  provides `LLKHF_LOWER_IL_INJECTED (0x02)` alongside `LLKHF_INJECTED (0x10)`;
  checking it would catch injection from lower-integrity processes. Low severity
  for a local keyboard tool, but free to harden.

- [ ] **`describe_foreground_block()` uses the wrong elevation test.** It infers
  "higher integrity" from `OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION)`
  failing. That right is deliberately permissive and *succeeds* on ordinary
  elevated apps — measured: `fg_pid=9380 fg_query=True fg_elevated=1`. So every
  `blocked=false` it has ever logged is unreliable. Replace with
  `OpenProcessToken` + `GetTokenInformation(TokenElevation)`.

- [ ] **`Platform::system_key_repeat_ms` is unimplemented on Windows** (trait
  default returns `None`). Wants `SystemParametersInfo` with
  `SPI_GETKEYBOARDDELAY` / `SPI_GETKEYBOARDSPEED`. Needed by any
  repeat-rate-aware logic, including the hook-silence backstop idea in
  `tmp/clx-z-review-notes-6.md`.

- [ ] **`MAX_STEPS_IN_FLIGHT = 2` drops rapid separate desktop presses.**
  `vd_api::post` coalesces desktop steps to bound a queue that the runaway used
  to bury. Deliberate for now, but it means four quick presses do not reliably
  travel four desktops. If release detection is genuinely fixed the backlog
  becomes unreachable, and the cap may be droppable entirely rather than tuned.

- [ ] **`%TEMP%\capslockx_vd.log` has no rotation.** It reached 6.4 MB during
  the runaway (9,640 × `CoCreateInstance FAILED` + `manager call failed –
  re-acquiring` in one session, since fixed with a 500 ms backoff). The hook log
  is `CLX_DEBUG`-gated so it only grows when asked; the vd log is not.

- [ ] **`lab/clx-lang/` playground UI is behind its parser.** `clx.js` now lexes
  and parses `when app:"…" { … }` scopes and `js { … }` action bodies, with
  `lab/clx-lang/parse.test.cjs` green at 27/27, but the page does not yet show a
  scope column, ship a scoped example, or syntax-highlight the new keywords.
  §3 and §4 of `lab/clx-lang/index.html` are written and flagged as design-only.

---

### 发展路线 🛰️ RoadMap

CapsLockX 的核心理念是：简化系统操作逻辑，提升操作效率，且不与原有习惯键位冲突。

1. [x] 按 CapsLockX + - 键显示对应帮助（目前的显示样式相当草率）
2. [ ] i18n （eh 这个真得有）
3. [ ] 自动更新（虽然 git pull 一下也不是不行）
4. [ ] 初次使用上手教程（这个现在有点简陋……）
5. [ ] 插件管理器（虽然文件系统也可以搞定）
6. [ ] 自动配置同步功能（虽然一般来说扔 onedrive 就够）
7. [ ] 易用的选项配置的 UI 界面（虽然改 ini 也不是什么难事）
8. [ ] 执行外部代码（Python、Nodejs、外部 AHK、Bash、……）（虽然写个脚本 run 一下也并不算麻烦）

---

## Rust Port – Active Investigation

### BUG: Shift+HJKL Text Selection (IN PROGRESS)

**Problem:** CapsLock+Shift+HJKL should select text (Shift+Arrow) but only moves cursor.

**Root Cause:** AccModel callback fires on background thread (`clx-acc-ticker`). `SendInput` from this thread causes the OS to insert **phantom modifier key-up events** between injected events, cancelling the Shift state before Arrow keys arrive.

**Approaches tried & results:**

| # | Approach | Result |
|---|----------|--------|
| 1 | Plain `key_tap(Arrow)` relying on held Shift | No selection — phantom Shift UP from OS cancels Shift before Arrow arrives |
| 2 | Separate `key_down(Shift)` + `key_tap(Arrow)` | Phantom Shift UP between separate SendInput calls |
| 3 | Atomic `key_tap_n_with_mod` (single SendInput batch) | Phantoms still within batch; works on 2nd+ ticks but not 1st |
| 4 | Suppress phantom Shift UP in hook (GetAsyncKeyState) | Fixes selection but **causes stuck Shift** — our injected Shift keeps GetAsyncKeyState returning "down", blocking real releases |
| 5 | `GetAsyncKeyState` for shift detection in cursor_action | Correctly detects shift, doesn't fix OS-level Shift state for arrows |
| 6 | Skip modifier injection if already held (GetAsyncKeyState check in key_tap_n_with_mod) | Phantom UP still appears after AHK's own Shift injection — not caused by our code |

**Key finding:** The phantom Shift UP (`inj=false extra=0x0`) appears immediately after ANY injected Shift DN — whether from our code or AHK. This is an OS-level behavior, possibly related to how `WH_KEYBOARD_LL` hooks interact with injected modifier keys. The phantom cancels the Shift state at the OS level before our Arrow key arrives.

**Next steps:**
- [ ] Try `PostMessage`/`SendMessage` WM_KEYDOWN to target window with shift flag in lParam (bypasses SendInput entirely)
- [ ] Try scan-code based injection (`KEYEVENTF_SCANCODE` flag)
- [ ] Try injecting on the hook thread (not the AccModel ticker thread) — use a message queue to marshal calls
- [ ] Check if the AHK version has the same phantom issue (it works, so how does it avoid it?)

**E2E test infra:** `rs/test-shift-select.ahk` creates its own AHK Gui Edit control (no Notepad dependency)

**E2E test results (5/8 pass):**
- PASS: CLX+L, CLX+H, CLX+K, CLX+J (plain cursor movement), Shift-not-stuck
- FAIL: CLX+Shift+L, Shift+CLX+L, CLX+Shift+H (selection)

### DONE: CycleWindows cross-monitor fix
- AHK + Rust: windows cycle within monitor -> next monitor -> virtual desktop switch
- `get_app_windows()` sorts by `(monitor_index, hwnd)` for stable monitor-grouped ordering

### DONE: AHK modules opt-in
- `--with-ahk` flag required to spawn AHK module loader (default: Rust-only)

### DONE: Admin elevation config
- `request_admin: true` in `%APPDATA%\CapsLockX\config.json` triggers UAC elevation on startup

### FEAT: Voice Input (CLX+V) — Streaming Speech-to-Text
**Plan:** [plan/voice-input/README.md](plan/voice-input/README.md) | [plan/voice-input/TODO.md](plan/voice-input/TODO.md)
- CLX+V click = toggle continuous listening; CLX+V hold = listen while held
- VAD splits audio into utterances, sends chunks to server incrementally
- 3-stage streaming pipeline: local Whisper (fast draft) → server Whisper (refined) → LLM typo-fix (polished)
- Text typed at cursor, each stage replaces previous in-place
- Server: new endpoint on brainstorm.snomiao.com combining Whisper + LLM
- Client: `cpal` for cross-platform audio, `webrtc-vad` for voice detection
- Phases: hotkey → audio capture → VAD → server endpoint → HTTP client → local Whisper (optional)

### BUG: macOS mouse drag doesn't move windows in realtime
**Problem:** Space+E (left click hold) + WASD to drag a window only moves the window when E is released, instead of moving it continuously like a trackpad drag.
**Cause:** `mouse_move` sends `CGEventType::MouseMoved` but while a button is held, macOS expects `CGEventType::LeftMouseDragged` events instead. Need to track button state and switch event type accordingly.
