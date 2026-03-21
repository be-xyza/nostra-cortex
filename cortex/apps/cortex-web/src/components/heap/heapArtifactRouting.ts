export const HEAP_ARTIFACT_QUERY_KEY = "artifact_id";

export function buildHeapArtifactHref(artifactId: string): string {
  const normalized = artifactId.trim();
  if (!normalized) {
    return "/explore";
  }
  return `/explore?${HEAP_ARTIFACT_QUERY_KEY}=${encodeURIComponent(normalized)}`;
}

export function readHeapArtifactIdFromSearchParams(
  searchParams: URLSearchParams,
): string | null {
  const value = searchParams.get(HEAP_ARTIFACT_QUERY_KEY)?.trim();
  return value ? value : null;
}
