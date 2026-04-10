import assert from "node:assert/strict";
import test from "node:test";

import type { ProviderRecord, ProviderTopology } from "../src/contracts.ts";
import {
  formatProviderHoverDetails,
  formatProviderLocalityLabel,
  formatProviderModelLabel,
  formatProviderTopologySummary,
  formatProviderTypeLabel,
  inferProviderLocalityKind,
} from "../src/components/system/providerTopology.ts";

test("inferProviderLocalityKind prefers explicit topology locality", () => {
  assert.equal(
    inferProviderLocalityKind({
      endpoint: "https://example.com",
      llmType: "OpenAI",
      topology: { localityKind: "Local" } as ProviderTopology,
    }),
    "Local",
  );
});

test("inferProviderLocalityKind distinguishes local tunneled and cloud endpoints", () => {
  assert.equal(
    inferProviderLocalityKind({
      endpoint: "http://127.0.0.1:11434",
      llmType: "Ollama",
    }),
    "Local",
  );

  assert.equal(
    inferProviderLocalityKind({
      endpoint: "https://nostra-cortex-ea1-git-feat-capability-s-35e1da-bexyzas-projects.vercel.app",
      llmType: "OpenRouter",
    }),
    "Tunneled",
  );

  assert.equal(
    inferProviderLocalityKind({
      endpoint: "https://api.openai.com/v1",
      llmType: "OpenAI",
    }),
    "Cloud",
  );
});

test("provider topology formatters stay simple for common users", () => {
  const provider: ProviderRecord = {
    id: "llm_adapter",
    name: "Primary LLM Adapter",
    providerType: "Llm" as const,
    endpoint: "http://127.0.0.1:11434",
    isActive: true,
    priority: 1,
    defaultModel: "gpt-5.4",
    supportedModels: ["gpt-5.4", "gpt-4.1"],
    topology: {
      familyId: "openrouter",
      profileId: "gpt-5.4",
      instanceId: "llm_adapter__127.0.0.1_11434",
      deviceId: "workstation-a",
      environmentId: "local-dev",
      localityKind: "Local" as const,
      lastSeenAt: "2026-03-23T00:00:00Z",
      discoverySource: "registry",
    },
  };

  assert.equal(formatProviderLocalityLabel(provider), "Local");
  assert.equal(formatProviderModelLabel(provider), "gpt-5.4");
  assert.equal(formatProviderTypeLabel(provider.providerType), "LLM");
  assert.match(formatProviderTopologySummary(provider), /family openrouter/);
  assert.match(formatProviderHoverDetails(provider), /Locality: Local/);
});

test("provider model formatter falls back to not configured", () => {
  assert.equal(
    formatProviderModelLabel({
      endpoint: "https://example.com",
      llmType: "OpenAI",
      defaultModel: "",
      supportedModels: [],
    } as ProviderRecord),
    "Not configured",
  );
});
