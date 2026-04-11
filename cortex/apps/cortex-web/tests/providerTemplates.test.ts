import assert from "node:assert/strict";
import test from "node:test";

import { providerTemplates, resolveProviderTemplate, slugifyProviderName } from "../src/components/system/providerTemplates.ts";

test("provider templates include the OpenRouter onboarding preset", () => {
  const template = resolveProviderTemplate("OpenRouter");
  assert.equal(template.id, "openrouter");
  assert.equal(template.defaultEndpoint, "https://openrouter.ai/api/v1");
  assert.equal(template.validateKey, true);
  assert.equal(template.validateChat, true);
});

test("provider templates infer from endpoint when the provider kind is missing", () => {
  const template = resolveProviderTemplate(undefined, "https://api.openai.com/v1");
  assert.equal(template.id, "openai");
  assert.equal(template.providerKind, "OpenAI");
});

test("provider name slugging is stable for generated ids", () => {
  assert.equal(slugifyProviderName("Primary OpenRouter Provider"), "primary_openrouter_provider");
});

test("template catalog keeps the guided onboarding surface small", () => {
  assert.ok(providerTemplates.length >= 4);
});
