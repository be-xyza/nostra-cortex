import { create } from "zustand";

import {
  appendConversationTurn,
  createConversationRecord,
  summarizeConversationText,
  type ConversationAnchor,
  type ConversationRecord,
} from "../components/conversations/conversationRegistry.ts";

const STORAGE_KEY = "cortex.conversations.registry.v1";
const ACTIVE_THREAD_KEY = "cortex.conversations.activeThreadId";

function hasWindow(): boolean {
  return typeof window !== "undefined";
}

function readJson<T>(key: string, fallback: T): T {
  if (!hasWindow()) return fallback;
  try {
    const raw = window.localStorage.getItem(key);
    if (!raw) return fallback;
    return JSON.parse(raw) as T;
  } catch {
    return fallback;
  }
}

function writeJson(key: string, value: unknown): void {
  if (!hasWindow()) return;
  try {
    window.localStorage.setItem(key, JSON.stringify(value));
  } catch {
    // Ignore storage failures in restricted or private browsing modes.
  }
}

function sortRecords(records: ConversationRecord[]): ConversationRecord[] {
  return [...records].sort((left, right) => {
    const updatedOrder = right.updatedAt.localeCompare(left.updatedAt);
    return updatedOrder !== 0 ? updatedOrder : right.threadId.localeCompare(left.threadId);
  });
}

function generateThreadId(): string {
  const crypto = globalThis.crypto as Crypto | undefined;
  if (crypto?.randomUUID) {
    return `thread-${crypto.randomUUID()}`;
  }
  return `thread-${Date.now()}-${Math.floor(Math.random() * 10000)}`;
}

function deriveTitleFromTurn(text: string): string {
  const summary = summarizeConversationText(text);
  return summary.length > 0 ? summary.slice(0, 72) : "Conversation";
}

function mergeConversationRecord(
  record: ConversationRecord,
  input: CreateConversationInput,
): ConversationRecord {
  const nextTitle =
    input.title?.trim() ||
    (record.title === "Conversation"
      ? input.anchor?.label?.trim() || record.anchor?.label?.trim() || record.title
      : record.title);
  return {
    ...record,
    title: nextTitle,
    anchor: record.anchor ?? input.anchor ?? null,
  };
}

function persistState(records: ConversationRecord[], activeThreadId: string | null): void {
  writeJson(STORAGE_KEY, records);
  if (activeThreadId) {
    writeJson(ACTIVE_THREAD_KEY, activeThreadId);
  } else if (hasWindow()) {
    try {
      window.localStorage.removeItem(ACTIVE_THREAD_KEY);
    } catch {
      // Ignore storage failures.
    }
  }
}

function loadInitialRecords(): ConversationRecord[] {
  return sortRecords(readJson<ConversationRecord[]>(STORAGE_KEY, []));
}

function loadInitialActiveThreadId(): string | null {
  const value = readJson<string | null>(ACTIVE_THREAD_KEY, null);
  return value?.trim() ? value.trim() : null;
}

export interface CreateConversationInput {
  threadId?: string;
  title?: string;
  anchor?: ConversationAnchor | null;
  createdAt?: string;
}

export interface AppendConversationTurnInput {
  threadId: string;
  role: "user" | "agent";
  text: string;
  timestamp?: string;
}

interface ConversationStoreState {
  records: ConversationRecord[];
  activeThreadId: string | null;
  getConversation: (threadId: string) => ConversationRecord | null;
  setRecords: (records: ConversationRecord[]) => void;
  setActiveThreadId: (threadId: string | null) => void;
  upsertConversation: (record: ConversationRecord) => ConversationRecord;
  createConversation: (input?: CreateConversationInput) => ConversationRecord;
  ensureConversation: (input: CreateConversationInput) => ConversationRecord;
  appendConversationTurn: (input: AppendConversationTurnInput) => ConversationRecord;
}

function upsertRecord(records: ConversationRecord[], record: ConversationRecord): ConversationRecord[] {
  const next = records.filter((entry) => entry.threadId !== record.threadId);
  next.push(record);
  return sortRecords(next);
}

function resolveConversationRecord(
  records: ConversationRecord[],
  input: CreateConversationInput,
): ConversationRecord {
  const threadId = input.threadId?.trim() || generateThreadId();
  const existing = records.find((record) => record.threadId === threadId);
  if (existing) {
    return mergeConversationRecord(existing, input);
  }
  return createConversationRecord({
    threadId,
    title: input.title,
    anchor: input.anchor ?? null,
    createdAt: input.createdAt,
  });
}

export const useConversationStore = create<ConversationStoreState>((set, get) => ({
  records: loadInitialRecords(),
  activeThreadId: loadInitialActiveThreadId(),

  getConversation: (threadId) => get().records.find((record) => record.threadId === threadId) ?? null,

  setRecords: (records) => {
    const nextRecords = sortRecords(records);
    set((state) => {
      const activeThreadId =
        state.activeThreadId && nextRecords.some((record) => record.threadId === state.activeThreadId)
          ? state.activeThreadId
          : nextRecords[0]?.threadId ?? null;
      persistState(nextRecords, activeThreadId);
      return {
        records: nextRecords,
        activeThreadId,
      };
    });
  },

  setActiveThreadId: (threadId) => {
    const normalized = threadId?.trim() || null;
    set({ activeThreadId: normalized });
    persistState(get().records, normalized);
  },

  upsertConversation: (record) => {
    let nextRecord = record;
    set((state) => {
      const records = upsertRecord(state.records, nextRecord);
      persistState(records, state.activeThreadId);
      return { records };
    });
    return nextRecord;
  },

  createConversation: (input = {}) => {
    const nextRecord = resolveConversationRecord(get().records, input);
    set((state) => {
      const records = upsertRecord(state.records, nextRecord);
      const activeThreadId = nextRecord.threadId;
      persistState(records, activeThreadId);
      return {
        records,
        activeThreadId,
      };
    });
    return nextRecord;
  },

  ensureConversation: (input = {}) => {
    const existing = input.threadId?.trim()
      ? get().records.find((record) => record.threadId === input.threadId?.trim())
      : null;
    if (existing) {
      const merged = mergeConversationRecord(existing, input);
      if (merged !== existing) {
        set((state) => {
          const records = upsertRecord(state.records, merged);
          persistState(records, state.activeThreadId);
          return { records };
        });
      }
      return merged;
    }
    return get().createConversation(input);
  },

  appendConversationTurn: (input) => {
    const timestamp = input.timestamp ?? new Date().toISOString();
    let nextRecord: ConversationRecord | null = null;
    set((state) => {
      const current = state.records.find((record) => record.threadId === input.threadId.trim());
      const baseRecord = current ?? createConversationRecord({
        threadId: input.threadId,
        title: input.role === "user" ? deriveTitleFromTurn(input.text) : undefined,
        createdAt: timestamp,
      });
      const withTurn = appendConversationTurn(baseRecord, {
        role: input.role,
        text: input.text,
        timestamp,
      });
      if (withTurn.title === "Conversation" && input.role === "user") {
        withTurn.title = deriveTitleFromTurn(input.text);
      }
      nextRecord = withTurn;
      const records = upsertRecord(state.records, withTurn);
      persistState(records, state.activeThreadId ?? withTurn.threadId);
      return {
        records,
        activeThreadId: state.activeThreadId ?? withTurn.threadId,
      };
    });
    return nextRecord ?? createConversationRecord({ threadId: input.threadId, createdAt: timestamp });
  },
}));

export function createConversationThreadId(): string {
  return generateThreadId();
}
