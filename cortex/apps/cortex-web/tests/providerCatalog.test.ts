import assert from "node:assert/strict";
import test from "node:test";

import { buildProviderCatalogState } from "../src/components/system/providerCatalog.ts";

test("buildProviderCatalogState prefers a freshly entered key for catalog refresh", () => {
  const state = buildProviderCatalogState({
    providerId: "openrouter_primary",
    hasStoredAuth: true,
    draftApiKey: "sk-live",
    catalogSize: 0,
  });

  assert.equal(state.refreshMode, "draft_auth");
  assert.equal(state.canRefresh, true);
  assert.match(state.helperText, /entered auth secret/i);
});

test("buildProviderCatalogState allows existing providers to refresh from a stored auth binding", () => {
  const state = buildProviderCatalogState({
    providerId: "openrouter_primary",
    hasStoredAuth: true,
    draftApiKey: "",
    catalogSize: 0,
  });

  assert.equal(state.refreshMode, "stored_auth");
  assert.equal(state.canRefresh, true);
  assert.doesNotMatch(state.helperText, /after validation/i);
});

test("buildProviderCatalogState keeps copy honest when no auth secret is available", () => {
  const state = buildProviderCatalogState({
    providerId: "openrouter_primary",
    hasStoredAuth: false,
    draftApiKey: "",
    catalogSize: 0,
  });

  assert.equal(state.refreshMode, "unavailable");
  assert.equal(state.canRefresh, false);
  assert.match(state.helperText, /add or paste an auth secret/i);
});

test("buildProviderCatalogState allows anonymous refresh for local runtimes", () => {
  const state = buildProviderCatalogState({
    providerId: "ollama_local",
    hasStoredAuth: false,
    draftApiKey: "",
    catalogSize: 0,
    allowsAnonymousDiscovery: true,
  });

  assert.equal(state.refreshMode, "anonymous");
  assert.equal(state.canRefresh, true);
  assert.match(state.helperText, /without any auth binding/i);
});
