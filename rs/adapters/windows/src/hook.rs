use std::sync::atomic::{AtomicU32, AtomicU64, AtomicUsize, Ordering};
use std::sync::mpsc;
/// Windows WH_KEYBOARD_LL hook – bridges Win32 key events to ClxEngine.
use std::sync::{Arc, Mutex, OnceLock};

use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetClassNameW, GetForegroundWindow, GetMessageW,
    GetWindowThreadProcessId, SetTimer, SetWindowsHookExW, UnhookWindowsHookEx, HHOOK,
    KBDLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL,
};

use crate::output::{WinPlatform, CLX_EXTRA_INFO};
use crate::shm::SharedState;
use crate::vk::vk_to_keycode;
use capslockx_core::{ClxConfig, ClxEngine, CoreResponse};

// ── Raw HHOOK stored as usize for atomic access ───────────────────────────────

static HOOK_RAW: AtomicUsize = AtomicUsize::new(0);

/// How many times the hook callback has run. Compared against
/// `raw_input::key_event_count()` by `check_hook_alive` to notice that Windows
/// has quietly removed our hook.
static HOOK_CALLS: AtomicU64 = AtomicU64::new(0);

// ── Engine (initialised once via init_engine before hook installs) ────────────

static ENGINE: OnceLock<Arc<ClxEngine>> = OnceLock::new();

// Serialize recovered releases with hook dispatch, so an old UP cannot reset
// a model between a new DOWN and its activation. The ticker uses try_lock:
// a callback that is itself stuck must not strand the ticker behind it.
static INPUT_STATE: Mutex<crate::release_state::ReleaseState> =
    Mutex::new(crate::release_state::ReleaseState::new());

// ── Shared memory (set from main before hook install) ────────────────────────

static SHM: OnceLock<SharedState> = OnceLock::new();

/// Last tray-active state for edge detection (0 = off, 1 = on, u32::MAX = uninitialised).
static LAST_TRAY_ACTIVE: AtomicU32 = AtomicU32::new(u32::MAX);

/// Channel sender to the tray/cursor worker thread. `None` until `init_tray_worker`
/// is called. The hook callback must NEVER call Tauri or SystemParametersInfoW
/// directly — those can block on GUI locks held by threads waiting on the hook,
/// creating a 3-way deadlock that leaves the process unkillable.
static TRAY_TX: OnceLock<mpsc::Sender<bool>> = OnceLock::new();

/// Spawn the tray/cursor-visibility worker thread. Must be called once before
/// `install_hook()`. The worker coalesces bursts of edge events so a rapid
/// on/off/on flicker only triggers the most recent state.
pub fn init_tray_worker() {
    let (tx, rx) = mpsc::channel::<bool>();
    if TRAY_TX.set(tx).is_err() {
        return;
    }
    let _ = std::thread::Builder::new()
        .name("clx-tray-worker".into())
        .spawn(move || {
            while let Ok(first) = rx.recv() {
                // Coalesce any backlog so we only apply the latest state.
                let mut latest = first;
                while let Ok(next) = rx.try_recv() {
                    latest = next;
                }
                // Firewall each iteration: `update_tray_icon` posts to the UI
                // thread (it must never block on it -- see its doc comment)
                // and `cursor_visibility` calls into Win32. A panic here would
                // otherwise kill this worker for the rest of the session (tray
                // icon + cursor visibility would silently stop tracking mode),
                // or abort the process if it unwound through a framework WndProc.
                let r = std::panic::catch_unwind(|| {
                    crate::update_tray_icon(latest);
                    if latest {
                        crate::cursor_visibility::enable();
                    } else {
                        crate::cursor_visibility::disable();
                    }
                });
                if r.is_err() {
                    crash_log_sync("[PANIC] recovered in clx-tray-worker");
                }
            }
        });
}

/// Store the shared memory handle so the hook callback can publish mode changes.
pub fn init_shared_state(shm: SharedState) {
    let _ = SHM.set(shm);
}

/// Get a reference to the shared memory state (if initialised).
pub fn get_shared_state() -> Option<&'static SharedState> {
    SHM.get()
}

// ── Win32 message constants ───────────────────────────────────────────────────

const WM_KEYDOWN: u32 = 0x0100;
const WM_KEYUP: u32 = 0x0101;
const WM_SYSKEYDOWN: u32 = 0x0104;
const WM_SYSKEYUP: u32 = 0x0105;
const LLKHF_UP: u32 = 0x80;
const LLKHF_INJECTED: u32 = 0x10;
const LLKHF_ALTDOWN: u32 = 0x20;

// ── Public API ────────────────────────────────────────────────────────────────

pub fn init_engine(config: ClxConfig) {
    // Drive AccModel ticks from the main thread (via SetTimer) instead of
    // background threads.  This ensures SendInput runs on the hook thread,
    // avoiding phantom modifier key-up events from cross-thread injection.
    capslockx_core::acc_model::set_external_tick(true);
    let platform = Arc::new(WinPlatform::new());
    ENGINE.set(ClxEngine::with_config(platform, config)).ok();
}

pub fn engine() -> Arc<ClxEngine> {
    ENGINE
        .get()
        .expect("init_engine must be called before engine()")
        .clone()
}

/// Which hook thread currently owns the registration.
///
/// A thread whose generation no longer matches has been superseded and retires
/// itself. This is how a wedged hook thread is replaced rather than repaired: the
/// blocked one cannot be rescued, so a fresh one takes over and the old one
/// leaves when (if) it ever comes back.
static HOOK_GENERATION: AtomicU64 = AtomicU64::new(0);

/// Install the WH_KEYBOARD_LL hook on a **dedicated thread** and spawn the
/// AccModel ticker.
///
/// The hook used to live on the Tauri/UI thread, because a low-level hook
/// installed elsewhere would not fire while one of clx's *own* windows had focus
/// — cross-thread hook delivery cannot get through the process's serialized input
/// queue. That reason is gone: prefs, the prompt and the overlay are all separate
/// processes now (`clx prefs-window` and friends), so this process owns no
/// focusable window for the hook to be blind to.
///
/// What the old arrangement cost, on the other hand, was measured: when a hook
/// callback blocked, it took the UI thread with it, and with the UI thread went
/// the message pump, the tray menu, and the `SetTimer` that the hook watchdog
/// was running on — so the one mechanism meant to notice a dead hook died
/// alongside it. A thread whose only job is to own the hook and pump its
/// messages can be abandoned and replaced; the UI thread cannot.
pub fn install_hook() {
    spawn_hook_thread(1);

    // Test affordance: drop our own hook after N ms, exactly as Windows does
    // when a callback overruns `LowLevelHooksTimeout`. The watchdog should then
    // notice and start a replacement.
    //
    // This exists because the watchdog's recovery path had already failed once in
    // the field without anyone noticing — it was sitting on the thread that dies.
    // A recovery mechanism nobody has ever seen recover is a guess, and there is
    // no other way to provoke this from outside the process.
    if let Ok(ms) = std::env::var("CLX_TEST_DROP_HOOK_MS") {
        if let Ok(ms) = ms.parse::<u64>() {
            std::thread::Builder::new()
                .name("clx-test-drop-hook".into())
                .spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(ms));
                    crash_log_sync("[hook] TEST: dropping the hook on purpose");
                    uninstall_hook();
                })
                .ok();
        }
    }

    // Cursor-visibility nudge timer stays on the UI thread: it calls
    // SystemParametersInfoW, which has no business anywhere near the hook.
    unsafe {
        SetTimer(None, 0, 250, Some(nudge_timer_proc));
    }

    // High-frequency ticker thread for AccModel physics (~166 FPS).
    // Uses timeBeginPeriod(1) for 1ms Sleep resolution, then sleeps ~6ms
    // per tick. SendInput for mouse movement works fine cross-thread.
    // A low-frequency SetTimer (250ms) handles cursor-visibility nudges
    // that don't need high frequency.
    std::thread::Builder::new()
        .name("clx-ticker".into())
        .spawn(|| {
            use std::sync::atomic::AtomicU64;
            static TICK_COUNT: AtomicU64 = AtomicU64::new(0);
            static LAST_LOG: AtomicU64 = AtomicU64::new(0);
            /// ~6 ms per tick, so every 42nd tick is roughly four times a second.
            static NEXT_HOOK_CHECK: AtomicU64 = AtomicU64::new(0);

            // Request 1ms timer resolution from Windows.
            unsafe {
                windows::Win32::Media::timeBeginPeriod(1);
            }

            loop {
                std::thread::sleep(std::time::Duration::from_millis(6)); // ~166 FPS

                if let Some(engine) = ENGINE.get() {
                    // Raw release reconciliation. See `RECONCILE_RELEASES`
                    // for why it was briefly switched off and why that was a
                    // false alarm; it runs here, on the ticker, never on the
                    // hook thread.
                    if RECONCILE_RELEASES {
                        reconcile_releases(engine);
                    }
                    engine.tick();
                }

                // Hook liveness, checked from here because this thread is the
                // one that survives. It ran on the UI thread's timer until a
                // blocked hook callback took that thread down and the watchdog
                // with it — the ticker kept logging 156 FPS throughout, for
                // fifteen minutes, while clx was deaf and nothing noticed.
                // Four times a second is plenty; the hot part of the check is
                // two atomic loads.
                if NEXT_HOOK_CHECK.fetch_add(1, Ordering::Relaxed) % 42 == 0 {
                    check_hook_alive();
                }

                // FPS logging — every 2 seconds.
                let count = TICK_COUNT.fetch_add(1, Ordering::Relaxed) + 1;
                let now_ms = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;
                let last = LAST_LOG.load(Ordering::Relaxed);
                if last == 0 {
                    LAST_LOG.store(now_ms, Ordering::Relaxed);
                    TICK_COUNT.store(0, Ordering::Relaxed);
                } else if now_ms - last >= 2000 {
                    let elapsed_s = (now_ms - last) as f64 / 1000.0;
                    let fps = count as f64 / elapsed_s;
                    debug_log(&format!(
                        "[CLX] tick: {:.1} FPS ({} ticks in {:.1}s)",
                        fps, count, elapsed_s
                    ));
                    LAST_LOG.store(now_ms, Ordering::Relaxed);
                    TICK_COUNT.store(0, Ordering::Relaxed);
                }
            }
        })
        .expect("failed to spawn ticker thread");
}

unsafe extern "system" fn nudge_timer_proc(_hwnd: HWND, _msg: u32, _id: usize, _time: u32) {
    // Also an `extern "system"` callback — same panic firewall rationale as
    // `keyboard_proc`. Swallow on panic (the nudge is best-effort cosmetics).
    let _ = std::panic::catch_unwind(|| {
        let active = LAST_TRAY_ACTIVE.load(Ordering::Relaxed);
        if active != 0 && active != u32::MAX {
            crate::cursor_visibility::nudge();
        }
    });
}

/// Replace the hook after Windows has removed it, or after its thread wedged.
///
/// Deliberately *not* an in-place reinstall on the calling thread. A low-level
/// hook must be owned by a thread that pumps messages, and the caller here is the
/// ticker, which does not. More importantly, the failure this recovers from
/// includes "the hook thread is blocked inside a callback and will never return",
/// which no amount of work on that thread can fix. So a new generation takes
/// over; the old thread retires if it is merely idle, and is abandoned if it is
/// stuck.
fn replace_hook_thread() {
    // Drop the stale registration so an abandoned thread's hook stops being
    // consulted. Unhooking one Windows already removed simply fails, which is
    // why the result is ignored.
    let old = HOOK_RAW.swap(0, Ordering::SeqCst) as *mut _;
    if !std::ptr::eq(old, std::ptr::null()) {
        unsafe {
            let _ = UnhookWindowsHookEx(HHOOK(old));
        }
    }
    let next = HOOK_GENERATION.load(Ordering::SeqCst) + 1;
    // Bumped before the new thread starts so a live old thread notices it has
    // been superseded even if the new one is slow to install.
    HOOK_GENERATION.store(next, Ordering::SeqCst);
    spawn_hook_thread(next);
}

/// Notice that the hook has stopped being called, and replace it.
///
/// Windows removes a low-level keyboard hook **without telling anyone** when its
/// callback exceeds `LowLevelHooksTimeout` (300 ms by default), and nothing used
/// to put it back: clx went silently deaf until restarted. Seen twice, from both
/// directions — a hook callback that blocked in `SendInput` and never returned,
/// and Narrator adding slow UI Automation work to the input path.
///
/// Runs on the ticker thread. That is not incidental: the first version of this
/// ran on the UI thread's `SetTimer`, and when a blocked callback took the UI
/// thread down it took the watchdog with it, so the check never fired in the one
/// case it existed for.
///
/// Raw input is the witness. It only receives events that no hook suppressed, so
/// in normal operation everything it sees was also offered to our hook. Raw
/// events piling up while the hook's call count sits still therefore means we
/// are no longer in the chain. Note what this cannot false-positive on: keys clx
/// suppresses reach neither counter, so holding a CLX trigger looks like silence
/// on both sides rather than like a dead hook.
///
/// Replacing has a second benefit. The chain runs most-recently-installed first,
/// so a hook installed after ours — Narrator's, for one — sits ahead of us and
/// can swallow a key before we ever see it. A fresh hook goes back to the front.
fn check_hook_alive() {
    /// Physical events that must accumulate with a motionless hook before we
    /// act. Four is two whole keystrokes (make + break), enough that a single
    /// straggler racing the counters cannot trip it.
    const STALE_BUDGET: u64 = 4;

    // Nothing to judge until a hook has been installed at all.
    if HOOK_GENERATION.load(Ordering::SeqCst) == 0 {
        return;
    }

    static LAST_RAW: AtomicU64 = AtomicU64::new(0);
    static LAST_HOOK: AtomicU64 = AtomicU64::new(0);
    static STALE_RAW: AtomicU64 = AtomicU64::new(0);

    let raw = crate::raw_input::key_event_count();
    let hook = HOOK_CALLS.load(Ordering::Relaxed);
    let raw_delta = raw.saturating_sub(LAST_RAW.swap(raw, Ordering::Relaxed));
    let hook_delta = hook.saturating_sub(LAST_HOOK.swap(hook, Ordering::Relaxed));

    // Any callback at all means we are still in the chain.
    if hook_delta > 0 {
        STALE_RAW.store(0, Ordering::Relaxed);
        return;
    }

    // Accumulated rather than required within one 250 ms window: a few
    // keystrokes spread over a couple of seconds still add up to a verdict.
    let stale = STALE_RAW.fetch_add(raw_delta, Ordering::Relaxed) + raw_delta;
    if stale < STALE_BUDGET {
        return;
    }
    STALE_RAW.store(0, Ordering::Relaxed);

    // crash_log_sync, not debug_log: this is durable and rare, and someone
    // reading a report of "clx went deaf" days later needs to find it.
    crash_log_sync(&format!(
        "[hook] {stale} physical key event(s) with no callback — hook gen {} is \
         gone or wedged; starting a replacement",
        HOOK_GENERATION.load(Ordering::SeqCst)
    ));
    replace_hook_thread();
}

/// Own the hook on a thread of its own, and pump the messages that keep it
/// serviced.
///
/// A low-level hook is delivered to the installing thread's message queue, so a
/// hook thread without a `GetMessage` loop receives nothing. The loop also gives
/// us the retirement check: a superseded generation exits and unhooks, which is
/// what makes replacing a wedged hook safe rather than merely hopeful.
fn spawn_hook_thread(generation: u64) {
    let spawned = std::thread::Builder::new()
        .name(format!("clx-hook-{generation}"))
        .spawn(move || unsafe {
            let hmod = GetModuleHandleW(None).unwrap_or_default();
            let hhook = match SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), hmod, 0) {
                Ok(hhook) => hhook,
                Err(error) => {
                    crash_log_sync(&format!("[hook] gen {generation}: install failed: {error}"));
                    return;
                }
            };
            HOOK_RAW.store(hhook.0 as usize, Ordering::SeqCst);
            HOOK_GENERATION.store(generation, Ordering::SeqCst);
            debug_log(&format!("[hook] gen {generation} installed and pumping"));

            let mut msg = MSG::default();
            while GetMessageW(&mut msg, None, 0, 0).0 > 0 {
                if crate::SHUTDOWN.load(Ordering::Relaxed) {
                    break;
                }
                if HOOK_GENERATION.load(Ordering::SeqCst) != generation {
                    debug_log(&format!("[hook] gen {generation} superseded — retiring"));
                    break;
                }
                DispatchMessageW(&msg);
            }
            let _ = UnhookWindowsHookEx(hhook);
        })
        .is_ok();
    if !spawned {
        crash_log_sync("[hook] could not spawn the hook thread — clx has no keyboard");
    }
}

pub fn uninstall_hook() {
    let raw = HOOK_RAW.swap(0, Ordering::SeqCst) as *mut _;
    if !std::ptr::eq(raw, std::ptr::null()) {
        unsafe {
            let _ = UnhookWindowsHookEx(HHOOK(raw));
        }
    }
}

// ── Hook callback ─────────────────────────────────────────────────────────────

unsafe extern "system" fn keyboard_proc(n_code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    // Panic firewall. This is an `extern "system"` boundary: if a panic unwound
    // out of here (engine dispatch, a module, a COM call), Rust would abort the
    // whole process (STATUS_STACK_BUFFER_OVERRUN / 0xC0000409). Catch it, log it,
    // and pass the key through so a transient bug degrades to one dropped hotkey
    // instead of taking clx down.
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
        keyboard_proc_inner(n_code, w_param, l_param)
    })) {
        Ok(r) => r,
        Err(_) => {
            crash_log_sync("[PANIC] recovered in keyboard_proc — passing key through");
            call_next(n_code, w_param, l_param)
        }
    }
}

unsafe fn keyboard_proc_inner(n_code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    // Counted before anything can return early: this is "was the hook called",
    // not "did the hook act". A relaxed increment costs nothing on this path.
    HOOK_CALLS.fetch_add(1, Ordering::Relaxed);

    if n_code < 0 {
        return call_next(n_code, w_param, l_param);
    }

    let kb = &*(l_param.0 as *const KBDLLHOOKSTRUCT);

    let flags = kb.flags.0;

    let msg = w_param.0 as u32;
    let is_up = (flags & LLKHF_UP) != 0;
    let pressed = matches!(msg, WM_KEYDOWN | WM_SYSKEYDOWN) && !is_up;
    let released = matches!(msg, WM_KEYUP | WM_SYSKEYUP) && is_up;

    let code = vk_to_keycode(kb.vkCode);
    let injected = (flags & LLKHF_INJECTED) != 0;
    let is_ours = injected && kb.dwExtraInfo == CLX_EXTRA_INFO;

    // Debug: log ALL key events with the currently focused window so we can tell
    // whether the hook fires while a given window (e.g. the prefs WebView2) has
    // focus. Gated by debug_enabled() so the GetForegroundWindow/class lookup
    // never runs on the hot path in production.
    if debug_enabled() && (pressed || released) {
        debug_log(&format!(
            "[hook] vk=0x{:02X} {:?} {} inj={} ours={} fg=[{}]",
            kb.vkCode,
            code,
            if pressed { "DN" } else { "UP" },
            injected,
            is_ours,
            fg_window_desc()
        ));
    }

    // Skip events injected by us
    if is_ours {
        return call_next(n_code, w_param, l_param);
    }

    if !pressed && !released {
        return call_next(n_code, w_param, l_param);
    }

    // AHK-parity Alt+Tab / Win+Tab switcher enhancement. While a task-switcher
    // window is focused and Alt is held, remap WASD→arrows, the media/volume
    // pad (Q E R F T G H J K L M), and X/C→close. Runs before engine dispatch
    // and only on non-injected Alt+<key> key-downs, so ordinary typing and CLX
    // mode are untouched. LLKHF_ALTDOWN gates the class lookup off the hot path.
    if pressed && (flags & LLKHF_ALTDOWN) != 0 && crate::alt_tab::try_handle(kb.vkCode) {
        return LRESULT(1);
    }

    let engine = ENGINE.get().expect("init_engine not called");
    let resp = {
        let mut input = INPUT_STATE.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(slot) =
            crate::release_state::key_slot(kb.scanCode, flags & 0x01 != 0, kb.vkCode)
        {
            input.hook_event(slot, kb.vkCode, pressed, kb.time);
        }
        engine.on_key_event(code, pressed)
    };
    if debug_enabled() {
        debug_log(&format!(
            "[hook] -> {:?} mode={}",
            resp,
            engine.state().mode()
        ));
    }

    publish_mode(engine);

    match resp {
        CoreResponse::Suppress => LRESULT(1),
        CoreResponse::PassThrough => call_next(n_code, w_param, l_param),
    }
}

/// Master switch for raw release reconciliation.
///
/// Briefly set to `false` on 2026-09-23 while chasing a hook death that looked
/// correlated with it. It was not: with reconciliation off, the hook still died
/// after the first Space. That whole session ran with five unkillable zombie
/// clx processes each holding a `WH_KEYBOARD_LL` registration, so every result
/// from it — including that one — is untrustworthy. Re-enabled to be retested
/// on a clean machine.
const RECONCILE_RELEASES: bool = true;

fn reconcile_releases(engine: &ClxEngine) {
    let mut input = match INPUT_STATE.try_lock() {
        Ok(input) => input,
        Err(std::sync::TryLockError::Poisoned(e)) => e.into_inner(),
        Err(std::sync::TryLockError::WouldBlock) => return,
    };
    let cancelled = crate::raw_input::take_cancel();
    if cancelled {
        input.clear();
        engine.emergency_stop();
    }
    let mut recovered = Vec::new();
    crate::raw_input::drain_releases(|slot, time| {
        if let Some(vk) = input.take_release(slot, time) {
            if engine.release_missed_key(vk_to_keycode(vk)) {
                recovered.push(vk);
            }
        }
    });
    drop(input); // Never make the hook wait for diagnostic disk I/O.
    if cancelled || !recovered.is_empty() {
        publish_mode(engine);
    }
    if cancelled {
        crash_log_sync("[input-reconcile] input desktop lost or observer resumed; cancelled input");
    }
    for vk in recovered {
        crash_log_sync(&format!(
            "[input-reconcile] recovered missed UP vk=0x{vk:02X}"
        ));
    }
}

fn publish_mode(engine: &ClxEngine) {
    // Publish current mode to shared memory so AHK extensions can read it.
    let mode = engine.state().mode();
    if let Some(shm) = SHM.get() {
        shm.write_mode(mode);
    }

    // Dispatch mode-edge transitions to the tray worker. The hook callback
    // must never call Tauri or SystemParametersInfoW directly — sending on an
    // mpsc channel is wait-free (one atomic CAS + a malloc) and safe here.
    // Cursor-visibility nudges are handled by the SetTimer tick instead.
    let active = u32::from(mode != 0);
    let prev = LAST_TRAY_ACTIVE.swap(active, Ordering::Relaxed);
    if prev != active {
        if let Some(tx) = TRAY_TX.get() {
            let _ = tx.send(active != 0);
        }
    }
}

/// Channel to the background log writer. `Some(_)` only when CLX_DEBUG is set.
/// `None` → logging disabled → `debug_log` is a no-op (no disk I/O, no alloc).
static DEBUG_TX: OnceLock<Option<mpsc::Sender<String>>> = OnceLock::new();

fn log_path() -> Option<std::path::PathBuf> {
    std::env::var("TEMP")
        .ok()
        .map(|tmp| std::path::PathBuf::from(format!(r"{}\capslockx_hook.log", tmp)))
}

/// Initialise debug logging. Call once early in `main`.
///
/// Logging is **off by default** and only enabled when `CLX_DEBUG` is set (and
/// not empty/"0"). This matters because `debug_log` is called several times per
/// keystroke from inside the WH_KEYBOARD_LL callback — the old version did a
/// synchronous file open+append+write there, stalling the system input path and
/// bloating a multi-MB log. When enabled, writes are handed to a background
/// thread so the hook callback never touches the disk.
pub fn init_debug_log() {
    let enabled = std::env::var("CLX_DEBUG")
        .map(|v| !v.is_empty() && v != "0")
        .unwrap_or(false);
    if !enabled {
        let _ = DEBUG_TX.set(None);
        return;
    }
    let (tx, rx) = mpsc::channel::<String>();
    let _ = std::thread::Builder::new()
        .name("clx-debug-log".into())
        .spawn(move || {
            use std::io::Write as _;
            let Some(path) = log_path() else { return };
            let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)
            else {
                return;
            };
            while let Ok(line) = rx.recv() {
                let _ = writeln!(file, "{} {}", now_ms(), line);
                // Drain any backlog before flushing so bursts batch into one flush.
                while let Ok(next) = rx.try_recv() {
                    let _ = writeln!(file, "{} {}", now_ms(), next);
                }
                let _ = file.flush();
            }
        });
    let _ = DEBUG_TX.set(Some(tx));
}

/// Milliseconds since the Unix epoch — used to timestamp log lines so a
/// separate focus poller can be time-correlated with hook activity.
fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// Cheap check used to gate expensive per-key logging (string formatting +
/// GetForegroundWindow) so it only runs when CLX_DEBUG is active.
#[inline]
fn debug_enabled() -> bool {
    matches!(DEBUG_TX.get(), Some(Some(_)))
}

/// Describe the currently focused (foreground) window: class name + PID. Uses
/// only non-blocking calls — notably NOT GetWindowTextW, which sends WM_GETTEXT
/// and could block the hook thread if the target window is busy.
fn fg_window_desc() -> String {
    unsafe {
        let hwnd = GetForegroundWindow();
        let mut buf = [0u16; 128];
        let n = GetClassNameW(hwnd, &mut buf);
        let class = String::from_utf16_lossy(&buf[..n.max(0) as usize]);
        let mut pid = 0u32;
        let _ = GetWindowThreadProcessId(hwnd, Some(&mut pid));
        format!("hwnd=0x{:X} class='{}' pid={}", hwnd.0 as usize, class, pid)
    }
}

/// Hot-path log: a cheap no-op unless CLX_DEBUG enabled the background writer.
/// Safe to call from the keyboard-hook callback — never blocks on disk.
#[inline]
pub fn debug_log(msg: &str) {
    if let Some(Some(tx)) = DEBUG_TX.get() {
        let _ = tx.send(msg.to_string());
    }
}

/// Synchronous, always-on log for rare critical events (panics) where we want
/// the record on disk even if the async writer isn't up. Not for the hot path.
pub fn debug_log_sync(msg: &str) {
    use std::io::Write as _;
    if let Some(path) = log_path() {
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
        {
            let _ = writeln!(f, "{}", msg);
        }
    }
}

/// Stable crash-log path under `%LOCALAPPDATA%\CapsLockX\crash.log`.
///
/// Unlike `capslockx_hook.log` (which lives in `%TEMP%` and gets wiped by Disk
/// Cleanup / Storage Sense), this survives so a post-mortem days later still has
/// the panic message + location. A 12-day-uptime abort on 2026-08-01 was lost
/// precisely because the only record was the TEMP hook log, which was gone by
/// the time anyone looked.
fn crash_log_path() -> Option<std::path::PathBuf> {
    std::env::var("LOCALAPPDATA").ok().map(|base| {
        std::path::PathBuf::from(base)
            .join("CapsLockX")
            .join("crash.log")
    })
}

/// Append a critical (crash/panic) record to the durable crash log. Best-effort,
/// synchronous, and allocation-light — safe to call from a panic hook.
pub fn crash_log_sync(msg: &str) {
    use std::io::Write as _;
    if let Some(path) = crash_log_path() {
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
        {
            let _ = writeln!(f, "{}", msg);
        }
    }
    // Mirror to the TEMP hook log too, so a single `clx` session's story stays
    // in one place while it lasts.
    debug_log_sync(msg);
}

#[inline(always)]
unsafe fn call_next(n_code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    let raw = HOOK_RAW.load(Ordering::Relaxed) as *mut _;
    CallNextHookEx(HHOOK(raw), n_code, w_param, l_param)
}
