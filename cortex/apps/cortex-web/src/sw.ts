/// <reference lib="webworker" />
declare let self: ServiceWorkerGlobalScope;

import { precacheAndRoute } from 'workbox-precaching';

// Precaching injected assets from Vite
precacheAndRoute(self.__WB_MANIFEST || []);

self.addEventListener('install', (event) => {
  console.log('[Service Worker] Installed. Becoming Cortex Gateway Adapter...');
  self.skipWaiting();
});

self.addEventListener('activate', (event) => {
  console.log('[Service Worker] Activated. Execution boundary initialized.');
  event.waitUntil(self.clients.claim());
});

import { 
  SEED_EVENTS, 
  INTRO_SPACE_ID, 
  MOCK_UX_WORKBENCH_MAIN,
  MOCK_UX_WORKBENCH_LABS,
  MOCK_UX_WORKBENCH_SYSTEM,
  MOCK_UX_WORKBENCH_SPACES,
  MOCK_UX_WORKBENCH_HEAP,
  MOCK_UX_WORKBENCH_STUDIO,
  MOCK_CAPABILITY_GRAPH,
  MOCK_SPACE_CAPABILITY_GRAPH,
  MOCK_CONTRIBUTION_GRAPH,
  MOCK_WORKFLOW_TOPOLOGY,
  buildMockActionPlan
} from './store/seedData';
import { getEventsBySpace, appendEvent, getEventsByArtifactId, getEventsBySpaceSince, seedIfEmpty, getSnapshot } from './store/eventStore';
import { reduceHeapBlocks } from './store/eventProcessor';

const DEFAULT_SPACE_ID = "01ARZ3NDEKTSV4RRFFQ69G5FAV";

/**
 * Route Cortex Requests (The Execution Boundary)
 */
async function routeCortexRequest(request: Request): Promise<Response> {
  const url = new URL(request.url);
  const path = url.pathname;
  const spaceId = url.searchParams.get('spaceId') || DEFAULT_SPACE_ID;
  console.log(`[Service Worker] Intercepted Cortex Action: ${path}`);
  
  // 1. GET /api/cortex/studio/heap/blocks
  if (path === '/api/cortex/studio/heap/blocks' && request.method === 'GET') {
    // Seed the store if it's empty for this space (restores mock content)
    await seedIfEmpty(spaceId);
    
    const events = await getEventsBySpace(spaceId);
    const blocks = reduceHeapBlocks(events);
    
    const includeDeleted = url.searchParams.get('includeDeleted') === 'true';
    const finalBlocks = includeDeleted ? blocks : blocks.filter(b => !b.deletedAt);

    return new Response(JSON.stringify({
      schemaVersion: "1.0.0",
      generatedAt: new Date().toISOString(),
      count: finalBlocks.length,
      hasMore: false,
      items: finalBlocks
    }), { status: 200, headers: { 'Content-Type': 'application/json' } });
  }

  // 1b. GET /api/cortex/studio/heap/changed_blocks
  if (path === '/api/cortex/studio/heap/changed_blocks' && request.method === 'GET') {
    const changedSince = url.searchParams.get('changedSince') || '';
    const events = await getEventsBySpaceSince(spaceId, changedSince);
    const allBlocks = reduceHeapBlocks(await getEventsBySpace(spaceId));
    
    // Filter blocks that were affected by these new events
    const affectedIds = new Set(events.map(e => e.payload.artifactId as string));
    const changed = allBlocks.filter(b => affectedIds.has(b.projection.artifactId) && !b.deletedAt);
    const deleted = allBlocks.filter(b => affectedIds.has(b.projection.artifactId) && b.deletedAt).map(b => b.projection.artifactId);

    return new Response(JSON.stringify({
      generatedAt: new Date().toISOString(),
      changed,
      deleted,
      hasMore: false,
    }), { status: 200, headers: { 'Content-Type': 'application/json' } });
  }

  // 2. POST /api/cortex/studio/heap/emit
  if (path === '/api/cortex/studio/heap/emit' && request.method === 'POST') {
    const payload = await request.json();
    const artifactId = `local-${Date.now()}-${Math.floor(Math.random() * 1000)}`;
    
    await appendEvent({
      id: `evt-${Date.now()}`,
      type: 'HeapBlockCreated',
      spaceId: payload.workspace_id || spaceId,
      timestamp: new Date().toISOString(),
      payload: {
        ...payload.block,
        artifactId,
        content: payload.content
      }
    });

    return new Response(JSON.stringify({ 
      accepted: true,
      artifactId 
    }), { status: 200, headers: { 'Content-Type': 'application/json' } });
  }

  // 3. POST /api/cortex/studio/heap/blocks/:id/pin
  const pinMatch = path.match(/\/api\/cortex\/studio\/heap\/blocks\/([^/]+)\/pin$/);
  if (pinMatch && request.method === 'POST') {
    const artifactId = pinMatch[1];
    await appendEvent({
      id: `evt-pin-${Date.now()}`,
      type: 'HeapBlockPinned',
      spaceId: spaceId,
      timestamp: new Date().toISOString(),
      payload: { artifactId }
    });
    return new Response(JSON.stringify({ 
      accepted: true, 
      artifactId, 
      action: "pin",
      updatedAt: new Date().toISOString()
    }), { status: 200, headers: { 'Content-Type': 'application/json' } });
  }

  // 4. POST /api/cortex/studio/heap/blocks/:id/delete
  const deleteMatch = path.match(/\/api\/cortex\/studio\/heap\/blocks\/([^/]+)\/delete$/);
  if (deleteMatch && request.method === 'POST') {
    const artifactId = deleteMatch[1];
    await appendEvent({
      id: `evt-delete-${Date.now()}`,
      type: 'HeapBlockDeleted',
      spaceId: spaceId,
      timestamp: new Date().toISOString(),
      payload: { artifactId }
    });
    return new Response(JSON.stringify({ 
      accepted: true, 
      artifactId, 
      action: "delete",
      updatedAt: new Date().toISOString()
    }), { status: 200, headers: { 'Content-Type': 'application/json' } });
  }

  // 5. GET /api/cortex/studio/heap/blocks/:id/history
  const historyMatch = path.match(/\/api\/cortex\/studio\/heap\/blocks\/([^/]+)\/history$/);
  if (historyMatch && request.method === 'GET') {
    const artifactId = historyMatch[1];
    const events = await getEventsByArtifactId(spaceId, artifactId);
    return new Response(JSON.stringify({
      artifactId,
      versions: events.map(e => ({
        eventId: e.id,
        eventType: e.type,
        timestamp: e.timestamp,
        payload: e.payload
      }))
    }), { status: 200, headers: { 'Content-Type': 'application/json' } });
  }

  // 5b. GET /api/cortex/workflow-definitions/:id/projections/:kind
  const topoMatch = path.match(/^\/api\/cortex\/workflow-definitions\/([^/]+)\/projections\/execution_topology_v1$/);
  if (topoMatch && request.method === "GET") {
    return new Response(JSON.stringify(MOCK_WORKFLOW_TOPOLOGY), {
      status: 200,
      headers: { "Content-Type": "application/json" }
    });
  }

  const defProjMatch = path.match(/^\/api\/cortex\/workflow-definitions\/([^/]+)\/projections\/([^/]+)$/);
  if (defProjMatch && request.method === "GET") {
    return new Response(JSON.stringify({
      kind: defProjMatch[2],
      projection: { note: "Mock generic projection for " + defProjMatch[2] }
    }), { status: 200, headers: { "Content-Type": "application/json" } });
  }

  const defMatch = path.match(/^\/api\/cortex\/workflow-definitions\/([^/]+)$/);
  if (defMatch && request.method === "GET") {
    return new Response(JSON.stringify({
      definitionId: defMatch[1],
      definition: {
        id: defMatch[1],
        digest: "sha256:mock-digest-" + defMatch[1].substring(0, 8),
        motif_kind: "sequential_agent_loop"
      }
    }), { status: 200, headers: { "Content-Type": "application/json" } });
  }

  // 6. GET /api/cortex/layout/spec
  if (path === '/api/cortex/layout/spec' && request.method === 'GET') {
    const layout = await getSnapshot("system:layout:spec");
    if (layout) return new Response(JSON.stringify(layout), { status: 200, headers: { 'Content-Type': 'application/json' } });
  }

  // 7. GET /api/system/whoami
  if (path === '/api/system/whoami' && request.method === 'GET') {
    const whoami = await getSnapshot("system:whoami");
    if (whoami) return new Response(JSON.stringify(whoami), { status: 200, headers: { 'Content-Type': 'application/json' } });
  }

  // 8. GET /api/spaces/*/navigation-plan
  if (path.match(/^\/api\/spaces\/[^/]+\/navigation-plan$/) && request.method === 'GET') {
    const navPlan = await getSnapshot("system:navigation:mock");
    if (navPlan) return new Response(JSON.stringify(navPlan), { status: 200, headers: { 'Content-Type': 'application/json' } });
  }

  // 9. GET /api/system/ux/workbench
  if (path === '/api/system/ux/workbench' && request.method === 'GET') {
    const route = url.searchParams.get('route') || '';
    const lookupKey = route ? `system:ux:workbench:${route}` : 'system:ux:workbench';
    let workbench = await getSnapshot(lookupKey);
    
    // Fallback to main if route-specific snapshot is missing
    if (!workbench) {
      workbench = await getSnapshot("system:ux:workbench");
    }

    if (workbench) return new Response(JSON.stringify(workbench), { status: 200, headers: { 'Content-Type': 'application/json' } });
  }

  // Contribution graph mock (ambient background graph)
  const graphMatch = path.match(/^\/api\/kg\/spaces\/[^/]+\/contribution-graph\/graph$/);
  if (graphMatch && request.method === 'GET') {
    return new Response(JSON.stringify(MOCK_CONTRIBUTION_GRAPH), {
      status: 200, headers: { 'Content-Type': 'application/json' }
    });
  }

  // Fallback to Network
  try {
    if (url.pathname === '/api/system/capability-graph') {
      return new Response(JSON.stringify(MOCK_CAPABILITY_GRAPH), {
        headers: { 'Content-Type': 'application/json' }
      });
    }

    // Space-level capability graph with overrides
    const capGraphMatch = url.pathname.match(/^\/api\/spaces\/([^/]+)\/capability-graph$/);
    if (capGraphMatch) {
      return new Response(JSON.stringify(MOCK_SPACE_CAPABILITY_GRAPH), {
        headers: { 'Content-Type': 'application/json' }
      });
    }

    if (url.pathname.startsWith('/api/spaces/') && url.pathname.endsWith('/action-plan')) {
      const spaceId = url.pathname.split('/')[3];
      const routeId = url.searchParams.get('route') || '/heap';
      return new Response(JSON.stringify(buildMockActionPlan(spaceId, routeId)), {
        headers: { 'Content-Type': 'application/json' }
      });
    }

    return await fetch(request);
  } catch (error) {
    return new Response(JSON.stringify({ 
      status: "offline-error",
      message: "Gateway unreachable and route not implemented locally."
    }), { status: 503, headers: { 'Content-Type': 'application/json' } });
  }
}

// Intercept specific domain/api calls
self.addEventListener('fetch', (event) => {
  const url = new URL(event.request.url);
  
  if (url.pathname.startsWith('/api/cortex') || url.pathname.startsWith('/api/system') || url.pathname.startsWith('/api/spaces') || url.pathname.startsWith('/api/kg')) {
    event.respondWith(routeCortexRequest(event.request));
    return;
  }
});
