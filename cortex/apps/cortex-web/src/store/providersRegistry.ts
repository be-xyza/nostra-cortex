import { create } from 'zustand';
import { workbenchApi } from '../api.ts';
import type {
  AuthBindingRecord,
  AuthBindingInventoryResponse,
  ExecutionBindingRecord,
  ExecutionBindingStatusResponse,
  OperatorProviderInventoryResponse,
  ProviderDiscoveryInventoryResponse,
  ProviderDiscoveryRecord,
  ProviderRecord,
  RuntimeHostRecord,
  RuntimeHostInventoryResponse,
  SystemProvidersResponse,
} from '../contracts.ts';

type ProviderRegistryStatus =
  | "idle"
  | "loading"
  | "discovering"
  | "ready"
  | "empty"
  | "booting"
  | "unavailable"
  | "access_denied"
  | "error";

interface ProvidersRegistryState {
  providers: ProviderRecord[];
  runtimeHosts: RuntimeHostRecord[];
  authBindings: AuthBindingRecord[];
  executionBindings: ExecutionBindingRecord[];
  discoveryRecords: ProviderDiscoveryRecord[];
  isLoading: boolean;
  status: ProviderRegistryStatus;
  error: string | null;
  fetchProviders: () => Promise<void>;
  refreshProviders: () => Promise<void>;
  discoverLocalProviders: () => Promise<void>;
  getProvidersByType: (type: ProviderRecord['providerType']) => ProviderRecord[];
  getLlmProviders: () => ProviderRecord[];
}

export interface ProviderRegistrySnapshot {
  providers: ProviderRecord[];
  runtimeHosts: RuntimeHostRecord[];
  authBindings: AuthBindingRecord[];
  executionBindings: ExecutionBindingRecord[];
  discoveryRecords: ProviderDiscoveryRecord[];
  status: "ready" | "empty";
}

interface ProviderRegistrySplitReads {
  providerInventory: OperatorProviderInventoryResponse;
  runtimeHostInventory: RuntimeHostInventoryResponse;
  authBindingInventory: AuthBindingInventoryResponse;
  executionBindingStatus: ExecutionBindingStatusResponse;
  providerDiscoveryInventory: ProviderDiscoveryInventoryResponse;
}

class SplitReadCompatibilityFallbackError extends Error {
  constructor() {
    super("provider registry split reads are not available yet");
    this.name = "SplitReadCompatibilityFallbackError";
  }
}

export function classifyProviderRegistryError(message: string): ProviderRegistryStatus {
  const statusMatch = message.match(/^(\d{3})\s/);
  const statusCode = statusMatch ? Number.parseInt(statusMatch[1] ?? "", 10) : NaN;
  if (statusCode === 503 || statusCode === 502 || statusCode === 504) {
    return "booting";
  }
  if (statusCode === 403) {
    return "access_denied";
  }
  if (statusCode === 404) {
    return "unavailable";
  }
  return "error";
}

function extractStatusCode(message: string): number | null {
  const statusMatch = message.match(/^(\d{3})\s/);
  return statusMatch ? Number.parseInt(statusMatch[1] ?? "", 10) : null;
}

function isSplitReadCompatibilityError(message: string): boolean {
  return extractStatusCode(message) === 404;
}

function isAccessDeniedError(message: string): boolean {
  return extractStatusCode(message) === 403;
}

function errorMessage(err: unknown): string {
  return err instanceof Error ? err.message : String(err);
}

function buildProviderRegistrySnapshot(
  providers: ProviderRecord[],
  runtimeHosts: RuntimeHostRecord[],
  authBindings: AuthBindingRecord[],
  executionBindings: ExecutionBindingRecord[],
  discoveryRecords: ProviderDiscoveryRecord[],
): ProviderRegistrySnapshot {
  const isEmpty =
    providers.length === 0 &&
    runtimeHosts.length === 0 &&
    authBindings.length === 0 &&
    executionBindings.length === 0 &&
    discoveryRecords.length === 0;

  return {
    providers,
    runtimeHosts,
    authBindings,
    executionBindings,
    discoveryRecords,
    status: isEmpty ? "empty" : "ready",
  };
}

export function buildProviderRegistrySnapshotFromSplitReads(
  reads: ProviderRegistrySplitReads,
): ProviderRegistrySnapshot {
  return buildProviderRegistrySnapshot(
    reads.providerInventory.providers,
    reads.runtimeHostInventory.runtimeHosts,
    reads.authBindingInventory.authBindings,
    reads.executionBindingStatus.executionBindings,
    reads.providerDiscoveryInventory.discoveryRecords,
  );
}

function buildProviderRegistrySnapshotFromAggregate(
  response: SystemProvidersResponse,
): ProviderRegistrySnapshot {
  return buildProviderRegistrySnapshot(
    response.providers,
    response.runtimeHosts ?? [],
    response.authBindings ?? [],
    response.executionBindings ?? [],
    response.discoveryRecords ?? [],
  );
}

function applyProviderRegistrySnapshot(
  set: (partial: Partial<ProvidersRegistryState>) => void,
  snapshot: ProviderRegistrySnapshot,
): void {
  set({
    providers: snapshot.providers,
    runtimeHosts: snapshot.runtimeHosts,
    authBindings: snapshot.authBindings,
    executionBindings: snapshot.executionBindings,
    discoveryRecords: snapshot.discoveryRecords,
    status: snapshot.status,
    error: null,
    isLoading: false,
  });
}

async function readSplitProviderRegistrySnapshot(): Promise<ProviderRegistrySnapshot> {
  let providerInventory: OperatorProviderInventoryResponse;
  try {
    providerInventory = await workbenchApi.getSystemProviderInventory();
  } catch (err) {
    const message = errorMessage(err);
    if (isAccessDeniedError(message)) {
      throw err;
    }
    if (isSplitReadCompatibilityError(message)) {
      throw new SplitReadCompatibilityFallbackError();
    }
    throw err;
  }

  try {
    const [runtimeHostInventory, authBindingInventory, executionBindingStatus, providerDiscoveryInventory] =
      await Promise.all([
        workbenchApi.getSystemRuntimeHosts(),
        workbenchApi.getSystemAuthBindings(),
        workbenchApi.getSystemExecutionBindings(),
        workbenchApi.getSystemProviderDiscovery(),
      ]);

    return buildProviderRegistrySnapshotFromSplitReads({
      providerInventory,
      runtimeHostInventory,
      authBindingInventory,
      executionBindingStatus,
      providerDiscoveryInventory,
    });
  } catch (err) {
    const message = errorMessage(err);
    if (isAccessDeniedError(message)) {
      throw err;
    }
    if (isSplitReadCompatibilityError(message)) {
      throw new SplitReadCompatibilityFallbackError();
    }
    throw err;
  }
}

export function providerRegistryStatusCopy(
  status: ProviderRegistryStatus,
  error: string | null,
): { title: string; body: string } | null {
  switch (status) {
    case "booting":
      return {
        title: "Provider registry is still booting.",
        body: "Refresh after the local shell finishes starting up.",
      };
    case "unavailable":
      return {
        title: "Provider registry is unavailable.",
        body: error ? "The registry endpoint could not be reached." : "The registry endpoint is not available yet.",
      };
    case "access_denied":
      return {
        title: "Provider registry requires operator access.",
        body: "Switch to an operator session to inspect provider, runtime host, and auth topology.",
      };
    case "error":
      return {
        title: "Unable to load providers.",
        body: "Refresh to try again. If the local shell is starting, the provider registry may not be ready yet.",
      };
    default:
      return null;
  }
}

export const useProvidersRegistry = create<ProvidersRegistryState>((set, get) => ({
  providers: [],
  runtimeHosts: [],
  authBindings: [],
  executionBindings: [],
  discoveryRecords: [],
  isLoading: false,
  status: "idle",
  error: null,

  fetchProviders: async () => {
    set({ isLoading: true, status: "loading", error: null });
    try {
      const snapshot = await readSplitProviderRegistrySnapshot();
      applyProviderRegistrySnapshot(set, snapshot);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      if (err instanceof SplitReadCompatibilityFallbackError) {
        try {
          const response = await workbenchApi.getSystemProviders();
          applyProviderRegistrySnapshot(set, buildProviderRegistrySnapshotFromAggregate(response));
          return;
        } catch (aggregateErr) {
          const aggregateMessage = errorMessage(aggregateErr);
          set({
            error: aggregateMessage,
            status: classifyProviderRegistryError(aggregateMessage),
            isLoading: false,
          });
          return;
        }
      }
      set({
        error: message,
        status: classifyProviderRegistryError(message),
        isLoading: false,
      });
    }
  },

  refreshProviders: async () => {
    await get().fetchProviders();
  },

  discoverLocalProviders: async () => {
    set({ isLoading: true, status: "discovering", error: null });
    try {
      const response = await workbenchApi.discoverSystemProviders();
      applyProviderRegistrySnapshot(set, buildProviderRegistrySnapshotFromAggregate(response));
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      set({
        error: message,
        status: classifyProviderRegistryError(message),
        isLoading: false,
      });
    }
  },

  getProvidersByType: (type) => {
    return get().providers.filter((p) => p.providerType === type);
  },

  getLlmProviders: () => {
    return get().providers.filter((p) => p.providerType === 'Llm');
  },
}));
