import { defineConfig, devices } from "@playwright/test";

// WebKit is the supported local macOS baseline for these focused UI tests.
// Chromium remains optional because bundled local launch behavior is still
// environment-sensitive on this machine.
const requestedBrowser = process.env.PLAYWRIGHT_BROWSER?.toLowerCase();
const browserTarget =
  requestedBrowser === "webkit" || requestedBrowser === "chromium" || requestedBrowser === "chrome"
    ? requestedBrowser
    : process.platform === "darwin"
      ? "webkit"
      : "chromium";
const browserName = browserTarget === "chrome" ? "chromium" : browserTarget;
const browserPreset =
  browserTarget === "webkit" ? devices["Desktop Safari"] : devices["Desktop Chrome"];
const launchOptions = browserTarget === "chrome" ? { channel: "chrome" as const } : {};

export default defineConfig({
  testDir: "./tests/playwright",
  fullyParallel: false,
  timeout: 60 * 1000,
  webServer: {
    command: "npm run dev -- --host 127.0.0.1 --port 4174",
    port: 4174,
    reuseExistingServer: !process.env.CI,
    timeout: 120 * 1000,
  },
  expect: {
    timeout: 15 * 1000,
  },
  use: {
    baseURL: "http://127.0.0.1:4174",
    trace: "retain-on-failure",
    screenshot: "only-on-failure",
    video: "off",
    serviceWorkers: "block",
  },
  reporter: [["list"]],
  projects: [
    {
      name: browserTarget,
      use: {
        browserName,
        ...browserPreset,
        ...launchOptions,
      },
    },
  ],
});
