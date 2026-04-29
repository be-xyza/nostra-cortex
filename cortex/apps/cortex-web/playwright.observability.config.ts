import { defineConfig, devices } from "@playwright/test";

export default defineConfig({
  testDir: "./tests/playwright",
  fullyParallel: false,
  timeout: 60 * 1000,
  expect: {
    timeout: 15 * 1000,
  },
  use: {
    baseURL: "https://observability.local",
    trace: "retain-on-failure",
    screenshot: "only-on-failure",
    video: "off",
    serviceWorkers: "block",
    ignoreHTTPSErrors: true,
    ...devices["Desktop Chrome"],
  },
  reporter: [["list"]],
  projects: [
    {
      name: "chromium",
      use: {
        browserName: "chromium",
      },
    },
  ],
});
