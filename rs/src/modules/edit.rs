/// CLX-Edit – cursor & page navigation via HJKL / YUIO / GT / PN
///
/// Mirrors AHK module `CLX-Edit.ahk` with `AccModel2D` acceleration.
use crate::acc_model::AccModel2D;
use crate::input;
use crate::vk::*;
use once_cell::sync::Lazy;
use std::sync::Arc;

/// Eagerly initialise all background ticker threads for this module.
pub fn init() {
    Lazy::force(&CURSOR);
    Lazy::force(&PAGE);
    Lazy::force(&TAB);
}

// ──────────────────────────────── AccModel instances ─────────────────────────

/// HJKL – directional cursor movement
static CURSOR: Lazy<AccModel2D> = Lazy::new(|| {
    AccModel2D::new(
        Arc::new(cursor_action),
        15.0, // h_accel_ratio  (SpeedRatioX * 15)
        15.0, // v_accel_ratio  (SpeedRatioY * 15)
        250.0, // max_speed
    )
});

/// YUIO – page / home / end navigation
static PAGE: Lazy<AccModel2D> = Lazy::new(|| {
    AccModel2D::new(Arc::new(page_action), 20.0, 20.0, 250.0)
});

/// PN – Tab / Shift+Tab
static TAB: Lazy<AccModel2D> = Lazy::new(|| {
    AccModel2D::new(Arc::new(tab_action), 15.0, 15.0, 250.0)
});

// ──────────────────────────────── action callbacks ───────────────────────────

fn cursor_action(dx: i32, dy: i32, phase: &str) {
    if !crate::state::is_clx_active() {
        CURSOR.stop();
        return;
    }
    match phase {
        "横中键" => {
            // Select word: move direction, then select back
            if dx > 0 {
                input::tap_key(VK_RIGHT); // TODO: Ctrl+Right Ctrl+Shift+Left
            } else {
                input::tap_key(VK_LEFT);
            }
            CURSOR.stop();
        }
        "纵中键" => {
            // Select line
            if dy > 0 {
                input::tap_key(VK_DOWN);
                input::tap_key(VK_HOME);
            } else {
                input::tap_key(VK_UP);
                input::tap_key(VK_HOME);
            }
            CURSOR.stop();
        }
        "移动" => {
            if dy < 0 { input::tap_key_n(VK_UP, -dy); }
            if dy > 0 { input::tap_key_n(VK_DOWN, dy); }
            if dx < 0 { input::tap_key_n(VK_LEFT, -dx); }
            if dx > 0 { input::tap_key_n(VK_RIGHT, dx); }
        }
        _ => {}
    }
}

fn page_action(dx: i32, dy: i32, phase: &str) {
    if !crate::state::is_clx_active() {
        PAGE.stop();
        return;
    }
    if phase != "移动" { return; }
    // dy<0 = up = PageUp, dy>0 = down = PageDown
    // dx<0 = left = Home, dx>0 = right = End
    if dy < 0 { input::tap_key_n(VK_PRIOR, -dy); }
    if dy > 0 { input::tap_key_n(VK_NEXT, dy); }
    if dx < 0 { input::tap_key_n(VK_HOME, -dx); }
    if dx > 0 { input::tap_key_n(VK_END, dx); }
}

fn tab_action(dx: i32, dy: i32, phase: &str) {
    if !crate::state::is_clx_active() {
        TAB.stop();
        return;
    }
    if phase != "移动" { return; }
    // dy<0 = Shift+Tab, dy>0 = Tab
    let _ = dx;
    if dy < 0 { input::tap_key_shifted_n(VK_TAB, -dy); }
    if dy > 0 { input::tap_key_n(VK_TAB, dy); }
}

// ──────────────────────────────── key dispatch ────────────────────────────────

pub fn on_key_down(vk: u32) -> bool {
    match vk {
        // ── HJKL: cursor ──────────────────────────────────────────────────
        VK_H => { CURSOR.press_left();  true }
        VK_L => { CURSOR.press_right(); true }
        VK_K => { CURSOR.press_up();    true }
        VK_J => { CURSOR.press_down();  true }
        // ── YUIO: page/home/end ───────────────────────────────────────────
        VK_U => { PAGE.press_up();    true }  // PageUp
        VK_I => { PAGE.press_down();  true }  // PageDown
        VK_Y => { PAGE.press_left();  true }  // Home
        VK_O => { PAGE.press_right(); true }  // End
        // ── GT: Enter / Delete ────────────────────────────────────────────
        VK_G => { input::tap_key(VK_RETURN); true }
        VK_T => { input::tap_key(VK_DELETE); true }
        // ── PN: Tab / Shift+Tab ───────────────────────────────────────────
        VK_P => { TAB.press_up();   true }  // Shift+Tab
        VK_N => { TAB.press_down(); true }  // Tab
        _ => false,
    }
}

pub fn on_key_up(vk: u32) -> bool {
    match vk {
        VK_H => { CURSOR.release_left();  true }
        VK_L => { CURSOR.release_right(); true }
        VK_K => { CURSOR.release_up();    true }
        VK_J => { CURSOR.release_down();  true }
        VK_U => { PAGE.release_up();    true }
        VK_I => { PAGE.release_down();  true }
        VK_Y => { PAGE.release_left();  true }
        VK_O => { PAGE.release_right(); true }
        VK_P => { TAB.release_up();   true }
        VK_N => { TAB.release_down(); true }
        VK_G | VK_T => true, // instantaneous, no release needed
        _ => false,
    }
}
