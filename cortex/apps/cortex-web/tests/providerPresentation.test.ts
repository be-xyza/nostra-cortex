import assert from "node:assert/strict";
import test from "node:test";

import {
  buildDefaultModelControlState,
  buildProviderWorkbenchChromeState,
} from "../src/components/system/providerPresentation.ts";

test("buildProviderWorkbenchChromeState keeps the registry layout stable while a docked panel is open", () => {
  const detailState = buildProviderWorkbenchChromeState({ kind: "provider", providerId: "openrouter_primary" });

  assert.equal(detailState.compactRegistryChrome, false);
  assert.equal(detailState.compactRegistryRows, false);
  assert.equal(detailState.contentPaddingClass, "");
  assert.equal(detailState.headerLayoutClass, "flex flex-col gap-4 xl:flex-row xl:items-end xl:justify-between");
});

test("buildProviderWorkbenchChromeState keeps full chrome for the neutral registry", () => {
  const neutralState = buildProviderWorkbenchChromeState({ kind: "none" });

  assert.equal(neutralState.compactRegistryChrome, false);
  assert.equal(neutralState.compactRegistryRows, false);
  assert.equal(neutralState.contentPaddingClass, "");
  assert.match(neutralState.description, /provider readiness/i);
});

test("buildDefaultModelControlState uses a selector when the catalog is loaded", () => {
  const loadedState = buildDefaultModelControlState(24);
  const emptyState = buildDefaultModelControlState(0);

  assert.equal(loadedState.control, "combobox");
  assert.match(loadedState.helperText, /24 loaded models/i);
  assert.equal(emptyState.control, "input");
  assert.match(emptyState.helperText, /No catalog is loaded yet/i);
});
