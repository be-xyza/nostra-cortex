export function resolveRequestSpaceId(
  url: URL,
  fallbackSpaceId: string,
): string {
  const canonical = resolveRequestedSpaceId(url.searchParams);
  return canonical || fallbackSpaceId;
}

export function resolveRequestedSpaceId(
  searchParams: URLSearchParams,
): string | null {
  return (
    searchParams.get("space_id")?.trim() ||
    searchParams.get("spaceId")?.trim() ||
    null
  );
}

export function resolveRequestedSpaceIdFromSearch(
  search: string,
): string | null {
  return resolveRequestedSpaceId(new URLSearchParams(search));
}

export function readWindowRequestedSpaceId(): string | null {
  if (typeof window === "undefined") {
    return null;
  }
  return resolveRequestedSpaceIdFromSearch(window.location.search);
}
