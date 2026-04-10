import { HeapBlockListItem, HeapBlockProjection, Json } from '../contracts.ts';
import { GlobalEvent } from './eventStore.ts';

/**
 * Reducer for Heap Blocks
 * 
 * Takes a stream of GlobalEvents and projects them into the HeapBlockListItem[]
 * expected by the UI.
 */
export function reduceHeapBlocks(events: GlobalEvent[]): HeapBlockListItem[] {
  const blocksMap = new Map<string, HeapBlockListItem>();

  for (const event of events) {
    switch (event.type) {
      case 'HeapBlockCreated': {
        const payload = event.payload as any;
        const artifactId = payload.artifactId || payload.id;
        
        const projection: HeapBlockProjection = {
          artifactId,
          title: payload.title || 'Untitled',
          blockType: payload.type || payload.blockType || 'note',
          updatedAt: event.timestamp,
          emittedAt: event.timestamp || payload.emittedAt,
          tags: payload.tags || payload.relations?.tags?.map((t: any) => t.to_block_id) || [],
          mentionsInline: payload.mentionsInline || payload.relations?.mentions?.map((m: any) => m.to_block_id) || [],
          pageLinks: payload.pageLinks || payload.relations?.links?.map((l: any) => l.to_block_id) || [],
          attributes: payload.attributes || payload.block?.attributes || {},
        };

        // Robust surface extraction
        let surfaceJson = {};
        if (payload.content) {
          const content = payload.content;
          if (content.a2ui) surfaceJson = content.a2ui.tree || {};
          else if (content.media) surfaceJson = { payload_type: 'media', media: content.media };
          else if (content.task) surfaceJson = { payload_type: 'task', text: content.task };
          else if (content.structured_data) surfaceJson = { payload_type: 'structured_data', structured_data: content.structured_data };
          else if (content.rich_text) surfaceJson = { payload_type: 'note', text: content.rich_text.plain_text };
        } else if (payload.surfaceJson) {
          surfaceJson = payload.surfaceJson;
        }

        blocksMap.set(artifactId, {
          projection,
          surfaceJson,
        });
        break;
      }

      case 'HeapBlockUpdated': {
        const payload = event.payload as any;
        const artifactId = payload.artifactId;
        const existing = blocksMap.get(artifactId);
        
        if (existing) {
          existing.projection = {
            ...existing.projection,
            ...payload.projection,
            updatedAt: event.timestamp,
          };
          if (payload.surfaceJson) {
            existing.surfaceJson = payload.surfaceJson;
          }
        }
        break;
      }

      case 'HeapBlockPinned': {
        const payload = event.payload as any;
        const artifactId = payload.artifactId;
        const existing = blocksMap.get(artifactId);
        if (existing) {
          existing.pinnedAt = event.timestamp;
          existing.projection.updatedAt = event.timestamp;
        }
        break;
      }

      case 'HeapBlockDeleted': {
        const payload = event.payload as any;
        const artifactId = payload.artifactId;
        const existing = blocksMap.get(artifactId);
        if (existing) {
          existing.deletedAt = event.timestamp;
          existing.projection.updatedAt = event.timestamp;
        }
        break;
      }
    }
  }

  return Array.from(blocksMap.values());
}
