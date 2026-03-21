import React, { useState } from "react";
import { useUiStore, type Comment } from "../../store/uiStore";

interface CommentSidebarProps {
    blockId: string;
    onClose: () => void;
}

const EMPTY_ARRAY: Comment[] = [];

export const CommentSidebar: React.FC<CommentSidebarProps> = ({ blockId, onClose }) => {
    const comments = useUiStore((state) => state.comments[blockId] || EMPTY_ARRAY);
    const addComment = useUiStore((state) => state.addComment);
    const sessionUser = useUiStore((state) => state.sessionUser);
    const [newComment, setNewComment] = useState("");

    const handleAddComment = () => {
        if (!newComment.trim()) return;
        const comment: Comment = {
            id: `comment-${Date.now()}`,
            author: sessionUser?.actorId || "anonymous",
            text: newComment.trim(),
            createdAt: new Date().toISOString(),
        };
        addComment(blockId, comment);
        setNewComment("");
    };

    return (
        <aside className="heap-comment-sidebar w-80 bg-slate-900/95 backdrop-blur-xl border-l border-slate-700/50 flex flex-col shrink-0 shadow-[-10px_0_30px_rgba(0,0,0,0.5)] animate-slide-left fixed right-0 top-0 h-full z-100 overflow-hidden text-slate-200">
            <div className="p-4 border-b border-slate-700/50 flex justify-between items-center bg-slate-800/30">
                <div className="flex items-center gap-2">
                    <span className="p-1.5 rounded-md bg-emerald-500/20 text-emerald-400">
                        <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 10h.01M12 10h.01M16 10h.01M9 16H5a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v8a2 2 0 01-2 2h-5l-5 5v-5z" />
                        </svg>
                    </span>
                    <div className="flex flex-col">
                        <h3 className="font-semibold text-sm">Discussions</h3>
                        <span className="text-[10px] text-slate-500 font-mono uppercase tracking-widest">Context Focus</span>
                    </div>
                </div>
                <button
                    onClick={onClose}
                    className="p-1.5 hover:bg-slate-700/50 rounded-md transition-colors text-slate-400 hover:text-slate-200"
                >
                    <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                    </svg>
                </button>
            </div>

            <div className="flex-1 overflow-y-auto p-4 custom-scrollbar space-y-4">
                {comments.length === 0 ? (
                    <div className="flex flex-col items-center justify-center h-48 text-slate-500 gap-2 opacity-60">
                        <svg className="w-8 h-8" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1} d="M17 8h2a2 2 0 012 2v6a2 2 0 01-2 2h-2v4l-4-4H9a1.994 1.994 0 01-1.414-.586m0 0L11 14h4a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2v4l.586-.586z" />
                        </svg>
                        <p className="text-xs">No comments yet.</p>
                    </div>
                ) : (
                    comments.map((c) => (
                        <div key={c.id} className="bg-slate-800/40 rounded-xl p-3 border border-slate-700/30 group hover:border-slate-600/50 transition-all">
                            <div className="flex justify-between items-center mb-2">
                                <span className="text-[10px] font-mono text-emerald-400 font-bold uppercase tracking-wider">
                                    {c.author}
                                </span>
                                <span className="text-[10px] text-slate-500">
                                    {new Date(c.createdAt).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
                                </span>
                            </div>
                            <p className="text-sm text-slate-300 leading-relaxed">
                                {c.text}
                            </p>
                        </div>
                    ))
                )}
            </div>

            <div className="p-4 border-t border-slate-700/50 bg-slate-900/50">
                <div className="relative">
                    <textarea
                        value={newComment}
                        onChange={(e) => setNewComment(e.target.value)}
                        placeholder="Write a comment..."
                        className="w-full bg-slate-950 border border-slate-700 rounded-xl p-3 text-sm focus:outline-none focus:ring-2 focus:ring-emerald-500/40 focus:border-emerald-500/50 resize-none min-h-[80px] custom-scrollbar"
                        onKeyDown={(e) => {
                            if (e.key === 'Enter' && !e.shiftKey) {
                                e.preventDefault();
                                handleAddComment();
                            }
                        }}
                    />
                    <button
                        onClick={handleAddComment}
                        disabled={!newComment.trim()}
                        className="absolute right-3 bottom-3 p-2 bg-emerald-600 hover:bg-emerald-500 disabled:opacity-50 disabled:hover:bg-emerald-600 text-white rounded-lg transition-all shadow-lg shadow-emerald-900/20"
                    >
                        <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8" />
                        </svg>
                    </button>
                </div>
                <p className="mt-2 text-[10px] text-slate-500 text-center">
                    Press Enter to send, Shift + Enter for new line • <b>System Intelligence</b> Active
                </p>
            </div>
        </aside>
    );
};
