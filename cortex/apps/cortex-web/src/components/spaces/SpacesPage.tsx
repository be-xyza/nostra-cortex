import React, { useState, useMemo, useEffect } from "react";
import {
  Globe,
  Shield,
  FlaskConical,
  Layout,
  Plus,
  Search,
  Activity,
  CheckCircle2,
  AlertTriangle,
  Sparkles,
  ArrowRight,
  MoreVertical,
  Users,
  Terminal,
  Layers,
  Settings,
  Copy,
  ExternalLink,
  Archive,
  Database,
  TrendingUp,
} from "lucide-react";
import * as DropdownMenu from "@radix-ui/react-dropdown-menu";
import { workbenchApi } from "../../api";
import {
  describeSpaceSourceMode,
  partitionSpacesBySource,
  useSpaceRegistrySnapshot,
  type Space
} from "../../store/spacesRegistry";
import { useUiStore } from "../../store/uiStore";
import { useUserPreferences } from "../../store/userPreferences";
import { useNavigate } from "react-router-dom";
import { useSelection } from "../../hooks/useSelection";
import { useEnforcementProfiles, type EnforcementId } from "../../store/enforcementProfilesStore";
import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";
import { SPACE_STUDIO_ROUTE } from "./spaceStudioRoutes";
import {
  resolveSpaceArchetypeProfile,
  type SpaceArchetypeIconKey,
} from "./spaceArchetypeProfiles";
import { SpaceDesignProfilePreviewPanel } from "./SpaceDesignProfilePreviewPanel";
import type { SpaceDesignProfilePreviewFixture } from "../../store/spaceDesignProfilePreviewContract";

function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

/* ─────────────────────────── Space Config ─────────────────────────── */

interface SpaceVisualConfig {
  icon: React.ReactNode;
  gradient: string;
  ringGlow: string;
  accentColor: string;
  bgShimmer: string;
}

const ICON_MAP: Record<SpaceArchetypeIconKey, React.ReactNode> = {
  globe: <Globe className="w-6 h-6" />,
  shield: <Shield className="w-6 h-6" />,
  flask: <FlaskConical className="w-6 h-6" />,
  layout: <Layout className="w-6 h-6" />,
  sparkles: <Sparkles className="w-6 h-6" />,
  database: <Database className="w-6 h-6" />,
};

function getVisuals(space: Space): SpaceVisualConfig {
  const profile = resolveSpaceArchetypeProfile(space.archetype);
  return {
    icon: ICON_MAP[profile.visuals.iconKey],
    gradient: profile.visuals.gradient,
    ringGlow: profile.visuals.ringGlow,
    accentColor: profile.visuals.accentColor,
    bgShimmer: profile.visuals.bgShimmer,
  };
}

/* ─────────────────────── SIQ Gate Simulation ──────────────────────── */

type SiqVerdict = "ready" | "not-ready";

interface SpaceSiqStatus {
  mode: EnforcementId;
  verdict: SiqVerdict;
  passCount: number;
  failCount: number;
}

function getSpaceSiq(space: Space): SpaceSiqStatus {
  if (space.config?.enforcement && space.stats) {
    const modeMap: Record<string, EnforcementId> = {
      'strict': 'hardgate',
      'flexible': 'softgate',
      'audit': 'observe'
    };
    return {
      mode: modeMap[space.config.enforcement] || 'observe',
      verdict: space.stats.growthPercentage >= 0 ? "ready" : "not-ready",
      passCount: Math.floor(space.stats.objectCount * 0.98),
      failCount: Math.floor(space.stats.objectCount * 0.02),
    };
  }

  // Fallback simulation for unknown spaces
  const hash = space.id.split("").reduce((acc, char) => acc + char.charCodeAt(0), 0);
  const modes: EnforcementId[] = ["observe", "softgate", "hardgate"];
  const mode = modes[hash % 3];
  return {
    mode,
    verdict: hash % 4 === 0 ? "not-ready" : "ready",
    passCount: 5 + (hash % 15),
    failCount: hash % 4 === 0 ? 1 : 0,
  };
}

/* ─────────────────────── Space Actions Menu ─────────────────────── */

interface SpaceActionsMenuProps {
  space: Space;
  onOpenDetails: (id: string) => void;
  onOpenSettings?: (id: string) => void;
}

const SpaceActionsMenu: React.FC<SpaceActionsMenuProps> = ({ space, onOpenDetails, onOpenSettings }) => {
  const [copied, setCopied] = useState(false);
  const actions = space.config?.actions || ['details', 'copy_id'];

  const handleCopyId = (e: React.MouseEvent) => {
    e.stopPropagation();
    navigator.clipboard.writeText(space.id);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <DropdownMenu.Root>
      <DropdownMenu.Trigger asChild>
        <button
          onClick={(e) => e.stopPropagation()}
          className="p-1 rounded-md hover:bg-white/10 text-white/30 hover:text-white/70 transition-all shrink-0 outline-none"
        >
          <MoreVertical className="w-3.5 h-3.5" />
        </button>
      </DropdownMenu.Trigger>

      <DropdownMenu.Portal>
        <DropdownMenu.Content
          className="z-50 min-w-[160px] bg-[#1a1c23] border border-white/10 rounded-lg p-1.5 shadow-2xl animate-in fade-in zoom-in-95 duration-100"
          sideOffset={5}
          align="end"
          onClick={(e) => e.stopPropagation()}
        >
          {actions.includes('details') && (
            <DropdownMenu.Item
              className="flex items-center gap-2 px-2.5 py-2 text-[11px] text-white/70 hover:text-white hover:bg-white/5 rounded-md cursor-pointer outline-none transition-colors"
              onClick={() => onOpenDetails(space.id)}
            >
              <ExternalLink className="w-3.5 h-3.5" />
              <span>View Details</span>
            </DropdownMenu.Item>
          )}

          {actions.includes('copy_id') && (
            <DropdownMenu.Item
              className="flex items-center gap-2 px-2.5 py-2 text-[11px] text-white/70 hover:text-white hover:bg-white/5 rounded-md cursor-pointer outline-none transition-colors"
              onClick={handleCopyId}
            >
              <Copy className={cn("w-3.5 h-3.5", copied && "text-emerald-400")} />
              <span>{copied ? "Copied!" : "Copy Full ID"}</span>
            </DropdownMenu.Item>
          )}

          {actions.includes('explore') && (
            <DropdownMenu.Item
              className="flex items-center gap-2 px-2.5 py-2 text-[11px] text-indigo-400/80 hover:text-indigo-300 hover:bg-white/5 rounded-md cursor-pointer outline-none transition-colors"
            >
              <Sparkles className="w-3.5 h-3.5" />
              <span>Explore Capability</span>
            </DropdownMenu.Item>
          )}

          {actions.includes('settings') && (
            <DropdownMenu.Item
              className="flex items-center gap-2 px-2.5 py-2 text-[11px] text-white/70 hover:text-white hover:bg-white/5 rounded-md cursor-pointer outline-none transition-colors"
              onClick={() => (onOpenSettings ? onOpenSettings(space.id) : onOpenDetails(space.id))}
            >
              <Settings className="w-3.5 h-3.5" />
              <span>Space Settings</span>
            </DropdownMenu.Item>
          )}

          {actions.includes('archive') && (
            <>
              <DropdownMenu.Separator className="h-px bg-white/5 my-1" />
              <DropdownMenu.Item
                className="flex items-center gap-2 px-2.5 py-2 text-[11px] text-red-400/70 hover:text-red-400 hover:bg-red-400/5 rounded-md cursor-pointer outline-none transition-colors"
              >
                <Archive className="w-3.5 h-3.5" />
                <span>Archive Space</span>
              </DropdownMenu.Item>
            </>
          )}
        </DropdownMenu.Content>
      </DropdownMenu.Portal>
    </DropdownMenu.Root>
  );
};

/* ─────────────────────── Compact Space Card ─────────────────────── */

interface SpaceCardProps {
  space: Space;
  reviewCount: number;
  isSelected: boolean;
  onSelect: (id: string, event: React.MouseEvent) => void;
  onOpenDetails: (id: string) => void;
  onOpenSettings?: (id: string) => void;
}

const SpaceCard: React.FC<SpaceCardProps> = ({ space, reviewCount, isSelected, onSelect, onOpenDetails, onOpenSettings }) => {
  const visuals = getVisuals(space);
  const siq = getSpaceSiq(space);
  const profiles = useEnforcementProfiles();
  const siqConfig = profiles[siq.mode];
  
  const stats = space.stats || {
    objectCount: Math.floor(space.id.length * 12.5),
    growthPercentage: (space.id.length % 8) + 2,
    memberCount: 3 + (space.id.length % 15)
  };

  return (
    <div
      onClick={(e) => onSelect(space.id, e)}
      onDoubleClick={() => onOpenDetails(space.id)}
      className={`group relative flex flex-col h-[160px] rounded-xl border transition-all duration-300 cursor-pointer overflow-hidden
        ${isSelected
          ? `border-white/20 bg-white/8 ${visuals.ringGlow}`
          : "border-white/6 bg-white/2 hover:border-white/12 hover:bg-white/4"
        }
      `}
    >
      {/* SHIMMER BACKGROUND */}
      <div className={`absolute inset-0 rounded-xl ${visuals.bgShimmer} opacity-0 group-hover:opacity-100 transition-opacity duration-500`} />

      {/* HEADER */}
      <div className="relative z-10 flex items-center justify-between p-3 border-b border-white/4">
        <div className="flex items-center gap-2.5 min-w-0">
          <div className={`w-8 h-8 rounded-lg bg-linear-to-br ${visuals.gradient} p-px shrink-0`}>
            <div className="w-full h-full rounded-lg bg-[#0f1117] flex items-center justify-center text-white/90">
              {React.cloneElement(visuals.icon as React.ReactElement, { className: "w-4 h-4" })}
            </div>
          </div>
          <div className="min-w-0">
            <h3 className="text-xs font-semibold text-white/90 truncate">
              {space.name}
            </h3>
            <span className="text-[8px] uppercase tracking-wider text-indigo-400/50 font-medium">
              {space.type}
            </span>
            <div className="mt-1 flex items-center gap-1.5 flex-wrap">
              <span className="inline-flex items-center rounded-full border border-white/10 bg-white/5 px-1.5 py-0.5 text-[7px] font-semibold uppercase tracking-wider text-white/50">
                {describeSpaceSourceMode(space)}
              </span>
              {space.readinessSummary && (
                <span className={`inline-flex items-center rounded-full border px-1.5 py-0.5 text-[7px] font-semibold uppercase tracking-wider ${
                  space.readinessSummary === "pass"
                    ? "border-emerald-400/20 bg-emerald-400/10 text-emerald-100"
                    : space.readinessSummary === "fail"
                      ? "border-red-400/20 bg-red-400/10 text-red-100"
                      : "border-amber-400/20 bg-amber-400/10 text-amber-100"
                }`}>
                  {space.readinessSummary.replace("_", " ")}
                </span>
              )}
            </div>
          </div>
        </div>
        
        <SpaceActionsMenu space={space} onOpenDetails={onOpenDetails} onOpenSettings={onOpenSettings} />
      </div>

      {/* BODY — Enforcement + Stats + Rich Data */}
      <div className="relative z-10 flex-1 flex flex-col justify-center px-3 py-2">
        <div className="flex items-center justify-between mb-2">
          <div className="flex items-center gap-2">
            <span className={`${siqConfig.color}`}>{siqConfig.icon}</span>
            <span className={`text-[9px] font-bold uppercase tracking-tight ${siqConfig.color}`}>
              {siqConfig.label}
            </span>
          </div>
          <div className="flex items-center gap-2">
            <div className="flex items-center gap-1">
              <div className="w-1.5 h-1.5 rounded-full bg-emerald-400" />
              <span className="text-[9px] text-white/35">{siq.passCount}</span>
            </div>
            {siq.failCount > 0 && (
              <div className="flex items-center gap-1">
                <div className="w-1.5 h-1.5 rounded-full bg-red-400" />
                <span className="text-[9px] text-white/35">{siq.failCount}</span>
              </div>
            )}
          </div>
        </div>

        <div className="flex items-center gap-3">
          <div className="flex items-center gap-1 text-[8px] text-white/20">
            <Database className="w-2.5 h-2.5" />
            <span>{stats.objectCount.toLocaleString()} objects</span>
          </div>
          <div className="flex items-center gap-1 text-[8px] text-emerald-400/40">
            <TrendingUp className="w-2.5 h-2.5" />
            <span>+{stats.growthPercentage}% growth</span>
          </div>
        </div>
        {reviewCount > 0 ? (
          <div className="mt-3 inline-flex w-fit items-center gap-1.5 rounded-full border border-amber-400/18 bg-amber-400/10 px-2.5 py-1 text-[9px] font-medium text-amber-100">
            <div className="h-1.5 w-1.5 rounded-full bg-amber-300" />
            {reviewCount === 1 ? "Needs review" : `${reviewCount} items need review`}
          </div>
        ) : null}
      </div>

      <div className="relative z-10 flex items-center justify-between px-3 py-2 bg-black/20 border-t border-white/4">
        <div className="flex items-center gap-2 min-w-0">
          <span 
            title={space.id}
            className="text-[7px] uppercase tracking-tight text-white/30 font-bold bg-white/5 px-1 py-px rounded border border-white/5 truncate max-w-[60px] cursor-help"
          >
            {space.id}
          </span>
          <div className="flex items-center gap-1 text-[9px] text-white/20 shrink-0">
            <Users className="w-2.5 h-2.5" />
            <span>{stats.memberCount}</span>
          </div>
        </div>
        <button 
          onClick={(e) => { e.stopPropagation(); onOpenDetails(space.id); }}
          className="flex items-center gap-1 text-[9px] text-white/40 hover:text-white/70 transition-colors font-medium shrink-0"
        >
          <span>Details</span>
          <ArrowRight className="w-2.5 h-2.5" />
        </button>
      </div>

      {/* SELECTION OVERLAY */}
      {isSelected && (
        <div className="absolute inset-0 border-2 border-white/10 rounded-xl pointer-events-none" />
      )}
    </div>
  );
};

/* ─────────────────────── Workbench Card ──────────────────────── */

interface WorkbenchCardProps {
  space: Space;
  memberSpaces: Space[];
  isSelected: boolean;
  onSelect: (id: string, event: React.MouseEvent) => void;
  onOpenDetails: (id: string) => void;
  onOpenSettings?: (id: string) => void;
}

const WorkbenchCard: React.FC<WorkbenchCardProps> = ({ space, memberSpaces, isSelected, onSelect, onOpenDetails, onOpenSettings }) => {
  const visuals = getVisuals(space);
  const stats = space.stats || {
    objectCount: memberSpaces.length * 1500,
    growthPercentage: 5,
    memberCount: memberSpaces.length
  };

  return (
    <div
      onClick={(e) => onSelect(space.id, e)}
      onDoubleClick={() => onOpenDetails(space.id)}
      className={`group relative flex flex-col h-[200px] rounded-xl border transition-all duration-300 cursor-pointer overflow-hidden
        ${isSelected
          ? `border-white/20 bg-white/8 ${visuals.ringGlow}`
          : "border-white/6 bg-white/2 hover:border-white/12 hover:bg-white/4"
        }
      `}
    >
      <div className={`absolute inset-0 rounded-xl ${visuals.bgShimmer} opacity-0 group-hover:opacity-100 transition-opacity duration-500`} />

      {/* HEADER */}
      <div className="relative z-10 flex items-center justify-between p-3 border-b border-white/4">
        <div className="flex items-center gap-2.5 min-w-0">
          <div className={`w-8 h-8 rounded-lg bg-linear-to-br ${visuals.gradient} p-px shrink-0`}>
            <div className="w-full h-full rounded-lg bg-[#0f1117] flex items-center justify-center text-white/90">
              <Layers className="w-4 h-4" />
            </div>
          </div>
          <div className="min-w-0">
            <h3 className="text-xs font-semibold text-white/90 truncate">{space.name}</h3>
            <span className="text-[8px] uppercase tracking-wider text-indigo-400/50 font-bold">
              {stats.memberCount} spaces aggregated
            </span>
          </div>
        </div>
        
        <SpaceActionsMenu space={space} onOpenDetails={onOpenDetails} onOpenSettings={onOpenSettings} />
      </div>

      {/* BODY — Space Membership Avatars */}
      <div className="relative z-10 flex-1 flex items-center px-4 py-3 gap-3 overflow-hidden">
        {memberSpaces.slice(0, 5).map((ms) => {
          const msVisuals = getVisuals(ms);
          return (
            <div
              key={ms.id}
              title={ms.name}
              className="w-12 h-12 rounded-xl bg-linear-to-br from-white/10 to-transparent p-px shrink-0 hover:scale-105 transition-transform group/avatar"
            >
              <div className={cn(
                "w-full h-full rounded-xl bg-[#0f1117] flex items-center justify-center text-[12px] font-black uppercase transition-colors border border-white/5 group-hover/avatar:border-white/20",
                msVisuals.accentColor
              )}>
                {ms.name.charAt(0)}
              </div>
            </div>
          );
        })}
        {memberSpaces.length > 5 && (
          <div 
            className="w-12 h-12 rounded-xl border border-white/10 bg-white/5 flex items-center justify-center cursor-help shrink-0 group/overflow relative"
            title={`+${memberSpaces.length - 5} more spaces: ${memberSpaces.slice(5).map(m => m.name).join(', ')}`}
          >
            <span className="text-[12px] text-white/40 font-bold">+{memberSpaces.length - 5}</span>
            
            {/* HOVER TOOLTIP LIST */}
            <div className="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 w-48 bg-[#1a1c23] border border-white/10 rounded-lg p-2.5 shadow-2xl opacity-0 group-hover/overflow:opacity-100 pointer-events-none transition-opacity z-50">
               <div className="text-[9px] uppercase tracking-wider text-white/20 font-black mb-1.5">Additional Spaces</div>
               <div className="flex flex-col gap-1">
                 {memberSpaces.slice(5).map(ms => (
                   <div key={ms.id} className="text-[10px] text-white/60 flex items-center gap-2">
                     <div className={cn("w-1.5 h-1.5 rounded-full bg-linear-to-br", getVisuals(ms).gradient)} />
                     <span className="truncate">{ms.name}</span>
                   </div>
                 ))}
               </div>
            </div>
          </div>
        )}
      </div>

      {/* FOOTER */}
      <div className="relative z-10 flex items-center justify-between px-3 py-2 bg-black/20 border-t border-white/4">
        <div className="flex items-center gap-2.5">
          <span className="text-[8px] uppercase tracking-wider text-white/40 font-bold bg-white/5 px-1.5 py-0.5 rounded border border-white/5">
            {space.id}
          </span>
          <div className="flex items-center gap-1 text-[9px] text-white/25">
            <Globe className="w-3 h-3" />
            <span>Shared overview</span>
          </div>
        </div>
        <button 
          onClick={(e) => { e.stopPropagation(); onOpenDetails(space.id); }}
          className="flex items-center gap-1 text-[9px] text-white/40 hover:text-white/70 transition-colors font-medium shrink-0"
        >
          <span>Details</span>
          <ArrowRight className="w-2.5 h-2.5" />
        </button>
      </div>

      {isSelected && (
        <div className="absolute inset-0 border-2 border-white/10 rounded-xl pointer-events-none" />
      )}
    </div>
  );
};

/* ─────────────────────── New Space / New Workbench CTAs ──────────────────────── */

const NewSpaceCard: React.FC<{ onClick: () => void }> = ({ onClick }) => (
  <button
    onClick={onClick}
    className="group flex items-center gap-3 h-[160px] rounded-xl border border-dashed border-white/8 bg-white/1
      hover:border-white/15 hover:bg-white/3 transition-all duration-200 cursor-pointer px-4"
  >
    <div className="w-8 h-8 rounded-lg bg-white/5 border border-white/8 flex items-center justify-center
      group-hover:bg-white/10 group-hover:border-white/15 transition-all">
      <Plus className="w-4 h-4 text-white/25 group-hover:text-white/50 transition-colors" />
    </div>
    <div className="text-left">
      <span className="text-xs font-medium text-white/35 group-hover:text-white/60 transition-colors block">
        Draft Space
      </span>
      <span className="text-[9px] text-white/15 group-hover:text-white/30 transition-colors">
        Start in Labs first
      </span>
    </div>
  </button>
);

const NewWorkbenchCard: React.FC<{ onClick: () => void }> = ({ onClick }) => (
  <button
    onClick={onClick}
    className="group flex items-center gap-3 h-[200px] rounded-xl border border-dashed border-white/8 bg-white/1
      hover:border-indigo-500/20 hover:bg-indigo-500/3 transition-all duration-200 cursor-pointer px-4"
  >
    <div className="w-8 h-8 rounded-lg bg-indigo-500/5 border border-indigo-500/10 flex items-center justify-center
      group-hover:bg-indigo-500/10 group-hover:border-indigo-500/20 transition-all">
      <Plus className="w-4 h-4 text-indigo-400/30 group-hover:text-indigo-400/60 transition-colors" />
    </div>
    <div className="text-left">
      <span className="text-xs font-medium text-white/35 group-hover:text-white/60 transition-colors block">
        New shared view
      </span>
      <span className="text-[9px] text-white/15 group-hover:text-white/30 transition-colors">
        Aggregate multiple spaces
      </span>
    </div>
  </button>
);

/* ─────────────────────── SpacesPage Component ─────────────────────── */

export const SpacesPage: React.FC = () => {
  const { spaces, registryResolved, registryDegraded } = useSpaceRegistrySnapshot();
  const profiles = useEnforcementProfiles();
  const activeSpaceIds = useUiStore((state) => state.activeSpaceIds);
  const setActiveSpaceIds = useUiStore((state) => state.setActiveSpaceIds);
  const [searchQuery, setSearchQuery] = useState("");
  const [reviewCounts, setReviewCounts] = useState<Record<string, number>>({});
  const [spaceDesignPreview, setSpaceDesignPreview] = useState<SpaceDesignProfilePreviewFixture | null>(null);
  const [spaceDesignPreviewLoading, setSpaceDesignPreviewLoading] = useState(false);
  const navigate = useNavigate();
  const registryMode = useUserPreferences((state) => state.registryMode);
  const setRegistryMode = useUserPreferences((state) => state.setRegistryMode);

  const workbenches = useMemo(() => spaces.filter(s => s.id === "meta"), [spaces]);
  const sovereignSpaces = useMemo(() => {
    const nonMeta = spaces.filter((s) => s.id !== "meta");
    if (!searchQuery.trim()) return nonMeta;
    return nonMeta.filter((s) =>
      s.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      s.type.toLowerCase().includes(searchQuery.toLowerCase())
    );
  }, [spaces, searchQuery]);
  const sovereignSpacesBySource = useMemo(
    () => partitionSpacesBySource(sovereignSpaces),
    [sovereignSpaces],
  );

  const {
    selectedIds,
    handleSelection: handleSelectionLogic,
  } = useSelection<Space>(activeSpaceIds);

  // Sync back to uiStore (Workbench Scope)
  React.useEffect(() => {
    if (selectedIds.length > 0 && JSON.stringify(selectedIds) !== JSON.stringify(activeSpaceIds)) {
      setActiveSpaceIds(selectedIds);
    }
  }, [selectedIds, activeSpaceIds, setActiveSpaceIds]);

  const handleSelect = (id: string, event: React.MouseEvent) => {
    handleSelectionLogic(event, id, spaces, (s: Space) => s.id);
  };

  const handleOpenDetails = (id: string) => {
    navigate(`/spaces/${id}`);
  };
  const handleOpenSettings = (id: string) => {
    navigate(`/spaces/${id}?tab=routing`);
  };

  // Summary stats
  const totalSpaces = sovereignSpaces.length;
  const readyCount = sovereignSpaces.filter((s: Space) => s.readinessSummary === "pass").length;
  const sourceCounts = useMemo(() => {
    return sovereignSpaces.reduce(
      (acc, space) => {
        const mode = space.sourceMode ?? "registered";
        if (mode === "observed") {
          acc.observed += 1;
        } else if (mode === "preview") {
          acc.preview += 1;
        } else if (mode === "draft") {
          acc.draft += 1;
        } else {
          acc.registered += 1;
        }
        return acc;
      },
      { registered: 0, observed: 0, preview: 0, draft: 0 },
    );
  }, [sovereignSpaces]);

  useEffect(() => {
    let cancelled = false;

    Promise.all(
      sovereignSpaces.map(async (space) => {
        try {
          const response = await workbenchApi.getHeapBlocks({
            spaceId: space.id,
            blockType: "proposal",
            limit: 1,
          });
          return [space.id, response.count ?? response.items.length] as const;
        } catch {
          return [space.id, 0] as const;
        }
      }),
    ).then((entries) => {
      if (cancelled) {
        return;
      }
      setReviewCounts(Object.fromEntries(entries));
    });

    return () => {
      cancelled = true;
    };
  }, [sovereignSpaces]);

  useEffect(() => {
    let cancelled = false;
    setSpaceDesignPreviewLoading(true);
    workbenchApi.getSpaceDesignProfilePreview()
      .then((preview) => {
        if (!cancelled) {
          setSpaceDesignPreview(preview);
        }
      })
      .catch(() => {
        if (!cancelled) {
          setSpaceDesignPreview(null);
        }
      })
      .finally(() => {
        if (!cancelled) {
          setSpaceDesignPreviewLoading(false);
        }
      });

    return () => {
      cancelled = true;
    };
  }, []);

  return (
    <div className="flex-1 overflow-y-auto px-6 py-8 max-w-6xl mx-auto">
      {/* Header */}
      <div className="mb-6">
        <div className="flex items-center gap-3 mb-1">
          <div className="w-9 h-9 rounded-xl bg-linear-to-br from-indigo-500/20 to-purple-500/20 flex items-center justify-center">
            <Sparkles className="w-4.5 h-4.5 text-indigo-400" />
          </div>
          <div>
            <h1 className="text-xl font-bold text-white/90 tracking-tight">Spaces</h1>
            <p className="text-xs text-white/35">
              Integrity Grid · {totalSpaces} spaces · {readyCount} ready · {sourceCounts.registered} registered · {sourceCounts.observed} observed · {sourceCounts.preview} preview · {sourceCounts.draft} draft
            </p>
          </div>
        </div>
      </div>

      {/* Search & Filter Bar */}
      <div className="mb-5 flex items-center justify-between gap-4">
        <div className="relative max-w-sm flex-1">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-white/20" />
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="Search spaces…"
            className="w-full pl-9 pr-4 py-2 rounded-lg bg-white/4 border border-white/6 text-xs text-white/80 placeholder-white/20
              focus:outline-none focus:border-white/15 focus:bg-white/6 transition-all duration-200"
          />
        </div>
        <DropdownMenu.Root>
          <DropdownMenu.Trigger asChild>
            <button className="flex items-center justify-center w-9 h-9 rounded-lg bg-white/4 border border-white/6 text-white/60 hover:text-white/90 hover:bg-white/10 transition-colors" aria-label="Spaces Settings">
              <Settings className="w-4 h-4" />
            </button>
          </DropdownMenu.Trigger>
          <DropdownMenu.Portal>
            <DropdownMenu.Content
              className="min-w-[200px] bg-slate-800/90 backdrop-blur-md border border-white/10 rounded-xl p-1.5 shadow-2xl z-50 text-xs"
              align="end"
              sideOffset={8}
            >
              <DropdownMenu.Label className="px-2 py-1.5 text-[10px] uppercase tracking-wider text-white/40 font-semibold">
                Spaces Settings
              </DropdownMenu.Label>
              <DropdownMenu.Item
                className="flex items-center justify-between px-2 py-2 rounded-lg outline-none cursor-default hover:bg-white/10 focus:bg-white/10 group transition-colors"
                onClick={() => setRegistryMode(registryMode === 'preview' ? 'live' : 'preview')}
              >
                <span className="flex items-center gap-2 text-white/80 group-hover:text-white">
                  <Database className="w-4 h-4 text-white/50 group-hover:text-blue-400 transition-colors" />
                  Preview Fixtures
                </span>
                <div className={`w-8 h-4 rounded-full p-0.5 transition-colors duration-200 flex items-center ${registryMode === 'preview' ? 'bg-blue-500 justify-end' : 'bg-white/10 group-hover:bg-white/20 justify-start'}`}>
                  <div className={`w-3 h-3 bg-white rounded-full shadow-sm`} />
                </div>
              </DropdownMenu.Item>
            </DropdownMenu.Content>
          </DropdownMenu.Portal>
        </DropdownMenu.Root>
      </div>

      {/* SIQ Legend — compact */}
      <div className="flex items-center gap-3 mb-5 text-[9px] text-white/25">
        <span className="flex items-center gap-1"><Activity className="w-2.5 h-2.5" /> Enforcement:</span>
        {Object.values(profiles).map((config) => (
          <span 
            key={config.id} 
            className={`flex items-center gap-1 ${config.color} cursor-help`}
            title={config.description}
          >
            {config.icon}
            {config.label}
          </span>
        ))}
        <span className="ml-1 flex items-center gap-1 text-emerald-400/50">
          <CheckCircle2 className="w-2.5 h-2.5" /> Ready
        </span>
        <span className="flex items-center gap-1 text-amber-400/50">
          <AlertTriangle className="w-2.5 h-2.5" /> Not Ready
        </span>
      </div>

      <SpaceDesignProfilePreviewPanel
        preview={spaceDesignPreview}
        loading={spaceDesignPreviewLoading}
      />

      {/* ═══════════════ Workbenches Section ═══════════════ */}
      <div className="mb-8">
        <h2 className="text-[9px] uppercase tracking-[0.25em] font-black text-white/20 mb-4 flex items-center gap-3">
          <Terminal className="w-3 h-3" />
          <span>Shared views</span>
          <div className="h-px flex-1 bg-white/5" />
        </h2>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
          {workbenches.map((space) => (
            <WorkbenchCard
              key={space.id}
              space={space}
              memberSpaces={sovereignSpaces}
              isSelected={selectedIds.includes(space.id)}
              onSelect={handleSelect}
              onOpenDetails={handleOpenDetails}
              onOpenSettings={handleOpenSettings}
            />
          ))}
          <NewWorkbenchCard onClick={() => navigate("/spaces#new-workbench")} />
        </div>
      </div>

      {/* ═══════════════ Sovereign Spaces Section ═══════════════ */}
      <div className="mb-8">
        <h2 className="text-[9px] uppercase tracking-[0.25em] font-black text-white/20 mb-4 flex items-center gap-3">
          <Globe className="w-3 h-3" />
          <span>Sovereign Spaces</span>
          <div className="h-px flex-1 bg-white/5" />
        </h2>
        {!registryResolved && sovereignSpaces.length === 0 && (
          <div className="mb-3 rounded-xl border border-white/6 bg-white/2 px-4 py-3 text-xs text-white/45">
            Discovering registered and observed live Spaces…
          </div>
        )}
        {registryDegraded && sovereignSpaces.length > 0 && (
          <div className="mb-3 rounded-xl border border-amber-400/15 bg-amber-400/8 px-4 py-3 text-xs text-amber-100/80">
            Gateway unavailable. Showing the last known live Space registry while the connection recovers.
          </div>
        )}
        {registryResolved && sovereignSpaces.length === 0 && (
          <div className="mb-3 rounded-xl border border-white/6 bg-white/2 px-4 py-3 text-xs text-white/45">
            No registered or observed live Spaces are available yet. Preview fixtures are still available in the settings menu when you intentionally want demo content.
          </div>
        )}
        {([
          ["registered", "Registered Spaces"],
          ["observed", "Observed Live Evidence"],
          ["draft", "Draft Spaces"],
          ["preview", "Preview Spaces"],
        ] as const).map(([bucket, label]) => {
          const entries = sovereignSpacesBySource[bucket];
          if (entries.length === 0) {
            return null;
          }
          return (
            <div key={bucket} className="mb-6">
              <div className="mb-3 flex items-center gap-3">
                <h3 className="text-[9px] uppercase tracking-[0.22em] font-black text-white/25">
                  {label}
                </h3>
                <div className="h-px flex-1 bg-white/5" />
                <span className="text-[9px] text-white/20">{entries.length}</span>
              </div>
              <div className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-3">
                {entries.map((space) => (
                  <SpaceCard
                    key={space.id}
                    space={space}
                    reviewCount={reviewCounts[space.id] ?? 0}
                    isSelected={selectedIds.includes(space.id)}
                    onSelect={handleSelect}
                    onOpenDetails={handleOpenDetails}
                    onOpenSettings={handleOpenSettings}
                  />
                ))}
                {bucket === "draft" && <NewSpaceCard onClick={() => navigate(SPACE_STUDIO_ROUTE)} />}
              </div>
            </div>
          );
        })}
        {sovereignSpacesBySource.draft.length === 0 && (
          <div className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-3">
            <NewSpaceCard onClick={() => navigate(SPACE_STUDIO_ROUTE)} />
          </div>
        )}
      </div>
    </div>
  );
};
