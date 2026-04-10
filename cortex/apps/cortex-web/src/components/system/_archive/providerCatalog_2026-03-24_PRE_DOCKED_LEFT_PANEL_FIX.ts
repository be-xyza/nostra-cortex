export type ProviderCatalogRefreshMode = "draft_key" | "stored_credential" | "unavailable";

export interface ProviderCatalogState {
  refreshMode: ProviderCatalogRefreshMode;
  canRefresh: boolean;
  helperText: string;
}

export function buildProviderCatalogState(input: {
  providerId?: string;
  hasStoredCredential: boolean;
  draftApiKey: string;
  catalogSize: number;
}): ProviderCatalogState {
  const draftApiKey = input.draftApiKey.trim();
  if (draftApiKey) {
    return {
      refreshMode: "draft_key",
      canRefresh: true,
      helperText:
        input.catalogSize > 0
          ? `${input.catalogSize} models are available from the latest catalog refresh.`
          : "Use the entered key to refresh the provider catalog before choosing a default model.",
    };
  }

  if (input.hasStoredCredential && input.providerId) {
    return {
      refreshMode: "stored_credential",
      canRefresh: true,
      helperText:
        input.catalogSize > 0
          ? `${input.catalogSize} models are available from the saved provider catalog.`
          : "Refresh the provider catalog with the saved credential for this provider.",
    };
  }

  return {
    refreshMode: "unavailable",
    canRefresh: false,
    helperText:
      input.catalogSize > 0
        ? `${input.catalogSize} models are available from the saved provider catalog.`
        : "Add or paste a key to refresh this provider catalog.",
  };
}
