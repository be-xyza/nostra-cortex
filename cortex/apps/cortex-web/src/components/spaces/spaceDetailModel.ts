import type { HeapBlockListItem } from "../../contracts.ts";
import { buildHeapArtifactHref } from "../heap/heapArtifactRouting.ts";
import type { Space } from "../../store/spacesRegistry.ts";

export interface SpaceDetailRow {
  label: string;
  value: string;
}

export interface SpaceDetailPerson {
  id: string;
  name: string;
  roleLabel: string;
}

export interface SpaceRecentWorkItem {
  label: string;
  value: string;
  href?: string;
  actionLabel?: string;
}

export interface SpaceDetailModel {
  statusLabel: string;
  statusMessage: string;
  aboutRows: SpaceDetailRow[];
  people: SpaceDetailPerson[];
  recentWork: SpaceRecentWorkItem[];
}

function titleCaseWords(value: string): string {
  return value
    .split(/[\s_-]+/)
    .filter(Boolean)
    .map((segment) => segment.charAt(0).toUpperCase() + segment.slice(1))
    .join(" ");
}

export function formatSpaceMemberName(memberId: string): string {
  if (memberId === "systems-steward") {
    return "Systems Steward";
  }
  if (memberId.startsWith("agent:")) {
    return titleCaseWords(memberId.slice("agent:".length));
  }
  return titleCaseWords(memberId);
}

export function describeSpaceMemberRole(memberId: string, owner?: string): string {
  if (owner && memberId === owner) {
    return "Owner";
  }
  if (memberId.startsWith("agent:")) {
    return "Agent";
  }
  return "Member";
}

export function formatSpaceCreatedAt(createdAt?: string): string {
  if (!createdAt) {
    return "Not available yet";
  }

  const parsed = new Date(createdAt);
  if (Number.isNaN(parsed.getTime())) {
    return createdAt;
  }

  return new Intl.DateTimeFormat("en-US", {
    month: "long",
    day: "numeric",
    year: "numeric",
  }).format(parsed);
}

export function buildPromotionReceiptRecentWorkItem(
  receipt?: HeapBlockListItem | null,
): SpaceRecentWorkItem | null {
  if (!receipt || receipt.projection.blockType !== "space_promotion_receipt") {
    return null;
  }

  const createdAt = formatSpaceCreatedAt(receipt.projection.emittedAt || receipt.projection.updatedAt);
  return {
    label: "Created from draft",
    value: `This space was created from a saved draft on ${createdAt}.`,
  };
}

export function buildAgentExecutionRecentWorkItem(
  executionRecord?: HeapBlockListItem | null,
): SpaceRecentWorkItem | null {
  if (!executionRecord || executionRecord.projection.blockType !== "agent_execution_record") {
    return null;
  }

  const createdAt = formatSpaceCreatedAt(
    executionRecord.projection.emittedAt || executionRecord.projection.updatedAt,
  );
  const benchmark = (
    executionRecord.surfaceJson as {
      benchmark?: {
        overall_grade?: string;
      };
    }
  ).benchmark;
  const grade = String(benchmark?.overall_grade ?? "").trim().toUpperCase();

  const value =
    grade === "FAIL"
      ? `Eudaemon last reviewed this space on ${createdAt} and flagged that it needs attention.`
      : `Eudaemon last reviewed this space on ${createdAt} and shared a new update.`;

  return {
    label: "Latest Eudaemon update",
    value,
  };
}

export function buildProposalReviewRecentWorkItem(
  proposal?: HeapBlockListItem | null,
): SpaceRecentWorkItem | null {
  if (!proposal || proposal.projection.blockType !== "proposal") {
    return null;
  }

  return {
    label: "Needs review",
    value: "A new recommendation is ready for review in this space.",
    href: buildHeapArtifactHref(proposal.projection.artifactId),
    actionLabel: "Open",
  };
}

function describeDraftLineage(space: Space): string | null {
  const lineage = space.metadata?.lineage;
  if (!lineage) {
    return null;
  }

  if (lineage.note?.trim()) {
    return lineage.note.trim();
  }
  if (lineage.sourceMode === "template") {
    return "Started from a saved template draft.";
  }
  if (lineage.sourceMode === "reference") {
    return "Started from a referenced draft.";
  }
  if (lineage.draftId) {
    return "Started from a saved draft.";
  }
  return null;
}

function describeSpaceAccess(space: Space, peopleCount: number): string {
  const governanceScope = space.metadata?.governance?.scope;
  if (governanceScope === "personal") {
    return "Personal space for the owner";
  }
  if (governanceScope === "private") {
    return peopleCount > 1
      ? `Private space for ${peopleCount} people or agents`
      : "Private space for approved members";
  }
  if (governanceScope === "public") {
    return "Public space with broader shared access";
  }
  return peopleCount > 1
    ? `${peopleCount} people or agents can work here`
    : "Only approved members can work here";
}

export function buildSpaceDetailModel(space: Space): SpaceDetailModel {
  const isActive = (space.status ?? "").toLowerCase() === "active";
  const statusLabel = isActive ? "Active" : "Needs attention";
  const archetypeLabel = space.archetype ? `${space.archetype.toLowerCase()} work` : "work";
  const ownerId = space.owner || "systems-steward";
  const peopleIds = Array.from(
    new Set([ownerId, ...(space.members?.length ? space.members : [])]),
  );
  const aboutRows: SpaceDetailRow[] = [
    { label: "Purpose", value: space.description || `A space for ${archetypeLabel}.` },
    { label: "Owner", value: formatSpaceMemberName(ownerId) },
    { label: "Created", value: formatSpaceCreatedAt(space.createdAt) },
    {
      label: "Access",
      value: describeSpaceAccess(space, peopleIds.length),
    },
  ];
  const lineageSummary = describeDraftLineage(space);
  if (lineageSummary) {
    aboutRows.push({ label: "Started from", value: lineageSummary });
  }

  const people = peopleIds.map((memberId) => ({
    id: memberId,
    name: formatSpaceMemberName(memberId),
    roleLabel: describeSpaceMemberRole(memberId, space.owner),
  }));

  const agentPeople = people.filter((person) => person.roleLabel === "Agent");
  const recentWork: SpaceRecentWorkItem[] =
    agentPeople.length > 0
      ? [
          {
            label: "Connected agent",
            value: `${agentPeople[0]?.name} can publish updates in this space.`,
          },
          {
            label: "Latest work",
            value: "Open this space to see the newest notes, proposals, and agent updates.",
          },
        ]
      : [
          {
            label: "Latest work",
            value: "Open this space to see the newest notes and updates.",
          },
        ];

  return {
    statusLabel,
    statusMessage: isActive
      ? "This space is active and ready to open."
      : "This space needs attention before new work should happen here.",
    aboutRows,
    people,
    recentWork,
  };
}
