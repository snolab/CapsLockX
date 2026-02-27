//! evdev keyboard hook – bridges Linux input events to ClxEngine.
//!
//! One thread is spawned per physical keyboard.  Each thread opens the device
//! with EVIOCGRAB (exclusive), so the OS never receives events that CLX
//! decides to suppress.
//!
//! No self-inject detection is required: uinput creates a separate device
//! node from the grabbed physical keyboard, so hook threads never read back
//! their own output.

use std::path::PathBuf;
use std::sync::Arc;

use evdev::{InputEventKind, Key};
use once_cell::sync::Lazy;

use capslockx_core::{ClxEngine, CoreResponse};

use crate::key_map::evdev_key_to_keycode;
use crate::output::{passthrough, LinuxPlatform};

// ── Engine (created once on first use) ───────────────────────────────────────

static ENGINE: Lazy<Arc<ClxEngine>> = Lazy::new(|| {
    let platform = Arc::new(LinuxPlatform::new());
    ClxEngine::new(platform)
});

// ── Public API ────────────────────────────────────────────────────────────────

/// Enumerate physical keyboards, grab each one, and spawn an event thread.
pub fn install_hooks() {
    // Force engine initialisation now, before any events can arrive.
    Lazy::force(&ENGINE);

    let keyboards = find_keyboards();
    if keyboards.is_empty() {
        eprintln!("[CLX] WARNING: no keyboards found (no device with KEY_CAPSLOCK)");
        return;
    }

    for path in keyboards {
        let path_clone = path.clone();
        std::thread::Builder::new()
            .name(format!("clx-kbd:{}", path.display()))
            .spawn(move || run_device(path_clone))
            .expect("failed to spawn keyboard thread");
    }
}

// ── Keyboard enumeration ──────────────────────────────────────────────────────

/// Return the paths of all input devices that advertise `KEY_CAPSLOCK`.
fn find_keyboards() -> Vec<PathBuf> {
    evdev::enumerate()
        .filter(|(_, dev)| {
            dev.supported_keys()
                .map_or(false, |keys| keys.contains(Key(58)))  // KEY_CAPSLOCK = 58
        })
        .map(|(path, _)| path)
        .collect()
}

// ── Per-device event loop ─────────────────────────────────────────────────────

fn run_device(path: PathBuf) {
    let mut device = match evdev::Device::open(&path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("[CLX] failed to open {:?}: {}", path, e);
            return;
        }
    };

    if let Err(e) = device.grab() {
        if e.kind() == std::io::ErrorKind::PermissionDenied {
            eprintln!("[CLX] Permission denied grabbing {:?}", path);
            eprintln!("[CLX] Add yourself to the 'input' group and log out/in:");
            eprintln!("[CLX]   sudo usermod -aG input $USER");
            eprintln!("[CLX] Or create a udev rule:");
            eprintln!("[CLX]   echo 'KERNEL==\"event*\", GROUP=\"input\", MODE=\"0660\"' \\");
            eprintln!("[CLX]        | sudo tee /etc/udev/rules.d/99-input.rules");
        } else {
            eprintln!("[CLX] failed to grab {:?}: {}", path, e);
        }
        return;
    }

    eprintln!("[CLX] grabbed keyboard {:?}", path);

    loop {
        // fetch_events() blocks until at least one event is ready.
        let events: Vec<_> = match device.fetch_events() {
            Ok(iter) => iter.collect(),
            Err(e) => {
                eprintln!("[CLX] fetch_events error on {:?}: {}", path, e);
                break;
            }
        };

        for event in events {
            // Only act on EV_KEY events; ignore EV_SYN, EV_MSC, etc.
            if let InputEventKind::Key(key) = event.kind() {
                let value = event.value();
                if value == 2 { continue; }  // skip auto-repeat

                let pressed = value == 1;
                let code = evdev_key_to_keycode(key);

                match ENGINE.on_key_event(code, pressed) {
                    CoreResponse::Suppress => {
                        // Dropped – device is exclusively grabbed, OS never sees it.
                    }
                    CoreResponse::PassThrough => {
                        passthrough(event);
                    }
                }
            }
        }
    }

    eprintln!("[CLX] keyboard thread exiting for {:?}", path);
}
