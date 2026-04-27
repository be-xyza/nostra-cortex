import { expect, test } from "@playwright/test";

const RESEARCH_SPACE_ID = "01KM4C04QY37V9RV9H2HH9J1NM";
const REVIEW_ARTIFACT_ID = "local-dev-research-hermes-review-001";

function noDeltaResponse() {
  return {
    schemaVersion: "1.0.0",
    generatedAt: "2026-04-25T00:00:00Z",
    count: 0,
    hasMore: false,
    changed: [],
    deleted: [],
  };
}

test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => {
    indexedDB.deleteDatabase("cortex-event-store");
    window.localStorage.setItem("cortex.shell.space.id", RESEARCH_SPACE_ID);
  });

  await page.route(/\/api\/cortex\/studio\/heap\/changed_blocks/, async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify(noDeltaResponse()),
    });
  });

  await page.route(/\/api\/cortex\/studio\/heap\/blocks(\?.*)?$/, async (route) => {
    await route.fulfill({
      status: 503,
      contentType: "application/json",
      body: JSON.stringify({ error: "force local-dev heap fallback" }),
    });
  });

  await page.route(/\/api\/cortex\/studio\/heap\/blocks\/.*\/a2ui\/feedback/, async (route) => {
    await route.fulfill({
      status: 503,
      contentType: "application/json",
      body: JSON.stringify({ error: "force local-dev feedback fallback" }),
    });
  });
});

test("review receipt opens generated feedback artifact without losing space context", async ({ page }) => {
  await page.goto(`/explore?space_id=${RESEARCH_SPACE_ID}&artifact_id=${REVIEW_ARTIFACT_ID}`);

  await expect(page.getByRole("button", { name: "Approve" })).toBeVisible();
  await expect(page.getByText("Approval stores steward feedback")).toBeVisible();

  await page.getByRole("button", { name: "Approve" }).click();

  await expect(page.getByText("Decision receipt")).toBeVisible();
  await expect(page.getByText("Approval recorded")).toBeVisible();
  await expect(page.getByRole("button", { name: "Open feedback" })).toBeVisible();

  await page.getByRole("button", { name: "Open feedback" }).click();

  await expect(page).toHaveURL(new RegExp(`space_id=${RESEARCH_SPACE_ID}.*artifact_id=local-dev-feedback-${REVIEW_ARTIFACT_ID}`));
  await expect(page.getByText("Decision receipt")).toBeVisible();
  await expect(page.getByText("local-dev-feedback", { exact: false })).toBeVisible();
  await expect(page.getByText("Stored feedback")).toBeVisible();
});
