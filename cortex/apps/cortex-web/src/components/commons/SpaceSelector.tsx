import React, { useState, useRef, useEffect } from "react";
import { ChevronDown, Check, Globe, Layout, Shield, FlaskConical, Plus, Settings, X } from "lucide-react";
import { Link } from "react-router-dom";
import { useUiStore } from "../../store/uiStore";
import { partitionSpacesBySource, useSpaceRegistrySnapshot } from "../../store/spacesRegistry";
import { resolveSpaceSelectorTriggerState } from "./spaceSelectorPresentation";

interface SpaceSelectorProps {
    className?: string;
    collapsed?: boolean;
    isCentered?: boolean;
}

const SPACE_CONFIG: Record<string, { 
    icon: React.ReactNode, 
    color: string, 
    ringColor: string, 
    label: string 
}> = {
    meta: {
        icon: <Globe className="w-4 h-4" />,
        color: "text-blue-400",
        ringColor: "ring-blue-500/50",
        label: "Meta Workbench"
    },
    system: {
        icon: <Shield className="w-4 h-4" />,
        color: "text-amber-400",
        ringColor: "ring-amber-500/50",
        label: "System Space"
    },
    research: {
        icon: <FlaskConical className="w-4 h-4" />,
        color: "text-purple-400",
        ringColor: "ring-purple-500/50",
        label: "Research Lab"
    },
    default: {
        icon: <Layout className="w-4 h-4" />,
        color: "text-emerald-400",
        ringColor: "ring-emerald-500/50",
        label: "Standard Space"
    }
};

export const SpaceSelector: React.FC<SpaceSelectorProps> = ({ 
    className = "",
    collapsed = false,
    isCentered = false
}) => {
    const [isOpen, setIsOpen] = useState(false);
    const dropdownRef = useRef<HTMLDivElement>(null);
    const { spaces: availableSpaces, registryResolved, registryDegraded } = useSpaceRegistrySnapshot();
    const activeSpaceIds = useUiStore((state) => state.activeSpaceIds);
    const setActiveSpaceIds = useUiStore((state) => state.setActiveSpaceIds);
    const activeWorkbenchSession = useUiStore((state) => state.activeWorkbenchSession);

    useEffect(() => {
        const handleClickOutside = (event: MouseEvent) => {
            if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
                setIsOpen(false);
            }
        };
        document.addEventListener("mousedown", handleClickOutside);
        return () => document.removeEventListener("mousedown", handleClickOutside);
    }, []);

    const selectSpace = (id: string, e: React.MouseEvent) => {
        e.stopPropagation();
        setActiveSpaceIds([id]);
        setIsOpen(false);
    };

    const toggleSpace = (id: string, e: React.MouseEvent) => {
        e.stopPropagation();
        
        if (id === "meta") {
            setActiveSpaceIds(["meta"]);
            setIsOpen(false);
            return;
        }

        let newIds = [...activeSpaceIds.filter(sid => sid !== "meta")];
        if (newIds.includes(id)) {
            if (newIds.length > 1) {
                newIds = newIds.filter((sid) => sid !== id);
            }
        } else {
            newIds.push(id);
        }
        
        setActiveSpaceIds(newIds);
    };

    const isMeta = activeSpaceIds.includes("meta");
    const activeSpaces = availableSpaces.filter(s => activeSpaceIds.includes(s.id));
    const nonMetaSpaces = availableSpaces.filter((space) => space.id !== "meta");
    const groupedSpaces = partitionSpacesBySource(nonMetaSpaces);
    
    // Determine visual config for the trigger
    const primarySpaceId = isMeta ? "meta" : (activeSpaces[0]?.id || "default");
    const config = SPACE_CONFIG[primarySpaceId] || SPACE_CONFIG.default;
    const isMulti = activeSpaceIds.length > 1 && !isMeta;
    const triggerState = resolveSpaceSelectorTriggerState({
        isMeta,
        isMulti,
        activeSpaceCount: activeSpaceIds.length,
        activeSpaceName: activeSpaces[0]?.name,
        activeSpaceSourceMode: activeSpaces[0]?.sourceMode,
        activeSpaceReadiness: activeSpaces[0]?.readinessSummary,
        registryResolved,
    });

    return (
        <div className={`relative ${className}`} ref={dropdownRef}>
            <button
                onClick={() => setIsOpen(!isOpen)}
                className={`flex items-center transition-all duration-300 hover:bg-white/5 
                    ${collapsed ? "justify-center p-1 rounded-full w-10 h-10" : "gap-2.5 p-1 rounded-xl pr-3"}
                    ${isCentered ? "mx-auto" : ""}
                    ${isMulti || isMeta ? "shadow-[0_0_15px_rgba(59,130,246,0.15)]" : ""}`}
                aria-label="Select Space"
                aria-haspopup="listbox"
                aria-expanded={isOpen}
            >
                {/* Circular Space Avatar */}
                <div className={`relative w-8 h-8 shrink-0 flex items-center justify-center rounded-full bg-cortex-900 border border-white/10 ring-2 ${isMulti || isMeta ? "ring-blue-500/50" : config.ringColor} ring-offset-2 ring-offset-cortex-surface-base shadow-lg transition-transform duration-300 ${isOpen ? "scale-110" : ""}`}>
                    <div className={`${isMulti ? "text-blue-400" : config.color}`}>
                        {isMulti ? <Globe className="w-4 h-4 animate-pulse" /> : config.icon}
                    </div>
                </div>

                {!collapsed && (
                    <div className="flex flex-col items-start -translate-y-px text-left truncate">
                        <div className="flex items-center gap-1 w-full overflow-hidden">
                            <span className="text-[10px] font-black uppercase tracking-wider text-cortex-100 truncate">
                                {triggerState.title}
                            </span>
                            <ChevronDown className={`w-3 h-3 text-cortex-500 shrink-0 transition-transform ${isOpen ? "rotate-180" : ""}`} />
                        </div>
                        <span className="text-[8px] font-bold text-cortex-400 tracking-tight leading-none uppercase truncate">
                            {triggerState.subtitle}
                        </span>
                    </div>
                )}
            </button>

            {isOpen && (
                <div className="absolute top-full left-0 w-64 mt-2 bg-slate-950/98 backdrop-blur-2xl border border-white/10 rounded-xl shadow-[0_8px_32px_-8px_rgba(0,0,0,0.8)] p-1 z-50 animate-in fade-in zoom-in-95 duration-100">
                    <div className="text-[9px] font-black text-cortex-ink-faint px-3 py-2 uppercase tracking-widest border-b border-white/5 mb-1">
                        Switch Space
                    </div>
                    
                    <button
                        onClick={(e) => selectSpace("meta", e)}
                        className={`w-full flex items-center justify-between px-3 py-2 rounded-lg text-[11px] transition-all group ${isMeta ? "bg-blue-500/10 text-blue-400" : "text-cortex-400 hover:bg-white/5 hover:text-white"}`}
                    >
                        <div className="flex items-center gap-3">
                            <div className={`w-6 h-6 flex items-center justify-center rounded-full bg-cortex-900 border border-white/5 shadow-inner ${SPACE_CONFIG.meta.color}`}>
                                {SPACE_CONFIG.meta.icon}
                            </div>
                            <span className="font-bold uppercase tracking-wide">Meta Workbench</span>
                        </div>
                        {isMeta && <Check className="w-3.5 h-3.5 text-blue-500" />}
                    </button>

                    <div className="my-1.5 px-3">
                        <div className="h-px bg-white/5 w-full" />
                    </div>

                    <div className="text-[9px] font-black text-cortex-ink-faint px-3 py-1 uppercase tracking-widest mb-1">
                        Sovereign Spaces
                    </div>

                    <div className="max-h-[240px] overflow-y-auto custom-scrollbar space-y-0.5">
                        {registryDegraded && (
                            <div className="mx-3 mb-2 rounded-lg border border-amber-400/15 bg-amber-400/8 px-2.5 py-2 text-[9px] text-amber-100/80">
                                Gateway unavailable. Showing the last known live Space registry.
                            </div>
                        )}
                        {([
                            ["registered", "Registered Spaces"],
                            ["observed", "Observed Live Evidence"],
                            ["draft", "Draft Spaces"],
                            ["preview", "Preview Spaces"],
                        ] as const).map(([bucket, label]) => {
                            const entries = groupedSpaces[bucket];
                            if (entries.length === 0) {
                                return null;
                            }
                            return (
                                <div key={bucket} className="mb-2">
                                    <div className="px-3 py-1 text-[8px] font-black uppercase tracking-[0.24em] text-cortex-ink-faint/80">
                                        {label}
                                    </div>
                                    <div className="space-y-0.5">
                                        {entries.map((space) => {
                            const isSelected = activeSpaceIds.includes(space.id) && !isMeta;
                            const spaceConf = SPACE_CONFIG[space.id] || SPACE_CONFIG.default;
                            return (
                                <div
                                    key={space.id}
                                    className={`w-full flex items-center justify-between px-3 py-2 rounded-lg text-[11px] transition-all group ${isSelected ? "bg-white/5" : "hover:bg-white/5"}`}
                                >
                                    <button 
                                        onClick={(e) => selectSpace(space.id, e)}
                                        className="flex items-center gap-3 flex-1 text-left"
                                    >
                                        <div className={`w-6 h-6 flex items-center justify-center rounded-full bg-cortex-900 border border-white/5 shadow-inner ${isSelected ? "text-blue-400" : spaceConf.color}`}>
                                            {spaceConf.icon}
                                        </div>
                                        <div className="flex flex-col items-start leading-tight">
                                            <span className={`font-bold uppercase tracking-wide ${isSelected ? "text-blue-400" : "text-cortex-200 group-hover:text-white"}`}>
                                                {space.name}
                                            </span>
                                            <span className="text-[7px] font-medium text-cortex-500 uppercase tracking-tighter">
                                                ID: {space.id}
                                            </span>
                                            <span className="mt-0.5 inline-flex items-center rounded-full border border-white/8 bg-white/4 px-1.5 py-0.5 text-[7px] font-semibold uppercase tracking-wider text-cortex-300">
                                                {(space.sourceMode === "observed" && "Observed Live Space")
                                                    || (space.sourceMode === "preview" && "Preview Space")
                                                    || (space.sourceMode === "draft" && "Draft Space")
                                                    || "Registered Space"}
                                                {space.readinessSummary ? ` · ${space.readinessSummary.replace("_", " ")}` : ""}
                                            </span>
                                        </div>
                                    </button>
                                    
                                    <div className="flex items-center gap-2">
                                        {!isSelected ? (
                                            <button
                                                onClick={(e) => toggleSpace(space.id, e)}
                                                className="p-1 rounded-md hover:bg-blue-500/20 text-cortex-500 hover:text-blue-400 transition-colors"
                                                title="Add to Workbench"
                                            >
                                                <Plus className="w-3.5 h-3.5" />
                                            </button>
                                        ) : (
                                            <>
                                                {activeSpaceIds.length > 1 && (
                                                     <button
                                                        onClick={(e) => toggleSpace(space.id, e)}
                                                        className="p-1 rounded-md hover:bg-red-500/20 text-blue-400 hover:text-red-400 transition-colors"
                                                        title="Remove from Workbench"
                                                     >
                                                         <X className="w-3.5 h-3.5" />
                                                     </button>
                                                )}
                                                <Check className="w-3.5 h-3.5 text-blue-500" />
                                            </>
                                        )}
                                    </div>
                                </div>
                            );
                                        })}
                                    </div>
                                </div>
                            );
                        })}
                    </div>

                    <div className="mt-2 pt-1 border-t border-white/5">
                        <Link
                            to="/spaces"
                            onClick={() => setIsOpen(false)}
                            className="w-full flex items-center justify-between px-3 py-2.5 rounded-lg text-[10px] font-black text-cortex-400 hover:bg-white/5 hover:text-white transition-all uppercase tracking-widest"
                        >
                            <div className="flex items-center gap-2">
                                <Settings className="w-3.5 h-3.5" />
                                <span>Manage All Spaces</span>
                            </div>
                        </Link>
                    </div>
                </div>
            )}
        </div>
    );
};
