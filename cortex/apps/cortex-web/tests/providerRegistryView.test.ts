import assert from "node:assert/strict";
import test from "node:test";

import type { ProviderRecord } from "../src/contracts.ts";
import {
  buildProviderRegistrySections,
  readProviderRegistryPanelState,
  validateProviderDraftInput,
  writeProviderRegistryPanelState,
} from "../src/components/system/providerRegistryView.ts";

function provider(overrides: Partial<ProviderRecord>): ProviderRecord {
  return {
    id: "provider-default",
    name: "Provider Default",
    providerType: "Llm",
    hostId: "host.managed.provider-default",
    endpoint: "https://example.com/v1",
    isActive: true,
    priority: 1,
    supportedModels: [],
    ...overrides,
  };
}

test("readProviderRegistryPanelState and writeProviderRegistryPanelState round-trip provider and discovery sheets", () => {
  const providerState = readProviderRegistryPanelState(
    new URLSearchParams("panel=provider&providerId=openrouter_primary&search=router"),
  );
  assert.deepEqual(providerState, {
    kind: "provider",
    providerId: "openrouter_primary",
  });

  const discoveryParams = writeProviderRegistryPanelState(
    new URLSearchParams("search=router&readiness=ready"),
    { kind: "discovery", seedModel: "llama3.1:8b", providerId: "openrouter_primary" },
  );
  assert.equal(discoveryParams.get("search"), "router");
  assert.equal(discoveryParams.get("readiness"), "ready");
  assert.equal(discoveryParams.get("panel"), "discovery");
  assert.equal(discoveryParams.get("seedModel"), "llama3.1:8b");
  assert.equal(discoveryParams.get("providerId"), "openrouter_primary");

  const cleared = writeProviderRegistryPanelState(discoveryParams, { kind: "none" });
  assert.equal(cleared.get("panel"), null);
  assert.equal(cleared.get("providerId"), null);
  assert.equal(cleared.get("seedModel"), null);
  assert.equal(cleared.get("search"), "router");
});

test("buildProviderRegistrySections keeps a neutral registry model with grouped sorted filtered providers", () => {
  const sections = buildProviderRegistrySections(
    [
      provider({
        id: "z-cloud",
        name: "Zed Cloud",
        providerFamily: "OpenAI",
        hostId: "host.managed.openai",
        defaultModel: "gpt-5.4",
        authState: "linked",
      }),
      provider({
        id: "local-ollama",
        name: "Local Ollama",
        endpoint: "http://127.0.0.1:11434/v1",
        providerFamily: "Ollama",
        hostId: "host.local.primary",
        defaultModel: "llama3.1:8b",
        authState: "not_required",
      }),
      provider({
        id: "embed-a",
        name: "Embedding A",
        providerType: "Embedding",
        endpoint: "https://embed.example.com",
        isActive: false,
      }),
    ],
    {
      searchTerm: "",
      providerType: "Llm",
      readiness: "all",
    },
  );

  assert.equal(sections.length, 1);
  assert.equal(sections[0]?.providerType, "Llm");
  assert.deepEqual(
    sections[0]?.providers.map((item) => item.id),
    ["local-ollama", "z-cloud"],
  );
});

test("buildProviderRegistrySections search matches provider family and host identity without exposing raw ids in the row model", () => {
  const sections = buildProviderRegistrySections(
    [
      provider({
        id: "ollama_local",
        name: "Ollama",
        endpoint: "http://127.0.0.1:11434",
        providerFamily: "Ollama",
        hostId: "host.local.primary",
        defaultModel: "llama3.1:8b",
        authState: "not_required",
      }),
    ],
    {
      searchTerm: "local",
      providerType: "all",
      readiness: "all",
    },
  );

  assert.equal(sections.length, 1);
  assert.equal(sections[0]?.providers[0]?.id, "ollama_local");
});

test("validateProviderDraftInput does not require an api key for a new provider", () => {
  assert.equal(
    validateProviderDraftInput({
      providerId: "local_provider",
      providerName: "Local Provider",
      providerEndpoint: "http://127.0.0.1:11434/v1",
      metadataJson: "{\n  \"note\": \"local runtime\"\n}",
    }),
    null,
  );

  assert.equal(
    validateProviderDraftInput({
      providerId: "",
      providerName: "Missing Id",
      providerEndpoint: "http://127.0.0.1:11434/v1",
      metadataJson: "{}",
    }),
    "Provider ID, name, and endpoint are required.",
  );
});
