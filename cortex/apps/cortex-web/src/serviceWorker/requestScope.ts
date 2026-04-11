export function resolveRequestSpaceId(
  url: URL,
  fallbackSpaceId: string,
): string {
  const canonical =
    url.searchParams.get("space_id")?.trim() ||
    url.searchParams.get("spaceId")?.trim();
  return canonical || fallbackSpaceId;
}
