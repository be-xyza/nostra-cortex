export type SpaceArchetypeIconKey =
  | "globe"
  | "shield"
  | "flask"
  | "layout"
  | "sparkles"
  | "database";

export interface SpaceArchetypeVisuals {
  iconKey: SpaceArchetypeIconKey;
  gradient: string;
  ringGlow: string;
  accentColor: string;
  bgShimmer: string;
}

export interface SpaceStudioTemplate {
  id: string;
  archetypeId: string;
  name: string;
  summary: string;
  purpose: string;
  accessSummary: string;
  templateId: string;
}

export interface SpaceArchetypeProfile {
  id: string;
  label: string;
  aliases: string[];
  explorePolicyId: string;
  visuals: SpaceArchetypeVisuals;
  spaceStudioTemplates: SpaceStudioTemplate[];
}

const GENERAL_TEMPLATES: SpaceStudioTemplate[] = [
  {
    id: "team-room",
    archetypeId: "default",
    name: "Team room",
    summary: "A shared space for people, updates, and ongoing work.",
    purpose: "Give a team one place to coordinate work and keep recent outputs visible.",
    accessSummary: "People and agents can both contribute here.",
    templateId: "tpl_team_room_v1",
  },
];

const RESEARCH_TEMPLATES: SpaceStudioTemplate[] = [
  {
    id: "research-starter",
    archetypeId: "research",
    name: "Research starter",
    summary: "A simple research space for initiatives, notes, and steward review.",
    purpose: "Collect research notes, decisions, and proposal drafts in one place.",
    accessSummary: "Stewards and agents can work here after review.",
    templateId: "tpl_research_starter_v1",
  },
];

const GOVERNANCE_TEMPLATES: SpaceStudioTemplate[] = [
  {
    id: "governance-charter",
    archetypeId: "governance",
    name: "Governance charter",
    summary: "A structured space for proposals, decisions, and lineage-aware review.",
    purpose: "Track proposals, approvals, and forks with explicit stewardship context.",
    accessSummary: "Stewards guide structural change while others can review and contribute.",
    templateId: "tpl_governance_charter_v1",
  },
];

const INTRO_TEMPLATES: SpaceStudioTemplate[] = [
  {
    id: "intro-story",
    archetypeId: "intro",
    name: "Intro story",
    summary: "A guided introductory space for onboarding, narrative, and orientation.",
    purpose: "Show people where to start, what matters, and how the space evolves over time.",
    accessSummary: "Stewards curate the story while contributors can add supporting materials.",
    templateId: "tpl_intro_story_v1",
  },
];

export const SPACE_ARCHETYPE_PROFILES: SpaceArchetypeProfile[] = [
  {
    id: "meta",
    label: "Meta",
    aliases: ["meta", "global"],
    explorePolicyId: "explore.list.default.v1",
    visuals: {
      iconKey: "globe",
      gradient: "from-blue-500 via-indigo-500 to-purple-600",
      ringGlow: "shadow-[0_0_20px_rgba(99,102,241,0.4)]",
      accentColor: "text-blue-400",
      bgShimmer: "bg-linear-to-br from-blue-500/10 via-indigo-500/5 to-transparent",
    },
    spaceStudioTemplates: [],
  },
  {
    id: "default",
    label: "General",
    aliases: ["default", "general", "team"],
    explorePolicyId: "explore.list.default.v1",
    visuals: {
      iconKey: "layout",
      gradient: "from-emerald-500 via-teal-500 to-cyan-500",
      ringGlow: "shadow-[0_0_20px_rgba(16,185,129,0.4)]",
      accentColor: "text-emerald-400",
      bgShimmer: "bg-linear-to-br from-emerald-500/10 via-teal-500/5 to-transparent",
    },
    spaceStudioTemplates: GENERAL_TEMPLATES,
  },
  {
    id: "intro",
    label: "Intro",
    aliases: ["intro", "introduction", "onboarding", "story"],
    explorePolicyId: "explore.list.story.v1",
    visuals: {
      iconKey: "sparkles",
      gradient: "from-sky-500 via-cyan-400 to-indigo-500",
      ringGlow: "shadow-[0_0_20px_rgba(56,189,248,0.35)]",
      accentColor: "text-sky-300",
      bgShimmer: "bg-linear-to-br from-sky-500/10 via-cyan-400/5 to-transparent",
    },
    spaceStudioTemplates: INTRO_TEMPLATES,
  },
  {
    id: "research",
    label: "Research",
    aliases: ["research", "lab", "labs"],
    explorePolicyId: "explore.list.density.v1",
    visuals: {
      iconKey: "flask",
      gradient: "from-purple-500 via-fuchsia-500 to-pink-500",
      ringGlow: "shadow-[0_0_20px_rgba(168,85,247,0.4)]",
      accentColor: "text-purple-400",
      bgShimmer: "bg-linear-to-br from-purple-500/10 via-fuchsia-500/5 to-transparent",
    },
    spaceStudioTemplates: RESEARCH_TEMPLATES,
  },
  {
    id: "governance",
    label: "Governance",
    aliases: ["governance", "lineage"],
    explorePolicyId: "explore.list.lineage.v1",
    visuals: {
      iconKey: "shield",
      gradient: "from-amber-500 via-orange-500 to-red-500",
      ringGlow: "shadow-[0_0_20px_rgba(245,158,11,0.4)]",
      accentColor: "text-amber-400",
      bgShimmer: "bg-linear-to-br from-amber-500/10 via-orange-500/5 to-transparent",
    },
    spaceStudioTemplates: GOVERNANCE_TEMPLATES,
  },
  {
    id: "system",
    label: "System",
    aliases: ["system", "operations"],
    explorePolicyId: "explore.list.lineage.v1",
    visuals: {
      iconKey: "database",
      gradient: "from-slate-500 via-slate-400 to-zinc-600",
      ringGlow: "shadow-[0_0_20px_rgba(148,163,184,0.28)]",
      accentColor: "text-slate-300",
      bgShimmer: "bg-linear-to-br from-slate-500/10 via-slate-400/5 to-transparent",
    },
    spaceStudioTemplates: [],
  },
];

export const DEFAULT_SPACE_ARCHETYPE_PROFILE =
  SPACE_ARCHETYPE_PROFILES.find((profile) => profile.id === "default")!;

function normalizeSpaceArchetypeKey(value?: string): string | undefined {
  const normalized = value?.trim().toLowerCase();
  return normalized && normalized.length > 0 ? normalized : undefined;
}

export function resolveSpaceArchetypeProfile(
  archetype?: string,
): SpaceArchetypeProfile {
  const normalized = normalizeSpaceArchetypeKey(archetype);
  if (!normalized) {
    return DEFAULT_SPACE_ARCHETYPE_PROFILE;
  }

  return (
    SPACE_ARCHETYPE_PROFILES.find(
      (profile) =>
        profile.id === normalized || profile.aliases.includes(normalized),
    ) ?? DEFAULT_SPACE_ARCHETYPE_PROFILE
  );
}

export function getSpaceStudioTemplatesForArchetype(
  archetype?: string,
): SpaceStudioTemplate[] {
  const profile = resolveSpaceArchetypeProfile(archetype);
  if (profile.id === DEFAULT_SPACE_ARCHETYPE_PROFILE.id) {
    return [...profile.spaceStudioTemplates];
  }
  return [
    ...profile.spaceStudioTemplates,
    ...DEFAULT_SPACE_ARCHETYPE_PROFILE.spaceStudioTemplates,
  ];
}

export function resolveSpaceStudioTemplate(
  templateId?: string,
): SpaceStudioTemplate | undefined {
  const normalized = normalizeSpaceArchetypeKey(templateId);
  if (!normalized) {
    return undefined;
  }

  return SPACE_ARCHETYPE_PROFILES.flatMap((profile) => profile.spaceStudioTemplates).find(
    (template) =>
      template.id.toLowerCase() === normalized ||
      template.templateId.toLowerCase() === normalized,
  );
}

export const SPACE_STUDIO_TEMPLATES = SPACE_ARCHETYPE_PROFILES.flatMap(
  (profile) => profile.spaceStudioTemplates,
);
