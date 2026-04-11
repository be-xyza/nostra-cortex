import React, { useMemo } from "react";

export interface A2UIBlock {
  id: string;
  type: string;
  title: string;
  accent?: string;
}

export interface SpatialHeapGridProps {
  blocks: A2UIBlock[];
  emptyStateLabel?: string;
}

interface BSPNode {
  id: string;
  type: 'split' | 'leaf';
  direction?: 'h' | 'v';
  left?: BSPNode;
  right?: BSPNode;
  blockId?: string;
}

export const SPATIAL_HEAP_GRID_PAGE_SIZE = 7;

function getWeight(n: BSPNode): number {
  if (n.type === 'leaf') return 1;
  return getWeight(n.left!) + getWeight(n.right!);
}

function buildPageTree(pageBlocks: A2UIBlock[], depth: number = 0): BSPNode {
  if (pageBlocks.length === 1) {
    return { id: pageBlocks[0].id, type: 'leaf', blockId: pageBlocks[0].id };
  }
  const mid = Math.floor(pageBlocks.length / 2);
  return {
    id: `split-${depth}-${pageBlocks[0].id}`,
    type: 'split',
    direction: depth % 2 === 0 ? 'v' : 'h',
    left: buildPageTree(pageBlocks.slice(0, mid), depth + 1),
    right: buildPageTree(pageBlocks.slice(mid), depth + 1)
  };
}

export function SpatialHeapGrid({
  blocks,
  emptyStateLabel = "No blocks to project in spatial BSP mode.",
}: SpatialHeapGridProps) {
  // Keep the renderer generic: callers provide already-shaped A2UI blocks and
  // this component only handles BSP packing for local Cortex experiments.
  const chunkedPages = useMemo(() => {
    if (blocks.length === 0) return [];
    const chunks: A2UIBlock[][] = [];
    for (let i = 0; i < blocks.length; i += SPATIAL_HEAP_GRID_PAGE_SIZE) {
      chunks.push(blocks.slice(i, i + SPATIAL_HEAP_GRID_PAGE_SIZE));
    }
    return chunks;
  }, [blocks]);

  const pageTrees = useMemo(() => {
    return chunkedPages.map(page => buildPageTree(page));
  }, [chunkedPages]);

  const renderBSPNode = (node: BSPNode, pageBlocks: A2UIBlock[]): React.ReactNode => {
    if (!node) return null;

    if (node.type === 'leaf') {
      const block = pageBlocks.find(b => b.id === node.blockId);
      if (!block) return null;
      return (
        <div key={block.id} className="flex flex-1 p-1 overflow-hidden transition-all duration-300 hover:scale-[1.01] hover:z-10 relative">
          <div 
            className="flex flex-col flex-1 min-w-0 bg-slate-900/40 backdrop-blur-md rounded-2xl border border-white/10 p-5 overflow-hidden shadow-2xl"
            style={{ borderTop: `4px solid ${block.accent || 'var(--ui-accent-blue)'}` }}
          >
            <div className="font-mono text-[10px] uppercase tracking-widest mb-2" style={{ color: block.accent || 'var(--ui-accent-blue)' }}>
              {block.type}
            </div>
            <div className="text-sm text-slate-200 leading-relaxed font-medium">
              {block.title}
            </div>
            <div className="flex-1" />
            <div className="h-8 flex mt-4 border-t border-white/5 pt-2 items-center text-[10px] text-slate-500 font-mono">
              ID: {block.id}
            </div>
          </div>
        </div>
      );
    }

    const leftWeight = getWeight(node.left!);
    const rightWeight = getWeight(node.right!);

    return (
      <div key={node.id} className={`flex flex-1 min-h-0 min-w-0 ${node.direction === 'v' ? 'flex-row' : 'flex-col'}`}>
        <div style={{ flex: leftWeight }} className="flex min-h-0 min-w-0">
          {renderBSPNode(node.left!, pageBlocks)}
        </div>
        <div style={{ flex: rightWeight }} className="flex min-h-0 min-w-0">
          {renderBSPNode(node.right!, pageBlocks)}
        </div>
      </div>
    );
  };

  if (blocks.length === 0) {
    return <div className="flex items-center justify-center h-full text-slate-500 font-mono">{emptyStateLabel}</div>;
  }

  return (
    <div className="flex-1 w-full h-full overflow-y-auto custom-scrollbar p-2" data-spatial-heap-grid="true">
      <div className="flex flex-col w-full h-full gap-4 pb-12">
        {pageTrees.map((tree, idx) => {
          const count = chunkedPages[idx].length;
          // Dynamically scale page height depending on the number of blocks to avoid stretching 1 block over the full screen
          const heightClass = count <= 2 ? 'min-h-[350px]' : count <= 4 ? 'min-h-[60vh] h-[60vh]' : 'min-h-[85vh] h-[85vh]';
          return (
            <div key={idx} className={`flex w-full ${heightClass}`}>
              {renderBSPNode(tree, chunkedPages[idx])}
            </div>
          );
        })}
      </div>
    </div>
  );
}
