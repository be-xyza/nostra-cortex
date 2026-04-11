import { defineConfig, devices } from "@playwright/test";

const requestedBrowser = process.env.PLAYWRIGHT_BROWSER?.toLowerCase();

const projectUse =
  requestedBrowser === "webkit"
    ? { ...devices["Desktop Safari"], browserName: "webkit" as const }
    : requestedBrowser === "chrome"
      ? { ...devices["Desktop Chrome"], browserName: "chromium" as const, channel: "chrome" }
      : { ...devices["Desktop Chrome"], browserName: "chromium" as const };

export default defineConfig({
  testDir: "./tests/playwright",
  fullyParallel: false,
  timeout: 60 * 1000,
  expect: {
    timeout: 15 * 1000,
  },
  use: {
    baseURL: 'http://127.0.0.1:4174',
    trace: "retain-on-failure",
    screenshot: "only-on-failure",
    video: "off",
    serviceWorkers: 'block',
  },
  reporter: [["list"]],
  webServer: {
    command: "npm run dev -- --host 127.0.0.1 --port 4174",
    url: "http://127.0.0.1:4174",
    reuseExistingServer: true,
    timeout: 120 * 1000,
  },
  projects: [
    {
      name: requestedBrowser ?? "chromium",
      use: projectUse,
    },
  ],
});
