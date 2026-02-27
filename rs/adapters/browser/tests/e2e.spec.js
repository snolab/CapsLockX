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
