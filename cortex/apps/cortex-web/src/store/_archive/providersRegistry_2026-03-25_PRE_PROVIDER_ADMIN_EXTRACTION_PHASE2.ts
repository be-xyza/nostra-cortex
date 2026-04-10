import { create } from 'zustand';
import { workbenchApi } from '../api.ts';
import type {
  AuthBindingRecord,
  ExecutionBindingRecord,
  ProviderDiscoveryRecord,
  ProviderRecord,
  RuntimeHostRecord,
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
      const response = await workbenchApi.getSystemProviders();
      const isEmpty = response.providers.length === 0 && response.authBindings?.length === 0;
      set({
        providers: response.providers,
        runtimeHosts: response.runtimeHosts ?? [],
        authBindings: response.authBindings ?? [],
        executionBindings: response.executionBindings ?? [],
        discoveryRecords: response.discoveryRecords ?? [],
        status: isEmpty ? "empty" : "ready",
        error: null,
        isLoading: false,
      });
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
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
      const isEmpty = response.providers.length === 0 && response.authBindings?.length === 0;
      set({
        providers: response.providers,
        runtimeHosts: response.runtimeHosts ?? [],
        authBindings: response.authBindings ?? [],
        executionBindings: response.executionBindings ?? [],
        discoveryRecords: response.discoveryRecords ?? [],
        status: isEmpty ? "empty" : "ready",
        error: null,
        isLoading: false,
      });
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
