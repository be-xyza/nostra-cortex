import type {
  A2UISubmitFeedbackRequest,
  A2UISubmitFeedbackResponse,
  HeapBlockListItem,
} from "./contracts.ts";
import { reduceHeapBlocks } from "./store/eventProcessor.ts";
import { appendEvent, getEventsBySpace, type PreviewGlobalEvent } from "./store/eventStore.ts";

export const LOCAL_DEV_RESEARCH_SPACE_ID = "01KM4C04QY37V9RV9H2HH9J1NM";
export const LOCAL_DEV_SYSTEM_SPACE_ID = "system";

const LOCAL_DEV_BOOTSTRAP_ENABLED_DEFAULT = "true";
const LOCAL_DEV_SUPPORTED_SPACE_IDS = new Set([
  LOCAL_DEV_SYSTEM_SPACE_ID,
  LOCAL_DEV_RESEARCH_SPACE_ID,
]);

interface LocalDevSeedSpec {
  artifactId: string;
  emittedAt: string;
  spaceId: string;
  title: string;
  summary: string;
  recommendation: string;
  requestedAgentRole: string;
  authorityScope: string;
  requiredCapabilities: string[];
  sourceRefs: string[];
  tags: string[];
}

const LOCAL_DEV_HEAP_SEEDS: readonly LocalDevSeedSpec[] = [
  {
    artifactId: "local-dev-system-hermes-review-001",
    emittedAt: "2026-04-23T19:20:00.000Z",
    spaceId: LOCAL_DEV_SYSTEM_SPACE_ID,
    title: "Hermes Advisory Review Pilot",
    summary:
      "Localhost bootstrap lane validating that Hermes-style advisory blocks can render in heap review surfaces before any real emission test.",
    recommendation:
      "Review the attached readiness summary, record steward feedback, and confirm the artifact lineage path is visible in the existing heap UX.",
    requestedAgentRole: "steward",
    authorityScope: "review_only",
    requiredCapabilities: [
      "heap_review",
      "artifact_lineage",
      "steward_feedback_recording",
    ],
    sourceRefs: [
      "initiative-132-hermes-system-cartography-pass",
      "initiative-132-hermes-visibility-approval-projection-pass",
    ],
    tags: ["hermes", "pilot", "system"],
  },
  {
    artifactId: "local-dev-research-hermes-review-001",
    emittedAt: "2026-04-23T19:32:00.000Z",
    spaceId: LOCAL_DEV_RESEARCH_SPACE_ID,
    title: "Initiative 132 Research Review Surface",
    summary:
      "Research-space validation copy of the Hermes advisory review pattern for Initiative 132 evidence and follow-up discussion.",
    recommendation:
      "Use this space after system validation to confirm the same review flow works in the canonical Initiative 132 research context.",
    requestedAgentRole: "steward",
    authorityScope: "initiative_132_advisory_review",
    requiredCapabilities: [
      "initiative_review",
      "heap_projection_validation",
      "steward_feedback_recording",
    ],
    sourceRefs: [
      "initiative-132-system-cartography-heap-projection",
      "Research · 01KM4C04QY37V9RV9H2HH9J1NM",
    ],
    tags: ["hermes", "initiative-132", "research"],
  },
];

function envValue(name: string): string | undefined {
  return (import.meta as unknown as { env?: Record<string, string | undefined> }).env?.[name];
}

export function isLocalDevelopmentHost(hostname?: string): boolean {
  const resolvedHostname =
    hostname
    ?? (typeof globalThis !== "undefined" && "location" in globalThis
      ? globalThis.location?.hostname
      : undefined)
    ?? "";
  return resolvedHostname === "localhost" || resolvedHostname === "127.0.0.1";
}

export function isLocalDevBootstrapEnabled(hostname?: string): boolean {
  if (!isLocalDevelopmentHost(hostname)) {
    return false;
  }
  const configured =
    envValue("VITE_LOCAL_DEV_BOOTSTRAP")
    ?? LOCAL_DEV_BOOTSTRAP_ENABLED_DEFAULT;
  return configured.trim().toLowerCase() !== "false";
}

export function shouldUseLocalDevSpaceBootstrap(spaceId?: string | null): boolean {
  const normalized = spaceId?.trim();
  if (!normalized) {
    return false;
  }
  return isLocalDevBootstrapEnabled() && LOCAL_DEV_SUPPORTED_SPACE_IDS.has(normalized);
}

function currentLocalDevSpaceIdFromLocation(): string | null {
  if (typeof globalThis === "undefined" || !("location" in globalThis)) {
    return null;
  }
  const search = globalThis.location?.search;
  if (!search) {
    return null;
  }
  const params = new URLSearchParams(search);
  const spaceId = params.get("space_id");
  return shouldUseLocalDevSpaceBootstrap(spaceId) ? spaceId : null;
}

function buildSeedEvent(seed: LocalDevSeedSpec): PreviewGlobalEvent {
  return {
    id: `local-dev-seed:${seed.artifactId}`,
    type: "HeapBlockCreated",
    spaceId: seed.spaceId,
    timestamp: seed.emittedAt,
    payload: {
      artifactId: seed.artifactId,
      blockType: "structured_data",
      title: seed.title,
      emittedAt: seed.emittedAt,
      tags: seed.tags,
      attributes: {
        bootstrap: "localhost_dev",
        pilot_ready: "false",
      },
      content: {
        structured_data: {
          type: "agent_solicitation",
          role: "hermes.advisory",
          requested_agent_role: seed.requestedAgentRole,
          authority_scope: seed.authorityScope,
          review_outcome_mode: "record_only",
          required_capabilities: seed.requiredCapabilities,
          source_refs: seed.sourceRefs,
          summary: seed.summary,
          message: seed.recommendation,
          budget: {
            currency: "tokens",
            max: 4000,
          },
        },
      },
    },
  };
}

async function ensureLocalDevSeeded(spaceId: string): Promise<void> {
  if (!shouldUseLocalDevSpaceBootstrap(spaceId)) {
    return;
  }

  const existingEvents = await getEventsBySpace(spaceId);
  const existingArtifactIds = new Set(
    existingEvents
      .map((event) => event.payload.artifactId)
      .filter((value): value is string => typeof value === "string" && value.trim().length > 0),
  );

  const missingSeeds = LOCAL_DEV_HEAP_SEEDS.filter(
    (seed) => seed.spaceId === spaceId && !existingArtifactIds.has(seed.artifactId),
  );

  for (const seed of missingSeeds) {
    await appendEvent(buildSeedEvent(seed));
  }
}

export async function getLocalDevBootstrapHeapBlocks(spaceId: string): Promise<HeapBlockListItem[]> {
  await ensureLocalDevSeeded(spaceId);
  const events = await getEventsBySpace(spaceId);
  return reduceHeapBlocks(events).filter((block) => !block.deletedAt);
}

async function resolveSpaceIdForArtifact(artifactId: string): Promise<string | null> {
  for (const spaceId of LOCAL_DEV_SUPPORTED_SPACE_IDS) {
    const events = await getEventsBySpace(spaceId);
    if (events.some((event) => event.payload.artifactId === artifactId)) {
      return spaceId;
    }
  }
  return currentLocalDevSpaceIdFromLocation();
}

export async function submitLocalDevBootstrapFeedback(
  artifactId: string,
  feedbackData: A2UISubmitFeedbackRequest["feedbackData"],
  actorRole = "operator",
  actorId = "cortex-web",
): Promise<A2UISubmitFeedbackResponse> {
  const spaceId = await resolveSpaceIdForArtifact(artifactId);
  if (!spaceId || !shouldUseLocalDevSpaceBootstrap(spaceId)) {
    throw new Error(`No localhost bootstrap artifact found for ${artifactId}`);
  }

  const storedAt = new Date().toISOString();
  const decision =
    typeof feedbackData.decision === "string" && feedbackData.decision.trim().length > 0
      ? feedbackData.decision.trim().toLowerCase()
      : "recorded";
  const feedbackText =
    typeof feedbackData.feedback === "string" ? feedbackData.feedback : "";
  const feedbackArtifactId = `local-dev-feedback-${artifactId}-${Date.now()}`;

  await appendEvent({
    id: `local-dev-feedback:${feedbackArtifactId}`,
    type: "HeapBlockCreated",
    spaceId,
    timestamp: storedAt,
    payload: {
      artifactId: feedbackArtifactId,
      blockType: "structured_data",
      title: `Steward Feedback · ${decision}`,
      emittedAt: storedAt,
      tags: ["steward_feedback", "localhost_dev"],
      mentionsInline: [artifactId],
      pageLinks: [artifactId],
      attributes: {
        bootstrap: "localhost_dev",
        actor_role: actorRole,
      },
      content: {
        structured_data: {
          type: "steward_feedback",
          artifact_id: feedbackArtifactId,
          parent_artifact_id: artifactId,
          decision,
          feedback: feedbackText,
          submitted_by: actorId,
          submitted_at: storedAt,
        },
      },
    },
  });

  return {
    accepted: true,
    artifactId,
    feedbackArtifactId,
    storedAt,
    reviewOutcomeMode: "record_only",
  };
}
