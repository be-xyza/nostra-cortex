# Playwright Notes

- Local macOS baseline for focused `cortex-web` browser tests is `webkit`.
- `playwright.heap.config.ts` owns the Vite dev server on `127.0.0.1:4174`.
- The layout experiments use `"/__test/*"` harness routes so renderer tests avoid `ShellLayout` bootstrap and live gateway dependencies.
- The layout browser coverage is split into `layout-catalogue.spec.ts` and `layout-registry.spec.ts` so catalogue and registry failures can be isolated cleanly.
- The layout harness spec resolves service-worker registration immediately so Vite's dev PWA path cannot stall startup.
- `npm run test:layout-playwright` now uses the validated Chrome-channel path by default on this machine.
- Use `npm run test:layout-playwright:webkit` only when explicitly debugging the local WebKit runtime.
- Latest known macOS machine finding: the layout harness routes and raw Playwright Chrome context are healthy, but local Playwright WebKit can still fail before any page navigation on this machine.
- Bundled Playwright Chromium launch failures on this machine are environment-specific and are not treated as blockers for the layout harness slice.
