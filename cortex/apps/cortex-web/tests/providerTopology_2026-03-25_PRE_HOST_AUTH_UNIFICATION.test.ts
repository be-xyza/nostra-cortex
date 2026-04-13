import assert from "node:assert/strict";
import test from "node:test";

import type { ProviderRecord, ProviderTopology } from "../src/contracts.ts";
import {
  formatProviderBindingLabel,
  formatProviderCredentialState,
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

test("provider readiness separates ready neutral attention and disabled states", () => {
  assert.deepEqual(
    getProviderReadiness({
      isActive: true,
      hasCredential: true,
    } as ProviderRecord),
    {
      state: "ready",
      label: "Ready",
      detail: "Credential is available.",
    },
  );

  assert.deepEqual(
    getProviderReadiness({
      isActive: true,
      hasCredential: false,
      credentialBindingId: "",
    } as ProviderRecord),
    {
      state: "neutral",
      label: "Uses default",
      detail: "No provider-specific credential is linked.",
    },
  );

  assert.deepEqual(
    getProviderReadiness({
      isActive: true,
      hasCredential: false,
      credentialBindingId: "binding-primary",
    } as ProviderRecord),
    {
      state: "attention",
      label: "Needs credential",
      detail: "A linked credential record is missing.",
    },
  );

  assert.deepEqual(
    getProviderReadiness({
      isActive: false,
      hasCredential: true,
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
        hasCredential: true,
        credentialBindingId: "cred-openrouter",
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
      llmType: "Ollama",
      endpoint: "http://127.0.0.1:11434",
      isActive: true,
      hasCredential: false,
      credentialBindingId: "",
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
      detail: "Local runtime discovery surfaced 1 model without a provider-specific credential.",
      issues: [],
    },
  );
});

test("provider credential formatter avoids false negative unbound copy", () => {
  assert.equal(
    formatProviderCredentialState({
      hasCredential: true,
      credentialBindingId: "binding-primary",
    } as ProviderRecord),
    "Connected",
  );

  assert.equal(
    formatProviderCredentialState({
      hasCredential: false,
      credentialBindingId: "",
    } as ProviderRecord),
    "System default",
  );

  assert.equal(
    formatProviderCredentialState({
      hasCredential: false,
      credentialBindingId: "binding-primary",
    } as ProviderRecord),
    "Credential missing",
  );
});
