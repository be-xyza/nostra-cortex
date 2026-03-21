import React from "react";
import { useLayoutPreferences } from "../../store/layoutPreferences";

/**
 * Floating toast bar that appears when the user has unsaved layout changes.
 * Stays visible at the bottom until the user commits or reverts.
 */
export function LayoutChangeToast() {
  const pendingSpaceId = useLayoutPreferences((s) => s.pendingSpaceId);
  const commitPending = useLayoutPreferences((s) => s.commitPending);
  const revertPending = useLayoutPreferences((s) => s.revertPending);

  if (!pendingSpaceId) return null;

  return (
    <div className="fixed bottom-6 left-1/2 -translate-x-1/2 z-[60] flex items-center gap-3 px-5 py-2.5 bg-cortex-950/95 border border-cortex-700/60 rounded-full backdrop-blur-xl shadow-2xl animate-in slide-in-from-bottom duration-300 pointer-events-auto">
      {/* Indicator dot */}
      <span className="w-2 h-2 rounded-full bg-amber-400 animate-pulse shadow-[0_0_8px_rgba(245,158,11,0.5)]" />

      <span className="text-[11px] font-bold text-cortex-200 tracking-wide">
        Layout modified
      </span>

      <div className="w-px h-4 bg-cortex-700/50" />

      <button
        onClick={commitPending}
        className="px-3 py-1 rounded-full text-[11px] font-black tracking-wider text-emerald-300 border border-emerald-500/30 bg-emerald-500/10 hover:bg-emerald-500/20 transition-all active:scale-95"
      >
        Save
      </button>

      <button
        onClick={revertPending}
        className="px-3 py-1 rounded-full text-[11px] font-black tracking-wider text-cortex-400 border border-cortex-700/40 bg-cortex-900/40 hover:bg-cortex-800/60 hover:text-cortex-200 transition-all active:scale-95"
      >
        Revert
      </button>
    </div>
  );
}
