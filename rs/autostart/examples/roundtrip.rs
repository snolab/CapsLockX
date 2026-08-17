//! Manual round-trip check for the launch-at-login logic.
//!
//! MUST be run from a directory containing clx.exe (i.e. copy it to the repo
//! root first) — `target_exe()` refuses to register a non-clx binary, so from
//! `target/release/examples/` every enable() correctly fails. Run elevated so
//! the schtasks fast path is exercised without a UAC prompt:
//!
//!     cargo build -p clx-autostart --example roundtrip --release
//!     cp target/release/examples/roundtrip.exe .          # next to clx.exe
//!     <elevated> ./roundtrip.exe
//!
//! Ends by restoring whatever state it found, so running it never changes the
//! machine's actual autostart setting.

fn main() {
    let started_enabled = clx_autostart::is_enabled();
    println!("target exe    : {:?}", clx_autostart::target_exe());
    println!("task name     : {}", clx_autostart::TASK_NAME);
    println!("initial state : {started_enabled}");

    let off = clx_autostart::set_enabled(false);
    println!("after disable : {off}  (expect false)");

    let on = clx_autostart::set_enabled(true);
    println!("after enable  : {on}  (expect true)");

    let restored = clx_autostart::set_enabled(started_enabled);
    println!("restored to   : {restored}  (expect {started_enabled})");

    let ok = !off && on && restored == started_enabled;
    println!("RESULT        : {}", if ok { "PASS" } else { "FAIL" });
}
