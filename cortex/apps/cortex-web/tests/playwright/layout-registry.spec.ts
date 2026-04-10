import { expect, test } from "@playwright/test";

test.beforeEach(async ({ page }) => {
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

test("workbench widget routing still resolves SpatialHeapGrid through WidgetRegistry", async ({ page }) => {
  await page.goto("/__test/layout-registry", { waitUntil: "domcontentloaded" });

  await expect(page.getByRole("heading", { name: "Layout Registry Harness" })).toBeVisible();
  await expect(page.getByText("Registry contract block one")).toBeVisible();
  await expect(page.getByText("Registry contract block two")).toBeVisible();
  await expect(page.getByText("Registry contract block three")).toBeVisible();
});
