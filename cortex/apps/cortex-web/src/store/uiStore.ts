import { create } from 'zustand';
import { readWindowRequestedSpaceId } from "../serviceWorker/requestScope.ts";

const ACTOR_ID_STORAGE_KEY = "cortex.shell.actor.id";
const ROLE_STORAGE_KEY = "cortex.shell.actor.role";
const ROLE_OPTIONS = ["viewer", "editor", "operator", "steward", "admin"] as const;
type CortexRole = typeof ROLE_OPTIONS[number];

export interface Comment {
    id: string;
    author: string;
    text: string;
    createdAt: string;
    authority?: {
        persistence: "local_ui_state";
        durableEvidence: false;
        governedHeapRecord: false;
        exportableAsEvidence: false;
        recommendedPersistenceTarget: "undecided";
    };
}

function normalizeRole(role: string): CortexRole | null {
    const normalized = role.trim().toLowerCase();
    return (ROLE_OPTIONS as readonly string[]).includes(normalized) ? (normalized as CortexRole) : null;
}

function readLocalStorage(key: string): string | null {
    if (typeof window === "undefined") return null;
    try {
        return window.localStorage.getItem(key);
    } catch {
        return null;
    }
}

function writeLocalStorage(key: string, value: string): void {
    if (typeof window === "undefined") return;
    try {
        window.localStorage.setItem(key, value);
    } catch {
        // ignore storage failures (private mode, quota, etc.)
    }
}

function defaultActorId(): string {
    const stored = readLocalStorage(ACTOR_ID_STORAGE_KEY);
    if (stored?.trim()) return stored.trim();
    const generated = `cortex-web-${Date.now()}-${Math.floor(Math.random() * 10000)}`;
    writeLocalStorage(ACTOR_ID_STORAGE_KEY, generated);
    return generated;
}

function defaultRole(): CortexRole {
    const stored = readLocalStorage(ROLE_STORAGE_KEY);
    const storedRole = stored ? normalizeRole(stored) : null;
    if (storedRole) return storedRole;
    const envRole =
        (((import.meta as unknown as { env?: Record<string, string | undefined> }).env?.VITE_ACTOR_ROLE as string | undefined) ?? "operator");
    return normalizeRole(envRole) ?? "operator";
}

function defaultSpaceId(): string {
    const requested = readWindowRequestedSpaceId();
    if (requested) {
        return requested;
    }
    const stored = readLocalStorage("cortex.shell.space.id");
    if (stored?.trim()) {
        const parts = stored.split(",");
        return parts[0].trim();
    }
    const env = ((import.meta as unknown as { env?: Record<string, string | undefined> }).env ?? {});
    const envSpace = (env.VITE_SPACE_ID as string | undefined)?.trim();
    if (envSpace) {
        return envSpace;
    }
    return "meta";
}

interface UiState {
    activeRoute: string | null;
    sessionUser: { actorId: string; role: string } | null;
    activeSpaceIds: string[]; // Support multi-space selection
    websocketStatus: 'DISCONNECTED' | 'CONNECTING' | 'CONNECTED';
    loadingFlags: Record<string, boolean>;
    comments: Record<string, Comment[]>;
    activeWorkbenchSession: { id: string, name: string } | null;
    wasWorkbenchNamedManually: boolean;
    namingModalOpen: boolean;
    pendingWorkbenchAction: (() => void) | null;
    setActiveRoute: (route: string) => void;
    setSessionUser: (user: { actorId: string; role: string }) => void;
    setActiveSpaceIds: (spaceIds: string[]) => void;
    setActiveWorkbenchSession: (session: { id: string, name: string } | null) => void;
    setWebsocketStatus: (status: 'DISCONNECTED' | 'CONNECTING' | 'CONNECTED') => void;
    setLoadingFlag: (key: string, isLoaded: boolean) => void;
    addComment: (blockId: string, comment: Comment) => void;
    setWorkbenchNamedManually: (named: boolean) => void;
    setNamingModalOpen: (open: boolean) => void;
    setPendingWorkbenchAction: (action: (() => void) | null) => void;
}

export const useUiStore = create<UiState>((set) => ({
    activeRoute: null,
    sessionUser: { actorId: defaultActorId(), role: defaultRole() },
    activeSpaceIds: [defaultSpaceId()],
    websocketStatus: 'DISCONNECTED',
    loadingFlags: {},
    comments: {},
    activeWorkbenchSession: null,
    wasWorkbenchNamedManually: false,
    namingModalOpen: false,
    pendingWorkbenchAction: null,
    setActiveRoute: (route) => set({ activeRoute: route }),
    setSessionUser: (user) => {
        if (user?.actorId?.trim()) {
            writeLocalStorage(ACTOR_ID_STORAGE_KEY, user.actorId.trim());
        }
        if (user?.role?.trim()) {
            const normalized = normalizeRole(user.role);
            if (normalized) {
                writeLocalStorage(ROLE_STORAGE_KEY, normalized);
            }
        }
        set({ sessionUser: user });
    },
    setActiveSpaceIds: (spaceIds) => {
        // Persist the first space for shell continuity if desired, or all as CSV
        writeLocalStorage("cortex.shell.space.id", spaceIds.join(","));
        set((state) => ({ 
            activeSpaceIds: spaceIds,
            // Reset naming flag if we go back to a single space
            wasWorkbenchNamedManually: spaceIds.length > 1 ? state.wasWorkbenchNamedManually : false
        }));
    },
    setActiveWorkbenchSession: (session) => {
        set({
            activeWorkbenchSession: session,
            wasWorkbenchNamedManually: !!session,
        });
    },
    setWorkbenchNamedManually: (named) => set({ wasWorkbenchNamedManually: named }),
    setNamingModalOpen: (open) => set({ namingModalOpen: open }),
    setPendingWorkbenchAction: (action) => set({ pendingWorkbenchAction: action }),
    setWebsocketStatus: (status) => set({ websocketStatus: status }),
    setLoadingFlag: (key, isLoading) => set((state) => ({
        loadingFlags: { ...state.loadingFlags, [key]: isLoading }
    })),
    addComment: (blockId, comment) => set((state) => ({
        comments: {
            ...state.comments,
            [blockId]: [...(state.comments[blockId] || []), comment]
        }
    })),
}));
