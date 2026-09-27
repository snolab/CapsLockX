//! Watch what the user actually types, so `#DPW#` can fire.
//!
//! The hook sees virtual keys; a hotstring needs *characters*. `#` is Shift+3 on a
//! US layout and something else elsewhere, so the mapping has to come from
//! Windows rather than a table of assumptions — that is `ToUnicodeEx`, against the
//! thread's current keyboard layout.
//!
//! **The hook callback does none of this.** It pushes the raw event into a channel
//! and returns. Everything else — the layout call, the matching, the backspaces,
//! running the action — happens on this module's worker thread. That split is not
//! stylistic: injecting from inside a hook callback is what froze clx for a day,
//! and `ToUnicodeEx` is a call into the layout DLL that has no business on the
//! input path either.
//!
//! What is deliberately *not* kept: anything before the leader character. See
//! `capslockx_core::hotstring` for why that bound matters.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::OnceLock;

use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, GetKeyboardLayout, ToUnicodeEx,
};
use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;

use capslockx_core::bindings::Action;
use capslockx_core::hotstring::{Hotstrings, DEFAULT_LEADER};
use capslockx_core::key_code::KeyCode;
use capslockx_core::Platform as _;

/// One observed key-down, as the hook saw it.
struct Typed {
    vk: u16,
    scan: u32,
    /// Foreground window at the time. A different window means the buffer is no
    /// longer adjacent to the cursor, so a partial trigger must be abandoned
    /// rather than backspaced into someone else's text.
    hwnd: isize,
    /// Modifier state, sampled in the hook rather than read later.
    ///
    /// `GetKeyboardState` reports the *calling thread's* key state, and it only
    /// updates as that thread reads key messages from its queue. The worker never
    /// reads any, so its table is permanently empty — `ToUnicodeEx` then sees no
    /// Shift and turns `Shift+3` into `3`, so `#DPW#` arrives as `3dpw3` and no
    /// trigger can ever match. That is exactly how the first version failed.
    shift: bool,
    ctrl: bool,
    alt: bool,
    caps: bool,
}

static TX: OnceLock<mpsc::Sender<Typed>> = OnceLock::new();
static ENABLED: AtomicBool = AtomicBool::new(false);

/// Start the watcher with the triggers from the bindings file. A no-op when there
/// are none, which is the common case — nobody pays for a feature they have not
/// configured.
pub fn start(triggers: Vec<(String, Action)>) {
    if triggers.is_empty() || TX.get().is_some() {
        return;
    }
    let mut hotstrings = Hotstrings::new(DEFAULT_LEADER);
    for (trigger, action) in triggers {
        match hotstrings.add(&trigger, action) {
            Ok(()) => {}
            Err(why) => eprintln!("[CLX] hotstring: {why}"),
        }
    }
    if hotstrings.is_empty() {
        return;
    }
    eprintln!("[CLX] hotstrings: {} trigger(s) watching", hotstrings.len());

    let (tx, rx) = mpsc::channel::<Typed>();
    let spawned = std::thread::Builder::new()
        .name("clx-hotstring".into())
        .spawn(move || worker(rx, hotstrings))
        .is_ok();
    if spawned {
        let _ = TX.set(tx);
        ENABLED.store(true, Ordering::Release);
    }
}

/// Called from the hook for every key-down that reached the app.
///
/// Must stay trivial: a load and a channel send. No layout calls, no allocation
/// beyond the message, nothing that can block.
#[inline]
pub fn observe(vk: u16, scan: u32) {
    if !ENABLED.load(Ordering::Relaxed) {
        return;
    }
    if let Some(tx) = TX.get() {
        // GetAsyncKeyState is global rather than tied to a thread's message queue,
        // which is why the modifiers are sampled here and carried along.
        const VK_SHIFT: i32 = 0x10;
        const VK_CONTROL: i32 = 0x11;
        const VK_MENU: i32 = 0x12;
        const VK_CAPITAL: i32 = 0x14;
        let down = |vk: i32| unsafe { GetAsyncKeyState(vk) as u16 & 0x8000 != 0 };
        let toggled = |vk: i32| unsafe { GetAsyncKeyState(vk) as u16 & 0x0001 != 0 };

        let hwnd = unsafe { GetForegroundWindow() }.0 as isize;
        let _ = tx.send(Typed {
            vk,
            scan,
            hwnd,
            shift: down(VK_SHIFT),
            ctrl: down(VK_CONTROL),
            alt: down(VK_MENU),
            caps: toggled(VK_CAPITAL),
        });
    }
}

fn worker(rx: mpsc::Receiver<Typed>, mut hotstrings: Hotstrings) {
    let platform = crate::output::WinPlatform::new();
    let mut last_hwnd = 0isize;

    while let Ok(event) = rx.recv() {
        if crate::SHUTDOWN.load(Ordering::Relaxed) {
            return;
        }

        // Focus moved: whatever was buffered is no longer next to the cursor.
        if event.hwnd != last_hwnd {
            hotstrings.reset();
            last_hwnd = event.hwnd;
        }

        let Some(c) = character_for(&event) else {
            // Not a character — an arrow, Home, a function key. These move the
            // caret or do something else entirely, so a partial trigger is stale.
            hotstrings.reset();
            continue;
        };

        if trace() {
            eprintln!("[CLX] hotstring: saw {c:?}");
        }
        let Some(matched) = hotstrings.push(c) else {
            continue;
        };
        let erase = matched.erase;
        let action = matched.action.clone();

        // The app already received the trigger text; take it back before adding
        // anything. Erasing first also means a failure leaves the user's typing
        // intact rather than half-deleted.
        for _ in 0..erase {
            platform.key_tap(KeyCode::Backspace);
        }

        match action {
            Action::Plugin { command, args } => {
                match capslockx_core::plugin::run(&command, &args, &platform) {
                    capslockx_core::plugin::Outcome::Exited(0) => {}
                    other => eprintln!("[CLX] hotstring: {command}: {other:?}"),
                }
            }
            Action::Script { path, args } => {
                match capslockx_core::script::run(&path, &args, &platform) {
                    capslockx_core::script::Outcome::Performed(_) => {}
                    capslockx_core::script::Outcome::Failed(why) => {
                        eprintln!("[CLX] hotstring: {path}: {why}")
                    }
                }
            }
        }
    }
}

/// The character this key produces under the current layout, if any.
///
/// `ToUnicodeEx` with the real keyboard state, so Shift, AltGr and non-US layouts
/// are Windows' problem rather than ours. Dead keys report no character and are
/// skipped; getting those right would mean tracking composition state, which a
/// hotstring does not need.
fn character_for(event: &Typed) -> Option<char> {
    let (vk, scan) = (event.vk, event.scan);
    unsafe {
        // Built from the modifiers the hook sampled, not read from this thread —
        // see the note on `Typed::shift`.
        let mut state = [0u8; 256];
        if event.shift {
            state[0x10] = 0x80; // VK_SHIFT
        }
        if event.ctrl {
            state[0x11] = 0x80; // VK_CONTROL
        }
        if event.alt {
            state[0x12] = 0x80; // VK_MENU — with Ctrl, this is AltGr
        }
        if event.caps {
            state[0x14] = 0x01; // VK_CAPITAL, a toggle rather than a hold
        }
        let layout = GetKeyboardLayout(0);
        let mut buf = [0u16; 8];
        // Flag 4 = do not change the kernel's dead-key state: this is an
        // observation, and it must not disturb the composition the user is in the
        // middle of.
        let n = ToUnicodeEx(vk as u32, scan, &state, &mut buf, 4, layout);
        if n != 1 {
            return None;
        }
        char::from_u32(buf[0] as u32).filter(|c| !c.is_control())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An unmodified key press, which is what the hook hands the worker for
    /// ordinary typing.
    fn typed(vk: u16) -> Typed {
        Typed {
            vk,
            scan: 0,
            hwnd: 0,
            shift: false,
            ctrl: false,
            alt: false,
            caps: false,
        }
    }

    /// Layout-dependent, so this asserts only what holds on every layout: keys
    /// that carry no printable character yield nothing. Arrow and function keys
    /// give `ToUnicodeEx` nothing at all; Return does produce `\r`, and the point
    /// is that it is filtered out rather than buffered as text — a hotstring must
    /// never be matched across a newline.
    #[test]
    fn control_keys_produce_no_character() {
        for vk in [0x25u16, 0x70, 0x0D] {
            assert_eq!(
                character_for(&typed(vk)),
                None,
                "vk {vk:#x} should carry no printable character"
            );
        }
    }

    /// The modifiers travel with the event because the worker cannot read them:
    /// `GetKeyboardState` describes the calling thread's queue, and the worker
    /// pumps none. If this ever regresses, `Shift+3` decodes as `3` and `#DPW#`
    /// arrives as `3dpw3`, which is exactly how the first version failed — so
    /// assert the two decode differently rather than trusting the comment.
    #[test]
    fn shift_is_taken_from_the_event_not_the_thread() {
        // VK_3 on any layout: the shifted and unshifted characters differ.
        let plain = character_for(&typed(0x33));
        let shifted = character_for(&Typed {
            shift: true,
            ..typed(0x33)
        });
        if let (Some(p), Some(s)) = (plain, shifted) {
            assert_ne!(p, s, "Shift made no difference — modifiers are being lost");
        }
    }

    #[test]
    fn observing_before_start_is_harmless() {
        // ENABLED is false until `start` succeeds; this must not panic or block.
        observe(0x41, 0);
    }
}

/// `CLX_TRACE_TYPING=1` logs each character the watcher derived.
///
/// Off by default and deliberately separate from `CLX_DEBUG`: this is a record of
/// what the user typed, which is the most sensitive thing clx can log, so turning
/// it on has to be a specific decision rather than a side effect of debugging
/// something else.
fn trace() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| {
        matches!(
            std::env::var("CLX_TRACE_TYPING").ok().as_deref(),
            Some("1") | Some("true") | Some("on")
        )
    })
}
