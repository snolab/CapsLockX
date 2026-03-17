//! MacPlatform – implements the `Platform` trait using Core Graphics.
//!
//! Uses `CGEventPost` to inject keyboard and mouse events.
//! Tags injected events with a user-data field so the hook can skip them.

use core_graphics::event::{
    CGEvent, CGEventTapLocation, CGEventType, CGMouseButton, EventField,
    ScrollEventUnit,
};
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
use core_graphics::geometry::CGPoint;

use capslockx_core::{KeyCode, Platform};
use capslockx_core::platform::MouseButton;

use crate::key_map::keycode_to_cg_keycode;

/// Tag value written to EVENT_SOURCE_USER_DATA on injected events so the
/// hook callback can recognise and pass them through.
pub const SELF_INJECT_TAG: i64 = 0x434C5831; // "CLX1"

// ── MacPlatform ──────────────────────────────────────────────────────────────

pub struct MacPlatform;

impl MacPlatform {
    pub fn new() -> Self {
        Self
    }

    /// Create an event source for synthetic events.
    fn source() -> CGEventSource {
        CGEventSource::new(CGEventSourceStateID::CombinedSessionState)
            .expect("failed to create CGEventSource")
    }

    /// Tag an event so our hook knows it's self-injected.
    fn tag(event: &CGEvent) {
        event.set_integer_value_field(EventField::EVENT_SOURCE_USER_DATA, SELF_INJECT_TAG);
    }
}

impl Platform for MacPlatform {
    // ── Keyboard output ───────────────────────────────────────────────────────

    fn key_down(&self, key: KeyCode) {
        if let Some(cg_key) = keycode_to_cg_keycode(key) {
            let source = Self::source();
            if let Ok(event) = CGEvent::new_keyboard_event(source, cg_key, true) {
                Self::tag(&event);
                event.post(CGEventTapLocation::HID);
            }
        }
    }

    fn key_up(&self, key: KeyCode) {
        if let Some(cg_key) = keycode_to_cg_keycode(key) {
            let source = Self::source();
            if let Ok(event) = CGEvent::new_keyboard_event(source, cg_key, false) {
                Self::tag(&event);
                event.post(CGEventTapLocation::HID);
            }
        }
    }

    // ── Mouse output ──────────────────────────────────────────────────────────

    fn mouse_move(&self, dx: i32, dy: i32) {
        let source = Self::source();
        // Get current mouse position.
        if let Ok(dummy) = CGEvent::new(source.clone()) {
            let loc = dummy.location();
            let new_loc = CGPoint::new(loc.x + dx as f64, loc.y + dy as f64);
            if let Ok(event) = CGEvent::new_mouse_event(
                source,
                CGEventType::MouseMoved,
                new_loc,
                CGMouseButton::Left,
            ) {
                Self::tag(&event);
                event.post(CGEventTapLocation::HID);
            }
        }
    }

    fn scroll_v(&self, delta: i32) {
        let source = Self::source();
        if let Ok(event) = CGEvent::new_scroll_event(
            source,
            ScrollEventUnit::LINE,
            1, // wheel_count
            delta,
            0,
            0,
        ) {
            Self::tag(&event);
            event.post(CGEventTapLocation::HID);
        }
    }

    fn scroll_h(&self, delta: i32) {
        let source = Self::source();
        if let Ok(event) = CGEvent::new_scroll_event(
            source,
            ScrollEventUnit::LINE,
            2, // wheel_count
            0,
            delta,
            0,
        ) {
            Self::tag(&event);
            event.post(CGEventTapLocation::HID);
        }
    }

    fn mouse_button(&self, button: MouseButton, pressed: bool) {
        let source = Self::source();
        // Get current position.
        let loc = if let Ok(dummy) = CGEvent::new(source.clone()) {
            dummy.location()
        } else {
            CGPoint::new(0.0, 0.0)
        };

        let (event_type, cg_button) = match (button, pressed) {
            (MouseButton::Left, true)    => (CGEventType::LeftMouseDown,  CGMouseButton::Left),
            (MouseButton::Left, false)   => (CGEventType::LeftMouseUp,    CGMouseButton::Left),
            (MouseButton::Right, true)   => (CGEventType::RightMouseDown, CGMouseButton::Right),
            (MouseButton::Right, false)  => (CGEventType::RightMouseUp,   CGMouseButton::Right),
            (MouseButton::Middle, true)  => (CGEventType::OtherMouseDown, CGMouseButton::Center),
            (MouseButton::Middle, false) => (CGEventType::OtherMouseUp,   CGMouseButton::Center),
        };

        if let Ok(event) = CGEvent::new_mouse_event(source, event_type, loc, cg_button) {
            Self::tag(&event);
            event.post(CGEventTapLocation::HID);
        }
    }

    // ── Window management: use Cmd shortcuts on macOS ──────────────────────────

    fn close_tab(&self) {
        // Cmd+W
        self.key_down(KeyCode::LWin);
        self.key_tap(KeyCode::W);
        self.key_up(KeyCode::LWin);
    }

    fn close_window(&self) {
        // Cmd+W
        self.key_down(KeyCode::LWin);
        self.key_tap(KeyCode::W);
        self.key_up(KeyCode::LWin);
    }

    fn kill_window(&self) {
        // Cmd+Q to quit the app
        self.key_down(KeyCode::LWin);
        self.key_tap(KeyCode::Q);
        self.key_up(KeyCode::LWin);
    }
}
