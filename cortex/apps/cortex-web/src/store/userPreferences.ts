import { create } from "zustand";

export type GraphVariant = "off" | "2d" | "3d";
export type MotionStyle = "static" | "drift" | "orbit";
export type RegistryMode = "auto" | "preview" | "live";

interface UserPreferences {
    ambientGraphVariant: GraphVariant;
    ambientGraphMotion: MotionStyle;
    reduceMotion: boolean;
    theme: string;
    registryMode: RegistryMode;
}

interface UserPreferencesState extends UserPreferences {
    setAmbientGraphVariant: (variant: GraphVariant) => void;
    setAmbientGraphMotion: (motion: MotionStyle) => void;
    setReduceMotion: (reduce: boolean) => void;
    setTheme: (theme: string) => void;
    setRegistryMode: (mode: RegistryMode) => void;
}

const STORAGE_KEY = "cortex.global.preferences";

const DEFAULT_PREFS: UserPreferences = {
    ambientGraphVariant: "off",
    ambientGraphMotion: "drift",
    reduceMotion: false,
    theme: "default",
    registryMode: "auto",
};

function readFromStorage(): UserPreferences {
    if (typeof window === "undefined") return DEFAULT_PREFS;
    try {
        const raw = window.localStorage.getItem(STORAGE_KEY);
        return raw ? { ...DEFAULT_PREFS, ...JSON.parse(raw) } : DEFAULT_PREFS;
    } catch {
        return DEFAULT_PREFS;
    }
}

function writeToStorage(prefs: UserPreferences): void {
    if (typeof window === "undefined") return;
    try {
        window.localStorage.setItem(STORAGE_KEY, JSON.stringify(prefs));
    } catch {
        // ignore storage failures
    }
}

export const useUserPreferences = create<UserPreferencesState>((set, get) => ({
    ...readFromStorage(),

    setAmbientGraphVariant: (variant) => {
        set({ ambientGraphVariant: variant });
        writeToStorage({ ...get() });
    },
    setAmbientGraphMotion: (motion) => {
        set({ ambientGraphMotion: motion });
        writeToStorage({ ...get() });
    },
    setReduceMotion: (reduce) => {
        set({ reduceMotion: reduce });
        writeToStorage({ ...get() });
    },
    setTheme: (theme) => {
        set({ theme: theme });
        writeToStorage({ ...get() });
    },
    setRegistryMode: (mode) => {
        set({ registryMode: mode });
        writeToStorage({ ...get() });
    },
}));

