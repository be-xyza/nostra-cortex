import { useState, useCallback, useRef, type DragEvent } from "react";

export interface SortableItem<T = unknown> {
  id: string;
  data: T;
}

interface UseSortableListReturn<T> {
  /** Items in their current (possibly reordered) order. */
  orderedItems: SortableItem<T>[];
  /** Whether any drag operation has changed the order. */
  isDirty: boolean;
  /** Current dragged item ID (for styling). */
  draggedId: string | null;
  /** Drop-target item ID (for styling). */
  overId: string | null;
  /** Attach to each sortable container element. */
  getItemProps: (id: string) => {
    draggable: true;
    onDragStart: (e: DragEvent) => void;
    onDragOver: (e: DragEvent) => void;
    onDragEnter: (e: DragEvent) => void;
    onDragLeave: (e: DragEvent) => void;
    onDrop: (e: DragEvent) => void;
    onDragEnd: (e: DragEvent) => void;
    "data-sortable-id": string;
  };
  /** Programmatically reset to the original order. */
  reset: () => void;
  /** Get the current ID ordering. */
  getOrder: () => string[];
}

/**
 * Lightweight HTML5 DnD hook for reordering a list.
 *
 * Usage:
 * ```tsx
 * const { orderedItems, getItemProps, isDirty } = useSortableList(items);
 * return orderedItems.map(item => (
 *   <div key={item.id} {...getItemProps(item.id)}>{item.data.label}</div>
 * ));
 * ```
 */
export function useSortableList<T>(
  items: SortableItem<T>[],
  onReorder?: (newOrder: string[]) => void,
): UseSortableListReturn<T> {
  const [order, setOrder] = useState<string[]>(() => items.map((i) => i.id));
  const [draggedId, setDraggedId] = useState<string | null>(null);
  const [overId, setOverId] = useState<string | null>(null);
  const originalRef = useRef<string[]>(items.map((i) => i.id));

  // Keep original in sync if items change externally
  if (items.length !== originalRef.current.length || items.some((item, i) => item.id !== originalRef.current[i])) {
    const newIds = items.map((i) => i.id);
    originalRef.current = newIds;
    setOrder(newIds);
  }

  const itemMap = new Map(items.map((item) => [item.id, item]));

  const orderedItems: SortableItem<T>[] = order
    .filter((id) => itemMap.has(id))
    .map((id) => itemMap.get(id)!);

  // Append any new items not yet in order
  for (const item of items) {
    if (!order.includes(item.id)) {
      orderedItems.push(item);
    }
  }

  const isDirty = order.some((id, i) => id !== originalRef.current[i]);

  const moveItem = useCallback(
    (fromId: string, toId: string) => {
      if (fromId === toId) return;
      setOrder((prev) => {
        const next = [...prev];
        const fromIndex = next.indexOf(fromId);
        const toIndex = next.indexOf(toId);
        if (fromIndex === -1 || toIndex === -1) return prev;
        next.splice(fromIndex, 1);
        next.splice(toIndex, 0, fromId);
        onReorder?.(next);
        return next;
      });
    },
    [onReorder],
  );

  const getItemProps = useCallback(
    (id: string) => ({
      draggable: true as const,
      "data-sortable-id": id,
      onDragStart: (e: DragEvent) => {
        setDraggedId(id);
        e.dataTransfer.effectAllowed = "move";
        e.dataTransfer.setData("text/plain", id);
        // Slight delay for the visual ghost
        requestAnimationFrame(() => {
          const el = e.currentTarget as HTMLElement;
          el.style.opacity = "0.4";
        });
      },
      onDragOver: (e: DragEvent) => {
        e.preventDefault();
        e.dataTransfer.dropEffect = "move";
      },
      onDragEnter: (e: DragEvent) => {
        e.preventDefault();
        setOverId(id);
      },
      onDragLeave: (_e: DragEvent) => {
        setOverId((current) => (current === id ? null : current));
      },
      onDrop: (e: DragEvent) => {
        e.preventDefault();
        const fromId = e.dataTransfer.getData("text/plain");
        if (fromId && fromId !== id) {
          moveItem(fromId, id);
        }
        setOverId(null);
        setDraggedId(null);
      },
      onDragEnd: (e: DragEvent) => {
        (e.currentTarget as HTMLElement).style.opacity = "1";
        setDraggedId(null);
        setOverId(null);
      },
    }),
    [moveItem],
  );

  const reset = useCallback(() => {
    setOrder([...originalRef.current]);
    onReorder?.(originalRef.current);
  }, [onReorder]);

  const getOrder = useCallback(() => [...order], [order]);

  return {
    orderedItems,
    isDirty,
    draggedId,
    overId,
    getItemProps,
    reset,
    getOrder,
  };
}
