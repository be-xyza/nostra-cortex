import type { Json, SystemProviderRuntimeStatusResponse } from "../../contracts.ts";

function hasLoadedUpstreamModels(payload: Json | undefined): boolean {
  if (!payload || typeof payload !== "object") {
    return false;
  }

  if (Array.isArray(payload)) {
    return payload.length > 0;
  }

  const record = payload as Record<string, unknown>;
  const data = record.data;
  return Array.isArray(data) && data.length > 0;
}

export function resolveAdapterStatusError(
  status: Pick<
    SystemProviderRuntimeStatusResponse,
    "adapterHealthError" | "openapiError" | "upstreamModels" | "upstreamModelsError"
  > | null,
): string | null {
  if (!status) {
    return null;
  }

  if (status.upstreamModelsError) {
    return status.upstreamModelsError;
  }

  if (hasLoadedUpstreamModels(status.upstreamModels)) {
    return null;
  }

  return humanizeProviderDiagnostic(status.openapiError ?? status.adapterHealthError ?? null);
}

export function humanizeProviderDiagnostic(message: string | null | undefined): string | null {
  const trimmed = message?.trim();
  if (!trimmed) {
    return null;
  }

  if (trimmed.includes("provider_probe_models_empty")) {
    return "The provider did not return any models from its catalog endpoint.";
  }
  if (trimmed.includes("openapi_parse_failed")) {
    return "OpenAPI metadata could not be parsed from the provider endpoint.";
  }
  if (trimmed.includes("health_parse_failed")) {
    return "The provider health response could not be parsed.";
  }
  if (trimmed.includes("models_http_401")) {
    return "The provider rejected model discovery. Check the credential or key scope.";
  }
  if (trimmed.includes("models_http_404")) {
    return "The provider does not expose a models catalog at the configured endpoint.";
  }

  return trimmed
    .replace(/^llm_adapter_/, "")
    .replace(/^provider_probe_/, "")
    .replace(/_/g, " ");
}
