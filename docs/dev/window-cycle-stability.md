# Window Cycle Stability (macOS)

## Problem

`Space+Z` (cycle windows) and `Space+C` (arrange/cascade windows) used different
window orderings, causing the cycle position to jump unpredictably when the two
operations were interleaved (`z, c, z, c, z, c`).

## Root Causes (discovered incrementally)

### 1. Z-order based ordering is inherently unstable

**Symptom**: Cycle order changed every time a fresh snapshot was taken.

**Cause**: `list_all_windows()` returned windows in CGWindowList z-order (front-to-back).
Every `AXRaise`, manual click, or `arrange_windows` call changed the z-order, so the
next snapshot had a different ordering.

**Fix**: Sort the window list by `window_id` (CGWindowID). These IDs are assigned at
window creation time and never change, giving a deterministic order independent of
z-order.

### 2. Title-based window matching activated wrong windows

**Symptom**: Cycle picked the right window but a different Chrome window came to front.

**Cause**: `activate_window()` matched AX windows by title. Chrome has many windows
with identical or similar titles (e.g., two "Inbox" tabs). Title matching picked the
first match, which could be the wrong physical window.

**Fix**: Use `_AXUIElementGetWindow()` (private macOS API) to get the CGWindowID from
each AXUIElement, and match by ID instead of title. This is 100% reliable.

### 3. cycle_windows and arrange_windows used different orderings

**Symptom**: `z, c, z, c` jumped around — the cascade reordered windows differently
than the cycle expected.

**Cause**: `cycle_windows` sorted by `window_id`, but `arrange_windows` used
`get_all_ax_window_refs_stable()` which returned windows in z-order (or pid+title
order). The two orderings were completely different.

**Fix**: Make `get_all_ax_window_refs_stable()` also sort by `window_id`, so both
operations use the same stable ordering.

### 4. arrange_windows changed which window was frontmost

**Symptom**: After `Space+C`, the next `Space+Z` started from a different position
because cascade put a different window on top.

**Cause**: Cascade positions windows sequentially; the last one ends up on top. This
changed the frontmost window, and the next cycle's FRESH snapshot anchored to the
new frontmost instead of the previously cycled-to window.

**Fix**: After arrange, re-activate the window that was focused before the arrange
(read from the cycle state). This preserves the cycle position.

### 5. Windows on other Spaces appeared in cycle but couldn't be raised

**Symptom**: Some windows in the cycle list couldn't be brought to front — AXRaise
succeeded internally but CGWindowList still showed a different frontmost.

**Cause**: CGWindowList with `kCGWindowListOptionOnScreenOnly` includes windows from
all Spaces, not just the current one. AXRaise reorders windows within the app's
internal list but can't bring a window from another Space to the current Space.

**Fix**: Filter windows by `kCGWindowIsOnscreen` flag from CGWindowList. Additionally
use `onscreen_wids` set (all on-screen CGWindowIDs regardless of title) to include
windows like Terminal that have empty `kCGWindowName` in CGWindowList.

### 6. Cache expiry caused re-anchoring jumps

**Symptom**: After a pause or when `Space+C` was used between `Space+Z` presses,
the cycle jumped to a different position.

**Cause**: The 5-second cache timeout expired between operations. `arrange_windows`
didn't update the cache's `last_use` timestamp, so `z, c, z` with >5s total time
caused a FRESH snapshot with re-anchoring.

**Fix**: `arrange_windows` bumps `CYCLE.last_use` to keep the cache alive.

### 7. UTF-8 string slicing panic in event tap callback

**Symptom**: CapsLockX crashed with `panic_cannot_unwind` when cycling to a window
with a Japanese title.

**Cause**: Debug logging used byte slicing (`&t[..t.len().min(20)]`) which panicked
on multi-byte UTF-8 characters. This happened inside the CGEventTap callback which
is `extern "C"` and cannot unwind.

**Fix**: Use `t.chars().take(20).collect::<String>()` for safe Unicode truncation.

## Key APIs Used

- **`CGWindowListCopyWindowInfo`**: Enumerate all windows with metadata (pid, wid, layer, title, isOnscreen)
- **`_AXUIElementGetWindow`** (private): Get CGWindowID from AXUIElement — the reliable bridge between CG and AX worlds
- **`AXUIElementPerformAction("AXRaise")`**: Raise a window within its app
- **`NSRunningApplication activateWithOptions:3`**: Bring an app to front (AllWindows + IgnoringOtherApps)
- **`AXUIElementSetAttributeValue("AXMain"/"AXFocused", true)`**: Set main/focused window

## Architecture

```
list_all_windows()
  │
  ├── CGWindowList (z-order, with isOnscreen filter + onscreen_wids set)
  ├── NSWorkspace.runningApplications (off-screen GUI apps)
  └── AX windows per pid (_AXUIElementGetWindow for CGWindowID)
  │
  └── sorted by window_id → stable ordering
          │
          ├── cycle_windows(): advance index, activate_window()
          └── arrange_windows(): cascade/tile in same order, restore focus
```

## Testing

`test-cycle` binary (`cargo run -p capslockx-macos --release --bin test-cycle`)
directly calls the AX/CG APIs without key injection, activating each window and
verifying frontmost. Does not require CapsLockX to be running.
