export type ProviderCatalogRefreshMode = "draft_auth" | "stored_auth" | "anonymous" | "unavailable";

export interface ProviderCatalogState {
  refreshMode: ProviderCatalogRefreshMode;
  canRefresh: boolean;
  helperText: string;
}

export function buildProviderCatalogState(input: {
  providerId?: string;
  hasStoredAuth: boolean;
  draftApiKey: string;
  catalogSize: number;
  allowsAnonymousDiscovery?: boolean;
}): ProviderCatalogState {
  const draftApiKey = input.draftApiKey.trim();
  if (draftApiKey) {
    return {
      refreshMode: "draft_auth",
      canRefresh: true,
      helperText:
        input.catalogSize > 0
          ? `${input.catalogSize} models are available from the latest catalog refresh.`
          : "Use the entered auth secret to refresh the provider catalog before choosing a default model.",
    };
  }

  if (input.hasStoredAuth && input.providerId) {
    return {
      refreshMode: "stored_auth",
      canRefresh: true,
      helperText:
        input.catalogSize > 0
          ? `${input.catalogSize} models are available from the saved provider catalog.`
          : "Refresh the provider catalog with the saved auth binding for this provider.",
    };
  }

  if (input.allowsAnonymousDiscovery) {
    return {
      refreshMode: "anonymous",
      canRefresh: true,
      helperText:
        input.catalogSize > 0
          ? `${input.catalogSize} models are available from the latest local runtime catalog.`
          : "Refresh the provider catalog directly from the local runtime without any auth binding.",
    };
  }

  return {
    refreshMode: "unavailable",
    canRefresh: false,
    helperText:
      input.catalogSize > 0
        ? `${input.catalogSize} models are available from the saved provider catalog.`
        : "Add or paste an auth secret to refresh this provider catalog.",
  };
}
