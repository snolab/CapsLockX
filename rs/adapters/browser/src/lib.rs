//! CapsLockX – Browser / WASM adapter.
//!
//! Compiles to a `.wasm` + JS glue file via `wasm-pack`.  The WASM module
//! installs `keydown` / `keyup` capture-phase listeners on `window` and
//! drives the AccModel physics ticker with `setInterval(16)`.
//!
//! # Build
//! ```sh
//! # with wasm-pack (recommended – produces pkg/ ready for bundlers):
//! wasm-pack build --target web rs/adapters/browser
//!
//! # raw cargo (produces .wasm only, no JS glue):
//! cargo build -p capslockx-browser --target wasm32-unknown-unknown
//! ```
//!
//! # Usage
//! ```js
//! import init, { start } from './pkg/capslockx_browser.js';
//! await init();   // loads + initialises the WASM module
//! start();        // installs keyboard hook
//! ```
//!
//! # Notes
//! - Requires Rust ≥ 1.77 for `std::time::Instant` support on wasm32.
//! - Mouse cursor movement is a **no-op** in browser (cannot move system
//!   cursor from JS/WASM).  HJKL cursor keys and RF scroll work normally.
//! - Window management (Z/X/C/V) is a **no-op** – use Ctrl+W for close-tab.

#[cfg(target_arch = "wasm32")]
mod key_map;
#[cfg(target_arch = "wasm32")]
mod platform;
#[cfg(target_arch = "wasm32")]
mod glue;

#[cfg(target_arch = "wasm32")]
pub use glue::start;
