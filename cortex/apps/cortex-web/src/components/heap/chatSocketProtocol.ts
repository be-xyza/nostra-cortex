export type ChatState = "idle" | "streaming" | "processing";

export interface ChatClientAttachmentDescriptor {
  name: string;
  type: string;
  size: number;
}

export interface ChatPanelMessage {
  id: string;
  role: "user" | "agent";
  text: string;
  timestamp: string;
  contextRefs?: string[];
  attachments?: ChatClientAttachmentDescriptor[];
}

export interface ChatPanelStateSnapshot {
  messages: ChatPanelMessage[];
  chatState: ChatState;
  error: string | null;
}

export interface ChatClientMessageEnvelope {
  type: "message";
  text: string;
  contextRefs?: string[];
  attachments?: ChatClientAttachmentDescriptor[];
  threadId?: string;
}

export type ChatServerEnvelope =
  | { type: "processing" }
  | { type: "streaming"; id: string; delta: string; timestamp: string }
  | { type: "message"; id: string; text: string; timestamp: string }
  | { type: "error"; code: string; message: string };

export function buildChatClientMessageEnvelope(input: {
  text: string;
  contextRefs?: string[];
  attachments?: ChatClientAttachmentDescriptor[];
  threadId?: string;
}): ChatClientMessageEnvelope {
  const envelope: ChatClientMessageEnvelope = {
    type: "message",
    text: input.text,
  };
  if (input.contextRefs?.length) {
    envelope.contextRefs = input.contextRefs;
  }
  if (input.attachments?.length) {
    envelope.attachments = input.attachments;
  }
  if (input.threadId?.trim()) {
    envelope.threadId = input.threadId.trim();
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
        messages[messages.length - 1] = {
          ...last,
          text: last.text + envelope.delta,
          timestamp: envelope.timestamp,
        };
      } else {
        messages.push({
          id: envelope.id,
          role: "agent",
          text: envelope.delta,
          timestamp: envelope.timestamp,
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
      const last = messages[messages.length - 1];
      if (last?.role === "agent" && last.id === envelope.id) {
        messages[messages.length - 1] = {
          ...last,
          text: envelope.text,
          timestamp: envelope.timestamp,
        };
      } else {
        messages.push({
          id: envelope.id,
          role: "agent",
          text: envelope.text,
          timestamp: envelope.timestamp,
        });
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
        error: envelope.message,
      };
  }
}
