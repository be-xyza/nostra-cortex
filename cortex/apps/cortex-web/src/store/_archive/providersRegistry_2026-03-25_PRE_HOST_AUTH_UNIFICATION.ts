import { create } from 'zustand';
import { workbenchApi } from '../api.ts';
import type {
  AdapterBindingRecord,
  ProviderCredentialBindingRecord,
  ProviderDiscoveryRecord,
  ProviderRecord,
} from '../contracts.ts';

type ProviderRegistryStatus =
  | "idle"
  | "loading"
  | "discovering"
  | "ready"
  | "empty"
  | "booting"
  | "unavailable"
  | "error";

interface ProvidersRegistryState {
  providers: ProviderRecord[];
  credentialBindings: ProviderCredentialBindingRecord[];
  adapterBindings: AdapterBindingRecord[];
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

function classifyProviderRegistryError(message: string): ProviderRegistryStatus {
  const statusMatch = message.match(/^(\d{3})\s/);
  const statusCode = statusMatch ? Number.parseInt(statusMatch[1] ?? "", 10) : NaN;
  if (statusCode === 503 || statusCode === 502 || statusCode === 504) {
    return "booting";
  }
  if (statusCode === 404) {
    return "unavailable";
  }
  return "error";
}

export const useProvidersRegistry = create<ProvidersRegistryState>((set, get) => ({
  providers: [],
  credentialBindings: [],
  adapterBindings: [],
  discoveryRecords: [],
  isLoading: false,
  status: "idle",
  error: null,

  fetchProviders: async () => {
    set({ isLoading: true, status: "loading", error: null });
    try {
      const response = await workbenchApi.getSystemProviders();
      const isEmpty = response.providers.length === 0 && response.credentialBindings?.length === 0;
      set({
        providers: response.providers,
        credentialBindings: response.credentialBindings ?? [],
        adapterBindings: response.adapterBindings ?? [],
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
      const isEmpty = response.providers.length === 0 && response.credentialBindings?.length === 0;
      set({
        providers: response.providers,
        credentialBindings: response.credentialBindings ?? [],
        adapterBindings: response.adapterBindings ?? [],
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
