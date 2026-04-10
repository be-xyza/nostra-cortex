import assert from "node:assert/strict";
import test from "node:test";

import { humanizeProviderDiagnostic, resolveAdapterStatusError } from "../src/components/system/providerDiagnostics.ts";

test("resolveAdapterStatusError suppresses parse warnings when upstream models loaded", () => {
  assert.equal(
    resolveAdapterStatusError({
      upstreamModels: { data: [{ id: "openai/gpt-5.4" }] },
      openapiError: "runtime_openapi_parse_failed:expected value at line 1 column 1",
      adapterHealthError: "runtime_health_parse_failed:expected value at line 1 column 1",
    }),
    null,
  );
});

test("resolveAdapterStatusError keeps the upstream models error when catalog loading fails", () => {
  assert.equal(
    resolveAdapterStatusError({
      upstreamModelsError: "runtime_models_http_500:oops",
      openapiError: "runtime_openapi_parse_failed:expected value at line 1 column 1",
      adapterHealthError: "runtime_health_parse_failed:expected value at line 1 column 1",
    }),
    "runtime_models_http_500:oops",
  );
});

test("humanizeProviderDiagnostic rewrites empty-model probe errors into operator copy", () => {
  assert.equal(
    humanizeProviderDiagnostic("provider_probe_models_empty"),
    "The provider did not return any models from its catalog endpoint.",
  );
});
