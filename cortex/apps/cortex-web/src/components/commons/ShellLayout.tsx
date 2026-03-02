import { useState, useEffect, ReactNode } from "react";
import { Link, useLocation } from "react-router-dom";
import { ShellLayoutSpec, NavigationEntrySpec } from "../../contracts";
import { workbenchApi } from "../../api";
import { useUiStore } from "../../store/uiStore";

interface ShellLayoutProps {
    children?: ReactNode;
}

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
    const [error, setError] = useState<string | null>(null);
    const location = useLocation();
    const setActiveRoute = useUiStore((state) => state.setActiveRoute);

    useEffect(() => {
        setActiveRoute(location.pathname);
    }, [location.pathname, setActiveRoute]);

    useEffect(() => {
        workbenchApi.getShellLayout()
            .then(spec => {
                setLayoutSpec(spec);
            })
            .catch(err => setError(err.message));
    }, []);

    if (error) {
        return <div className="error-banner">Failed to load shell layout: {error}</div>;
    }

    if (!layoutSpec) {
        return <div className="placeholder">Loading Cortex Shell...</div>;
    }

    const entries = layoutSpec.navigationGraph?.entries ?? [];
    const activeRoute = resolveActiveRoute(location.pathname, entries);

    return (
        <div className="shell-layout">
            <aside className="shell-sidebar">
                <div className="shell-brand">
                    <h2>Nostra Cortex</h2>
                </div>
                <nav className="shell-nav flex flex-col gap-1">
                    {entries.map((entry: NavigationEntrySpec) => {
                        const routePath = normalizeRouteId(entry.routeId);
                        const isActive = activeRoute === routePath;
                        return (
                            <Link
                                key={entry.routeId}
                                to={routePath}
                                className={`nav-item flex items-center gap-2 p-2 rounded no-underline text-cortex-ink-muted hover:bg-cortex-bg-elev ${isActive ? "active text-cortex-ink bg-cortex-bg-elev" : ""}`}
                                title={entry.label}
                            >
                                <span className="nav-icon w-6 text-center">{entry.icon || "•"}</span>
                                <span className="nav-label">{entry.label}</span>
                            </Link>
                        );
                    })}
                </nav>
            </aside>
            <main className="shell-main">
                {children}
            </main>
        </div>
    );
}
