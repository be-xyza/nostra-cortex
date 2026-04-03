import { useEffect, useMemo, useState } from 'react';
import { workbenchApi } from '../api.ts';
import type {
    GraphPhysicsConfig,
    SpaceReadinessStatus,
    SpaceRegistryRecord,
    SpaceSourceMode,
} from '../contracts.ts';
import { useUiStore } from './uiStore.ts';
import { useUserPreferences } from './userPreferences.ts';

const INTRO_SPACE_ID = '01ARZ3NDEKTSV4RRFFQ69G5FAV';
const LIVE_SPACES_CACHE_KEY = 'cortex.liveSpaces.v1';
export type SpaceRegistryMode = 'auto' | 'preview' | 'live';
export type SpaceSourceBucketKey = 'registered' | 'observed' | 'preview' | 'draft';

/**
 * Default "Branching Universe" physics configuration.
 * Principally aligned with Neo4j-style visualization:
 * - Strong repulsion (charge) spreads nodes wide.
 * - Explicit link distance ensures branches have room.
 * - Minimal center gravity allows expansion.
 */
export const DEFAULT_GRAPH_PHYSICS: GraphPhysicsConfig = {
    repulsionStrength: -1200,
    linkDistance: 120,
    centerGravity: 0.005,
};

export interface SpaceStats {
    objectCount: number;
    growthPercentage: number;
    memberCount: number;
}

export interface SpaceConfig {
    actions: ('details' | 'copy_id' | 'archive' | 'explore' | 'settings')[];
    enforcement: 'audit' | 'flexible' | 'strict';
}

export interface Space {
    id: string;
    name: string;
    type: 'global' | 'user' | 'system';
    icon?: string;
    description?: string;
    owner?: string;
    status?: string;
    createdAt?: string;
    members?: string[];
    archetype?: string;
    sourceMode?: SpaceSourceMode | 'preview' | 'draft';
    readinessSummary?: SpaceReadinessStatus;
    readiness?: {
        registry: SpaceReadinessStatus;
        navigationPlan: SpaceReadinessStatus;
        agentRuns: SpaceReadinessStatus;
        contributionGraphArtifact: SpaceReadinessStatus;
        contributionGraphRuns: SpaceReadinessStatus;
        capabilityGraph: SpaceReadinessStatus;
        summary: SpaceReadinessStatus;
    };
    stats?: SpaceStats;
    config?: SpaceConfig;
    metadata?: {
        lineage?: {
            draftId?: string;
            sourceMode?: string;
            note?: string;
        };
        governance?: {
            scope?: "personal" | "private" | "public";
            visibilityState?: "owner_only" | "members_only" | "discoverable";
        };
        theme?: {
            ambientGraphVariant?: 'off' | '2d' | '3d';
            ambientGraphMotion?: 'static' | 'drift' | 'orbit';
            graphPhysics?: GraphPhysicsConfig;
        }
    };
}

const META_SPACE: Space = {
    id: 'meta',
    name: 'Platform Overview',
    type: 'global',
    archetype: 'Meta',
    icon: 'globe',
    sourceMode: 'registered',
    readinessSummary: 'pass',
    readiness: {
        registry: 'pass',
        navigationPlan: 'pass',
        agentRuns: 'pass',
        contributionGraphArtifact: 'in_progress',
        contributionGraphRuns: 'in_progress',
        capabilityGraph: 'pass',
        summary: 'pass',
    },
    stats: { objectCount: 24500, growthPercentage: 8, memberCount: 12 },
    config: { actions: ['details', 'settings', 'archive'], enforcement: 'flexible' }
};

export const PREVIEW_SPACES: Space[] = [
    {
        id: INTRO_SPACE_ID,
        name: 'Nostra Intro Demo',
        type: 'user',
        archetype: 'Intro',
        icon: 'star',
        description: 'Demo Space for local preview content',
        sourceMode: 'preview',
        readinessSummary: 'in_progress',
        readiness: {
            registry: 'in_progress',
            navigationPlan: 'in_progress',
            agentRuns: 'in_progress',
            contributionGraphArtifact: 'in_progress',
            contributionGraphRuns: 'in_progress',
            capabilityGraph: 'in_progress',
            summary: 'in_progress',
        },
        stats: { objectCount: 1240, growthPercentage: 12, memberCount: 45 },
        config: { actions: ['details', 'copy_id', 'explore'], enforcement: 'audit' }
    },
    {
        id: 'nostra-governance-v0',
        name: 'Governance Demo',
        type: 'system',
        archetype: 'Governance',
        icon: 'shield',
        description: 'Demo Space for governance preview flows',
        sourceMode: 'preview',
        readinessSummary: 'in_progress',
        readiness: {
            registry: 'in_progress',
            navigationPlan: 'in_progress',
            agentRuns: 'in_progress',
            contributionGraphArtifact: 'in_progress',
            contributionGraphRuns: 'in_progress',
            capabilityGraph: 'in_progress',
            summary: 'in_progress',
        },
        stats: { objectCount: 850, growthPercentage: 3, memberCount: 150 },
        config: { actions: ['details', 'copy_id'], enforcement: 'strict' }
    },
    {
        id: 'system',
        name: 'System Demo',
        type: 'system',
        archetype: 'System',
        icon: 'settings',
        description: 'Demo Space for system preview content',
        sourceMode: 'preview',
        readinessSummary: 'in_progress',
        readiness: {
            registry: 'in_progress',
            navigationPlan: 'in_progress',
            agentRuns: 'in_progress',
            contributionGraphArtifact: 'in_progress',
            contributionGraphRuns: 'in_progress',
            capabilityGraph: 'in_progress',
            summary: 'in_progress',
        },
        stats: { objectCount: 15400, growthPercentage: 1, memberCount: 3 },
        config: { actions: ['details', 'settings'], enforcement: 'strict' }
    },
    {
        id: 'research',
        name: 'Research Demo',
        type: 'user',
        archetype: 'Research',
        icon: 'flask',
        description: 'Demo Space for research preview content',
        sourceMode: 'preview',
        readinessSummary: 'in_progress',
        readiness: {
            registry: 'in_progress',
            navigationPlan: 'in_progress',
            agentRuns: 'in_progress',
            contributionGraphArtifact: 'in_progress',
            contributionGraphRuns: 'in_progress',
            capabilityGraph: 'in_progress',
            summary: 'in_progress',
        },
        stats: { objectCount: 520, growthPercentage: 24, memberCount: 8 },
        config: { actions: ['details', 'copy_id', 'explore', 'archive'], enforcement: 'flexible' }
    },
    {
        id: 'default',
        name: 'Default Demo',
        type: 'user',
        archetype: 'General',
        icon: 'box',
        description: 'Demo Space for default preview content',
        sourceMode: 'preview',
        readinessSummary: 'in_progress',
        readiness: {
            registry: 'in_progress',
            navigationPlan: 'in_progress',
            agentRuns: 'in_progress',
            contributionGraphArtifact: 'in_progress',
            contributionGraphRuns: 'in_progress',
            capabilityGraph: 'in_progress',
            summary: 'in_progress',
        },
        stats: { objectCount: 87, growthPercentage: 0, memberCount: 1 },
        config: { actions: ['details', 'copy_id'], enforcement: 'audit' }
    },
];

export function resolveSpaceRegistryMode(value?: string): SpaceRegistryMode {
    const normalized = value?.trim().toLowerCase();
    if (normalized === 'preview' || normalized === 'live') {
        return normalized;
    }
    return 'auto';
}

export function getRegistryFallbackSpaces(mode: SpaceRegistryMode): Space[] {
    if (mode === 'live') {
        return [META_SPACE];
    }
    if (mode === 'preview' || mode === 'auto') {
        return [META_SPACE, ...PREVIEW_SPACES];
    }
    return [META_SPACE];
}

export function getRegistryBootstrapSpaces(mode: SpaceRegistryMode): Space[] {
    if (mode === 'preview') {
        return getRegistryFallbackSpaces(mode);
    }
    return [META_SPACE];
}

function normalizeCachedSpaces(value: unknown): Space[] {
    if (!Array.isArray(value)) {
        return [];
    }
    return value
        .filter((entry): entry is Space => Boolean(entry) && typeof entry === 'object' && typeof (entry as Space).id === 'string')
        .filter((space) => space.id !== 'meta')
        .filter((space) => space.sourceMode !== 'preview');
}

function readCachedLiveSpaces(): Space[] {
    if (typeof window === 'undefined') {
        return [];
    }
    try {
        const raw = window.localStorage.getItem(LIVE_SPACES_CACHE_KEY);
        if (!raw) {
            return [];
        }
        return normalizeCachedSpaces(JSON.parse(raw));
    } catch {
        return [];
    }
}

function writeCachedLiveSpaces(spaces: Space[]): void {
    if (typeof window === 'undefined') {
        return;
    }
    try {
        const serializable = spaces.filter((space) => space.id !== 'meta' && space.sourceMode !== 'preview');
        window.localStorage.setItem(LIVE_SPACES_CACHE_KEY, JSON.stringify(serializable));
    } catch {
        // Best-effort cache only.
    }
}

export function resolveRegistryFailureSpaces(
    registryMode: SpaceRegistryMode,
    currentSpaces: Space[],
    cachedLiveSpaces: Space[] = [],
): Space[] {
    const currentLiveSpaces = currentSpaces.filter((space) => space.id !== 'meta' && space.sourceMode !== 'preview');
    if (currentLiveSpaces.length > 0) {
        return [META_SPACE, ...currentLiveSpaces];
    }
    if (cachedLiveSpaces.length > 0) {
        return [META_SPACE, ...cachedLiveSpaces];
    }
    if (registryMode !== 'live') {
        return getRegistryFallbackSpaces(registryMode);
    }
    return [META_SPACE];
}

export function resolveCanonicalActiveSpaceIds(
    activeSpaceIds: string[],
    availableSpaces: Pick<Space, 'id'>[],
    options?: { deferInvalidation?: boolean },
): string[] {
    const validIds = new Set(availableSpaces.map((space) => space.id));
    const canonicalIds = activeSpaceIds.filter((spaceId, index) => {
        if (!validIds.has(spaceId)) {
            return false;
        }
        return activeSpaceIds.indexOf(spaceId) === index;
    });
    if (canonicalIds.length > 0) {
        return canonicalIds;
    }
    if (options?.deferInvalidation && activeSpaceIds.length > 0) {
        return activeSpaceIds;
    }
    const firstNonMetaSpace = availableSpaces.find((space) => space.id !== 'meta');
    return [firstNonMetaSpace?.id ?? 'meta'];
}

function isSystemOwnedSpace(record: SpaceRegistryRecord): boolean {
    const owner = record.owner.toLowerCase();
    return owner.startsWith('system') || owner.startsWith('agent:');
}

export function describeSpaceSourceMode(space: Pick<Space, 'sourceMode'>): string {
    switch (space.sourceMode) {
        case 'observed':
            return 'Observed Live Space';
        case 'preview':
            return 'Preview Space';
        case 'draft':
            return 'Draft Space';
        default:
            return 'Registered Space';
    }
}

export function describeSpaceReadiness(space: Pick<Space, 'readinessSummary'>): string {
    switch (space.readinessSummary) {
        case 'pass':
            return 'pass';
        case 'fail':
            return 'fail';
        case 'in_progress':
            return 'in progress';
        default:
            return 'unknown';
    }
}

export function partitionSpacesBySource(spaces: Space[]): Record<SpaceSourceBucketKey, Space[]> {
    return spaces.reduce<Record<SpaceSourceBucketKey, Space[]>>(
        (acc, space) => {
            const mode = space.sourceMode ?? 'registered';
            if (mode === 'observed' || mode === 'preview' || mode === 'draft') {
                acc[mode].push(space);
            } else {
                acc.registered.push(space);
            }
            return acc;
        },
        {
            registered: [],
            observed: [],
            preview: [],
            draft: [],
        },
    );
}

function compactSpaceSuffix(spaceId: string): string {
    return spaceId.slice(0, 8);
}

export function getSpaceDisplayName(record: SpaceRegistryRecord): string {
    const archetype = record.archetype?.trim();
    if (archetype) {
        return `${archetype} · ${compactSpaceSuffix(record.spaceId)}`;
    }
    return record.spaceId;
}

export function mapSpaceRegistryRecordToSpace(record: SpaceRegistryRecord): Space {
    const systemOwned = isSystemOwnedSpace(record);
    const type: Space['type'] = systemOwned ? 'system' : 'user';
    const active = record.status.toLowerCase() === 'active';
    const archetype = record.archetype?.trim() || (systemOwned ? 'System' : 'General');

    return {
        id: record.spaceId,
        name: getSpaceDisplayName(record),
        type,
        icon: systemOwned ? 'shield' : archetype?.toLowerCase() === 'research' ? 'flask' : 'box',
        description: archetype ? `${archetype} Space` : 'Registered Space',
        owner: record.owner,
        status: record.status,
        createdAt: record.createdAt,
        members: record.members,
        archetype: archetype || undefined,
        sourceMode: record.sourceMode ?? 'registered',
        readinessSummary: record.readinessSummary,
        readiness: record.readiness,
        stats: {
            objectCount: 0,
            growthPercentage: 0,
            memberCount: record.members.length,
        },
        config: {
            actions: active ? ['details', 'copy_id', 'explore'] : ['details', 'copy_id'],
            enforcement: systemOwned ? 'strict' : archetype?.toLowerCase() === 'research' ? 'flexible' : 'audit',
        },
        metadata: {
            lineage:
                record.draftId || record.draftSourceMode || record.lineageNote
                    ? {
                        draftId: record.draftId ?? undefined,
                        sourceMode: record.draftSourceMode ?? undefined,
                        note: record.lineageNote ?? undefined,
                    }
                    : undefined,
            governance:
                record.governanceScope || record.visibilityState
                    ? {
                        scope: record.governanceScope ?? undefined,
                        visibilityState: record.visibilityState ?? undefined,
                    }
                    : undefined,
        },
    };
}

/**
 * Registry of available spaces.
 * Fetches canonical space truth from the Cortex gateway and only shows preview fixtures when explicitly enabled.
 */
export function useAvailableSpaces() {
    return useAvailableSpacesState().spaces;
}

export function useSpaceRegistrySnapshot() {
    return useAvailableSpacesState();
}

function useAvailableSpacesState() {
    const storeRegistryMode = useUserPreferences((state) => state.registryMode);
    const registryMode = resolveSpaceRegistryMode(storeRegistryMode);
    const cachedLiveSpaces = registryMode === 'live' ? readCachedLiveSpaces() : [];

    const [spaces, setSpaces] = useState<Space[]>(() => {
        if (registryMode === 'preview') {
            return getRegistryBootstrapSpaces(registryMode);
        }
        if (registryMode === 'live' && cachedLiveSpaces.length > 0) {
            return [META_SPACE, ...cachedLiveSpaces];
        }
        return getRegistryBootstrapSpaces(registryMode);
    });
    const [registryResolved, setRegistryResolved] = useState<boolean>(registryMode === 'preview');
    const [registryDegraded, setRegistryDegraded] = useState<boolean>(false);

    useEffect(() => {
        let cancelled = false;

        if (registryMode === 'preview') {
            setSpaces(getRegistryBootstrapSpaces(registryMode));
            setRegistryResolved(true);
            setRegistryDegraded(false);
            return () => {
                cancelled = true;
            };
        }

        setRegistryResolved(false);
        setRegistryDegraded(false);

        workbenchApi.getSpaces()
            .then((response) => {
                if (cancelled) {
                    return;
                }
                const resolvedSpaces = [META_SPACE, ...response.items.map(mapSpaceRegistryRecordToSpace)];
                setSpaces(resolvedSpaces);
                setRegistryResolved(true);
                setRegistryDegraded(false);
                writeCachedLiveSpaces(resolvedSpaces);
            })
            .catch(() => {
                if (cancelled) {
                    return;
                }
        setSpaces((currentSpaces) =>
                    resolveRegistryFailureSpaces(registryMode, currentSpaces, readCachedLiveSpaces()),
                );
                setRegistryResolved(true);
                setRegistryDegraded(true);
            });

        return () => {
            cancelled = true;
        };
    }, [registryMode]);

    return { spaces, registryMode, registryResolved, registryDegraded };
}

export function useCanonicalActiveSpaces() {
    const { spaces, registryMode, registryResolved } = useSpaceRegistrySnapshot();
    const activeSpaceIds = useUiStore((state) => state.activeSpaceIds);
    const setActiveSpaceIds = useUiStore((state) => state.setActiveSpaceIds);

    useEffect(() => {
        const canonicalIds = resolveCanonicalActiveSpaceIds(activeSpaceIds, spaces, {
            deferInvalidation: !registryResolved && registryMode !== 'preview',
        });
        if (canonicalIds.length !== activeSpaceIds.length || canonicalIds.some((spaceId, index) => spaceId !== activeSpaceIds[index])) {
            setActiveSpaceIds(canonicalIds);
        }
    }, [activeSpaceIds, registryMode, registryResolved, setActiveSpaceIds, spaces]);
}

/**
 * Helper to resolve the primary space ID or "meta" context
 */
export function useActiveSpaceContext() {
    const activeSpaceIds = useUiStore((state) => state.activeSpaceIds);
    
    return useMemo(() => {
        if (activeSpaceIds.includes('meta') || activeSpaceIds.length === 0) {
            return 'meta';
        }
        // If multiple are selected but not meta, we return the first as primary,
        // but fetchers should use the whole array.
        return activeSpaceIds[0];
    }, [activeSpaceIds]);
}

export function useActiveSpaceRecord() {
    const spaces = useAvailableSpaces();
    const activeSpaceId = useActiveSpaceContext();

    return useMemo(() => {
        return spaces.find((space) => space.id === activeSpaceId);
    }, [spaces, activeSpaceId]);
}
