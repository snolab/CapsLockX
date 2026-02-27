// @ts-check
import { test, expect } from "@playwright/test";

const PAGE = "/www/";

// ── Helpers ──────────────────────────────────────────────────────────────────

/** Navigate to PAGE and wait for the WASM module to initialise. */
async function loadPage(page) {
  await page.goto(PAGE);
  await expect(page.locator("#wasm-badge")).toHaveText("WASM ✓", { timeout: 10_000 });
  // Ensure the textarea is focused so all key events land there.
  await page.click("#editor");
}

/**
 * Inject a listener that records synthetic (isTrusted=false) keydown codes
 * into window.__synth[].  Must be called after WASM loads so the collector
 * fires AFTER the WASM dispatch (same tick, capture phase, same element).
 */
async function installSynthCollector(page) {
  await page.evaluate(() => {
    window.__synth = [];
    // Use capture=true so we see events before they might be stopped.
    window.addEventListener(
      "keydown",
      (e) => {
        if (!e.isTrusted) window.__synth.push(e.code);
      },
      true,
    );
  });
}

async function getSynth(page) {
  return page.evaluate(() => window.__synth ?? []);
}

/** Hold CapsLock+key for `ms` ms, then release both. */
async function holdClx(page, key, ms = 200) {
  await page.keyboard.down("CapsLock");
  await page.keyboard.down(key);
  await page.waitForTimeout(ms);
  await page.keyboard.up(key);
  await page.keyboard.up("CapsLock");
  await page.waitForTimeout(50); // drain in-flight ticks
}

/** Place textarea value + cursor, return initial selectionStart. */
async function setEditorCursor(page, text, pos) {
  await page.evaluate(
    ({ text, pos }) => {
      const ta = document.getElementById("editor");
      ta.value = text;
      ta.setSelectionRange(pos, pos);
      ta.focus();
    },
    { text, pos },
  );
  return pos;
}

// ── WASM bootstrap ───────────────────────────────────────────────────────────

test("WASM module loads – badge turns green", async ({ page }) => {
  await page.goto(PAGE);
  const badge = page.locator("#wasm-badge");
  await expect(badge).toHaveText("WASM ✓", { timeout: 10_000 });
  await expect(badge).toHaveClass(/ok/);
  // Error banner must remain hidden.
  await expect(page.locator("#err")).toBeHidden();
});

// ── Status indicator ─────────────────────────────────────────────────────────

test.describe("Status indicator", () => {
  test("shows CLX OFF on load", async ({ page }) => {
    await loadPage(page);
    await expect(page.locator("#status")).toHaveText("CLX OFF");
  });

  test("CapsLock hold → CLX HELD", async ({ page }) => {
    await loadPage(page);
    await page.keyboard.down("CapsLock");
    await expect(page.locator("#status")).toHaveText("CLX HELD");
    await page.keyboard.up("CapsLock");
  });

  test("CapsLock released → CLX OFF", async ({ page }) => {
    await loadPage(page);
    await page.keyboard.down("CapsLock");
    await page.keyboard.up("CapsLock");
    await expect(page.locator("#status")).toHaveText("CLX OFF");
  });

  test("Space hold → CLX HELD", async ({ page }) => {
    await loadPage(page);
    await page.keyboard.down("Space");
    await expect(page.locator("#status")).toHaveText("CLX HELD");
    await page.keyboard.up("Space");
  });

  test("CapsLock+Space chord → CLX LOCKED", async ({ page }) => {
    await loadPage(page);
    await page.keyboard.down("CapsLock");
    await page.keyboard.down("Space");
    await expect(page.locator("#status")).toHaveText("CLX LOCKED");
    await page.keyboard.up("Space");
    await page.keyboard.up("CapsLock");
  });

  test("locked mode: releasing Space while CapsLock held keeps CLX LOCKED", async ({ page }) => {
    await loadPage(page);
    // Chord: CapsLock + Space → locked = true.
    await page.keyboard.down("CapsLock");
    await page.keyboard.down("Space");
    await page.keyboard.up("Space");
    // CapsLock still held; Space released but capsHeld=true keeps locked alive.
    await expect(page.locator("#status")).toHaveText("CLX LOCKED");
    // Release CapsLock → both keys up → locked = false.
    await page.keyboard.up("CapsLock");
    await expect(page.locator("#status")).toHaveText("CLX OFF");
  });
});

// ── Key-log (bubble-phase passthrough) ───────────────────────────────────────

test.describe("Key log (bubble-phase)", () => {
  test("regular keypress appears in log", async ({ page }) => {
    await loadPage(page);
    await page.keyboard.press("a");
    // Bubble-phase listener records trusted events.
    await expect(page.locator("#log-box")).toContainText("KeyA");
  });

  test("CapsLock keydown is suppressed – absent from log", async ({ page }) => {
    await loadPage(page);
    await page.keyboard.down("CapsLock");
    await page.waitForTimeout(80);
    await page.keyboard.up("CapsLock");
    // The log only shows events that survived capture phase; CapsLock is eaten.
    const logText = await page.locator("#log-box").innerText();
    expect(logText).not.toContain("CapsLock");
  });

  test("CLX+H keydown is suppressed – H absent from log", async ({ page }) => {
    await loadPage(page);
    await page.keyboard.down("CapsLock");
    await page.keyboard.down("KeyH");
    await page.waitForTimeout(80);
    await page.keyboard.up("KeyH");
    await page.keyboard.up("CapsLock");
    const logText = await page.locator("#log-box").innerText();
    // H should NOT appear (it was intercepted in capture phase).
    expect(logText).not.toContain("KeyH");
  });
});

// ── Synthetic output events ───────────────────────────────────────────────────
//
// When CLX is active, the AccModel drives synthetic KeyboardEvents
// (isTrusted=false) on the focused element.  We capture them with our own
// listener because the page log ignores synthetic events.

test.describe("Synthetic cursor events", () => {
  async function expectSynth(page, triggerKey, expectedCode) {
    await loadPage(page);
    await installSynthCollector(page);

    await page.keyboard.down("CapsLock");
    await page.keyboard.down(triggerKey);
    // Wait for at least two ticker cycles (~32 ms each) plus some slack.
    await page.waitForTimeout(150);
    await page.keyboard.up(triggerKey);
    await page.keyboard.up("CapsLock");
    // Drain any in-flight ticks.
    await page.waitForTimeout(50);

    const synth = await getSynth(page);
    expect(synth).toContain(expectedCode);
  }

  test("CLX+H dispatches ArrowLeft", ({ page }) => expectSynth(page, "KeyH", "ArrowLeft"));
  test("CLX+L dispatches ArrowRight", ({ page }) => expectSynth(page, "KeyL", "ArrowRight"));
  test("CLX+K dispatches ArrowUp", ({ page }) => expectSynth(page, "KeyK", "ArrowUp"));
  test("CLX+J dispatches ArrowDown", ({ page }) => expectSynth(page, "KeyJ", "ArrowDown"));
  test("CLX+Y dispatches Home", ({ page }) => expectSynth(page, "KeyY", "Home"));
  test("CLX+O dispatches End", ({ page }) => expectSynth(page, "KeyO", "End"));
  test("CLX+G dispatches Enter", ({ page }) => expectSynth(page, "KeyG", "Enter"));
  test("CLX+T dispatches Delete", ({ page }) => expectSynth(page, "KeyT", "Delete"));
});

// ── AccModel acceleration ────────────────────────────────────────────────────

test("sustained CLX+H produces multiple ArrowLeft events (acceleration)", async ({ page }) => {
  await loadPage(page);
  await installSynthCollector(page);

  await page.keyboard.down("CapsLock");
  await page.keyboard.down("KeyH");
  // Hold for ~400 ms – the AccModel should fire many ticks with growing
  // velocity, producing more than 3 ArrowLeft events.
  await page.waitForTimeout(400);
  await page.keyboard.up("KeyH");
  await page.keyboard.up("CapsLock");
  await page.waitForTimeout(50); // drain trailing ticks

  const synth = await getSynth(page);
  const leftEvents = synth.filter((c) => c === "ArrowLeft");
  expect(leftEvents.length).toBeGreaterThan(3);
});

// ── clear-log button ─────────────────────────────────────────────────────────

test("clear button empties the key log", async ({ page }) => {
  await loadPage(page);
  // Generate some log entries.
  await page.keyboard.press("a");
  await page.keyboard.press("b");
  await expect(page.locator("#log-box")).not.toBeEmpty();

  await page.click("#clear-log");
  await expect(page.locator("#log-box")).toBeEmpty();
});

// ── HJKL actually moves the textarea cursor ───────────────────────────────────
//
// These tests set a known selectionStart, hold CLX+key, then assert the
// cursor moved in the expected direction.  Synthetic ArrowKey events dispatched
// via dispatchEvent() do move the cursor in Chromium textareas.

test.describe("HJKL moves text cursor (selectionStart)", () => {
  test("CLX+H moves cursor left", async ({ page }) => {
    await loadPage(page);
    await setEditorCursor(page, "abcde fghij", 5);

    await holdClx(page, "KeyH", 200);

    const pos = await page.evaluate(() => document.getElementById("editor").selectionStart);
    expect(pos).toBeLessThan(5);
  });

  test("CLX+L moves cursor right", async ({ page }) => {
    await loadPage(page);
    await setEditorCursor(page, "abcde fghij", 5);

    await holdClx(page, "KeyL", 200);

    const pos = await page.evaluate(() => document.getElementById("editor").selectionStart);
    expect(pos).toBeGreaterThan(5);
  });

  test("CLX+K moves cursor up (multi-line)", async ({ page }) => {
    await loadPage(page);
    // Place cursor on line 2, column 3
    await setEditorCursor(page, "line1\nline2\nline3", 9); // 'i' in line2

    await holdClx(page, "KeyK", 200);

    const pos = await page.evaluate(() => document.getElementById("editor").selectionStart);
    // Cursor should now be on line 1 (position < 6)
    expect(pos).toBeLessThan(6);
  });

  test("CLX+J moves cursor down (multi-line)", async ({ page }) => {
    await loadPage(page);
    await setEditorCursor(page, "line1\nline2\nline3", 3); // 'e' in line1

    await holdClx(page, "KeyJ", 200);

    const pos = await page.evaluate(() => document.getElementById("editor").selectionStart);
    // Cursor should now be on line 2 or 3 (position > 5)
    expect(pos).toBeGreaterThan(5);
  });

  test("CLX+Y jumps to line start (Home)", async ({ page }) => {
    await loadPage(page);
    await setEditorCursor(page, "hello world", 7);

    // Y uses AccModel (page model), so we need to hold long enough for ticks.
    await holdClx(page, "KeyY", 200);

    const pos = await page.evaluate(() => document.getElementById("editor").selectionStart);
    expect(pos).toBe(0);
  });

  test("CLX+O jumps to line end (End)", async ({ page }) => {
    await loadPage(page);
    await setEditorCursor(page, "hello world", 0);

    await holdClx(page, "KeyO", 200);

    const pos = await page.evaluate(() => document.getElementById("editor").selectionStart);
    expect(pos).toBe(11); // end of "hello world"
  });
});

// ── WASD virtual cursor ───────────────────────────────────────────────────────
//
// After holding CLX+D the WASM mouse_move fires clx:mouse_move events which
// move the virtual cursor to the right.  The cursor overlay becomes visible
// and window.__clxGetCursorPos() reflects the updated position.

test.describe("WASD virtual cursor", () => {
  test("CLX+D moves virtual cursor right", async ({ page }) => {
    await loadPage(page);

    // Initialise known position via real mouse move (sets vx/vy in JS).
    await page.mouse.move(400, 300);
    await page.waitForTimeout(30);

    const before = await page.evaluate(() => window.__clxGetCursorPos());

    await holdClx(page, "KeyD", 300);

    const after = await page.evaluate(() => window.__clxGetCursorPos());

    expect(after.x).toBeGreaterThan(before.x);
    expect(after.visible).toBe(true);
  });

  test("CLX+A moves virtual cursor left", async ({ page }) => {
    await loadPage(page);
    await page.mouse.move(600, 300);
    await page.waitForTimeout(30);

    const before = await page.evaluate(() => window.__clxGetCursorPos());

    await holdClx(page, "KeyA", 300);

    const after = await page.evaluate(() => window.__clxGetCursorPos());
    expect(after.x).toBeLessThan(before.x);
  });

  test("CLX+S moves virtual cursor down", async ({ page }) => {
    await loadPage(page);
    await page.mouse.move(500, 200);
    await page.waitForTimeout(30);

    const before = await page.evaluate(() => window.__clxGetCursorPos());

    await holdClx(page, "KeyS", 300);

    const after = await page.evaluate(() => window.__clxGetCursorPos());
    expect(after.y).toBeGreaterThan(before.y);
  });

  test("real mousemove snaps cursor back and hides overlay", async ({ page }) => {
    await loadPage(page);
    // Activate virtual cursor
    await holdClx(page, "KeyD", 200);
    expect(await page.evaluate(() => window.__clxGetCursorPos().visible)).toBe(true);

    // Move real mouse – should snap and hide overlay
    await page.mouse.move(300, 300);
    await page.waitForTimeout(30);

    const state = await page.evaluate(() => window.__clxGetCursorPos());
    expect(state.visible).toBe(false);
    expect(state.x).toBeCloseTo(300, 0);
    expect(state.y).toBeCloseTo(300, 0);
  });
});

// ── Scroll at virtual cursor (R / F) ─────────────────────────────────────────
//
// Position the virtual cursor over the scroll-box, then press CLX+F.
// The scroll-box's scrollTop should increase (not window.scrollY).

test.describe("R/F scroll at virtual cursor", () => {
  test("CLX+F scrolls scroll-box when cursor is over it", async ({ page }) => {
    await loadPage(page);

    // Scroll the box into the viewport first — the default 720 px viewport
    // is too short for the grid layout so the box starts below the fold.
    const scrollBox = page.locator("#scroll-box");
    await scrollBox.scrollIntoViewIfNeeded();

    // Move real mouse to the centre of the scroll-box to set vx/vy there.
    const bb = await scrollBox.boundingBox();
    await page.mouse.move(bb.x + bb.width / 2, bb.y + bb.height / 2);
    await page.waitForTimeout(30);

    const scrollBefore = await page.evaluate(() => document.getElementById("scroll-box").scrollTop);

    await holdClx(page, "KeyF", 400);

    const scrollAfter = await page.evaluate(() => document.getElementById("scroll-box").scrollTop);
    expect(scrollAfter).toBeGreaterThan(scrollBefore);
  });

  test("CLX+R scrolls scroll-box up when cursor is over it", async ({ page }) => {
    await loadPage(page);

    // Scroll the box down first so there is room to scroll up.
    await page.evaluate(() => {
      document.getElementById("scroll-box").scrollTop = 200;
    });

    const scrollBox = page.locator("#scroll-box");
    await scrollBox.scrollIntoViewIfNeeded();

    const bb = await scrollBox.boundingBox();
    await page.mouse.move(bb.x + bb.width / 2, bb.y + bb.height / 2);
    await page.waitForTimeout(30);

    const scrollBefore = await page.evaluate(() => document.getElementById("scroll-box").scrollTop);

    await holdClx(page, "KeyR", 400);

    const scrollAfter = await page.evaluate(() => document.getElementById("scroll-box").scrollTop);
    expect(scrollAfter).toBeLessThan(scrollBefore);
  });

  test("CLX+F scrolls window when cursor is NOT over a scrollable element", async ({ page }) => {
    await loadPage(page);

    // Put a lot of content so the page is scrollable
    await page.evaluate(() => {
      document.body.style.paddingBottom = "2000px";
    });

    // Move cursor to the header (not over a scrollable box)
    await page.mouse.move(200, 10);
    await page.waitForTimeout(30);

    const winScrollBefore = await page.evaluate(() => window.scrollY);

    await holdClx(page, "KeyF", 400);

    const winScrollAfter = await page.evaluate(() => window.scrollY);
    expect(winScrollAfter).toBeGreaterThan(winScrollBefore);
  });
});

// ── N / P focus cycling ───────────────────────────────────────────────────────
//
// CLX+N dispatches clx:focus(1) which cycles querySelectorAll tabbable
// elements forward; CLX+P goes backward.

test.describe("N/P focus cycling", () => {
  test("CLX+N moves focus from editor to next tabbable element", async ({ page }) => {
    await loadPage(page);

    const startId = await page.evaluate(() => document.activeElement?.id);
    expect(startId).toBe("editor");

    // A single N press: AccModel needs ~2 ticks (≥32 ms) to fire.
    await holdClx(page, "KeyN", 100);

    const newId = await page.evaluate(() => document.activeElement?.id);
    expect(newId).not.toBe("editor");
    expect(newId).toBeTruthy();
  });

  test("CLX+P moves focus backward from editor", async ({ page }) => {
    await loadPage(page);

    await holdClx(page, "KeyP", 100);

    // Focus should have moved to the last tabbable element (wraps around)
    const newId = await page.evaluate(() => document.activeElement?.id);
    expect(newId).not.toBe("editor");
    expect(newId).toBeTruthy();
  });

  test("CLX+N cycles through focus-field inputs", async ({ page }) => {
    await loadPage(page);

    // Advance focus until we reach focus-field-1
    const ids = [];
    for (let i = 0; i < 10; i++) {
      await holdClx(page, "KeyN", 80);
      const id = await page.evaluate(() => document.activeElement?.id ?? "");
      ids.push(id);
      if (id === "focus-field-1") break;
    }
    expect(ids).toContain("focus-field-1");
  });

  test("CLX+N then CLX+P returns focus to the same element", async ({ page }) => {
    await loadPage(page);

    // Move forward once
    await holdClx(page, "KeyN", 100);
    const afterN = await page.evaluate(() => document.activeElement?.id);

    // Move backward once (back to editor)
    await holdClx(page, "KeyP", 100);
    const afterP = await page.evaluate(() => document.activeElement?.id);

    expect(afterP).toBe("editor");
    expect(afterN).not.toBe("editor");
  });
});
