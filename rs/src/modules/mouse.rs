/// CLX-Mouse – virtual mouse via WASD + QE buttons + RF scroll
///
/// Mirrors AHK module `CLX-Mouse.ahk`.
use crate::acc_model::AccModel2D;
use crate::input;
use crate::vk::*;
use once_cell::sync::Lazy;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// Eagerly initialise all background ticker threads for this module.
pub fn init() {
    Lazy::force(&MOUSE);
    Lazy::force(&SCROLL);
}

// DPI scaling: 96 DPI baseline, multiply by actual DPI ratio
// For now we use a fixed ratio; a real implementation would call GetDpiForSystem()
const MOUSE_SPEED: f64 = 240.0; // pixels/s at baseline (120 * 2, roughly matching AHK defaults)
const SCROLL_SPEED: f64 = 480.0; // scroll wheel delta/s

static LEFT_BTN_DOWN: AtomicBool = AtomicBool::new(false);
static RIGHT_BTN_DOWN: AtomicBool = AtomicBool::new(false);

// ──────────────────────────────── AccModel instances ─────────────────────────

static MOUSE: Lazy<AccModel2D> = Lazy::new(|| {
    AccModel2D::new(Arc::new(mouse_action), MOUSE_SPEED, MOUSE_SPEED, f64::INFINITY)
});

static SCROLL: Lazy<AccModel2D> = Lazy::new(|| {
    AccModel2D::new(Arc::new(scroll_action), SCROLL_SPEED, SCROLL_SPEED, f64::INFINITY)
});

// ──────────────────────────────── action callbacks ───────────────────────────

fn mouse_action(dx: i32, dy: i32, phase: &str) {
    if !crate::state::is_clx_active() {
        MOUSE.stop();
        return;
    }
    match phase {
        "移动" => {
            if dx != 0 || dy != 0 {
                input::send_mouse_move(dx, dy);
            }
        }
        "横中键" | "纵中键" => {
            MOUSE.stop();
        }
        _ => {}
    }
}

/// scroll: up = positive dy (wheel up), down = negative dy
fn scroll_action(dx: i32, dy: i32, phase: &str) {
    if !crate::state::is_clx_active() {
        SCROLL.stop();
        return;
    }
    if phase != "移动" { return; }
    // Vertical scroll: R = up, F = down
    // dy is negative when R (up) is held because AccModel: press_up sets up_tick
    // and up means negative dy in the model convention (-y is up on screen)
    // We want R to scroll the page UP (wheel delta positive)
    if dy < 0 { input::send_scroll_v(-dy * 3); }  // R pressed → scroll up
    if dy > 0 { input::send_scroll_v(-dy * 3); }  // F pressed → scroll down (negative)
    if dx != 0 { input::send_scroll_h(dx * 3); }
}

// ──────────────────────────────── key dispatch ────────────────────────────────

pub fn on_key_down(vk: u32) -> bool {
    match vk {
        // ── WASD: mouse movement ──────────────────────────────────────────
        VK_W => { MOUSE.press_up();    true }
        VK_S => { MOUSE.press_down();  true }
        VK_A => { MOUSE.press_left();  true }
        VK_D => { MOUSE.press_right(); true }
        // ── QE: right / left mouse buttons ───────────────────────────────
        VK_Q => {
            if !RIGHT_BTN_DOWN.swap(true, Ordering::Relaxed) {
                input::mouse_right_down();
            }
            true
        }
        VK_E => {
            if !LEFT_BTN_DOWN.swap(true, Ordering::Relaxed) {
                input::mouse_left_down();
            }
            true
        }
        // ── RF: vertical scroll (R = up, F = down) ────────────────────────
        VK_R => { SCROLL.press_up();   true }
        VK_F => { SCROLL.press_down(); true }
        _ => false,
    }
}

pub fn on_key_up(vk: u32) -> bool {
    match vk {
        VK_W => { MOUSE.release_up();    true }
        VK_S => { MOUSE.release_down();  true }
        VK_A => { MOUSE.release_left();  true }
        VK_D => { MOUSE.release_right(); true }
        VK_Q => {
            if RIGHT_BTN_DOWN.swap(false, Ordering::Relaxed) {
                input::mouse_right_up();
            }
            true
        }
        VK_E => {
            if LEFT_BTN_DOWN.swap(false, Ordering::Relaxed) {
                input::mouse_left_up();
            }
            true
        }
        VK_R => { SCROLL.release_up();   true }
        VK_F => { SCROLL.release_down(); true }
        _ => false,
    }
}
