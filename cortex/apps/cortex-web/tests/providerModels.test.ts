import assert from "node:assert/strict";
import test from "node:test";

import { buildProviderModelOptions, buildSelectableModelOptions, extractModelNames } from "../src/components/system/providerModels.ts";

test("extractModelNames normalizes array and data payloads", () => {
  assert.deepEqual(
    extractModelNames({
      data: [
        { id: "openai/gpt-5.4" },
        { name: "llama3.1:8b" },
        "qwen2.5-coder",
        { id: "  " },
      ],
    }),
    ["openai/gpt-5.4", "llama3.1:8b", "qwen2.5-coder"],
  );
});

test("extractModelNames reads Ollama-style models payloads", () => {
  assert.deepEqual(
    extractModelNames({
      models: [
        { name: "llama3.2:8b" },
        { model: "qwen2.5-coder" },
      ],
    }),
    ["llama3.2:8b", "qwen2.5-coder"],
  );
});

test("buildSelectableModelOptions keeps the current custom model visible", () => {
  assert.deepEqual(
    buildSelectableModelOptions(
      "custom-model",
      ["openai/gpt-5.4", "openai/gpt-4.1"],
      ["openai/gpt-4.1", "llama3.1:8b"],
    ),
    ["custom-model", "openai/gpt-5.4", "openai/gpt-4.1", "llama3.1:8b"],
  );
});

test("buildProviderModelOptions ignores adapter discovery for remote provider detail", () => {
  assert.deepEqual(
    buildProviderModelOptions({
      currentModel: "openrouter/sonoma",
      validatedModels: ["openrouter/sonoma", "openrouter/horizon"],
      savedProviderModels: ["openrouter/sonoma"],
      adapterDiscoveryModels: ["llama3.2:8b", "qwen2.5-coder"],
      adapterRuntimeModel: "llama3.2:8b",
      panelKind: "provider",
      providerId: "openrouter_primary",
      templateId: "openrouter",
    }),
    ["openrouter/sonoma", "openrouter/horizon"],
  );
});

test("buildProviderModelOptions includes adapter discovery for local create flow", () => {
  assert.deepEqual(
    buildProviderModelOptions({
      currentModel: "",
      validatedModels: [],
      savedProviderModels: [],
      adapterDiscoveryModels: ["llama3.2:8b", "qwen2.5-coder"],
      adapterRuntimeModel: "llama3.2:8b",
      panelKind: "create",
      templateId: "ollama",
    }),
    ["llama3.2:8b", "qwen2.5-coder"],
  );
});

test("buildProviderModelOptions no longer depends on legacy special provider ids for detail catalogs", () => {
  assert.deepEqual(
    buildProviderModelOptions({
      currentModel: "llama3.2:8b",
      validatedModels: [],
      savedProviderModels: ["llama3.2:8b", "qwen2.5-coder"],
      adapterDiscoveryModels: ["openai/gpt-5.4", "openai/gpt-4.1"],
      adapterRuntimeModel: "openai/gpt-5.4",
      panelKind: "provider",
      providerId: "ollama_local",
      templateId: "ollama",
    }),
    ["llama3.2:8b", "qwen2.5-coder"],
  );
});
