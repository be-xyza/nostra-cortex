import type {
  ChatAgentIdentity,
  ChatConversationAnchorContract,
  ChatMessagePart,
} from "../../contracts.ts";

export type ChatState = "idle" | "streaming" | "processing";

export interface ChatPanelMessage {
  id: string;
  role: "user" | "agent";
  text: string;
  timestamp: string;
  content: ChatMessagePart[];
  contextRefs?: string[];
  agent?: ChatAgentIdentity;
}

export interface ChatPanelStateSnapshot {
  messages: ChatPanelMessage[];
  chatState: ChatState;
  error: string | null;
}

export interface ChatClientMessageEnvelope {
  type: "message";
  content: ChatMessagePart[];
  threadId?: string;
  spaceId?: string;
  context?: {
    blockIds?: string[];
    sourceAnchor?: ChatConversationAnchorContract | null;
  };
}

export type ChatServerEnvelope =
  | { type: "processing"; agent?: ChatAgentIdentity }
  | { type: "streaming"; id: string; delta: string; timestamp: string; agent?: ChatAgentIdentity }
  | { type: "message"; id: string; text: string; timestamp: string; content: ChatMessagePart[]; agent?: ChatAgentIdentity }
  | { type: "error"; code: string; message: string };

function textPart(text: string): ChatMessagePart {
  return {
    type: "text",
    text,
  };
}

function extractMessageText(content: ChatMessagePart[], fallback = ""): string {
  const text = content
    .filter((part): part is Extract<ChatMessagePart, { type: "text" }> => part.type === "text")
    .map((part) => part.text)
    .join("\n\n")
    .trim();
  return text || fallback;
}

export function buildChatClientMessageEnvelope(input: {
  text: string;
  threadId?: string;
  spaceId?: string;
  contextBlockIds?: string[];
  sourceAnchor?: ChatConversationAnchorContract | null;
}): ChatClientMessageEnvelope {
  const trimmedText = input.text.trim();
  const envelope: ChatClientMessageEnvelope = {
    type: "message",
    content: trimmedText ? [textPart(trimmedText)] : [],
  };
  if (input.threadId?.trim()) {
    envelope.threadId = input.threadId.trim();
  }
  if (input.spaceId?.trim()) {
    envelope.spaceId = input.spaceId.trim();
  }
  if (input.contextBlockIds?.length || input.sourceAnchor) {
    envelope.context = {};
    if (input.contextBlockIds?.length) {
      envelope.context.blockIds = input.contextBlockIds;
    }
    if (input.sourceAnchor) {
      envelope.context.sourceAnchor = input.sourceAnchor;
    }
  }
  return envelope;
}

export function isChatServerEnvelope(value: unknown): value is ChatServerEnvelope {
  if (!value || typeof value !== "object") return false;
  const envelope = value as Record<string, unknown>;
  return (
    envelope.type === "processing" ||
    envelope.type === "streaming" ||
    envelope.type === "message" ||
    envelope.type === "error"
  );
}

export function normalizeChatErrorMessage(code: string, message: string): string {
  const normalized = message.trim();
  const lower = normalized.toLowerCase();

  if (
    code === "gateway_error" &&
    (lower.includes("coloroutofrange") ||
      lower.includes("terminal colors") ||
      lower.includes("local ic tooling failed"))
  ) {
    return "Local IC host is temporarily unavailable. This is a known IC tooling issue in this environment, not a heap/chat content failure. Use the canonical `icp-cli` lane for local command execution.";
  }

  return normalized;
}

export function applyChatServerEnvelope(
  snapshot: ChatPanelStateSnapshot,
  envelope: ChatServerEnvelope,
): ChatPanelStateSnapshot {
  switch (envelope.type) {
    case "processing":
      return {
        ...snapshot,
        chatState: "processing",
        error: null,
      };
    case "streaming": {
      const messages = [...snapshot.messages];
      const last = messages[messages.length - 1];
      if (last?.role === "agent" && last.id === envelope.id) {
        const nextContent = last.content.length > 0 ? [...last.content] : [textPart(last.text)];
        const firstTextPart = nextContent.find(
          (part): part is Extract<ChatMessagePart, { type: "text" }> => part.type === "text",
        );
        if (firstTextPart) {
          firstTextPart.text += envelope.delta;
        } else {
          nextContent.unshift(textPart(envelope.delta));
        }
        messages[messages.length - 1] = {
          ...last,
          text: extractMessageText(nextContent, last.text + envelope.delta),
          content: nextContent,
          timestamp: envelope.timestamp,
          agent: envelope.agent ?? last.agent,
        };
      } else {
        const content = [textPart(envelope.delta)];
        messages.push({
          id: envelope.id,
          role: "agent",
          text: envelope.delta,
          content,
          timestamp: envelope.timestamp,
          agent: envelope.agent,
        });
      }
      return {
        messages,
        chatState: "streaming",
        error: null,
      };
    }
    case "message": {
      const messages = [...snapshot.messages];
      const normalizedContent = envelope.content.length > 0 ? envelope.content : [textPart(envelope.text)];
      const nextMessage: ChatPanelMessage = {
        id: envelope.id,
        role: "agent",
        text: extractMessageText(normalizedContent, envelope.text),
        content: normalizedContent,
        timestamp: envelope.timestamp,
        agent: envelope.agent,
      };
      const last = messages[messages.length - 1];
      if (last?.role === "agent" && last.id === envelope.id) {
        messages[messages.length - 1] = nextMessage;
      } else {
        messages.push(nextMessage);
      }
      return {
        messages,
        chatState: "idle",
        error: null,
      };
    }
    case "error":
      return {
        ...snapshot,
        chatState: "idle",
        error: normalizeChatErrorMessage(envelope.code, envelope.message),
      };
  }
}

export function buildUserChatPanelMessage(input: {
  id: string;
  text: string;
  timestamp: string;
  contextRefs?: string[];
}): ChatPanelMessage {
  const text = input.text.trim();
  return {
    id: input.id,
    role: "user",
    text,
    content: text ? [textPart(text)] : [],
    timestamp: input.timestamp,
    contextRefs: input.contextRefs,
  };
}
