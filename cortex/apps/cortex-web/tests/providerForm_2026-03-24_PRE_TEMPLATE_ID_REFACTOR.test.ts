import assert from "node:assert/strict";
import test from "node:test";

import { applyProviderTemplate, createEmptyProviderForm } from "../src/components/system/providerForm.ts";

test("switching provider templates clears stale model state", () => {
  const current = {
    ...createEmptyProviderForm(),
    templateId: "ollama" as const,
    providerId: "local_adapter",
    name: "Local Adapter",
    endpoint: "http://127.0.0.1:11434/v1",
    defaultModel: "llama3.1:8b",
    adapterSetRef: "adapter.local",
    credentialBindingId: "cred-local",
  };

  const next = applyProviderTemplate("openrouter", current);

  assert.equal(next.templateId, "openrouter");
  assert.equal(next.providerId, "local_adapter");
  assert.equal(next.name, "Local Adapter");
  assert.equal(next.endpoint, "https://openrouter.ai/api/v1");
  assert.equal(next.defaultModel, "");
  assert.equal(next.adapterSetRef, "");
  assert.equal(next.credentialBindingId, "");
});
