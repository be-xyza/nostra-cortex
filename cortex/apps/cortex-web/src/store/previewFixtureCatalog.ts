import type { HeapBlockListItem, HeapDeletedListItem } from "../contracts.ts";
import { HEAP_BLOCK_CAPABILITY_INVENTORY_SNAPSHOT_ID } from "./heapBlockCapabilityInventoryContract.ts";
import { SHELL_SURFACE_INVENTORY_SNAPSHOT_ID } from "./shellSurfaceInventoryContract.ts";
import { SPACE_DESIGN_PROFILE_PREVIEW_SNAPSHOT_ID } from "./spaceDesignProfilePreviewContract.ts";
import { INTRO_SPACE_ID, SEED_EVENTS } from "./seedData.ts";

export const PREVIEW_SNAPSHOT_IDS = new Set([
    "system:whoami",
    "system:layout:spec",
    "system:navigation:mock",
    "system:ux:workbench",
    "system:ux:workbench:/labs",
    "system:ux:workbench:/labs/execution-canvas",
    "system:ux:workbench:/system",
    "system:ux:workbench:/spaces",
    "system:ux:workbench:/heap",
    "system:ux:workbench:/studio",
    HEAP_BLOCK_CAPABILITY_INVENTORY_SNAPSHOT_ID,
    SHELL_SURFACE_INVENTORY_SNAPSHOT_ID,
    SPACE_DESIGN_PROFILE_PREVIEW_SNAPSHOT_ID,
]);

export const PREVIEW_EVENT_IDS = new Set(SEED_EVENTS.map((event) => event.id));

export const PREVIEW_ARTIFACT_IDS = new Set(
    SEED_EVENTS
        .map((event) => event.payload.artifactId)
        .filter((artifactId): artifactId is string => typeof artifactId === "string" && artifactId.length > 0),
);

export function isPreviewSnapshotId(id: string): boolean {
    return PREVIEW_SNAPSHOT_IDS.has(id);
}

export function isPreviewArtifactId(id?: string | null): boolean {
    return typeof id === "string" && PREVIEW_ARTIFACT_IDS.has(id);
}

export function isPreviewEventRecord(event: {
    id?: string;
    spaceId?: string;
    payload?: Record<string, unknown>;
}): boolean {
    if (event.id && PREVIEW_EVENT_IDS.has(event.id)) {
        return true;
    }
    if (event.spaceId === INTRO_SPACE_ID && isPreviewArtifactId(asString(event.payload?.artifactId))) {
        return true;
    }
    return isPreviewArtifactId(asString(event.payload?.artifactId));
}

export function isPreviewHeapBlock(block: Pick<HeapBlockListItem, "projection">): boolean {
    return isPreviewArtifactId(block.projection.artifactId);
}

export function isPreviewDeletedBlock(item: Pick<HeapDeletedListItem, "artifactId">): boolean {
    return isPreviewArtifactId(item.artifactId);
}

export function filterPreviewHeapBlocks<T extends Pick<HeapBlockListItem, "projection">>(
    blocks: T[],
): T[] {
    return blocks.filter((block) => !isPreviewHeapBlock(block));
}

export function filterPreviewDeletedBlocks<T extends Pick<HeapDeletedListItem, "artifactId">>(
    items: T[],
): T[] {
    return items.filter((item) => !isPreviewDeletedBlock(item));
}

function asString(value: unknown): string | null {
    if (typeof value !== "string") {
        return null;
    }
    const normalized = value.trim();
    return normalized.length > 0 ? normalized : null;
}
