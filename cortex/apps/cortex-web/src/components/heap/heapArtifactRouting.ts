export const HEAP_ARTIFACT_QUERY_KEY = "artifact_id";

export function buildHeapArtifactHref(artifactId: string, spaceId?: string | null): string {
  const normalized = artifactId.trim();
  if (!normalized) {
    return "/explore";
  }
  const params = new URLSearchParams();
  if (spaceId?.trim()) {
    params.set("space_id", spaceId.trim());
  }
  params.set(HEAP_ARTIFACT_QUERY_KEY, normalized);
  return `/explore?${params.toString()}`;
}

export function readHeapArtifactIdFromSearchParams(
  searchParams: URLSearchParams,
): string | null {
  const value = searchParams.get(HEAP_ARTIFACT_QUERY_KEY)?.trim();
  return value ? value : null;
}
