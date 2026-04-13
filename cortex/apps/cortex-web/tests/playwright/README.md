# Playwright Notes

- `playwright.heap.config.ts` owns the Vite dev server on `127.0.0.1:4174` and reuses it when one is already running.
- The layout experiments use `"/__test/*"` harness routes so renderer tests avoid `ShellLayout` bootstrap and live gateway dependencies.
- The layout browser coverage is split into `layout-catalogue.spec.ts` and `layout-registry.spec.ts` so catalogue and registry failures can be isolated cleanly.
- The layout harness spec resolves service-worker registration immediately so Vite's dev PWA path cannot stall startup.
- `PLAYWRIGHT_BROWSER=chrome` selects the Chrome channel, `PLAYWRIGHT_BROWSER=webkit` selects WebKit, and the default remains the standard Chromium device profile used by the existing heap/browser scripts.
- `npm run test:layout-playwright` and `npm run test:layout-playwright:chrome` intentionally drive the Chrome-channel path for the layout harness.
- Use `npm run test:layout-playwright:webkit` only when explicitly debugging the local WebKit runtime.
