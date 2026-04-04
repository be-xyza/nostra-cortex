const MAX_PREVIEW_LENGTH = 140;

export type ConversationAnchorKind = "page" | "view" | "block" | "component";

export interface ConversationAnchor {
  kind: ConversationAnchorKind;
  label: string;
  href: string;
  routeId?: string;
  artifactId?: string;
  viewId?: string;
  blockId?: string;
  componentId?: string;
}

export interface ConversationTurn {
  role: "user" | "agent";
  text: string;
  timestamp: string;
}

export interface ConversationRecord {
  threadId: string;
  title: string;
  anchor: ConversationAnchor | null;
  messageCount: number;
  lastMessagePreview: string;
  createdAt: string;
  updatedAt: string;
  recentTurns: ConversationTurn[];
}

function normalizeWhitespace(value: string): string {
  return value.replace(/\s+/g, " ").trim();
}

function stripMarkdown(text: string): string {
  return text
    .replace(/```[\s\S]*?```/g, " ")
    .replace(/`([^`]+)`/g, "$1")
    .replace(/^#{1,6}\s+/gm, "")
    .replace(/^\s*[-*+]\s+/gm, "")
    .replace(/^\s*\d+\.\s+/gm, "")
    .replace(/>\s?/gm, "")
    .replace(/\[(.*?)\]\((.*?)\)/g, "$1")
    .replace(/[_~*]/g, "");
}

export function summarizeConversationText(text: string): string {
  const stripped = stripMarkdown(text);
  const normalized = normalizeWhitespace(stripped);
  if (normalized.length <= MAX_PREVIEW_LENGTH) {
    return normalized;
  }
  return `${normalized.slice(0, MAX_PREVIEW_LENGTH - 1).trimEnd()}…`;
}

export function buildConversationSourceHref(anchor: ConversationAnchor): string {
  return anchor.href.trim();
}

export function createConversationRecord(input: {
  threadId: string;
  title?: string;
  anchor?: ConversationAnchor | null;
  createdAt?: string;
}): ConversationRecord {
  const createdAt = input.createdAt ?? new Date().toISOString();
  const title = input.title?.trim() || input.anchor?.label?.trim() || "Conversation";
  return {
    threadId: input.threadId.trim(),
    title,
    anchor: input.anchor ?? null,
    messageCount: 0,
    lastMessagePreview: "",
    createdAt,
    updatedAt: createdAt,
    recentTurns: [],
  };
}

export function appendConversationTurn(
  record: ConversationRecord,
  turn: ConversationTurn,
): ConversationRecord {
  const preview = summarizeConversationText(turn.text);
  const recentTurns = [...record.recentTurns, turn].slice(-8);
  return {
    ...record,
    messageCount: record.messageCount + 1,
    lastMessagePreview: preview,
    updatedAt: turn.timestamp,
    recentTurns,
  };
}
