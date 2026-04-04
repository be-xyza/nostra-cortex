import React, { useState, useRef, useEffect, useCallback, useMemo } from "react";
import { useNavigate } from "react-router-dom";
import {
    X,
    Send,
    Loader2,
    MessagesSquare,
    Maximize2,
    Minimize2,
    ArrowUpRight,
} from "lucide-react";
import { gatewayWsBase, resolveWorkbenchSpaceId, workbenchApi } from "../../api";
import { resolveChatHints, type ChatHint } from "./chatHintRegistry";
import {
    applyChatServerEnvelope,
    buildChatClientMessageEnvelope,
    buildUserChatPanelMessage,
    isChatServerEnvelope,
    type ChatPanelMessage,
    type ChatState,
} from "./chatSocketProtocol";
import type { HeapViewContextSnapshot } from "./heapViewRegistry.ts";
import { buildConversationSourceHref } from "../conversations/conversationRegistry.ts";
import { useConversationStore } from "../../store/conversationStore.ts";
import { A2UIInterpreter, type A2UINode } from "../a2ui/A2UIInterpreter";

interface ChatPanelProps {
    isOpen: boolean;
    onClose: () => void;
    /** Block IDs selected in the Heap grid — attached as context refs */
    contextBlockIds?: string[];
    /** Current active view mode for hint resolution */
    viewMode?: string;
    /** Optional derived heap view context for context-aware hints */
    heapViewContext?: HeapViewContextSnapshot | null;
    /** Initial thread ID to resume a conversation */
    threadId?: string;
    /** Gateway base URL for WebSocket */
    gatewayUrl?: string;
}

export function ChatPanel({
    isOpen,
    onClose,
    contextBlockIds = [],
    viewMode = "Explore",
    heapViewContext = null,
    threadId,
    gatewayUrl,
}: ChatPanelProps) {
    const [messages, setMessages] = useState<ChatPanelMessage[]>([]);
    const [input, setInput] = useState("");
    const [chatState, setChatState] = useState<ChatState>("idle");
    const [chatError, setChatError] = useState<string | null>(null);
    const [isExpanded, setIsExpanded] = useState(false);
    const messagesEndRef = useRef<HTMLDivElement | null>(null);
    const inputRef = useRef<HTMLTextAreaElement | null>(null);
    const wsRef = useRef<WebSocket | null>(null);
    const chatStateRef = useRef<ChatState>("idle");
    const chatErrorRef = useRef<string | null>(null);
    const navigate = useNavigate();
    const conversationRecord = useConversationStore((state) =>
        threadId?.trim()
            ? state.records.find((record) => record.threadId === threadId.trim()) ?? null
            : null,
    );
    const ensureConversation = useConversationStore((state) => state.ensureConversation);
    const upsertConversation = useConversationStore((state) => state.upsertConversation);
    const appendConversationTurn = useConversationStore((state) => state.appendConversationTurn);
    const resolvedGatewayUrl = useMemo(() => gatewayUrl?.trim() || gatewayWsBase(), [gatewayUrl]);

    // Resolve hints from registry based on context
    const hints = useMemo(
        () => resolveChatHints(viewMode, contextBlockIds.length, heapViewContext),
        [viewMode, contextBlockIds.length, heapViewContext],
    );

    const scrollToBottom = useCallback(() => {
        messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
    }, []);

    useEffect(() => {
        scrollToBottom();
    }, [messages, scrollToBottom]);

    useEffect(() => {
        if (isOpen) {
            setTimeout(() => inputRef.current?.focus(), 200);
        }
    }, [isOpen]);

    useEffect(() => {
        chatStateRef.current = chatState;
    }, [chatState]);

    useEffect(() => {
        chatErrorRef.current = chatError;
    }, [chatError]);

    useEffect(() => {
        setMessages([]);
        setChatState("idle");
        setChatError(null);
    }, [threadId]);

    useEffect(() => {
        if (!threadId?.trim()) return;
        let cancelled = false;
        workbenchApi
            .getChatConversation(threadId.trim())
            .then((conversation) => {
                if (cancelled) return;
                setMessages(
                    conversation.messages.map((message) => ({
                        id: message.id,
                        role: message.role,
                        text: message.text,
                        content: message.content,
                        timestamp: message.timestamp,
                        agent: message.agent,
                    })),
                );
                upsertConversation({
                    threadId: conversation.threadId,
                    title: conversation.title,
                    anchor: conversation.anchor,
                    messageCount: conversation.messageCount,
                    lastMessagePreview: conversation.lastMessagePreview,
                    createdAt: conversation.createdAt,
                    updatedAt: conversation.updatedAt,
                    recentTurns: conversation.recentTurns,
                });
            })
            .catch(() => {
                if (cancelled) return;
                setMessages([]);
            });
        return () => {
            cancelled = true;
        };
    }, [threadId, upsertConversation]);

    useEffect(() => {
        if (!threadId?.trim()) return;
        ensureConversation({ threadId: threadId.trim() });
    }, [ensureConversation, threadId]);

    // WebSocket connection for streaming mode
    useEffect(() => {
        if (!isOpen || !resolvedGatewayUrl) return;

        const wsUrl = resolvedGatewayUrl
            .replace(/^http/, "ws")
            .replace(/\/$/, "");
        const ws = new WebSocket(`${wsUrl}/ws/chat${threadId ? `?thread=${threadId}` : ""}`);

        ws.onopen = () => {
            setChatState("idle");
            setChatError(null);
        };

        ws.onmessage = (event) => {
            try {
                const data = JSON.parse(event.data);
                if (!isChatServerEnvelope(data)) return;
                setMessages(prev => {
                    const snapshot = applyChatServerEnvelope(
                        {
                            messages: prev,
                            chatState: chatStateRef.current,
                            error: chatErrorRef.current,
                        },
                        data,
                    );
                    chatStateRef.current = snapshot.chatState;
                    chatErrorRef.current = snapshot.error;
                    setChatState(snapshot.chatState);
                    setChatError(snapshot.error);
                    return snapshot.messages;
                });
                if (data.type === "message" && threadId?.trim()) {
                    appendConversationTurn({
                        threadId: threadId.trim(),
                        role: "agent",
                        text: data.text,
                        timestamp: data.timestamp,
                    });
                }
            } catch {
                // Non-JSON messages ignored
            }
        };

        ws.onclose = () => {
            chatStateRef.current = "idle";
            setChatState("idle");
        };

        wsRef.current = ws;
        return () => {
            ws.close();
            wsRef.current = null;
        };
    }, [appendConversationTurn, isOpen, resolvedGatewayUrl, threadId]);

    const sendMessage = useCallback((overrideText?: string) => {
        const trimmed = (overrideText ?? input).trim();
        if (!trimmed) return;

        const userMsg: ChatPanelMessage = buildUserChatPanelMessage({
            id: `user-${Date.now()}`,
            text: trimmed,
            timestamp: new Date().toISOString(),
            contextRefs: contextBlockIds.length > 0 ? contextBlockIds : undefined,
        });

        setMessages(prev => [...prev, userMsg]);
        if (threadId?.trim()) {
            appendConversationTurn({
                threadId: threadId.trim(),
                role: "user",
                text: userMsg.text,
                timestamp: userMsg.timestamp,
            });
        }
        setInput("");
        setChatError(null);

        if (wsRef.current?.readyState === WebSocket.OPEN) {
            wsRef.current.send(
                JSON.stringify(
                    buildChatClientMessageEnvelope({
                        text: trimmed,
                        threadId,
                        spaceId: resolveWorkbenchSpaceId(),
                        contextBlockIds: contextBlockIds,
                        sourceAnchor: conversationRecord?.anchor ?? null,
                    }),
                )
            );
            chatStateRef.current = "processing";
            setChatState("processing");
        } else {
            // Async fallback
            chatStateRef.current = "processing";
            setChatState("processing");
            setTimeout(() => {
                setMessages(prev => [
                    ...prev,
                        {
                            id: `agent-${Date.now()}`,
                            role: "agent",
                            text: "Eudaemon is working on this. The result will appear in your Inbox when ready.",
                            content: [{
                                type: "text",
                                text: "Eudaemon is working on this. The result will appear in your Inbox when ready.",
                            }],
                            timestamp: new Date().toISOString(),
                        },
                ]);
                if (threadId?.trim()) {
                    appendConversationTurn({
                        threadId: threadId.trim(),
                        role: "agent",
                        text: "Eudaemon is working on this. The result will appear in your Inbox when ready.",
                        timestamp: new Date().toISOString(),
                    });
                }
                chatStateRef.current = "idle";
                setChatState("idle");
            }, 1500);
        }
    }, [appendConversationTurn, contextBlockIds, conversationRecord?.anchor, input, threadId]);

    const handleHintClick = useCallback((hint: ChatHint) => {
        if (hint.prompt) {
            setInput(hint.prompt);
            setTimeout(() => inputRef.current?.focus(), 50);
        }
    }, []);

    const handleKeyDown = (e: React.KeyboardEvent) => {
        if (e.key === "Enter" && !e.shiftKey) {
            e.preventDefault();
            sendMessage();
        }
    };

    if (!isOpen) return null;

    const panelWidth = isExpanded ? "w-[480px]" : "w-[360px]";

    return (
        <div
            className={`chat-panel fixed right-0 top-0 bottom-0 ${panelWidth} z-50 bg-cortex-surface-panel/95 backdrop-blur-2xl border-l border-white/5 flex flex-col shadow-2xl transition-all duration-300`}
        >
            {/* Header */}
            <div className="flex items-center gap-3 px-4 py-3 border-b border-white/5 shrink-0">
                <MessagesSquare className="w-4 h-4 text-indigo-400" />
                <div className="min-w-0 flex-1">
                    <span className="block text-xs font-bold text-cortex-50 tracking-tight">
                        Chat
                    </span>
                    {conversationRecord && (
                        <div className="mt-0.5 truncate text-[10px] text-cortex-500">
                            {conversationRecord.title}
                            {conversationRecord.anchor ? ` · ${conversationRecord.anchor.label}` : ""}
                        </div>
                    )}
                </div>
                {contextBlockIds.length > 0 && (
                    <span className="text-[9px] font-mono bg-indigo-500/10 text-indigo-300 px-2 py-0.5 rounded-full">
                        {contextBlockIds.length} context
                    </span>
                )}
                {conversationRecord?.anchor && (
                    <button
                        onClick={() => navigate(buildConversationSourceHref(conversationRecord.anchor!))}
                        className="p-1 rounded hover:bg-white/5 text-cortex-500 hover:text-cortex-300 transition-colors"
                        title="Open source"
                    >
                        <ArrowUpRight className="w-3.5 h-3.5" />
                    </button>
                )}
                <button
                    onClick={() => setIsExpanded(!isExpanded)}
                    className="p-1 rounded hover:bg-white/5 text-cortex-500 hover:text-cortex-300 transition-colors"
                >
                    {isExpanded ? (
                        <Minimize2 className="w-3.5 h-3.5" />
                    ) : (
                        <Maximize2 className="w-3.5 h-3.5" />
                    )}
                </button>
                <button
                    onClick={onClose}
                    className="p-1 rounded hover:bg-white/5 text-cortex-500 hover:text-cortex-300 transition-colors"
                >
                    <X className="w-4 h-4" />
                </button>
            </div>

            {/* Messages */}
            <div className="flex-1 overflow-y-auto p-4 space-y-3 custom-scrollbar">
                {chatError && (
                    <div className={`rounded-xl px-3 py-2 text-[11px] ${
                        chatError.includes("temporarily unavailable")
                            ? "border border-amber-400/30 bg-amber-400/10 text-amber-100"
                            : "border border-rose-500/30 bg-rose-500/10 text-rose-200"
                    }`}>
                        {chatError}
                        {chatError.includes("icp-cli") ? (
                            <div className="mt-2 text-[10px] text-amber-50/80">
                                Keep the canonical `icp-cli` lane available on PATH and keep terminal color disabled for local command execution.
                            </div>
                        ) : null}
                    </div>
                )}
                {messages.length === 0 && (
                    <div className="flex flex-col items-center justify-center h-full text-center gap-4">
                        <MessagesSquare className="w-8 h-8 text-cortex-700" />
                        <p className="text-xs text-cortex-500">
                            Ask Eudaemon anything, or make a request.
                            {contextBlockIds.length > 0 && (
                                <span className="block mt-1 text-indigo-400/60">
                                    {contextBlockIds.length} block{contextBlockIds.length === 1 ? "" : "s"} resolved as canonical context.
                                </span>
                            )}
                        </p>
                        {/* Hint chips — data-driven from registry */}
                        {hints.length > 0 && (
                            <div className="flex flex-wrap justify-center gap-1.5 max-w-[280px]">
                                {hints.map(hint => (
                                    <button
                                        key={hint.id}
                                        onClick={() => handleHintClick(hint)}
                                        className="text-[10px] px-2.5 py-1.5 rounded-full bg-cortex-surface-base/60 border border-white/5 text-cortex-400 hover:text-cortex-200 hover:border-indigo-500/20 hover:bg-indigo-500/5 transition-all"
                                    >
                                        {hint.label}
                                    </button>
                                ))}
                            </div>
                        )}
                    </div>
                )}
                {messages.map((msg) => (
                    <div
                        key={msg.id}
                        className={`flex ${msg.role === "user" ? "justify-end" : "justify-start"}`}
                        >
                            <div
                                className={`max-w-[85%] px-3 py-2 rounded-xl text-xs leading-relaxed ${
                                msg.role === "user"
                                    ? "bg-blue-500/10 text-blue-100 rounded-br-sm"
                                    : "bg-cortex-surface-base/60 text-cortex-300 border border-white/5 rounded-bl-sm"
                            }`}
                        >
                            <ChatMessageBody message={msg} />
                            {msg.agent ? (
                                <div className="mt-2 text-[9px] uppercase tracking-[0.24em] text-cortex-500">
                                    {msg.agent.label}
                                </div>
                            ) : null}
                            <span className="block text-[9px] mt-1 opacity-40 font-mono">
                                {formatChatTime(msg.timestamp)}
                            </span>
                        </div>
                    </div>
                ))}

                {chatState === "processing" && (
                    <div className="flex justify-start">
                        <div className="bg-cortex-surface-base/60 border border-white/5 rounded-xl rounded-bl-sm px-3 py-2 flex items-center gap-2">
                            <Loader2 className="w-3 h-3 text-indigo-400 animate-spin" />
                            <span className="text-[10px] text-cortex-500">Eudaemon is working...</span>
                        </div>
                    </div>
                )}

                <div ref={messagesEndRef} />
            </div>

            {/* Input */}
            <div className="shrink-0 border-t border-white/5 p-3">
                <div className="flex items-end gap-2 bg-cortex-surface-base/40 rounded-xl border border-white/5 p-2">
                    <textarea
                        ref={inputRef}
                        value={input}
                        onChange={(e) => setInput(e.target.value)}
                        onKeyDown={handleKeyDown}
                        placeholder="Message Eudaemon..."
                        rows={1}
                        className="flex-1 bg-transparent text-xs text-cortex-50 placeholder-cortex-600 resize-none focus:outline-none max-h-24 leading-relaxed py-1"
                    />
                    <button
                        onClick={() => sendMessage()}
                        disabled={!input.trim() || chatState === "processing"}
                        className="p-1.5 text-blue-400 hover:text-blue-300 disabled:text-cortex-700 transition-colors shrink-0"
                    >
                        <Send className="w-3.5 h-3.5" />
                    </button>
                </div>
            </div>
        </div>
    );
}

function ChatMessageBody({ message }: { message: ChatPanelMessage }) {
    const content = message.content.length > 0
        ? message.content
        : [{ type: "text", text: message.text } as const];

    return (
        <div className="space-y-2">
            {content.map((part, index) => {
                if (part.type === "text") {
                    return (
                        <p key={`${message.id}:text:${index}`} className="whitespace-pre-wrap">
                            {part.text}
                        </p>
                    );
                }
                if (part.type === "a2ui") {
                    const node = normalizeA2UINode(part.tree);
                    if (!node) {
                        return null;
                    }
                    return (
                        <div
                            key={`${message.id}:a2ui:${index}`}
                            className="overflow-hidden rounded-lg border border-white/10 bg-black/10 p-2"
                        >
                            <A2UIInterpreter node={node} />
                        </div>
                    );
                }
                return (
                    <a
                        key={`${message.id}:pointer:${index}`}
                        href={part.href}
                        className="block rounded-lg border border-cyan-400/20 bg-cyan-400/8 px-3 py-2 text-[11px] text-cyan-100 hover:bg-cyan-400/14"
                    >
                        <span className="block font-semibold">{part.label}</span>
                        {part.description ? (
                            <span className="mt-1 block text-cyan-100/70">{part.description}</span>
                        ) : null}
                    </a>
                );
            })}
        </div>
    );
}

function normalizeA2UINode(value: unknown): A2UINode | null {
    if (!value || typeof value !== "object") {
        return null;
    }
    const maybeNode = value as Record<string, unknown>;
    if (Array.isArray(maybeNode.components)) {
        return {
            type: "Container",
            componentProperties: { layout: "vertical" },
            children: {
                explicitList: (maybeNode.components as unknown[]).filter(Boolean) as A2UINode[],
            },
        };
    }
    return value as A2UINode;
}

function formatChatTime(iso: string): string {
    try {
        return new Date(iso).toLocaleTimeString([], {
            hour: "2-digit",
            minute: "2-digit",
        });
    } catch {
        return "";
    }
}
