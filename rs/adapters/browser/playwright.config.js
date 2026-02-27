import { defineConfig, devices } from "@playwright/test";

export default defineConfig({
  testDir: "./tests",
  fullyParallel: false,
  retries: 1,
  reporter: "list",

  use: {
    baseURL: "http://localhost:4321",
    headless: true,
    // Capture traces on first retry for debugging.
    trace: "on-first-retry",
  },

  projects: [{ name: "chromium", use: { ...devices["Desktop Chrome"] } }],

  // Reuse the already-running `npx serve` if present; otherwise start one.
  webServer: {
    command: "npx serve .",
    url: "http://localhost:4321",
    reuseExistingServer: true,
    timeout: 30_000,
  },
});
