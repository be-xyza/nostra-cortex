import assert from "node:assert/strict";
import test from "node:test";

import { applyProviderTemplate, buildTemplateProviderId, createEmptyProviderForm } from "../src/components/system/providerForm.ts";

test("template-based provider ids stay unique per template", () => {
  assert.equal(buildTemplateProviderId("openai", []), "openai_primary");
  assert.equal(buildTemplateProviderId("openai", ["openai_primary"]), "openai_primary_2");
  assert.equal(buildTemplateProviderId("manual", ["custom_provider", "custom_provider_2"]), "custom_provider_3");
  assert.equal(buildTemplateProviderId("ollama", []), "ollama_local");
});

test("empty provider forms start from a neutral custom template", () => {
  const form = createEmptyProviderForm();

  assert.equal(form.templateId, "manual");
  assert.equal(form.providerKind, "");
  assert.equal(form.endpoint, "");
  assert.equal(form.useAsDefaultLlm, false);
});

test("switching provider templates clears stale model state", () => {
  const current = {
    ...createEmptyProviderForm(),
    templateId: "ollama" as const,
    providerId: "local_adapter",
    name: "Local Adapter",
    hostId: "host.local.primary",
    endpoint: "http://127.0.0.1:11434/v1",
    defaultModel: "llama3.1:8b",
    authBindingId: "auth.none.local_adapter",
  };

  const next = applyProviderTemplate("openrouter", current);

  assert.equal(next.templateId, "openrouter");
  assert.equal(next.providerId, "local_adapter");
  assert.equal(next.name, "Local Adapter");
  assert.equal(next.endpoint, "https://openrouter.ai/api/v1");
  assert.equal(next.defaultModel, "");
  assert.equal(next.authBindingId, "");
  assert.equal(next.useAsDefaultLlm, false);
});
