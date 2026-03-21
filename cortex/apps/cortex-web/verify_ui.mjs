import { chromium } from 'playwright';

(async () => {
  const browser = await chromium.launch();
  const context = await browser.newContext();
  const page = await context.newPage();
  
  // Clear site data to force SW re-seed
  await context.clearCookies();
  await context.addInitScript(() => {
    window.indexedDB.deleteDatabase('cortex-event-store');
  });
  
  await page.setViewportSize({ width: 1280, height: 800 });

  console.log("Capturing Explore and Sidebar...");
  await page.goto('http://localhost:4173/explore', { waitUntil: 'networkidle' });
  // Wait a bit for transitions
  await page.waitForTimeout(1000);
  await page.screenshot({ path: '/Users/xaoj/.gemini/antigravity/brain/b5e803db-32fc-434d-ab2d-69682512b573/canvas_explore_verification.png', fullPage: true });

  console.log("Capturing Inbox...");
  await page.goto('http://localhost:4173/explore?heap_view=Inbox', { waitUntil: 'networkidle' });
  await page.waitForTimeout(1000);
  await page.screenshot({ path: '/Users/xaoj/.gemini/antigravity/brain/b5e803db-32fc-434d-ab2d-69682512b573/canvas_inbox_verification.png', fullPage: true });

  console.log("Expanding Space Selector...");
  await page.goto('http://localhost:4173/explore', { waitUntil: 'networkidle' });
  await page.click('[aria-label="Select Space"]');
  await page.waitForTimeout(500);
  await page.screenshot({ path: '/Users/xaoj/.gemini/antigravity/brain/b5e803db-32fc-434d-ab2d-69682512b573/space_selector_verification.png' });

  console.log("Capturing Spaces Page...");
  await page.goto('http://localhost:4173/spaces', { waitUntil: 'networkidle' });
  await page.waitForTimeout(1000);
  await page.screenshot({ path: '/Users/xaoj/.gemini/antigravity/brain/b5e803db-32fc-434d-ab2d-69682512b573/spaces_page_verification.png', fullPage: true });

  await browser.close();
  console.log("Done.");
})();
