import assert from "node:assert/strict";
import test from "node:test";

import type { ProviderRecord, ProviderTopology } from "../src/contracts.ts";
import {
  formatProviderAccessLabel,
  formatProviderBindingLabel,
  formatProviderHoverDetails,
  formatProviderLocalityLabel,
  formatProviderModelLabel,
  formatProviderTopologySummary,
  formatProviderTypeLabel,
  getProviderOperationalReadiness,
  getProviderReadiness,
  inferProviderLocalityKind,
} from "../src/components/system/providerTopology.ts";

test("inferProviderLocalityKind prefers explicit topology locality", () => {
  assert.equal(
    inferProviderLocalityKind({
      endpoint: "https://example.com",
      providerFamily: "OpenAI",
      topology: { localityKind: "Local" } as ProviderTopology,
    }),
    "Local",
  );
});

test("inferProviderLocalityKind distinguishes local tunneled and cloud endpoints", () => {
  assert.equal(
    inferProviderLocalityKind({
      endpoint: "http://127.0.0.1:11434",
      providerFamily: "Ollama",
    }),
    "Local",
  );

  assert.equal(
    inferProviderLocalityKind({
      endpoint: "https://nostra-cortex-ea1-git-feat-capability-s-35e1da-bexyzas-projects.vercel.app",
      providerFamily: "OpenRouter",
    }),
    "Tunneled",
  );

  assert.equal(
    inferProviderLocalityKind({
      endpoint: "https://api.openai.com/v1",
      providerFamily: "OpenAI",
    }),
    "Cloud",
  );
});

test("provider topology formatters stay simple for common users", () => {
  const provider: ProviderRecord = {
    id: "openrouter_primary",
    name: "OpenRouter Primary",
    providerType: "Llm" as const,
    endpoint: "http://127.0.0.1:11434",
    isActive: true,
    priority: 1,
    defaultModel: "gpt-5.4",
    supportedModels: ["gpt-5.4", "gpt-4.1"],
    topology: {
      familyId: "openrouter",
      profileId: "gpt-5.4",
      instanceId: "openrouter_primary__127.0.0.1_11434",
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

test("binding labels stay operator-readable", () => {
  assert.equal(formatProviderBindingLabel("llm.default"), "LLM default");
  assert.equal(formatProviderBindingLabel("embedding.default"), "Embedding Default");
});

test("provider access labels distinguish no-key linked inherited and missing auth states", () => {
  assert.equal(
    formatProviderAccessLabel({
      authState: "not_required",
    } as ProviderRecord),
    "No key required",
  );

  assert.equal(
    formatProviderAccessLabel({
      authState: "linked",
      authType: "api_key",
    } as ProviderRecord),
    "API key linked",
  );

  assert.equal(
    formatProviderAccessLabel({
      authState: "inherited",
      authType: "api_key",
    } as ProviderRecord),
    "Inherited runtime key",
  );

  assert.equal(
    formatProviderAccessLabel({
      authState: "missing",
      authType: "api_key",
    } as ProviderRecord),
    "Auth missing",
  );
});

test("provider model formatter falls back to not configured", () => {
  assert.equal(
    formatProviderModelLabel({
      endpoint: "https://example.com",
      providerFamily: "OpenAI",
      defaultModel: "",
      supportedModels: [],
    } as ProviderRecord),
    "Not configured",
  );
});

test("provider readiness separates ready neutral attention and disabled states", () => {
  assert.deepEqual(
    getProviderReadiness({
      isActive: true,
      authState: "linked",
    } as ProviderRecord),
    {
      state: "ready",
      label: "Ready",
      detail: "A linked auth binding is available.",
    },
  );

  assert.deepEqual(
    getProviderReadiness({
      isActive: true,
      authState: "inherited",
    } as ProviderRecord),
    {
      state: "neutral",
      label: "Inherited",
      detail: "This provider relies on inherited runtime auth.",
    },
  );

  assert.deepEqual(
    getProviderReadiness({
      isActive: true,
      authState: "missing",
    } as ProviderRecord),
    {
      state: "attention",
      label: "Needs auth",
      detail: "A linked auth binding is missing its secret.",
    },
  );

  assert.deepEqual(
    getProviderReadiness({
      isActive: false,
      authState: "linked",
    } as ProviderRecord),
    {
      state: "disabled",
      label: "Disabled",
      detail: "Provider is turned off.",
    },
  );
});

test("provider operational readiness flags validation issues over credential-only readiness", () => {
  assert.deepEqual(
    getProviderOperationalReadiness(
      {
        providerType: "Llm" as const,
        endpoint: "https://openrouter.ai/api",
        isActive: true,
        authState: "linked",
        supportedModels: ["openai/gpt-5.4"],
      } as ProviderRecord,
      {
        valid: false,
        supportedModels: [],
        modelsError: "provider_probe_models_empty",
        chatError: "provider_probe_missing_model_for_chat",
      },
    ),
    {
      state: "attention",
      label: "Needs attention",
      detail: "Latest validation or runtime checks reported issues, even though 1 saved model is still available.",
      issues: [
        "The provider did not return any models from its catalog endpoint.",
        "Choose a valid default model before chat validation can run.",
      ],
    },
  );
});

test("provider operational readiness keeps local credentialless runtimes usable", () => {
  assert.deepEqual(
    getProviderOperationalReadiness({
      providerType: "Llm" as const,
      providerFamily: "Ollama",
      endpoint: "http://127.0.0.1:11434",
      isActive: true,
      authState: "not_required",
      supportedModels: ["llama3.1:8b"],
      topology: {
        familyId: "ollama",
        instanceId: "ollama_local__http_127.0.0.1_11434",
        localityKind: "Local",
      } as ProviderTopology,
    } as ProviderRecord),
    {
      state: "ready",
      label: "Ready",
      detail: "Local runtime discovery surfaced 1 model without any provider-specific auth.",
      issues: [],
    },
  );
});

test("provider readiness keeps keyless local runtimes out of auth-missing states", () => {
  assert.deepEqual(
    getProviderReadiness({
      isActive: true,
      authState: "not_required",
    } as ProviderRecord),
    {
      state: "ready",
      label: "Ready",
      detail: "No auth secret is required for this provider.",
    },
  );
});
