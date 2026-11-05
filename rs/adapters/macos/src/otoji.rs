//! Talks to the otoji app (voice input moved there; CLX+V just triggers it).
//!
//! otoji-tray listens on a Unix datagram socket, `<otoji data dir>/tray-ctl.sock`,
//! one ASCII command per datagram (see otoji's README). If nobody is listening
//! and otoji.app is installed, launch it and retry once the socket appears.
//! The microphone and every voice permission stay with otoji — this file opens
//! nothing but that socket.

use capslockx_core::platform::{OtojiCommand, OtojiSendResult};
use std::os::unix::net::UnixDatagram;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const BUNDLE_ID: &str = "com.snomiao.otoji";
/// otoji loads a speech model before it is useful, but the tray binds its
/// control socket early — this only has to cover process startup.
const LAUNCH_TIMEOUT: Duration = Duration::from_secs(10);

/// Mirrors `otoji::notes::data_dir()` on macOS.
fn data_dir() -> PathBuf {
    if let Some(custom) = std::env::var_os("OTOJI_DATA_DIR") {
        return PathBuf::from(custom);
    }
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Library/Application Support/otoji")
}

fn socket_path() -> PathBuf {
    data_dir().join("tray-ctl.sock")
}

/// Fire one command at the socket. Fails when no tray is listening.
fn send_datagram(path: &Path, cmd: &str) -> std::io::Result<()> {
    UnixDatagram::unbound()?.send_to(cmd.as_bytes(), path)?;
    Ok(())
}

fn is_installed() -> bool {
    let app = "Applications/otoji.app";
    if Path::new("/").join(app).exists() {
        return true;
    }
    if dirs::home_dir().is_some_and(|h| h.join(app).exists()) {
        return true;
    }
    // Anywhere else Spotlight has indexed it.
    std::process::Command::new("mdfind")
        .arg(format!("kMDItemCFBundleIdentifier == '{BUNDLE_ID}'"))
        .output()
        .is_ok_and(|o| o.status.success() && !o.stdout.iter().all(u8::is_ascii_whitespace))
}

/// `open -g`: launch without stealing focus from whatever the user is typing in.
fn launch() -> bool {
    std::process::Command::new("open")
        .args(["-g", "-b", BUNDLE_ID])
        .status()
        .is_ok_and(|s| s.success())
}

/// Deliver `cmd`, launching otoji first if it is installed but not running.
pub fn send(cmd: OtojiCommand) -> OtojiSendResult {
    let path = socket_path();
    if send_datagram(&path, cmd.as_str()).is_ok() {
        return OtojiSendResult::Sent;
    }
    if cmd == OtojiCommand::PttEnd {
        // The tray went away mid-hold: there is no segment left to end, and
        // releasing a key must not launch an app.
        return OtojiSendResult::Sent;
    }
    if !is_installed() {
        return OtojiSendResult::NotInstalled;
    }
    if !launch() {
        return OtojiSendResult::NoResponse;
    }
    // otoji may start listening by itself, so a blind `toggle` could turn it
    // straight back off. The user pressed CLX+V to talk: make it `on`.
    let cmd = if cmd == OtojiCommand::Toggle {
        OtojiCommand::On
    } else {
        cmd
    };
    let started = Instant::now();
    while started.elapsed() < LAUNCH_TIMEOUT {
        if send_datagram(&path, cmd.as_str()).is_ok() {
            return OtojiSendResult::Launched;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    OtojiSendResult::NoResponse
}

/// Preferences → "Voice Settings…": otoji's settings window, launching the app
/// if needed. Returns false when otoji isn't installed.
pub fn open_settings() -> bool {
    let path = socket_path();
    if send_datagram(&path, "settings").is_ok() {
        return true;
    }
    if !is_installed() || !launch() {
        return false;
    }
    std::thread::spawn(move || {
        let started = Instant::now();
        while started.elapsed() < LAUNCH_TIMEOUT {
            if send_datagram(&path, "settings").is_ok() {
                return;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
    });
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_socket(name: &str) -> PathBuf {
        // Short path: sun_path is limited to ~100 bytes on macOS.
        std::env::temp_dir().join(format!("clx-{}-{name}.sock", std::process::id()))
    }

    #[test]
    fn command_reaches_a_listening_tray() {
        let path = temp_socket("ok");
        let _ = std::fs::remove_file(&path);
        let tray = UnixDatagram::bind(&path).unwrap();
        send_datagram(&path, OtojiCommand::Toggle.as_str()).unwrap();
        let mut buf = [0u8; 32];
        let n = tray.recv(&mut buf).unwrap();
        assert_eq!(&buf[..n], b"toggle");
        drop(tray);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn no_listener_is_an_error() {
        let path = temp_socket("missing");
        let _ = std::fs::remove_file(&path);
        assert!(send_datagram(&path, "toggle").is_err());
        // Stale socket file left behind by a dead tray.
        let tray = UnixDatagram::bind(&path).unwrap();
        drop(tray);
        assert!(send_datagram(&path, "toggle").is_err());
        let _ = std::fs::remove_file(&path);
    }
}
