import { useState, useEffect, ReactNode, useMemo } from "react";
import { Link, useLocation } from "react-router-dom";
import { ShellLayoutSpec, NavigationEntrySpec, CompiledNavigationPlan } from "../../contracts";
import { workbenchApi } from "../../api";
import { useUiStore } from "../../store/uiStore";

interface ShellLayoutProps {
    children?: ReactNode;
}

type ShellNavMode = "expanded" | "rail" | "hidden";
const NAV_MODE_STORAGE_KEY = "cortex.shell.nav.mode";

function normalizeRouteId(routeId: string): string {
    if (!routeId) {
        return "/";
    }
    return routeId.startsWith("/") ? routeId : `/${routeId}`;
}

function resolveActiveRoute(pathname: string, entries: NavigationEntrySpec[]): string | null {
    let bestMatch: string | null = null;
    for (const entry of entries) {
        const routePath = normalizeRouteId(entry.routeId);
        const matchesRoute = routePath === "/"
            ? pathname === "/"
            : pathname === routePath || pathname.startsWith(`${routePath}/`);
        if (!matchesRoute) {
            continue;
        }
        if (!bestMatch || routePath.length > bestMatch.length) {
            bestMatch = routePath;
        }
    }
    return bestMatch;
}

export function ShellLayout({ children }: ShellLayoutProps) {
    const [layoutSpec, setLayoutSpec] = useState<ShellLayoutSpec | null>(null);
    const [compiledPlan, setCompiledPlan] = useState<CompiledNavigationPlan | null>(null);
    const [error, setError] = useState<string | null>(null);
    const dynamicNavEnabled =
        (((import.meta as unknown as { env?: Record<string, string | undefined> }).env?.VITE_DYNAMIC_NAV_ENABLED as string | undefined) ?? "true").toLowerCase() !== "false";
    const playgroundEnabled =
        (((import.meta as unknown as { env?: Record<string, string | undefined> }).env?.VITE_CHAT_HEAP_PLAYGROUND_ENABLED as string | undefined) ?? "true").toLowerCase() !== "false";
    const [navMode, setNavMode] = useState<ShellNavMode>("expanded");
    const [isMobile, setIsMobile] = useState(false);
    const location = useLocation();
    const setActiveRoute = useUiStore((state) => state.setActiveRoute);
    const activeSpaceId =
        (((import.meta as unknown as { env?: Record<string, string | undefined> }).env?.VITE_SPACE_ID as string | undefined) ?? "nostra-governance-v0").trim() || "nostra-governance-v0";
    const actorRole =
        (((import.meta as unknown as { env?: Record<string, string | undefined> }).env?.VITE_ACTOR_ROLE as string | undefined) ?? "operator").trim() || "operator";

    useEffect(() => {
        setActiveRoute(location.pathname);
    }, [location.pathname, setActiveRoute]);

    useEffect(() => {
        const load = async () => {
            try {
                const spec = await workbenchApi.getShellLayout();
                setLayoutSpec(spec);
                try {
                    const plan = await workbenchApi.getSpaceNavigationPlan(activeSpaceId, {
                        actorRole,
                        intent: "navigate",
                        density: "comfortable"
                    });
                    setCompiledPlan(plan);
                } catch {
                    setCompiledPlan(null);
                }
            } catch (err) {
                const message = err instanceof Error ? err.message : "unknown error";
                setError(message);
                setCompiledPlan(null);
            }
        };
        void load();
    }, [activeSpaceId, actorRole]);

    useEffect(() => {
        if (!dynamicNavEnabled) return;
        if (typeof window === "undefined") return;
        const readMode = (): ShellNavMode => {
            const stored = window.localStorage.getItem(NAV_MODE_STORAGE_KEY);
            if (stored === "expanded" || stored === "rail" || stored === "hidden") {
                return stored;
            }
            return window.innerWidth <= 1120 ? "rail" : "expanded";
        };
        setNavMode(readMode());
    }, [dynamicNavEnabled]);

    useEffect(() => {
        if (!dynamicNavEnabled) return;
        if (typeof window === "undefined") return;
        const media = window.matchMedia("(max-width: 980px)");
        const applyMobileState = () => {
            const mobile = media.matches;
            setIsMobile(mobile);
            if (mobile && navMode === "rail") {
                setNavMode("hidden");
            }
        };
        applyMobileState();
        media.addEventListener("change", applyMobileState);
        return () => media.removeEventListener("change", applyMobileState);
    }, [dynamicNavEnabled, navMode]);

    useEffect(() => {
        if (!dynamicNavEnabled) return;
        if (typeof window === "undefined") return;
        window.localStorage.setItem(NAV_MODE_STORAGE_KEY, navMode);
    }, [dynamicNavEnabled, navMode]);

    const entries = useMemo(() => {
        const baseEntries = (layoutSpec?.navigationGraph?.entries ?? []).filter((entry) => playgroundEnabled || entry.routeId !== "/playground");
        if (!compiledPlan?.entries?.length) {
            return baseEntries;
        }
        const rankByRoute = new Map<string, number>();
        for (const entry of compiledPlan.entries) {
            rankByRoute.set(normalizeRouteId(entry.routeId), entry.rank);
        }
        const filtered = baseEntries.filter((entry) => rankByRoute.has(normalizeRouteId(entry.routeId)));
        filtered.sort((left, right) => {
            const leftRank = rankByRoute.get(normalizeRouteId(left.routeId)) ?? Number.MAX_SAFE_INTEGER;
            const rightRank = rankByRoute.get(normalizeRouteId(right.routeId)) ?? Number.MAX_SAFE_INTEGER;
            if (leftRank !== rightRank) {
                return leftRank - rightRank;
            }
            return left.label.localeCompare(right.label);
        });
        return filtered;
    }, [layoutSpec, playgroundEnabled, compiledPlan]);
    const activeRoute = resolveActiveRoute(location.pathname, entries);
    const totalBadgeCount = useMemo(
        () => entries.reduce((sum, entry) => sum + (entry.navMeta?.badgeCount || 0), 0),
        [entries]
    );
    const attentionEntries = useMemo(
        () => entries.filter((entry) => entry.navMeta?.attention),
        [entries]
    );
    const navVisible = !dynamicNavEnabled || (isMobile ? navMode !== "hidden" : true);
    const showRail = dynamicNavEnabled && !isMobile && navMode === "rail";

    if (error) {
        return <div className="error-banner">Failed to load shell layout: {error}</div>;
    }

    if (!layoutSpec) {
        return <div className="placeholder">Loading Cortex Shell...</div>;
    }

    const toggleNav = () => {
        if (!dynamicNavEnabled) return;
        if (isMobile) {
            setNavMode((mode) => (mode === "hidden" ? "expanded" : "hidden"));
            return;
        }
        setNavMode((mode) => (mode === "expanded" ? "rail" : "expanded"));
    };

    return (
        <div className="shell-layout">
            {isMobile && navVisible && (
                <button
                    className="shell-sidebar-overlay"
                    aria-label="Close navigation panel"
                    onClick={() => setNavMode("hidden")}
                />
            )}
            <aside
                className={`shell-sidebar ${showRail ? "shell-sidebar--rail" : ""} ${isMobile ? "shell-sidebar--mobile" : ""} ${navVisible ? "shell-sidebar--visible" : "shell-sidebar--hidden"}`}
                aria-label="Global navigation"
            >
                <div className="shell-brand">
                    <button
                        className="shell-brand__toggle"
                        aria-label={navVisible ? "Collapse navigation" : "Expand navigation"}
                        onClick={toggleNav}
                    >
                        {isMobile ? (navVisible ? "✕" : "☰") : (showRail ? "☰" : "◧")}
                    </button>
                    {!showRail && (
                        <div className="shell-brand__title-group">
                            <h2>Nostra Cortex</h2>
                            <div className="shell-brand__meta">
                                {attentionEntries.length > 0 && (
                                    <span className="shell-brand__attention" title={attentionEntries.map((entry) => entry.navMeta?.attentionLabel || entry.label).join(", ")}>
                                        {attentionEntries.length} attention
                                    </span>
                                )}
                                <span className="shell-brand__badge" title="Aggregated route badge count">
                                    {totalBadgeCount}
                                </span>
                            </div>
                        </div>
                    )}
                </div>
                <nav className="shell-nav flex flex-col gap-1">
                    {entries.map((entry: NavigationEntrySpec) => {
                        const routePath = normalizeRouteId(entry.routeId);
                        const isActive = activeRoute === routePath;
                        const badgeCount = entry.navMeta?.badgeCount ?? 0;
                        const badgeTone = entry.navMeta?.badgeTone || "default";
                        const label = entry.label;
                        return (
                            <Link
                                key={entry.routeId}
                                to={routePath}
                                className={`nav-item flex items-center gap-2 p-2 rounded no-underline text-cortex-ink-muted hover:bg-cortex-bg-elev ${isActive ? "active text-cortex-ink bg-cortex-bg-elev" : ""}`}
                                title={label}
                                aria-label={label}
                                onClick={() => {
                                    if (isMobile) setNavMode("hidden");
                                }}
                            >
                                <span className="nav-icon w-6 text-center">{entry.icon || "•"}</span>
                                {!showRail && <span className="nav-label">{label}</span>}
                                {!showRail && badgeCount > 0 && (
                                    <span className={`nav-badge nav-badge--${badgeTone}`}>{badgeCount}</span>
                                )}
                                {!showRail && entry.navMeta?.attention && <span className="nav-attention-dot" aria-hidden="true" />}
                            </Link>
                        );
                    })}
                </nav>
            </aside>
            <main className="shell-main">
                {dynamicNavEnabled && isMobile && !navVisible && (
                    <button
                        className="shell-main__menu-btn"
                        onClick={() => setNavMode("expanded")}
                        aria-label="Open navigation menu"
                    >
                        ☰ Menu
                    </button>
                )}
                {children}
            </main>
        </div>
    );
}
