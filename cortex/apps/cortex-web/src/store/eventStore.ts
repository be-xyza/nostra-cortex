import { openDB, DBSchema, IDBPDatabase } from 'idb';
import { SEED_EVENTS, INTRO_SPACE_ID, MOCK_WHOAMI, MOCK_LAYOUT_SPEC, MOCK_NAVIGATION_PLAN, MOCK_UX_WORKBENCH_MAIN, MOCK_UX_WORKBENCH_LABS, MOCK_UX_WORKBENCH_EXECUTION_CANVAS, MOCK_UX_WORKBENCH_SYSTEM, MOCK_UX_WORKBENCH_SPACES, MOCK_UX_WORKBENCH_HEAP, MOCK_UX_WORKBENCH_STUDIO } from './seedData.ts';
import { useUserPreferences } from './userPreferences.ts';
import { PREVIEW_SNAPSHOT_IDS, isPreviewEventRecord, isPreviewSnapshotId } from './previewFixtureCatalog.ts';

export interface GlobalEvent {
  id: string;
  type: string;
  spaceId: string;
  timestamp: string;
  payload: Record<string, unknown>;
}

export type PlatformEntityState = Record<string, unknown>;

/**
 * Event Store Schema
 * 
 * Fulfills the Pivot: Event log is the primary database.
 * All UI state derives from GlobalEvents.
 */
interface CortexEventDB extends DBSchema {
  events: {
    key: string;
    value: GlobalEvent;
    indexes: {
      'by-space': string;
      'by-timestamp': string;
      'by-type': string;
    };
  };
  snapshots: {
    key: string;
    value: {
      id: string; // Typically the aggregate ID
      state: PlatformEntityState;
      version: number;
      updatedAt: string;
    };
    indexes: {
      'by-updated': string;
    };
  };
}

/**
 * Read the event log for a given space since a specific timestamp.
 */
export async function getEventsBySpaceSince(spaceId: string, sinceTs: string): Promise<GlobalEvent[]> {
  await purgePreviewFixturesIfDisabled();
  if (!arePreviewFixturesEnabled() && spaceId === INTRO_SPACE_ID) {
    return [];
  }
  await seedMockSpaceIfNeeded(spaceId);
  const db = await initEventStore();
  const tx = db.transaction('events', 'readonly');
  const index = tx.store.index('by-space');
  
  // IDB doesn't support easy multi-index range queries without a compound index.
  // We'll fetch by space and filter in memory for now, as local logs are typically small.
  const events = await index.getAll(spaceId);
  return events
    .filter((event) => arePreviewFixturesEnabled() || !isPreviewEventRecord(event))
    .filter(e => e.spaceId === spaceId && e.timestamp > sinceTs)
    .sort((a, b) => new Date(a.timestamp).getTime() - new Date(b.timestamp).getTime());
}

const DB_NAME = 'cortex-event-store';
const DB_VERSION = 1;

/**
 * Initialize the IndexedDB Event Store
 */
export async function initEventStore(): Promise<IDBPDatabase<CortexEventDB>> {
  return openDB<CortexEventDB>(DB_NAME, DB_VERSION, {
    upgrade(db) {
      if (!db.objectStoreNames.contains('events')) {
        const eventStore = db.createObjectStore('events', { keyPath: 'id' });
        eventStore.createIndex('by-space', 'spaceId');
        eventStore.createIndex('by-timestamp', 'timestamp');
        eventStore.createIndex('by-type', 'type');
      }
      
      if (!db.objectStoreNames.contains('snapshots')) {
        const snapshotStore = db.createObjectStore('snapshots', { keyPath: 'id' });
        snapshotStore.createIndex('by-updated', 'updatedAt');
      }
    },
  });
}

/**
 * Seeds the mock space with initial events if it is currently empty.
 * This is called automatically when reading events for a space.
 */
async function seedMockSpaceIfNeeded(spaceId: string) {
  if (spaceId !== INTRO_SPACE_ID || !arePreviewFixturesEnabled()) return;

  const db = await initEventStore();
  const txCheck = db.transaction('events', 'readonly');
  const index = txCheck.store.index('by-space');
  const existingCount = await index.count(spaceId);

  if (existingCount === 0) {
    console.log(`[EventStore] Seeding empty ${spaceId} with ${SEED_EVENTS.length} events...`);
    const txWrite = db.transaction('events', 'readwrite');
    for (const event of SEED_EVENTS) {
      await txWrite.store.add(event);
    }
    await txWrite.done;
    console.log(`[EventStore] Seeding completed for ${spaceId}`);
  }
}

/**
 * Get a specific snapshot by ID from the `snapshots` store.
 */
export async function getSnapshot(id: string): Promise<PlatformEntityState | null> {
  await purgePreviewFixturesIfDisabled();
  if (!arePreviewFixturesEnabled() && isPreviewSnapshotId(id)) {
    return null;
  }
  await initSystemSnapshotsIfNeeded();
  const db = await initEventStore();
  const tx = db.transaction('snapshots', 'readonly');
  const snapshot = await tx.store.get(id);
  return snapshot ? snapshot.state : null;
}

/**
 * Save a snapshot into the `snapshots` store.
 */
export async function putSnapshot(id: string, state: PlatformEntityState, version: number): Promise<void> {
  const db = await initEventStore();
  const tx = db.transaction('snapshots', 'readwrite');
  await tx.store.put({
    id,
    state,
    version,
    updatedAt: new Date().toISOString()
  });
  await tx.done;
}

/**
 * Ensure system mock snapshots exist in IDB.
 */
export async function initSystemSnapshotsIfNeeded(): Promise<void> {
  await purgePreviewFixturesIfDisabled();
  if (!arePreviewFixturesEnabled()) {
    return;
  }
  const db = await initEventStore();
  
  // For Sovereign Local Host mock environments, we always re-seed the static mocks
  // to ensure updates to seedData.ts are immediately reflected on reload.
  const txWrite = db.transaction('snapshots', 'readwrite');
  await txWrite.store.put({
    id: "system:whoami",
    state: MOCK_WHOAMI as any,
    version: 1,
    updatedAt: new Date().toISOString()
  });
  await txWrite.store.put({
    id: "system:layout:spec",
    state: MOCK_LAYOUT_SPEC as any,
    version: 1,
    updatedAt: new Date().toISOString()
  });
  await txWrite.store.put({
    id: "system:navigation:mock",
    state: MOCK_NAVIGATION_PLAN as any,
    version: 1,
    updatedAt: new Date().toISOString()
  });
  
  // Workbench Variants
  const workbenches = [
    { id: "system:ux:workbench", state: MOCK_UX_WORKBENCH_MAIN },
    { id: "system:ux:workbench:/labs", state: MOCK_UX_WORKBENCH_LABS },
    { id: "system:ux:workbench:/labs/execution-canvas", state: MOCK_UX_WORKBENCH_EXECUTION_CANVAS },
    { id: "system:ux:workbench:/system", state: MOCK_UX_WORKBENCH_SYSTEM },
    { id: "system:ux:workbench:/spaces", state: MOCK_UX_WORKBENCH_SPACES },
    { id: "system:ux:workbench:/heap", state: MOCK_UX_WORKBENCH_HEAP },
    { id: "system:ux:workbench:/studio", state: MOCK_UX_WORKBENCH_STUDIO },
  ];

  for (const wb of workbenches) {
    await txWrite.store.put({
      id: wb.id,
      state: wb.state as any,
      version: 1,
      updatedAt: new Date().toISOString()
    });
  }

  await txWrite.done;
}

/**
 * Append an event to the local event log.
 */
export async function appendEvent(event: GlobalEvent): Promise<void> {
  const db = await initEventStore();
  const tx = db.transaction('events', 'readwrite');
  await tx.store.add(event);
  await tx.done;
  console.log(`[EventStore] Appended event ${event.id} of type ${event.type}`);
  
  // Future: Trigger UI Reducer updates via React state or Zustand
}

/**
 * Read the entire event log for a given space, sorted chronologically.
 */
export async function getEventsBySpace(spaceId: string): Promise<GlobalEvent[]> {
  await purgePreviewFixturesIfDisabled();
  if (!arePreviewFixturesEnabled() && spaceId === INTRO_SPACE_ID) {
    return [];
  }
  await seedMockSpaceIfNeeded(spaceId);
  const db = await initEventStore();
  const index = db.transaction('events', 'readonly').store.index('by-space');
  const events = await index.getAll(spaceId);
  return events
    .filter((event) => arePreviewFixturesEnabled() || !isPreviewEventRecord(event))
    .sort((a: GlobalEvent, b: GlobalEvent) => new Date(a.timestamp).getTime() - new Date(b.timestamp).getTime());
}
/**
 * Retrieve all events related to a specific artifact.
 * Scans events in the given space to find those where payload.artifactId matches.
 */
export async function getEventsByArtifactId(spaceId: string, artifactId: string): Promise<GlobalEvent[]> {
  const events = await getEventsBySpace(spaceId);
  return events.filter(e => e.payload.artifactId === artifactId);
}

/**
 * Seed the store with default events if it's currently empty for the given space.
 */
export async function seedIfEmpty(spaceId: string): Promise<void> {
  await purgePreviewFixturesIfDisabled();
  if (!arePreviewFixturesEnabled() && spaceId === INTRO_SPACE_ID) {
    return;
  }
  const existing = await getEventsBySpace(spaceId);
  const existingIds = new Set(existing.map(e => e.id));
  
  const seedEvents = SEED_EVENTS.filter(e => e.spaceId === spaceId);
  let addedCount = 0;

  for (const event of seedEvents) {
    if (!existingIds.has(event.id)) {
      await appendEvent(event);
      addedCount++;
    }
  }

  if (addedCount > 0) {
    console.log(`[EventStore] Seeded space ${spaceId} with ${addedCount} missing default events.`);
  }
}

function arePreviewFixturesEnabled(): boolean {
  return useUserPreferences.getState().registryMode === 'preview';
}

let previewFixturePurgePromise: Promise<void> | null = null;

async function purgePreviewFixturesIfDisabled(): Promise<void> {
  if (arePreviewFixturesEnabled()) {
    previewFixturePurgePromise = null;
    return;
  }

  if (!previewFixturePurgePromise) {
    previewFixturePurgePromise = (async () => {
      const db = await initEventStore();
      const snapshotTx = db.transaction('snapshots', 'readwrite');
      for (const snapshotId of PREVIEW_SNAPSHOT_IDS) {
        await snapshotTx.store.delete(snapshotId);
      }
      await snapshotTx.done;

      const eventTx = db.transaction('events', 'readwrite');
      const allEvents = await eventTx.store.getAll();
      for (const event of allEvents) {
        if (isPreviewEventRecord(event)) {
          await eventTx.store.delete(event.id);
        }
      }
      await eventTx.done;
    })().finally(() => {
      previewFixturePurgePromise = null;
    });
  }

  await previewFixturePurgePromise;
}
