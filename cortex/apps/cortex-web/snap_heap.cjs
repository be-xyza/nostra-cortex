const { chromium } = require('playwright');
const { mkdirSync } = require('node:fs');
const { join } = require('node:path');
(async () => {
  const outputDir = join('/tmp', 'cortex-web-heap-snapshots');
  mkdirSync(outputDir, { recursive: true });
  const browser = await chromium.launch();
  const page = await browser.newPage();
  await page.goto('http://localhost:4174/heap');
  await page.waitForLoadState('networkidle');
  await page.waitForTimeout(2000);
  await page.screenshot({ path: join(outputDir, 'heap_ui_refined_1.png'), fullPage: true });

  try {
     const block = page.locator('.heap-block-card').nth(1);
     await block.click();
     await page.waitForTimeout(1000);
     await page.screenshot({ path: join(outputDir, 'heap_ui_refined_selected.png'), fullPage: true });
     console.log("Captured screenshots.");
  } catch (e) {
     console.log("Error selecting block.", e);
  }
  await browser.close();
})();
