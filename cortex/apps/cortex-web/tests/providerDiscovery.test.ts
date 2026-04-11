import assert from "node:assert/strict";
import test from "node:test";

import { buildDiscoveryProviderForm, describeRuntimeHostInventory, isLocalDiscoveryRecord } from "../src/components/system/providerDiscovery.ts";

test("buildDiscoveryProviderForm seeds a provider from adapter discovery", () => {
  const form = buildDiscoveryProviderForm(
    {
      baseUrl: "http://127.0.0.1:11434/v1",
      model: "llama3.1:8b",
    },
    "llama3.1:8b",
  );

  assert.equal(form.templateId, "ollama");
  assert.equal(form.providerType, "Llm");
  assert.equal(form.providerKind, "Ollama");
  assert.equal(form.endpoint, "http://127.0.0.1:11434/v1");
  assert.equal(form.defaultModel, "llama3.1:8b");
  assert.equal(form.authBindingId, "");
  assert.equal(form.hostId, "host.local.primary");
  assert.equal(form.providerId, "ollama_local");
  assert.equal(form.name, "Local OpenAI-compatible Provider");
  assert.equal(form.enabled, true);
});

test("isLocalDiscoveryRecord recognizes local Ollama-style discovery entries", () => {
  assert.equal(
    isLocalDiscoveryRecord({
      providerKind: "Ollama",
      endpoint: "http://127.0.0.1:11434",
      topology: { familyId: "ollama", instanceId: "ollama_local", localityKind: "Local" },
    }),
    true,
  );
});

test("describeRuntimeHostInventory explains when a tracked host has no provider runtime yet", () => {
  assert.deepEqual(
    describeRuntimeHostInventory(
      {
        hostId: "host.vps.primary",
        name: "Eudaemon Alpha VPS",
        hostKind: "vps",
        endpoint: "ssh://root@204.168.175.150",
        localityKind: "Cloud",
        metadata: {
          remoteDiscoveryStatus: "no_runtime",
          remoteDiscoveryDetail: "No provider runtime detected over SSH.",
        },
      },
      0,
      0,
    ),
    {
      label: "No provider runtime detected yet",
      detail: "No provider runtime detected over SSH.",
    },
  );
});

test("describeRuntimeHostInventory summarizes discovered provider runtimes", () => {
  assert.deepEqual(
    describeRuntimeHostInventory(
      {
        hostId: "host.local.primary",
        name: "Local machine",
        hostKind: "local",
        endpoint: "http://127.0.0.1:11434",
        localityKind: "Local",
      },
      2,
      5,
    ),
    {
      label: "2 provider runtimes discovered",
      detail: "5 models are currently visible from providers on this host.",
    },
  );
});
