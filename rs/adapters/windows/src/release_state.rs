//! Match independent raw releases to presses actually seen by the hook.
//! Pure state logic: no polling, Windows calls, timers, or input injection.

pub const KEY_SLOTS: usize = 768;

/// Prefer the physical scan identity so a layout change during a hold cannot
/// turn a release into another logical key. Scan-less input uses its VK.
pub fn key_slot(scan: u32, extended: bool, vk: u32) -> Option<usize> {
    if vk >= 255 || scan > 255 {
        return None;
    }
    Some(if scan == 0 {
        512 + vk as usize
    } else {
        scan as usize + if extended { 256 } else { 0 }
    })
}

#[derive(Clone, Copy)]
struct Press {
    vk: u32,
    time: u32,
}

pub struct ReleaseState {
    pressed: [Option<Press>; KEY_SLOTS],
}

impl ReleaseState {
    pub const fn new() -> Self {
        Self {
            pressed: [None; KEY_SLOTS],
        }
    }

    pub fn hook_event(&mut self, slot: usize, vk: u32, down: bool, time: u32) {
        self.pressed[slot] = down.then_some(Press { vk, time });
    }

    /// Both timestamps are Win32 message times (wrapping milliseconds).
    /// Accept same-ms taps; an older queued release must not stop a newer hold.
    pub fn take_release(&mut self, slot: usize, time: u32) -> Option<u32> {
        let press = self.pressed[slot]?;
        if (time.wrapping_sub(press.time) as i32) < 0 {
            return None;
        }
        self.pressed[slot] = None;
        Some(press.vk)
    }

    pub fn clear(&mut self) {
        self.pressed.fill(None);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_up_is_recovered_without_ever_observing_raw_make() {
        let mut s = ReleaseState::new();
        s.hook_event(44, 0x5a, true, 100);
        assert_eq!(s.take_release(44, 101), Some(0x5a));
        assert_eq!(s.take_release(44, 101), None);
    }

    #[test]
    fn normal_up_needs_no_reconciliation() {
        let mut s = ReleaseState::new();
        s.hook_event(44, 0x5a, true, 100);
        s.hook_event(44, 0x5a, false, 110);
        assert_eq!(s.take_release(44, 110), None);
    }

    #[test]
    fn sub_tick_and_same_millisecond_taps_are_not_calibration_dependent() {
        for delay in 0..6 {
            let mut s = ReleaseState::new();
            s.hook_event(44, 0x5a, true, 100);
            assert_eq!(s.take_release(44, 100 + delay), Some(0x5a));
        }
    }

    #[test]
    fn stale_release_does_not_cancel_new_press_or_repeat() {
        let mut s = ReleaseState::new();
        s.hook_event(44, 0x5a, true, 120);
        assert_eq!(s.take_release(44, 110), None);
        assert_eq!(s.take_release(44, 121), Some(0x5a));
    }

    #[test]
    fn timestamps_survive_win32_wraparound() {
        let mut s = ReleaseState::new();
        s.hook_event(44, 0x5a, true, u32::MAX - 2);
        assert_eq!(s.take_release(44, 3), Some(0x5a));
        s.hook_event(44, 0x5a, true, 3);
        assert_eq!(s.take_release(44, u32::MAX - 2), None);
    }

    #[test]
    fn releasing_z_leaves_w_held_and_returns_original_layout_vk() {
        let mut s = ReleaseState::new();
        s.hook_event(44, 0x5a, true, 100);
        s.hook_event(17, 0x57, true, 100);
        assert_eq!(s.take_release(44, 110), Some(0x5a));
        assert_eq!(s.take_release(17, 120), Some(0x57));
        // Physical position 21 can be Z in a non-US layout. No conversion
        // from the raw event's (possibly changed) layout is needed on release.
        s.hook_event(21, 0x5a, true, 130);
        assert_eq!(s.take_release(21, 140), Some(0x5a));
    }

    #[test]
    fn physical_identity_distinguishes_extended_and_scanless_keys() {
        assert_ne!(key_slot(29, false, 0xa2), key_slot(29, true, 0xa3));
        assert_ne!(key_slot(0, false, 0x5a), key_slot(0, false, 0x57));
        assert_eq!(key_slot(21, false, 0x59), key_slot(21, false, 0x5a));
        assert_eq!(key_slot(255, false, 255), None);
    }
}
