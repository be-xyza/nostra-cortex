import { useCallback, useEffect, useMemo } from "react";
import { useNavigate, useSearchParams } from "react-router-dom";
import { ArrowUpRight, MessageSquarePlus, PanelRightOpen, RotateCcw } from "lucide-react";

import { workbenchApi } from "../../api";
import { buildConversationSourceHref } from "./conversationRegistry.ts";
import { createConversationThreadId, useConversationStore } from "../../store/conversationStore.ts";

function appendThreadParam(href: string, threadId: string): string {
  const url = new URL(href, "https://cortex.local");
  url.searchParams.set("thread", threadId);
  return `${url.pathname}${url.search}${url.hash}`;
}

function formatDateTime(value: string): string {
  return new Intl.DateTimeFormat(undefined, {
    dateStyle: "medium",
    timeStyle: "short",
  }).format(new Date(value));
}

export function ConversationsPage() {
  const navigate = useNavigate();
  const [searchParams, setSearchParams] = useSearchParams();
  const records = useConversationStore((state) => state.records);
  const activeThreadId = useConversationStore((state) => state.activeThreadId);
  const setActiveThreadId = useConversationStore((state) => state.setActiveThreadId);
  const setRecords = useConversationStore((state) => state.setRecords);
  const createConversation = useConversationStore((state) => state.createConversation);
  const ensureConversation = useConversationStore((state) => state.ensureConversation);

  const routeThreadId = searchParams.get("thread")?.trim() || null;
  const selectedThreadId = routeThreadId || activeThreadId || records[0]?.threadId || null;

  useEffect(() => {
    let cancelled = false;
    workbenchApi
      .listChatConversations()
      .then((response) => {
        if (cancelled) return;
        setRecords(response.items);
      })
      .catch(() => {
        if (cancelled) return;
      });
    return () => {
      cancelled = true;
    };
  }, [setRecords]);

  useEffect(() => {
    if (!selectedThreadId) return;
    setActiveThreadId(selectedThreadId);
  }, [selectedThreadId, setActiveThreadId]);

  useEffect(() => {
    if (!routeThreadId || records.some((record) => record.threadId === routeThreadId)) {
      return;
    }
    ensureConversation({ threadId: routeThreadId });
  }, [ensureConversation, records, routeThreadId]);

  const selectedRecord = useMemo(
    () => records.find((record) => record.threadId === selectedThreadId) ?? null,
    [records, selectedThreadId],
  );
  const selectedAnchor = selectedRecord?.anchor ?? null;

  const selectThread = useCallback(
    (threadId: string) => {
      setActiveThreadId(threadId);
      const next = new URLSearchParams(searchParams);
      next.set("thread", threadId);
      setSearchParams(next, { replace: true });
    },
    [searchParams, setActiveThreadId, setSearchParams],
  );

  const startNewConversation = useCallback(() => {
    const record = createConversation({
      threadId: createConversationThreadId(),
      title: "New conversation",
    });
    navigate(`/explore?thread=${encodeURIComponent(record.threadId)}`);
  }, [createConversation, navigate]);

  const openSource = useCallback((threadId: string, href: string) => {
    setActiveThreadId(threadId);
    navigate(href);
  }, [navigate, setActiveThreadId]);

  const resumeInSource = useCallback((threadId: string, href: string) => {
    setActiveThreadId(threadId);
    navigate(appendThreadParam(href, threadId));
  }, [navigate, setActiveThreadId]);

  return (
    <div className="min-h-[calc(100vh-88px)] bg-[radial-gradient(circle_at_top_left,rgba(59,130,246,0.16),transparent_28%),radial-gradient(circle_at_bottom_right,rgba(168,85,247,0.14),transparent_30%),linear-gradient(180deg,rgba(3,8,20,0.96),rgba(5,11,25,1))] text-slate-100">
      <div className="mx-auto flex min-h-[calc(100vh-88px)] w-full max-w-[1640px] flex-col gap-6 px-4 py-6 md:px-6">
        <div className="flex flex-col gap-4 rounded-[28px] border border-white/8 bg-white/[0.03] px-6 py-6 shadow-[0_24px_90px_-48px_rgba(0,0,0,0.85)] backdrop-blur-xl md:flex-row md:items-end md:justify-between">
          <div className="max-w-3xl">
            <div className="flex items-center gap-2 text-[11px] font-black uppercase tracking-[0.32em] text-cyan-300/80">
              <PanelRightOpen className="h-4 w-4" />
              Conversations
            </div>
            <h1 className="mt-3 text-3xl font-semibold tracking-tight text-white md:text-5xl">
              Resume conversations without losing the source.
            </h1>
            <p className="mt-3 max-w-2xl text-sm leading-6 text-slate-300/90 md:text-base">
              Threads are stored as working context. Each conversation keeps its anchor to the originating page,
              view, block, or component so we can return there explicitly when the discussion is done.
            </p>
          </div>
          <button
            type="button"
            onClick={startNewConversation}
            className="inline-flex items-center justify-center gap-2 rounded-full border border-cyan-400/30 bg-cyan-400/10 px-4 py-2.5 text-sm font-semibold text-cyan-100 shadow-[0_10px_30px_-16px_rgba(34,211,238,0.65)] transition hover:bg-cyan-400/18 hover:text-white"
          >
            <MessageSquarePlus className="h-4 w-4" />
            New conversation
          </button>
        </div>

        <div className="grid min-h-0 flex-1 gap-5 lg:grid-cols-[360px_minmax(0,1fr)]">
          <aside className="min-h-0 overflow-hidden rounded-[28px] border border-white/8 bg-white/[0.025] shadow-[0_24px_90px_-48px_rgba(0,0,0,0.8)]">
            <div className="border-b border-white/8 px-5 py-4">
              <div className="flex items-center justify-between">
                <h2 className="text-xs font-black uppercase tracking-[0.3em] text-slate-400">Recent threads</h2>
                <span className="rounded-full border border-white/10 bg-white/[0.04] px-2.5 py-1 text-[10px] font-semibold text-slate-300">
                  {records.length}
                </span>
              </div>
            </div>
            <div className="max-h-[calc(100vh-240px)] overflow-y-auto p-3">
              {records.length === 0 ? (
                <div className="rounded-[24px] border border-dashed border-white/10 bg-white/[0.02] px-4 py-8 text-sm leading-6 text-slate-300">
                  No saved threads yet. Start one from Explore or open a conversation from a view or block to make it resumable.
                </div>
              ) : (
                <div className="space-y-2">
                  {records.map((record) => {
                    const isActive = record.threadId === selectedThreadId;
                    return (
                      <button
                        key={record.threadId}
                        type="button"
                        onClick={() => selectThread(record.threadId)}
                        className={[
                          "w-full rounded-[22px] border px-4 py-4 text-left transition",
                          isActive
                            ? "border-cyan-400/30 bg-cyan-400/10 shadow-[0_18px_46px_-24px_rgba(34,211,238,0.45)]"
                            : "border-white/8 bg-white/[0.02] hover:border-white/12 hover:bg-white/[0.04]",
                        ].join(" ")}
                      >
                        <div className="flex items-start justify-between gap-3">
                          <div>
                            <div className="text-sm font-semibold text-white">{record.title}</div>
                            <div className="mt-1 text-[11px] uppercase tracking-[0.22em] text-slate-400">
                              {record.anchor?.kind ?? "conversation"}
                              {record.anchor?.label ? ` · ${record.anchor.label}` : ""}
                            </div>
                          </div>
                          <span className="rounded-full bg-white/[0.05] px-2 py-1 text-[10px] font-semibold text-slate-300">
                            {record.messageCount}
                          </span>
                        </div>
                        <p className="mt-3 line-clamp-2 text-sm leading-6 text-slate-300">
                          {record.lastMessagePreview || "No captured turns yet."}
                        </p>
                        <div className="mt-3 text-[11px] text-slate-500">
                          Updated {formatDateTime(record.updatedAt)}
                        </div>
                      </button>
                    );
                  })}
                </div>
              )}
            </div>
          </aside>

          <section className="min-h-0 overflow-hidden rounded-[28px] border border-white/8 bg-[linear-gradient(180deg,rgba(11,18,35,0.92),rgba(5,11,24,0.95))] shadow-[0_24px_90px_-48px_rgba(0,0,0,0.82)]">
            {!selectedRecord ? (
              <div className="flex h-full min-h-[480px] flex-col items-start justify-center px-8 py-8">
                <div className="max-w-xl">
                  <div className="text-xs font-black uppercase tracking-[0.32em] text-slate-400">No conversation selected</div>
                  <h2 className="mt-3 text-2xl font-semibold text-white">Pick a thread or create a new one.</h2>
                  <p className="mt-3 text-sm leading-6 text-slate-300">
                    Conversations keep their source anchor so we can return to the exact page, view, or block that started them.
                  </p>
                </div>
              </div>
            ) : (
              <div className="flex h-full min-h-0 flex-col">
                <div className="flex items-start justify-between gap-4 border-b border-white/8 px-6 py-5">
                  <div>
                    <div className="text-[11px] font-black uppercase tracking-[0.32em] text-slate-500">Selected thread</div>
                    <h2 className="mt-2 text-2xl font-semibold text-white">{selectedRecord.title}</h2>
                    <div className="mt-2 flex flex-wrap items-center gap-2 text-xs text-slate-400">
                      <span className="rounded-full border border-white/10 bg-white/[0.04] px-2.5 py-1">
                        {selectedRecord.threadId}
                      </span>
                      {selectedRecord.anchor && (
                        <span className="rounded-full border border-cyan-400/20 bg-cyan-400/10 px-2.5 py-1 text-cyan-100">
                          {selectedRecord.anchor.kind}: {selectedRecord.anchor.label}
                        </span>
                      )}
                    </div>
                  </div>
                  <div className="flex flex-wrap gap-2">
                    {selectedAnchor && (
                      <button
                        type="button"
                        onClick={() => openSource(selectedRecord.threadId, buildConversationSourceHref(selectedAnchor))}
                        className="inline-flex items-center gap-2 rounded-full border border-white/10 bg-white/[0.04] px-4 py-2 text-sm font-medium text-slate-200 transition hover:bg-white/[0.08]"
                      >
                        <ArrowUpRight className="h-4 w-4" />
                        Open source
                      </button>
                    )}
                      <button
                        type="button"
                        onClick={() =>
                          resumeInSource(
                            selectedRecord.threadId,
                            selectedAnchor ? buildConversationSourceHref(selectedAnchor) : "/explore",
                          )
                        }
                      className="inline-flex items-center gap-2 rounded-full border border-cyan-400/25 bg-cyan-400/12 px-4 py-2 text-sm font-semibold text-cyan-50 transition hover:bg-cyan-400/20"
                    >
                      <RotateCcw className="h-4 w-4" />
                      Resume in source
                    </button>
                  </div>
                </div>

                <div className="grid min-h-0 flex-1 gap-5 overflow-hidden p-6 xl:grid-cols-[minmax(0,1fr)_320px]">
                  <div className="min-h-0 overflow-y-auto rounded-[24px] border border-white/8 bg-white/[0.02] p-5">
                    <div className="text-xs font-black uppercase tracking-[0.3em] text-slate-500">Recent turns</div>
                    <div className="mt-4 space-y-3">
                      {selectedRecord.recentTurns.length === 0 ? (
                        <div className="rounded-[20px] border border-dashed border-white/10 bg-white/[0.02] px-4 py-6 text-sm text-slate-300">
                          This thread has no captured turns yet. Open it in the chat dialogue and send a message to begin tracking.
                        </div>
                      ) : (
                        selectedRecord.recentTurns.map((turn, index) => (
                          <div
                            key={`${turn.timestamp}-${index}`}
                            className={[
                              "rounded-[20px] border px-4 py-4",
                              turn.role === "agent"
                                ? "border-emerald-400/15 bg-emerald-400/6"
                                : "border-cyan-400/15 bg-cyan-400/6",
                            ].join(" ")}
                          >
                            <div className="flex items-center justify-between gap-4">
                              <div className="text-[11px] font-black uppercase tracking-[0.26em] text-slate-500">
                                {turn.role}
                              </div>
                              <div className="text-[11px] text-slate-500">{formatDateTime(turn.timestamp)}</div>
                            </div>
                            <div className="mt-3 whitespace-pre-wrap text-sm leading-6 text-slate-100">
                              {turn.text}
                            </div>
                          </div>
                        ))
                      )}
                    </div>
                  </div>

                  <aside className="min-h-0 space-y-4 overflow-y-auto">
                    <div className="rounded-[24px] border border-white/8 bg-white/[0.02] px-4 py-4">
                      <div className="text-[11px] font-black uppercase tracking-[0.3em] text-slate-500">Source</div>
                      <div className="mt-3 text-lg font-semibold text-white">
                        {selectedRecord.anchor?.label ?? "Conversation source unavailable"}
                      </div>
                      <p className="mt-2 text-sm leading-6 text-slate-300">
                        {selectedRecord.anchor
                          ? `This thread can return to the ${selectedRecord.anchor.kind} it came from.`
                          : "This thread does not yet have a saved source anchor."}
                      </p>
                    </div>

                    <div className="rounded-[24px] border border-white/8 bg-white/[0.02] px-4 py-4">
                      <div className="text-[11px] font-black uppercase tracking-[0.3em] text-slate-500">Thread details</div>
                      <dl className="mt-3 space-y-3 text-sm text-slate-300">
                        <div>
                          <dt className="text-[11px] uppercase tracking-[0.26em] text-slate-500">Created</dt>
                          <dd className="mt-1">{formatDateTime(selectedRecord.createdAt)}</dd>
                        </div>
                        <div>
                          <dt className="text-[11px] uppercase tracking-[0.26em] text-slate-500">Updated</dt>
                          <dd className="mt-1">{formatDateTime(selectedRecord.updatedAt)}</dd>
                        </div>
                        <div>
                          <dt className="text-[11px] uppercase tracking-[0.26em] text-slate-500">Messages</dt>
                          <dd className="mt-1">{selectedRecord.messageCount}</dd>
                        </div>
                      </dl>
                    </div>
                  </aside>
                </div>
              </div>
            )}
          </section>
        </div>
      </div>
    </div>
  );
}
