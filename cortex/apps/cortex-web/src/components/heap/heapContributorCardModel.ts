import type { HeapBlockListItem } from "../../contracts.ts";
import { summarizeHeapBlockText } from "./heapTextSummary.ts";

export type HeapContributorStatus = "Completed" | "Needs review" | "Suggestion" | "Evidence" | "System update";
export type HeapContributorSource = "Eudaemon" | "Runtime monitor" | "Operator review" | "System";
export type HeapContributorRelevance = "Actionable" | "Informational" | "Review recommended" | "Background";

export interface HeapContributorCardModel {
  displayTitle: string;
  plainSummary: string;
  friendlyTypeLabel: string;
  statusLabel: HeapContributorStatus;
  sourceLabel: HeapContributorSource;
  relevanceLabel: HeapContributorRelevance;
  digestable: boolean;
  technicalType: string;
}

export interface HeapContributorDetailModel {
  whatHappened: string;
  whyItMatters: string;
  nextStep: string;
}

const PLACEHOLDER_TITLE_PATTERN = /^(usage_report|self_optimization_proposal|agent_execution_record)\s+block$/i;
const PLACEHOLDER_SUMMARY_PATTERN = /^(usage report|self optimization proposal|agent execution record)\s+block$/i;

export function buildHeapContributorCardModel(block: Pick<HeapBlockListItem, "projection" | "surfaceJson">): HeapContributorCardModel {
  const blockType = String(block.projection.blockType || "note");
  const normalizedType = blockType.toLowerCase();
  const rawTitle = block.projection.title?.trim() || "";
  const rawSummary = summarizeHeapBlockText(block).trim();
  const evidenceLike = isEvidenceLike(block);

  if (normalizedType === "usage_report") {
    return {
      displayTitle: chooseTitle(rawTitle, "Activity summary"),
      plainSummary: chooseSummary(rawSummary, "System activity was recorded for this Space. Open details for the technical record."),
      friendlyTypeLabel: "Activity summary",
      statusLabel: "System update",
      sourceLabel: "Runtime monitor",
      relevanceLabel: "Background",
      digestable: true,
      technicalType: blockType,
    };
  }

  if (normalizedType === "self_optimization_proposal") {
    return {
      displayTitle: chooseTitle(rawTitle, "Suggested improvement"),
      plainSummary: chooseSummary(rawSummary, "A system improvement was suggested for review. Open details to inspect the proposal."),
      friendlyTypeLabel: "Suggested improvement",
      statusLabel: "Suggestion",
      sourceLabel: "Eudaemon",
      relevanceLabel: "Review recommended",
      digestable: true,
      technicalType: blockType,
    };
  }

  if (normalizedType === "agent_execution_record") {
    return {
      displayTitle: chooseTitle(rawTitle, "Agent work update"),
      plainSummary: chooseSummary(rawSummary, "Agent work was completed or recorded for this Space. Open details for the run record."),
      friendlyTypeLabel: "Agent work update",
      statusLabel: "Completed",
      sourceLabel: "Eudaemon",
      relevanceLabel: "Informational",
      digestable: true,
      technicalType: blockType,
    };
  }

  if (normalizedType === "eudaemon_evidence_note" || evidenceLike) {
    return {
      displayTitle: chooseTitle(rawTitle, "Evidence note"),
      plainSummary: cleanContributorSummary(rawSummary) || "Evidence was attached for review. Open details for provenance and supporting context.",
      friendlyTypeLabel: "Evidence note",
      statusLabel: "Evidence",
      sourceLabel: "Operator review",
      relevanceLabel: "Informational",
      digestable: false,
      technicalType: blockType,
    };
  }

  return {
    displayTitle: rawTitle || "Untitled update",
    plainSummary: cleanContributorSummary(rawSummary) || "No readable summary is available yet. Open details for the underlying record.",
    friendlyTypeLabel: titleCase(blockType.replace(/_/g, " ")),
    statusLabel: inferStatus(block),
    sourceLabel: inferSource(block),
    relevanceLabel: inferRelevance(block),
    digestable: false,
    technicalType: blockType,
  };
}

export function isPlaceholderHeapTitle(value: string): boolean {
  return PLACEHOLDER_TITLE_PATTERN.test(value.trim());
}

export function buildHeapContributorDetailModel(block: Pick<HeapBlockListItem, "projection" | "surfaceJson">): HeapContributorDetailModel {
  const card = buildHeapContributorCardModel(block);
  const type = card.technicalType.toLowerCase();

  if (type === "usage_report") {
    return {
      whatHappened: card.plainSummary,
      whyItMatters: "It gives a quick pulse on recent system activity without requiring contributors to inspect repeated runtime records.",
      nextStep: "Use it as background context, then focus review time on evidence notes, suggestions, and agent work updates.",
    };
  }

  if (type === "self_optimization_proposal") {
    return {
      whatHappened: card.plainSummary,
      whyItMatters: "It may point to a process or interface improvement that needs human judgment before it becomes part of the Space workflow.",
      nextStep: "Review the proposal, compare it against current priorities, and promote only the changes that are clearly useful.",
    };
  }

  if (type === "agent_execution_record") {
    return {
      whatHappened: card.plainSummary,
      whyItMatters: "It records work completed by an agent so contributors can understand recent activity without reading execution logs.",
      nextStep: "Check whether the work produced a useful artifact, needs review, or should be linked to a follow-up task.",
    };
  }

  if (card.statusLabel === "Evidence") {
    return {
      whatHappened: card.plainSummary,
      whyItMatters: "It preserves supporting context for a decision, publication, or review trail.",
      nextStep: "Confirm the evidence supports the surrounding work and add relationships if it should be connected to other records.",
    };
  }

  if (card.relevanceLabel === "Actionable" || card.relevanceLabel === "Review recommended") {
    return {
      whatHappened: card.plainSummary,
      whyItMatters: "It may affect what contributors should review, improve, or route next.",
      nextStep: "Decide whether this record needs action, discussion, or a clearer summary.",
    };
  }

  return {
    whatHappened: card.plainSummary,
    whyItMatters: "It adds context to the Space record and can help contributors understand recent activity.",
    nextStep: "Keep it for context, or link it to related work if it should be easier to find later.",
  };
}

function chooseTitle(rawTitle: string, fallback: string): string {
  if (!rawTitle || isPlaceholderHeapTitle(rawTitle)) {
    return fallback;
  }
  return rawTitle;
}

function chooseSummary(rawSummary: string, fallback: string): string {
  const cleaned = cleanContributorSummary(rawSummary);
  if (!cleaned || PLACEHOLDER_SUMMARY_PATTERN.test(cleaned)) {
    return fallback;
  }
  return cleaned;
}

function cleanContributorSummary(value: string): string | null {
  const normalized = value.replace(/\bverified operator principal\b/gi, "verified operator")
    .replace(/\bheap emission mode\b/gi, "publication flow")
    .replace(/\bbounded rich-text\b/gi, "single note")
    .replace(/\bpayload[_ ]type\b/gi, "content type")
    .replace(/\s+/g, " ")
    .trim();
  return normalized || null;
}

function isEvidenceLike(block: Pick<HeapBlockListItem, "projection" | "surfaceJson">): boolean {
  const title = block.projection.title?.toLowerCase() || "";
  const type = block.projection.blockType?.toLowerCase() || "";
  const tags = block.projection.tags || [];
  return type.includes("evidence")
    || title.includes("evidence")
    || title.includes("publication proof")
    || tags.some((tag) => tag.toLowerCase().includes("evidence"));
}

function inferStatus(block: Pick<HeapBlockListItem, "projection" | "surfaceJson">): HeapContributorStatus {
  const text = `${block.projection.title || ""} ${summarizeHeapBlockText(block)}`.toLowerCase();
  if (text.includes("review") || text.includes("proposal")) return "Needs review";
  if (text.includes("evidence") || text.includes("proof")) return "Evidence";
  if (text.includes("complete") || text.includes("completed")) return "Completed";
  return "System update";
}

function inferSource(block: Pick<HeapBlockListItem, "projection" | "surfaceJson">): HeapContributorSource {
  const text = `${block.projection.title || ""} ${summarizeHeapBlockText(block)}`.toLowerCase();
  if (text.includes("eudaemon")) return "Eudaemon";
  if (text.includes("operator") || text.includes("steward")) return "Operator review";
  return "System";
}

function inferRelevance(block: Pick<HeapBlockListItem, "projection" | "surfaceJson">): HeapContributorRelevance {
  const text = `${block.projection.title || ""} ${summarizeHeapBlockText(block)}`.toLowerCase();
  if (text.includes("needs review") || text.includes("proposal")) return "Review recommended";
  if (text.includes("task") || text.includes("action")) return "Actionable";
  return "Informational";
}

function titleCase(value: string): string {
  return value
    .split(/\s+/)
    .filter(Boolean)
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .join(" ");
}
