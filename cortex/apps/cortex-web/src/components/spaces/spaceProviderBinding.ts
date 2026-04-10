import type { SpaceRoutingRecord } from "../../contracts.ts";

export function parseAuthMetadataJson(metadataJson: string): Record<string, string> | undefined {
  const trimmed = metadataJson.trim();
  if (!trimmed || trimmed === "{}") {
    return undefined;
  }

  const parsed = JSON.parse(trimmed) as Record<string, unknown>;
  return Object.fromEntries(
    Object.entries(parsed).map(([key, value]) => [key, typeof value === "string" ? value : JSON.stringify(value)]),
  );
}

export function applyAuthBindingToSpaceRouting(
  routing: SpaceRoutingRecord,
  authBindingId: string,
  providerId?: string,
  defaultModel?: string,
  adapterSetRef?: string,
): SpaceRoutingRecord {
  return {
    ...routing,
    providerId: providerId?.trim() ? providerId.trim() : routing.providerId,
    authBindingId,
    defaultModel: defaultModel?.trim() ? defaultModel.trim() : routing.defaultModel,
    adapterSetRef: adapterSetRef?.trim() ? adapterSetRef.trim() : routing.adapterSetRef,
  };
}
