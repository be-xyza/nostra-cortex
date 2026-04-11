import assert from "node:assert/strict";
import test from "node:test";

import { applyProviderTemplate, buildTemplateProviderId, createEmptyProviderForm } from "../src/components/system/providerForm.ts";

test("template-based provider ids stay unique per template", () => {
  assert.equal(buildTemplateProviderId("openai", []), "openai_provider");
  assert.equal(buildTemplateProviderId("openai", ["openai_provider"]), "openai_provider_2");
  assert.equal(buildTemplateProviderId("manual", ["manual_provider", "manual_provider_2"]), "manual_provider_3");
});

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
