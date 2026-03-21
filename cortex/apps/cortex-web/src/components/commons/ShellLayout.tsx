import { useState, useEffect, ReactNode, useMemo } from "react";
import { Link, useLocation } from "react-router-dom";
import {
    LayoutGrid,
    Search,
    BrainCircuit,
    Activity,
    Settings,
    ShieldAlert,
    Menu,
    PanelLeftClose,
    PanelLeftOpen,
    ChevronDown,
    Mail,
    GitBranch,
    GitMerge,
    FileCode,
    Terminal,
    Inbox,
    Compass,
} from "lucide-react";
import { ShellLayoutSpec, NavigationEntrySpec, CompiledNavigationPlan, WhoAmIResponse } from "../../contracts";
import { gatewayBaseUrl, workbenchApi } from "../../api";
import { useUiStore } from "../../store/uiStore";
import { buildNavigationSections } from "./navSections";
import { LayoutChangeToast } from "./LayoutChangeToast";
import { useLayoutPreferences, type LayoutPreferences, EMPTY_PREFS } from "../../store/layoutPreferences.ts";
import { useActiveSpaceContext, useCanonicalActiveSpaces } from "../../store/spacesRegistry";
import {
    buildFallbackShellLayoutSpec,
    buildFallbackWhoami,
    formatShellBootstrapWarning,
} from "./shellBootstrapFallback.ts";
import { GripVertical } from "lucide-react";
import { WorkbenchNamingModal } from "./WorkbenchNamingModal";
import { SpaceSelector } from "./SpaceSelector";
import { RoleProfileSelector } from "./RoleProfileSelector";

interface ShellLayoutProps {
    children?: ReactNode;
}

// Map of route IDs or labels to Lucide icons
const ICON_MAP: Record<string, React.ReactNode> = {
    "/": <LayoutGrid className="w-5 h-5" />,
    "/explore": <Compass className="w-5 h-5" />,
    "/playground": <Activity className="w-5 h-5" />,
    "/spaces": <Search className="w-5 h-5" />,
    "/system": <ShieldAlert className="w-5 h-5" />,
    "/inbox": <Inbox className="w-5 h-5" />,
    "/workflows": <GitBranch className="w-5 h-5" />,
    "/contributions": <GitMerge className="w-5 h-5" />,
    "/artifacts": <FileCode className="w-5 h-5" />,
    "/logs": <Terminal className="w-5 h-5" />,
    "default": <LayoutGrid className="w-5 h-5" />,
};

function getIcon(entry: NavigationEntrySpec) {
    return ICON_MAP[entry.routeId] || ICON_MAP[entry.label.toLowerCase()] || ICON_MAP["default"];
}

type ShellNavMode = "expanded" | "rail" | "hidden";
const NAV_MODE_STORAGE_KEY = "cortex.shell.nav.mode";
const ROLE_OPTIONS = ["viewer", "editor", "operator", "steward", "admin"] as const;
type CortexRole = typeof ROLE_OPTIONS[number];

function normalizeRole(role: string): CortexRole | null {
    const normalized = role.trim().toLowerCase();
    return (ROLE_OPTIONS as readonly string[]).includes(normalized) ? (normalized as CortexRole) : null;
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
    useCanonicalActiveSpaces();
    const [layoutSpec, setLayoutSpec] = useState<ShellLayoutSpec | null>(null);
    const [compiledPlan, setCompiledPlan] = useState<CompiledNavigationPlan | null>(null);
    const [whoami, setWhoami] = useState<WhoAmIResponse | null>(null);
    const [whoamiError, setWhoamiError] = useState<string | null>(null);
    const [bootstrapWarning, setBootstrapWarning] = useState<string | null>(null);
    const dynamicNavEnabled =
        (((import.meta as unknown as { env?: Record<string, string | undefined> }).env?.VITE_DYNAMIC_NAV_ENABLED as string | undefined) ?? "true").toLowerCase() !== "false";
    const playgroundEnabled =
        (((import.meta as unknown as { env?: Record<string, string | undefined> }).env?.VITE_CHAT_HEAP_PLAYGROUND_ENABLED as string | undefined) ?? "true").toLowerCase() !== "false";
    const [navMode, setNavMode] = useState<ShellNavMode>("expanded");
    const [isMobile, setIsMobile] = useState(false);
    const location = useLocation();
    const setActiveRoute = useUiStore((state) => state.setActiveRoute);
    const sessionUser = useUiStore((state) => state.sessionUser);
    const setSessionUser = useUiStore((state) => state.setSessionUser);
    const activeSpaceIds = useUiStore((state) => state.activeSpaceIds);
    const setActiveSpaceIds = useUiStore((state) => state.setActiveSpaceIds);
    const namingModalOpen = useUiStore((state) => state.namingModalOpen);
    const setNamingModalOpen = useUiStore((state) => state.setNamingModalOpen);
    const pendingWorkbenchAction = useUiStore((state) => state.pendingWorkbenchAction);
    const setPendingWorkbenchAction = useUiStore((state) => state.setPendingWorkbenchAction);
    const setActiveWorkbenchSession = useUiStore((state) => state.setActiveWorkbenchSession);
    const activeWorkbenchSession = useUiStore((state) => state.activeWorkbenchSession);
    const wasWorkbenchNamedManually = useUiStore((state) => state.wasWorkbenchNamedManually);
    const activeSpaceId = useActiveSpaceContext();
    const actorRoleEnv =
        (((import.meta as unknown as { env?: Record<string, string | undefined> }).env?.VITE_ACTOR_ROLE as string | undefined) ?? "operator").trim() || "operator";
    const actorRole = sessionUser?.role?.trim() || actorRoleEnv;
    const actorId = sessionUser?.actorId?.trim() || "unknown";
    const configuredGatewayTarget = gatewayBaseUrl().trim() || "same-origin /api proxy";

    useEffect(() => {
        setActiveRoute(location.pathname);
    }, [location.pathname, setActiveRoute]);

    useEffect(() => {
        if (!sessionUser) return;
        setWhoamiError(null);
        void workbenchApi
            .getWhoami(actorRole, actorId)
            .then((payload) => setWhoami(payload))
            .catch((err) => {
                const message = err instanceof Error ? err.message : "unknown error";
                setWhoamiError(message);
                setWhoami(buildFallbackWhoami(actorId, actorRole));
            });
    }, [sessionUser, actorRole, actorId]);

    useEffect(() => {
        if (!sessionUser || !whoami) return;
        if (whoami.allowUnverifiedRoleHeader) return;
        if (sessionUser.role === whoami.effectiveRole) return;
        setSessionUser({ actorId: sessionUser.actorId, role: whoami.effectiveRole });
    }, [sessionUser, whoami, setSessionUser]);

    useEffect(() => {
        const load = async () => {
            try {
                const spec = await workbenchApi.getShellLayout();
                setLayoutSpec(spec);
                setBootstrapWarning(null);
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
                setLayoutSpec(buildFallbackShellLayoutSpec());
                setBootstrapWarning(
                    formatShellBootstrapWarning("layout", message, configuredGatewayTarget),
                );
                setCompiledPlan(null);
            }
        };
        void load();
    }, [activeSpaceId, actorRole, configuredGatewayTarget]);

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

    // Autohide listener for desktop: collapse to rail when clicking content
    useEffect(() => {
        if (!isMobile) {
            const handleContentClick = (e: MouseEvent) => {
                const mainArea = document.querySelector("main");
                if (mainArea?.contains(e.target as Node) && navMode === "expanded") {
                    setNavMode("rail");
                }
            };
            document.addEventListener("click", handleContentClick);
            return () => document.removeEventListener("click", handleContentClick);
        }
    }, [isMobile, navMode]);

    // Trigger initial load from localStorage on first access
    useEffect(() => {
        useLayoutPreferences.getState().getPreferences(activeSpaceId);
    }, [activeSpaceId]);

    const entries = useMemo(() => {
        const baseEntries = (layoutSpec?.navigationGraph?.entries ?? [])
            .filter((entry) => playgroundEnabled || entry.routeId !== "/playground")
            .filter((entry) => normalizeRouteId(entry.routeId) !== "/spaces");
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
    const layoutPrefs = useLayoutPreferences((s) => s.cache[activeSpaceId] ?? EMPTY_PREFS);
    const stageChange = useLayoutPreferences((s) => s.stageChange);
    const [previewLayoutPrefs, setPreviewLayoutPrefs] = useState<LayoutPreferences | null>(null);

    const navigationSections = useMemo(
        () => buildNavigationSections(entries, compiledPlan, previewLayoutPrefs || layoutPrefs),
        [entries, compiledPlan, layoutPrefs, previewLayoutPrefs]
    );

    // Nav drag-and-drop handlers
    const [draggedNavItem, setDraggedNavItem] = useState<string | null>(null);
    const [draggedSection, setDraggedSection] = useState<string | null>(null);

    const handleNavItemDragStart = (routeId: string, e: React.DragEvent) => {
        setDraggedNavItem(routeId);
        e.dataTransfer.effectAllowed = "move";
        e.dataTransfer.setData("text/plain", `item:${routeId}`);
        requestAnimationFrame(() => {
            (e.currentTarget as HTMLElement).style.opacity = "0.4";
        });
    };

    const handleNavItemDragOver = (targetRouteId: string, slot: string, e: React.DragEvent) => {
        e.preventDefault();
        e.stopPropagation();
        if (!draggedNavItem || draggedNavItem === targetRouteId) return;

        const section = navigationSections.find((s) => s.slot === slot);
        if (!section) return;

        const currentOrder = section.entries.map((entry) => entry.routeId);
        const fromIndex = currentOrder.indexOf(draggedNavItem);
        const toIndex = currentOrder.indexOf(targetRouteId);

        if (fromIndex === -1 || toIndex === -1) return;

        const newOrder = [...currentOrder];
        newOrder.splice(fromIndex, 1);
        newOrder.splice(toIndex, 0, draggedNavItem);

        setPreviewLayoutPrefs((prev) => ({
            ...(prev || layoutPrefs),
            navItems: {
                ...(prev?.navItems || layoutPrefs.navItems),
                [slot]: { itemOrder: newOrder, hidden: prev?.navItems?.[slot]?.hidden || layoutPrefs.navItems?.[slot]?.hidden || [] },
            },
        }));
    };

    const handleNavItemDrop = (targetRouteId: string, slot: string, e: React.DragEvent) => {
        e.preventDefault();
        e.stopPropagation();
        if (!previewLayoutPrefs) return;

        stageChange(activeSpaceId, () => previewLayoutPrefs);
        setPreviewLayoutPrefs(null);
        setDraggedNavItem(null);
    };

    const handleSectionDragStart = (slot: string, e: React.DragEvent) => {
        setDraggedSection(slot);
        e.dataTransfer.effectAllowed = "move";
        e.dataTransfer.setData("text/plain", `section:${slot}`);
    };

    const handleSectionDragOver = (targetSlot: string, e: React.DragEvent) => {
        e.preventDefault();
        e.stopPropagation();
        if (!draggedSection || draggedSection === targetSlot) return;

        const currentOrder = navigationSections.map((s) => s.slot);
        const fromIndex = currentOrder.indexOf(draggedSection);
        const toIndex = currentOrder.indexOf(targetSlot);

        if (fromIndex === -1 || toIndex === -1) return;

        const newOrder = [...currentOrder];
        newOrder.splice(fromIndex, 1);
        newOrder.splice(toIndex, 0, draggedSection);

        setPreviewLayoutPrefs((prev) => ({
            ...(prev || layoutPrefs),
            navSections: { itemOrder: newOrder, hidden: prev?.navSections?.hidden || layoutPrefs.navSections?.hidden || [] },
        }));
    };

    const handleSectionDrop = (targetSlot: string, e: React.DragEvent) => {
        e.preventDefault();
        e.stopPropagation();
        if (!previewLayoutPrefs) return;

        stageChange(activeSpaceId, () => previewLayoutPrefs);
        setPreviewLayoutPrefs(null);
        setDraggedSection(null);
    };

    const handleDragEnd = (e: React.DragEvent) => {
        (e.currentTarget as HTMLElement).style.opacity = "1";
        setDraggedNavItem(null);
        setDraggedSection(null);
        setPreviewLayoutPrefs(null);
    };

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
        <div className={`shell-layout ${activeSpaceIds.length > 1 ? "shell-layout--meta" : ""}`}>
            {activeSpaceIds.length > 1 && (
                <div className="fixed inset-x-0 top-0 h-[2px] bg-linear-to-r from-transparent via-blue-500/40 to-transparent z-50 animate-pulse" />
            )}
            {activeSpaceIds.length > 1 && activeWorkbenchSession && (
                <div className="fixed top-0 left-0 w-full h-full pointer-events-none opacity-5 ring-1 ring-inset ring-blue-500/20 z-10" />
            )}
            {isMobile && navVisible && (
                <button
                    className="shell-sidebar-overlay"
                    aria-label="Close navigation panel"
                    onClick={() => setNavMode("hidden")}
                />
            )}
            <aside
                className={`shell-sidebar ${showRail ? "shell-sidebar--rail scrollbar-hide" : "custom-scrollbar"} ${isMobile ? "shell-sidebar--mobile" : ""} ${navVisible ? "shell-sidebar--visible" : "shell-sidebar--hidden"}`}
                aria-label="Global navigation"
            >
                <div className="shell-brand flex flex-col gap-5">
                    <div className="flex items-center justify-between w-full">
                        {!showRail && (
                            <h2 className="text-xl font-black tracking-tighter text-cortex-50 flex items-center gap-2 select-none">
                                <span className="text-blue-500 text-2xl">◈</span>
                                <span>CORTEX</span>
                            </h2>
                        )}
                        <button
                            className={`shell-brand__toggle flex items-center justify-center hover:bg-cortex-800/40 transition-all rounded-lg bg-cortex-800/20 text-cortex-400 hover:text-cortex-100 ${showRail ? "w-10 h-10 mx-auto" : "w-8 h-8"}`}
                            aria-label={navVisible ? "Collapse navigation" : "Expand navigation"}
                            onClick={toggleNav}
                        >
                            {isMobile ? (navVisible ? "✕" : "☰") : (showRail ? <Menu className="w-5 h-5" /> : <PanelLeftClose className="w-5 h-5" />)}
                        </button>
                    </div>
                    {/* Selector Area (Persistent) */}
                    <div className="flex flex-col items-center gap-4 w-full px-2">
                        <SpaceSelector 
                            collapsed={showRail}
                            isCentered={true}
                        />
                        
                        <div className="relative flex items-center justify-center w-full">
                            <RoleProfileSelector 
                                whoami={whoami}
                                sessionUser={sessionUser}
                                setSessionUser={setSessionUser}
                                collapsed={showRail}
                                isCentered={true}
                            />
                            {!showRail && (
                                <div className="absolute right-1 flex items-center gap-1.5">
                                    {attentionEntries.length > 0 && (
                                        <span className="w-5 h-5 flex items-center justify-center rounded-full bg-red-500/10 text-red-400 text-[10px] font-black" title={attentionEntries.map((e) => e.navMeta?.attentionLabel || e.label).join(", ")}>
                                            {attentionEntries.length}
                                        </span>
                                    )}
                                    {totalBadgeCount > 0 && (
                                        <span className="px-2 py-1 rounded-lg bg-blue-500/10 text-blue-400 text-[10px] font-black">
                                            {totalBadgeCount}
                                        </span>
                                    )}
                                </div>
                            )}
                        </div>

                        {!showRail && activeSpaceIds.length > 1 && (
                            <div className="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-blue-500/5 border border-blue-500/10 animate-in fade-in duration-300">
                                <div className="w-1.5 h-1.5 rounded-full bg-blue-500 animate-pulse" />
                                <span className="text-[9px] font-black uppercase tracking-wider text-blue-400 leading-none">
                                    {wasWorkbenchNamedManually ? "Live Session" : "Drafting Workbench"}
                                </span>
                                {activeWorkbenchSession?.name && (
                                    <span className="text-[9px] font-medium text-slate-500 truncate italic leading-none">
                                        — {activeWorkbenchSession.name}
                                    </span>
                                )}
                            </div>
                        )}
                    </div>
                </div>
                <nav className="shell-nav flex flex-col gap-3 mt-2">
                    {(showRail ? [{ slot: "rail", label: "", entries }] : navigationSections).map((section) => (
                        <div
                            key={section.slot}
                            className={`flex flex-col gap-1 transition-all duration-300 ${draggedSection === section.slot ? "opacity-40" : ""}`}
                            onDragOver={(e) => handleSectionDragOver(section.slot, e)}
                            onDrop={(e) => handleSectionDrop(section.slot, e)}
                        >
                            {!showRail && section.label && (
                                <div
                                    className="px-3 pt-2 pb-1 text-[9px] font-bold uppercase tracking-[0.24em] text-cortex-ink-faint opacity-60 flex items-center gap-1 group cursor-grab active:cursor-grabbing"
                                    draggable
                                    onDragStart={(e) => handleSectionDragStart(section.slot, e)}
                                    onDragEnd={handleDragEnd}
                                >
                                    <GripVertical className="w-3 h-3 opacity-0 group-hover:opacity-60 transition-opacity shrink-0" />
                                    {section.label}
                                </div>
                            )}
                            {section.entries.map((entry: NavigationEntrySpec) => {
                                const routePath = normalizeRouteId(entry.routeId);
                                const isActive = activeRoute === routePath;
                                const badgeCount = entry.navMeta?.badgeCount ?? 0;
                                const badgeTone = entry.navMeta?.badgeTone || "default";
                                const label = entry.label;
                                const icon = getIcon(entry);
                                return (
                                    <div
                                        key={entry.routeId}
                                        className={`relative group transition-all duration-300 ${draggedNavItem === entry.routeId ? "opacity-40 scale-95" : ""}`}
                                        draggable
                                        onDragStart={(e) => handleNavItemDragStart(entry.routeId, e)}
                                        onDragOver={(e) => handleNavItemDragOver(entry.routeId, section.slot, e)}
                                        onDrop={(e) => handleNavItemDrop(entry.routeId, section.slot, e)}
                                        onDragEnd={handleDragEnd}
                                    >
                                        <Link
                                            to={routePath}
                                            className={`nav-item flex items-center transition-all duration-200 pointer-events-auto ${showRail ? "justify-center p-3" : "gap-3 py-2.5 px-3 rounded-lg"} ${isActive ? "active text-blue-400 bg-blue-500/10 border border-white/5 shadow-md" : "text-cortex-ink-faint hover:text-cortex-50 hover:bg-cortex-surface-panel"}`}
                                            title={label}
                                            aria-label={label}
                                            draggable={false}
                                            onClick={() => {
                                                if (isMobile) setNavMode("hidden");
                                            }}
                                        >
                                            {!showRail && <GripVertical className="w-3 h-3 opacity-0 group-hover:opacity-40 transition-opacity shrink-0 cursor-grab active:cursor-grabbing" />}
                                            <span className={`nav-icon shrink-0 flex items-center justify-center transition-transform duration-200 ${isActive ? "scale-110 opacity-100" : "opacity-70 group-hover:opacity-100"}`}>{icon}</span>
                                            {!showRail && <span className="nav-label text-[13px] font-medium tracking-wide">{label}</span>}
                                            {!showRail && badgeCount > 0 && (
                                                <span className={`nav-badge ml-auto text-[9px] px-1.5 py-0.5 rounded-full ${badgeTone === "critical" ? "bg-red-500/20 text-red-400 border border-red-500/30" : "bg-white/5 text-slate-400 border border-white/5"}`}>{badgeCount}</span>
                                            )}
                                            {!showRail && entry.navMeta?.attention && <span className="nav-attention-dot w-1.5 h-1.5 rounded-full bg-blue-500 shadow-[0_0_8px_rgba(59,130,246,0.5)]" aria-hidden="true" />}
                                        </Link>
                                    </div>
                                );
                            })}
                        </div>
                    ))}
                </nav>
            </aside>
            <main className={`shell-main ${location.pathname.startsWith("/system") || location.pathname.startsWith("/heap") || location.pathname.startsWith("/spaces") || location.pathname.startsWith("/explore") || location.pathname.startsWith("/inbox") || location.pathname.startsWith("/drafts") || location.pathname.startsWith("/activity") || location.pathname.startsWith("/pinned") || location.pathname.startsWith("/archive") ? "shell-main--full" : ""}`}>
                {dynamicNavEnabled && isMobile && !navVisible && (
                    <button
                        className="shell-main__menu-btn fixed top-4 left-4 z-40 bg-slate-900/80 backdrop-blur border border-white/5 p-2 rounded-lg text-slate-100 shadow-xl"
                        onClick={() => setNavMode("expanded")}
                        aria-label="Open navigation menu"
                    >
                        <Menu className="w-5 h-5" />
                    </button>
                )}
                {(bootstrapWarning || whoamiError) && (
                    <div className="mx-4 mt-4 rounded-2xl border border-amber-400/20 bg-amber-400/8 px-4 py-3 text-sm text-amber-100 shadow-[0_8px_32px_rgba(245,158,11,0.08)]">
                        <div className="font-medium">
                            Running in local preview fallback mode
                        </div>
                        <div className="mt-1 text-amber-100/80">
                            {bootstrapWarning || formatShellBootstrapWarning(
                                "identity",
                                whoamiError || "unknown identity error",
                                configuredGatewayTarget,
                            )}
                        </div>
                    </div>
                )}
                {children}
            </main>
            <LayoutChangeToast />
            <WorkbenchNamingModal
                isOpen={namingModalOpen}
                onClose={() => {
                    setNamingModalOpen(false);
                    setPendingWorkbenchAction(null);
                }}
                onConfirm={(name) => {
                    setActiveWorkbenchSession({ id: `wb-${Date.now()}`, name });
                    setNamingModalOpen(false);
                    if (pendingWorkbenchAction) {
                        pendingWorkbenchAction();
                    }
                }}
            />
        </div>
    );
}
