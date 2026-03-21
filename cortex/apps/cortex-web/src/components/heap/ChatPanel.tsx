import React, { useState, useRef, useEffect, useCallback, useMemo } from "react";
import {
    X,
    Send,
    Loader2,
    MessagesSquare,
    Paperclip,
    Maximize2,
    Minimize2,
    ImageIcon,
} from "lucide-react";
import { resolveChatHints, type ChatHint } from "./chatHintRegistry";

interface ChatAttachment {
    file: File;
    previewUrl?: string;
}

interface ChatMessage {
    id: string;
    role: "user" | "agent";
    text: string;
    timestamp: string;
    contextRefs?: string[];
    attachments?: Array<{ name: string; type: string; size: number }>;
}

interface ChatPanelProps {
    isOpen: boolean;
    onClose: () => void;
    /** Block IDs selected in the Heap grid — attached as context refs */
    contextBlockIds?: string[];
    /** Current active view mode for hint resolution */
    viewMode?: string;
    /** Initial thread ID to resume a conversation */
    threadId?: string;
    /** Gateway base URL for WebSocket */
    gatewayUrl?: string;
}

type ChatState = "idle" | "streaming" | "processing";

export function ChatPanel({
    isOpen,
    onClose,
    contextBlockIds = [],
    viewMode = "Explore",
    threadId,
    gatewayUrl,
}: ChatPanelProps) {
    const [messages, setMessages] = useState<ChatMessage[]>([]);
    const [input, setInput] = useState("");
    const [chatState, setChatState] = useState<ChatState>("idle");
    const [isExpanded, setIsExpanded] = useState(false);
    const [attachments, setAttachments] = useState<ChatAttachment[]>([]);
    const messagesEndRef = useRef<HTMLDivElement | null>(null);
    const inputRef = useRef<HTMLTextAreaElement | null>(null);
    const fileInputRef = useRef<HTMLInputElement | null>(null);
    const wsRef = useRef<WebSocket | null>(null);

    // Resolve hints from registry based on context
    const hints = useMemo(
        () => resolveChatHints(viewMode, contextBlockIds.length),
        [viewMode, contextBlockIds.length],
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

    // WebSocket connection for streaming mode
    useEffect(() => {
        if (!isOpen || !gatewayUrl) return;

        const wsUrl = gatewayUrl
            .replace(/^http/, "ws")
            .replace(/\/$/, "");
        const ws = new WebSocket(`${wsUrl}/ws/chat${threadId ? `?thread=${threadId}` : ""}`);

        ws.onopen = () => {
            setChatState("idle");
        };

        ws.onmessage = (event) => {
            try {
                const data = JSON.parse(event.data);
                if (data.type === "message") {
                    setMessages(prev => [
                        ...prev,
                        {
                            id: data.id ?? `agent-${Date.now()}`,
                            role: "agent",
                            text: data.text,
                            timestamp: data.timestamp ?? new Date().toISOString(),
                        },
                    ]);
                    setChatState("idle");
                } else if (data.type === "processing") {
                    setChatState("processing");
                } else if (data.type === "streaming") {
                    setChatState("streaming");
                    setMessages(prev => {
                        const last = prev[prev.length - 1];
                        if (last?.role === "agent" && last.id.startsWith("stream-")) {
                            return [
                                ...prev.slice(0, -1),
                                { ...last, text: last.text + (data.delta ?? "") },
                            ];
                        }
                        return [
                            ...prev,
                            {
                                id: `stream-${Date.now()}`,
                                role: "agent",
                                text: data.delta ?? "",
                                timestamp: new Date().toISOString(),
                            },
                        ];
                    });
                }
            } catch {
                // Non-JSON messages ignored
            }
        };

        ws.onclose = () => {
            setChatState("idle");
        };

        wsRef.current = ws;
        return () => {
            ws.close();
            wsRef.current = null;
        };
    }, [isOpen, gatewayUrl, threadId]);

    // File attachment handler
    const handleFileSelect = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
        const files = e.target.files;
        if (!files) return;

        const newAttachments: ChatAttachment[] = [];
        for (let i = 0; i < files.length; i++) {
            const file = files[i];
            const attachment: ChatAttachment = { file };
            if (file.type.startsWith("image/")) {
                attachment.previewUrl = URL.createObjectURL(file);
            }
            newAttachments.push(attachment);
        }
        setAttachments(prev => [...prev, ...newAttachments]);
        // Reset input so re-selecting the same file works
        e.target.value = "";
    }, []);

    const removeAttachment = useCallback((idx: number) => {
        setAttachments(prev => {
            const removed = prev[idx];
            if (removed.previewUrl) URL.revokeObjectURL(removed.previewUrl);
            return prev.filter((_, i) => i !== idx);
        });
    }, []);

    // Cleanup preview URLs on unmount
    useEffect(() => {
        return () => {
            attachments.forEach(a => {
                if (a.previewUrl) URL.revokeObjectURL(a.previewUrl);
            });
        };
    // eslint-disable-next-line react-hooks/exhaustive-deps
    }, []);

    const sendMessage = useCallback((overrideText?: string) => {
        const trimmed = (overrideText ?? input).trim();
        if (!trimmed && attachments.length === 0) return;

        const userMsg: ChatMessage = {
            id: `user-${Date.now()}`,
            role: "user",
            text: trimmed || (attachments.length > 0 ? `[${attachments.length} file${attachments.length > 1 ? "s" : ""} attached]` : ""),
            timestamp: new Date().toISOString(),
            contextRefs: contextBlockIds.length > 0 ? contextBlockIds : undefined,
            attachments: attachments.map(a => ({
                name: a.file.name,
                type: a.file.type,
                size: a.file.size,
            })),
        };

        setMessages(prev => [...prev, userMsg]);
        setInput("");
        setAttachments([]);

        if (wsRef.current?.readyState === WebSocket.OPEN) {
            wsRef.current.send(
                JSON.stringify({
                    type: "message",
                    text: trimmed,
                    contextRefs: userMsg.contextRefs,
                    attachments: userMsg.attachments,
                })
            );
            setChatState("processing");
        } else {
            // Async fallback
            setChatState("processing");
            setTimeout(() => {
                setMessages(prev => [
                    ...prev,
                    {
                        id: `agent-${Date.now()}`,
                        role: "agent",
                        text: "Eudaemon is working on this. The result will appear in your Inbox when ready.",
                        timestamp: new Date().toISOString(),
                    },
                ]);
                setChatState("idle");
            }, 1500);
        }
    }, [input, contextBlockIds, attachments]);

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
                <span className="text-xs font-bold text-cortex-50 tracking-tight flex-1">
                    Chat
                </span>
                {contextBlockIds.length > 0 && (
                    <span className="text-[9px] font-mono bg-indigo-500/10 text-indigo-300 px-2 py-0.5 rounded-full">
                        {contextBlockIds.length} context
                    </span>
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
                {messages.length === 0 && (
                    <div className="flex flex-col items-center justify-center h-full text-center gap-4">
                        <MessagesSquare className="w-8 h-8 text-cortex-700" />
                        <p className="text-xs text-cortex-500">
                            Ask Eudaemon anything, or make a request.
                            {contextBlockIds.length > 0 && (
                                <span className="block mt-1 text-indigo-400/60">
                                    {contextBlockIds.length} block{contextBlockIds.length === 1 ? "" : "s"} attached as context.
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
                            <p className="whitespace-pre-wrap">{msg.text}</p>
                            {msg.attachments && msg.attachments.length > 0 && (
                                <div className="flex flex-wrap gap-1 mt-1.5">
                                    {msg.attachments.map((a, i) => (
                                        <span key={i} className="text-[9px] bg-white/5 px-1.5 py-0.5 rounded text-cortex-500 flex items-center gap-1">
                                            {a.type.startsWith("image/") ? <ImageIcon className="w-2.5 h-2.5" /> : <Paperclip className="w-2.5 h-2.5" />}
                                            {a.name}
                                        </span>
                                    ))}
                                </div>
                            )}
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

            {/* Attachment preview strip */}
            {attachments.length > 0 && (
                <div className="shrink-0 border-t border-white/5 px-3 py-2 flex flex-wrap gap-2">
                    {attachments.map((a, i) => (
                        <div key={i} className="relative group/att">
                            {a.previewUrl ? (
                                <img
                                    src={a.previewUrl}
                                    alt={a.file.name}
                                    className="w-14 h-14 rounded-lg object-cover border border-white/10"
                                />
                            ) : (
                                <div className="w-14 h-14 rounded-lg bg-cortex-surface-base/60 border border-white/10 flex flex-col items-center justify-center">
                                    <Paperclip className="w-3.5 h-3.5 text-cortex-500" />
                                    <span className="text-[7px] text-cortex-600 mt-0.5 truncate max-w-[48px]">{a.file.name}</span>
                                </div>
                            )}
                            <button
                                onClick={() => removeAttachment(i)}
                                className="absolute -top-1 -right-1 w-4 h-4 rounded-full bg-red-500/80 text-white flex items-center justify-center text-[8px] opacity-0 group-hover/att:opacity-100 transition-opacity"
                            >
                                ×
                            </button>
                        </div>
                    ))}
                </div>
            )}

            {/* Input */}
            <div className="shrink-0 border-t border-white/5 p-3">
                <div className="flex items-end gap-2 bg-cortex-surface-base/40 rounded-xl border border-white/5 p-2">
                    <button
                        onClick={() => fileInputRef.current?.click()}
                        className="p-1.5 text-cortex-600 hover:text-cortex-400 transition-colors shrink-0"
                        title="Attach file or screenshot"
                    >
                        <Paperclip className="w-3.5 h-3.5" />
                    </button>
                    <input
                        ref={fileInputRef}
                        type="file"
                        multiple
                        accept="image/*,.pdf,.md,.txt,.json,.csv"
                        onChange={handleFileSelect}
                        className="hidden"
                    />
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
                        disabled={(!input.trim() && attachments.length === 0) || chatState === "processing"}
                        className="p-1.5 text-blue-400 hover:text-blue-300 disabled:text-cortex-700 transition-colors shrink-0"
                    >
                        <Send className="w-3.5 h-3.5" />
                    </button>
                </div>
            </div>
        </div>
    );
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
