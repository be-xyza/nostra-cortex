import assert from "node:assert/strict";
import test from "node:test";

import { workbenchApi } from "../src/api.ts";
import type {
  AuthBindingInventoryResponse,
  ExecutionBindingStatusResponse,
  OperatorProviderInventoryResponse,
  ProviderDiscoveryInventoryResponse,
  RuntimeHostInventoryResponse,
} from "../src/contracts.ts";
import { buildProviderRegistrySnapshotFromSplitReads, useProvidersRegistry } from "../src/store/providersRegistry.ts";

function providerInventoryResponse(): OperatorProviderInventoryResponse {
  return {
    schemaVersion: "1.0.0",
    generatedAt: "2026-03-25T00:00:00Z",
    providers: [
      {
        id: "openai_primary",
        name: "OpenAI Primary",
        providerType: "Llm",
        providerFamily: "OpenAI",
        hostId: "",
        endpoint: "https://api.openai.com/v1",
        isActive: true,
        priority: 1,
        defaultModel: "gpt-5.4",
        supportedModels: ["gpt-5.4"],
        authState: "linked",
      },
    ],
  };
}

function runtimeHostInventoryResponse(): RuntimeHostInventoryResponse {
  return {
    schemaVersion: "1.0.0",
    generatedAt: "2026-03-25T00:00:00Z",
    runtimeHosts: [
      {
        hostId: "host.local.primary",
        name: "Local Host",
        hostKind: "local",
        endpoint: "http://127.0.0.1:11434",
        localityKind: "Local",
        remoteDiscoveryEnabled: false,
        executionRoutable: true,
      },
    ],
  };
}

function authBindingInventoryResponse(): AuthBindingInventoryResponse {
  return {
    schemaVersion: "1.0.0",
    generatedAt: "2026-03-25T00:00:00Z",
    authBindings: [
      {
        authBindingId: "auth.openai_primary",
        targetKind: "provider",
        targetId: "openai_primary",
        authType: "api_key",
        label: "OpenAI key",
        hasSecret: true,
        updatedAt: "2026-03-25T00:00:00Z",
      },
    ],
  };
}

function executionBindingStatusResponse(): ExecutionBindingStatusResponse {
  return {
    schemaVersion: "1.0.0",
    generatedAt: "2026-03-25T00:00:00Z",
    executionBindings: [
      {
        bindingId: "llm.default",
        providerType: "Llm",
        boundProviderId: "openai_primary",
      },
    ],
  };
}

function providerDiscoveryInventoryResponse(): ProviderDiscoveryInventoryResponse {
  return {
    schemaVersion: "1.0.0",
    generatedAt: "2026-03-25T00:00:00Z",
    discoveryRecords: [
      {
        providerId: "openai_primary",
        providerType: "Llm",
        providerKind: "OpenAI",
        endpoint: "https://api.openai.com/v1",
        supportedModels: ["gpt-5.4"],
      },
    ],
  };
}

test("buildProviderRegistrySnapshotFromSplitReads composes the registry snapshot from operator reads", () => {
  const snapshot = buildProviderRegistrySnapshotFromSplitReads({
    providerInventory: providerInventoryResponse(),
    runtimeHostInventory: runtimeHostInventoryResponse(),
    authBindingInventory: authBindingInventoryResponse(),
    executionBindingStatus: executionBindingStatusResponse(),
    providerDiscoveryInventory: providerDiscoveryInventoryResponse(),
  });

  assert.equal(snapshot.status, "ready");
  assert.equal(snapshot.providers[0]?.id, "openai_primary");
  assert.equal(snapshot.runtimeHosts[0]?.hostId, "host.local.primary");
  assert.equal(snapshot.authBindings[0]?.authBindingId, "auth.openai_primary");
  assert.equal(snapshot.executionBindings[0]?.bindingId, "llm.default");
  assert.equal(snapshot.discoveryRecords[0]?.providerId, "openai_primary");
});

test("buildProviderRegistrySnapshotFromSplitReads marks the registry empty when inventories are empty", () => {
  const snapshot = buildProviderRegistrySnapshotFromSplitReads({
    providerInventory: {
      schemaVersion: "1.0.0",
      generatedAt: "2026-03-25T00:00:00Z",
      providers: [],
    },
    runtimeHostInventory: {
      schemaVersion: "1.0.0",
      generatedAt: "2026-03-25T00:00:00Z",
      runtimeHosts: [],
    },
    authBindingInventory: {
      schemaVersion: "1.0.0",
      generatedAt: "2026-03-25T00:00:00Z",
      authBindings: [],
    },
    executionBindingStatus: {
      schemaVersion: "1.0.0",
      generatedAt: "2026-03-25T00:00:00Z",
      executionBindings: [],
    },
    providerDiscoveryInventory: {
      schemaVersion: "1.0.0",
      generatedAt: "2026-03-25T00:00:00Z",
      discoveryRecords: [],
    },
  });

  assert.equal(snapshot.status, "empty");
  assert.deepEqual(snapshot.providers, []);
  assert.deepEqual(snapshot.authBindings, []);
});

test("fetchProviders falls back to aggregate provider inventory when split reads are unavailable", async (t) => {
  const original = {
    getSystemProviderInventory: workbenchApi.getSystemProviderInventory,
    getSystemRuntimeHosts: workbenchApi.getSystemRuntimeHosts,
    getSystemAuthBindings: workbenchApi.getSystemAuthBindings,
    getSystemExecutionBindings: workbenchApi.getSystemExecutionBindings,
    getSystemProviderDiscovery: workbenchApi.getSystemProviderDiscovery,
    getSystemProviders: workbenchApi.getSystemProviders,
  };

  useProvidersRegistry.setState({
    providers: [],
    runtimeHosts: [],
    authBindings: [],
    executionBindings: [],
    discoveryRecords: [],
    isLoading: false,
    status: "idle",
    error: null,
  });

  workbenchApi.getSystemProviderInventory = async () => {
    throw new Error("404 Not Found: split inventory unavailable");
  };
  workbenchApi.getSystemProviders = async () => ({
    schemaVersion: "1.0.0",
    generatedAt: "2026-03-25T00:00:00Z",
    providers: [
      {
        id: "fallback_provider",
        name: "Fallback Provider",
        providerType: "Llm",
        hostId: "host.local.primary",
        endpoint: "http://127.0.0.1:11434",
        isActive: true,
        priority: 1,
      },
    ],
    runtimeHosts: [],
    authBindings: [],
    executionBindings: [],
    discoveryRecords: [],
  });

  t.after(() => {
    workbenchApi.getSystemProviderInventory = original.getSystemProviderInventory;
    workbenchApi.getSystemRuntimeHosts = original.getSystemRuntimeHosts;
    workbenchApi.getSystemAuthBindings = original.getSystemAuthBindings;
    workbenchApi.getSystemExecutionBindings = original.getSystemExecutionBindings;
    workbenchApi.getSystemProviderDiscovery = original.getSystemProviderDiscovery;
    workbenchApi.getSystemProviders = original.getSystemProviders;
    useProvidersRegistry.setState({
      providers: [],
      runtimeHosts: [],
      authBindings: [],
      executionBindings: [],
      discoveryRecords: [],
      isLoading: false,
      status: "idle",
      error: null,
    });
  });

  await useProvidersRegistry.getState().fetchProviders();

  assert.equal(useProvidersRegistry.getState().status, "ready");
  assert.equal(useProvidersRegistry.getState().providers[0]?.id, "fallback_provider");
});

test("fetchProviders preserves 403 access denied from split reads", async (t) => {
  const original = {
    getSystemProviderInventory: workbenchApi.getSystemProviderInventory,
    getSystemProviders: workbenchApi.getSystemProviders,
  };

  useProvidersRegistry.setState({
    providers: [],
    runtimeHosts: [],
    authBindings: [],
    executionBindings: [],
    discoveryRecords: [],
    isLoading: false,
    status: "idle",
    error: null,
  });

  workbenchApi.getSystemProviderInventory = async () => {
    throw new Error("403 Forbidden: operator access required");
  };
  workbenchApi.getSystemProviders = async () => {
    throw new Error("aggregate should not be used after access denied");
  };

  t.after(() => {
    workbenchApi.getSystemProviderInventory = original.getSystemProviderInventory;
    workbenchApi.getSystemProviders = original.getSystemProviders;
    useProvidersRegistry.setState({
      providers: [],
      runtimeHosts: [],
      authBindings: [],
      executionBindings: [],
      discoveryRecords: [],
      isLoading: false,
      status: "idle",
      error: null,
    });
  });

  await useProvidersRegistry.getState().fetchProviders();

  assert.equal(useProvidersRegistry.getState().status, "access_denied");
  assert.match(useProvidersRegistry.getState().error ?? "", /^403 Forbidden/);
});
