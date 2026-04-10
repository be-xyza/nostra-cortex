export const PREVIEW_FIXTURES_COOKIE = "cortex.preview.fixtures";
export const USER_PREFERENCES_STORAGE_KEY = "cortex.global.preferences";
const PREVIEW_FIXTURES_COOKIE_MAX_AGE = 60 * 60 * 24 * 365;

export function normalizeRegistryMode(value?: string | null): "preview" | "live" {
    return value?.trim().toLowerCase() === "preview" ? "preview" : "live";
}

export function isPreviewFixturesEnabledMode(value?: string | null): boolean {
    return normalizeRegistryMode(value) === "preview";
}

export function readPreviewFixturesCookie(cookieHeader?: string | null): boolean {
    if (!cookieHeader) {
        return false;
    }

    return cookieHeader.split(";").some((pair) => {
        const [name, rawValue] = pair.split("=");
        if (name?.trim() !== PREVIEW_FIXTURES_COOKIE) {
            return false;
        }
        const value = rawValue?.trim().toLowerCase();
        return value === "1" || value === "true" || value === "preview";
    });
}

export function syncPreviewFixturesCookie(enabled: boolean): void {
    if (typeof document === "undefined") {
        return;
    }

    const maxAge = enabled ? PREVIEW_FIXTURES_COOKIE_MAX_AGE : 0;
    document.cookie = `${PREVIEW_FIXTURES_COOKIE}=${enabled ? "1" : "0"}; Path=/; Max-Age=${maxAge}; SameSite=Lax`;
}

export function syncPreviewFixturesState(enabled: boolean): void {
    syncPreviewFixturesCookie(enabled);

    if (typeof navigator === "undefined" || !("serviceWorker" in navigator)) {
        return;
    }

    const message = { type: "cortex.preview-fixtures", enabled };
    navigator.serviceWorker.controller?.postMessage(message);

    void navigator.serviceWorker.ready
        .then((registration) => {
            registration.active?.postMessage(message);
        })
        .catch(() => {
            // Ignore service worker readiness failures.
        });
}

export function readPreviewFixturesEnabledFromStorage(): boolean {
    if (typeof window === "undefined") {
        return false;
    }

    try {
        const raw = window.localStorage.getItem(USER_PREFERENCES_STORAGE_KEY);
        if (!raw) {
            return false;
        }
        const parsed = JSON.parse(raw) as { registryMode?: string | null };
        return isPreviewFixturesEnabledMode(parsed.registryMode);
    } catch {
        return false;
    }
}
