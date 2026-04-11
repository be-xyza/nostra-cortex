import { expect, test } from "@playwright/test";

test.beforeEach(async ({ page }) => {
  await page.route("**/api/**", async (route) => {
    await route.fulfill({
      status: 500,
      contentType: "text/plain",
      body: "playwright fallback",
    });
  });

  await page.addInitScript(() => {
    const serviceWorkerStub = {
      active: null,
      installing: null,
      waiting: null,
      scope: "/",
      unregister: () => Promise.resolve(true),
      update: () => Promise.resolve(),
      addEventListener: () => {},
      removeEventListener: () => {},
    };
    // @ts-ignore
    window.navigator.serviceWorker.register = () => Promise.resolve(serviceWorkerStub);
    // @ts-ignore
    window.navigator.serviceWorker.ready = Promise.resolve(serviceWorkerStub);
    // @ts-ignore
    window.navigator.serviceWorker.getRegistration = () => Promise.resolve(undefined);
    // @ts-ignore
    window.navigator.serviceWorker.getRegistrations = () => Promise.resolve([]);
    indexedDB.deleteDatabase("cortex-event-store");
  });
});

test("catalogue exposes the normalized experimental layout families", async ({ page }) => {
  await page.goto("/__test/layout-catalogue", { waitUntil: "domcontentloaded" });

  await expect(page.getByRole("heading", { name: "Layout Matrix Catalogue" })).toBeVisible();
  await expect(page.getByText(/Experimental Cortex Labs vocabulary only/i)).toBeVisible();
  await expect(page.getByRole("button", { name: "Lane Board" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Spatial BSP" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Force Graph" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Time Indexed" })).toBeVisible();
  await expect(page.getByText("Caffeine improves short-term recall")).toBeVisible();
  await expect(page.getByText("Spatial architectures require explicit topology contracts.")).toBeVisible();

  await page.getByRole("button", { name: "Force Graph" }).click();
  await expect(page.getByText(/Force-graph family remains experimental/i)).toBeVisible();

  await page.getByRole("button", { name: "Lane Board" }).click();
  await expect(page.getByText("Promoted outputs")).toBeVisible();
});

test("/labs/layout-catalogue renders the public catalogue surface", async ({ page }) => {
  await page.goto("/labs/layout-catalogue", { waitUntil: "domcontentloaded" });
  await expect(page.getByRole("heading", { name: "Layout Matrix Catalogue" })).toBeVisible();
});
