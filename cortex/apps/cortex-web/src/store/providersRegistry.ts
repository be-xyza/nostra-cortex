import { create } from 'zustand';
import { workbenchApi } from '../api.ts';
import type { ProviderRecord } from '../contracts.ts';

interface ProvidersRegistryState {
  providers: ProviderRecord[];
  isLoading: boolean;
  error: string | null;
  fetchProviders: () => Promise<void>;
  getProvidersByType: (type: ProviderRecord['providerType']) => ProviderRecord[];
  getLlmProviders: () => ProviderRecord[];
}

export const useProvidersRegistry = create<ProvidersRegistryState>((set, get) => ({
  providers: [],
  isLoading: false,
  error: null,

  fetchProviders: async () => {
    set({ isLoading: true, error: null });
    try {
      const response = await workbenchApi.getSystemProviders();
      set({ providers: response.providers, isLoading: false });
    } catch (err) {
      set({ error: err instanceof Error ? err.message : String(err), isLoading: false });
    }
  },

  getProvidersByType: (type) => {
    return get().providers.filter((p) => p.providerType === type);
  },

  getLlmProviders: () => {
    return get().providers.filter((p) => p.providerType === 'Llm');
  },
}));
