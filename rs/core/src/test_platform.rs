//! Mock Platform for unit tests. Records every call into a shared log.

use crate::key_code::KeyCode;
use crate::platform::{ArrangeMode, MouseButton, Platform, PttTrayState};
use std::sync::Mutex;

#[derive(Debug, Clone, PartialEq)]
pub enum Call {
    KeyDown(KeyCode),
    KeyUp(KeyCode),
    KeyTapExtended(KeyCode),
    MouseMove(i32, i32),
    ScrollV(i32),
    ScrollH(i32),
    MouseButton(MouseButton, bool),
    CycleWindows(i32),
    ArrangeWindows(ArrangeMode),
    CloseTab,
    CloseWindow,
    KillWindow,
    SetWindowTransparent(u8),
    RestoreWindow,
    ToggleWindowTopmost,
    SwitchToDesktop(u32),
    MoveWindowToDesktop(u32),
    TypeText(String),
    OpenPreferences,
    ShowVoiceOverlay,
    HideVoiceOverlay,
    UpdateVoiceSubtitle(String),
    SetPttTrayState(PttTrayState),
    GetSelectedText,
    GetClipboardText,
    SetClipboardText(String),
    ShowBrainstormOverlay(String),
    HideBrainstormOverlay,
    ShowPromptInput(String, String, String),
    Restart,
}

pub struct MockPlatform {
    pub calls: Mutex<Vec<Call>>,
    pub clipboard: Mutex<String>,
    pub selected: Mutex<String>,
    pub prompt_response: Mutex<Option<String>>,
}

impl MockPlatform {
    pub fn new() -> Self {
        Self {
            calls: Mutex::new(Vec::new()),
            clipboard: Mutex::new(String::new()),
            selected: Mutex::new(String::new()),
            prompt_response: Mutex::new(None),
        }
    }

    fn log(&self, c: Call) {
        self.calls.lock().unwrap().push(c);
    }

    pub fn calls(&self) -> Vec<Call> {
        self.calls.lock().unwrap().clone()
    }

    pub fn clear(&self) {
        self.calls.lock().unwrap().clear();
    }

    pub fn count(&self, pred: impl Fn(&Call) -> bool) -> usize {
        self.calls
            .lock()
            .unwrap()
            .iter()
            .filter(|c| pred(*c))
            .count()
    }

    /// Poll until `calls()` returns the expected vec or 500ms elapses.
    /// Use this after invoking a module that dispatches its side effects
    /// onto a background thread (e.g. window_manager close/arrange).
    pub fn wait_calls(&self, expected: &[Call]) -> Vec<Call> {
        let deadline = std::time::Instant::now() + std::time::Duration::from_millis(500);
        loop {
            let actual = self.calls();
            if actual == expected || std::time::Instant::now() >= deadline {
                return actual;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
    }
}

impl Default for MockPlatform {
    fn default() -> Self {
        Self::new()
    }
}

impl Platform for MockPlatform {
    fn key_down(&self, k: KeyCode) {
        self.log(Call::KeyDown(k));
    }
    fn key_up(&self, k: KeyCode) {
        self.log(Call::KeyUp(k));
    }
    fn key_tap_extended(&self, k: KeyCode) {
        self.log(Call::KeyTapExtended(k));
    }
    fn mouse_move(&self, dx: i32, dy: i32) {
        self.log(Call::MouseMove(dx, dy));
    }
    fn scroll_v(&self, d: i32) {
        self.log(Call::ScrollV(d));
    }
    fn scroll_h(&self, d: i32) {
        self.log(Call::ScrollH(d));
    }
    fn mouse_button(&self, b: MouseButton, p: bool) {
        self.log(Call::MouseButton(b, p));
    }
    fn cycle_windows(&self, dir: i32) {
        self.log(Call::CycleWindows(dir));
    }
    fn arrange_windows(&self, m: ArrangeMode) {
        self.log(Call::ArrangeWindows(m));
    }
    fn close_tab(&self) {
        self.log(Call::CloseTab);
    }
    fn close_window(&self) {
        self.log(Call::CloseWindow);
    }
    fn kill_window(&self) {
        self.log(Call::KillWindow);
    }
    fn set_window_transparent(&self, a: u8) {
        self.log(Call::SetWindowTransparent(a));
    }
    fn restore_window(&self) {
        self.log(Call::RestoreWindow);
    }
    fn toggle_window_topmost(&self) {
        self.log(Call::ToggleWindowTopmost);
    }
    fn switch_to_desktop(&self, i: u32) {
        self.log(Call::SwitchToDesktop(i));
    }
    fn move_window_to_desktop(&self, i: u32) {
        self.log(Call::MoveWindowToDesktop(i));
    }
    fn type_text(&self, t: &str) {
        self.log(Call::TypeText(t.into()));
    }
    fn open_preferences(&self) {
        self.log(Call::OpenPreferences);
    }
    fn show_voice_overlay(&self) {
        self.log(Call::ShowVoiceOverlay);
    }
    fn hide_voice_overlay(&self) {
        self.log(Call::HideVoiceOverlay);
    }
    fn update_voice_subtitle(&self, t: &str) {
        self.log(Call::UpdateVoiceSubtitle(t.into()));
    }
    fn set_ptt_tray_state(&self, s: PttTrayState) {
        self.log(Call::SetPttTrayState(s));
    }
    fn get_selected_text(&self) -> String {
        self.log(Call::GetSelectedText);
        self.selected.lock().unwrap().clone()
    }
    fn get_clipboard_text(&self) -> String {
        self.log(Call::GetClipboardText);
        self.clipboard.lock().unwrap().clone()
    }
    fn set_clipboard_text(&self, t: &str) {
        self.log(Call::SetClipboardText(t.into()));
        *self.clipboard.lock().unwrap() = t.into();
    }
    fn show_brainstorm_overlay(&self, t: &str) {
        self.log(Call::ShowBrainstormOverlay(t.into()));
    }
    fn hide_brainstorm_overlay(&self) {
        self.log(Call::HideBrainstormOverlay);
    }
    fn show_prompt_input(&self, title: &str, msg: &str, pre: &str) -> Option<String> {
        self.log(Call::ShowPromptInput(title.into(), msg.into(), pre.into()));
        self.prompt_response.lock().unwrap().clone()
    }
    fn restart(&self) {
        self.log(Call::Restart);
    }
}
