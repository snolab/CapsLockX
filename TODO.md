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

### BUG: macOS mouse drag doesn't move windows in realtime
**Problem:** Space+E (left click hold) + WASD to drag a window only moves the window when E is released, instead of moving it continuously like a trackpad drag.
**Cause:** `mouse_move` sends `CGEventType::MouseMoved` but while a button is held, macOS expects `CGEventType::LeftMouseDragged` events instead. Need to track button state and switch event type accordingly.
