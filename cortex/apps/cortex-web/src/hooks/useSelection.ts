import { useState, useCallback } from "react";

export interface SelectionHook<T> {
  selectedIds: string[];
  lastSelectedId: string | null;
  setSelectedIds: React.Dispatch<React.SetStateAction<string[]>>;
  /** Standardized click handler for selection logic */
  handleSelection: (event: React.MouseEvent | React.KeyboardEvent, id: string, allItems: T[], getId: (item: T) => string) => void;
  /** Clear selection */
  clearSelection: () => void;
  /** Check if id is selected */
  isSelected: (id: string) => boolean;
}

/**
 * Standardized hook for OS-level selection paradigms (Single, Shift+Click, Cmd/Ctrl+Click).
 * Principally aligned for pattern reuse across Spaces, Blocks, and Lists.
 */
export function useSelection<T>(initialSelected?: string[]): SelectionHook<T> {
  const [selectedIds, setSelectedIds] = useState<string[]>(initialSelected || []);
  const [lastSelectedId, setLastSelectedId] = useState<string | null>(null);

  const clearSelection = useCallback(() => {
    setSelectedIds([]);
    setLastSelectedId(null);
  }, []);

  const isSelected = useCallback((id: string) => selectedIds.includes(id), [selectedIds]);

  const handleSelection = useCallback(
    (event: React.MouseEvent | React.KeyboardEvent, id: string, allItems: T[], getId: (item: T) => string) => {
      const isShift = event.shiftKey;
      const isCmd = event.metaKey || event.ctrlKey;

      if (isCmd) {
        // Toggle item
        setSelectedIds((prev) => {
          const next = prev.includes(id) ? prev.filter((i) => i !== id) : [...prev, id];
          setLastSelectedId(id);
          return next;
        });
      } else if (isShift && lastSelectedId) {
        // Range select
        const allIds = allItems.map(getId);
        const startIdx = allIds.indexOf(lastSelectedId);
        const endIdx = allIds.indexOf(id);

        if (startIdx !== -1 && endIdx !== -1) {
          const start = Math.min(startIdx, endIdx);
          const end = Math.max(startIdx, endIdx);
          const rangeIds = allIds.slice(start, end + 1);

          setSelectedIds((prev) => {
            const nextSet = new Set([...prev, ...rangeIds]);
            return Array.from(nextSet);
          });
          // Note: Shift click doesn't update lastSelectedId in standard Finder behavior to allow expanding from same pivot,
          // but updating it as target is also common. We'll keep pivot stable if shift is held.
        }
      } else {
        // Single select (replace)
        setSelectedIds([id]);
        setLastSelectedId(id);
      }
    },
    [lastSelectedId]
  );

  return {
    selectedIds,
    lastSelectedId,
    setSelectedIds,
    handleSelection,
    clearSelection,
    isSelected,
  };
}
