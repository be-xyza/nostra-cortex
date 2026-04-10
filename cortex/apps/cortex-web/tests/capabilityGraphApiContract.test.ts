import assert from "node:assert/strict";
import test from "node:test";

import { workbenchApi } from "../src/api.ts";

test("capability graph contract remains backward compatible with additive metadata", async () => {
  const originalFetch = globalThis.fetch;
  let capturedUrl = "";

  globalThis.fetch = (async (input: URL | RequestInfo) => {
    capturedUrl = String(input);
    return new Response(
      JSON.stringify({
        schema_version: "1.1.0",
        generated_at: "2026-03-02T00:00:00Z",
        source_of_truth: "local_json",
        graph_hash: "abc123",
        capabilities_version: "v1-abc123",
        layout_hints: {
          engine: "react_flow",
          seed: "capability-graph-v2",
          cluster_by: "domain",
          groups: [{ key: "domain:system", label: "system", order: 0, color: "#38bdf8" }]
        },
        legend: {
          intent_type_colors: { monitor: "#38bdf8" },
          relationship_styles: { contains: "solid" },
          lock_semantics: "role_rank(required_role) > actor_role_rank"
        },
        nodes: [
          {
            id: "route:_system",
            title: "System",
            description: "System route",
            intent_type: "monitor",
            required_role: "viewer",
            cluster_key: "domain:system",
            domain: "system",
            visibility_state: "visible"
          }
        ],
        edges: [
          {
            from: "cortex.workbench.root",
            to: "route:_system",
            relationship: "drill_down",
            relationship_label: "Route Drill-Down",
            confidence: 98
          }
        ]
      }),
      { status: 200, headers: { "Content-Type": "application/json" } }
    );
  }) as typeof fetch;

  try {
    const graph = await workbenchApi.getCapabilityGraph();
    assert.ok(capturedUrl.endsWith("/api/system/capability-graph"));
    assert.equal(graph.schema_version, "1.1.0");
    assert.equal(graph.nodes[0]?.id, "route:_system");
    assert.equal(graph.edges[0]?.relationship, "drill_down");
    assert.equal(graph.layout_hints?.engine, "react_flow");
    assert.equal(typeof graph.graph_hash, "string");
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("shell layout contract carries optional navMeta and playground route", async () => {
  const originalFetch = globalThis.fetch;
  let capturedUrl = "";
  globalThis.fetch = (async (input: URL | RequestInfo) => {
    capturedUrl = String(input);
    return new Response(
      JSON.stringify({
        layoutId: "cortex.desktop.shell.v1",
        navigationGraph: {
          entries: [
            {
              routeId: "/explore",
              label: "Explore",
              icon: "EX",
              category: "core",
              requiredRole: "operator",
              navMeta: { badgeCount: 1, badgeTone: "info", attention: true, attentionLabel: "Live heap", collapsibleHint: "expanded" }
            },
            {
              routeId: "/playground",
              label: "Chat x Heap Playground",
              icon: "PG",
              category: "core",
              requiredRole: "operator",
              navMeta: { badgeCount: 1, badgeTone: "warn", attention: true, attentionLabel: "Orchestration", collapsibleHint: "expanded" }
            }
          ]
        }
      }),
      { status: 200, headers: { "Content-Type": "application/json" } }
    );
  }) as typeof fetch;

  try {
    const layout = await workbenchApi.getShellLayout();
    assert.ok(capturedUrl.endsWith("/api/cortex/layout/spec"));
    assert.ok(layout.navigationGraph.entries.some((entry) => entry.routeId === "/playground"));
    assert.equal(layout.navigationGraph.entries[0]?.navMeta?.badgeTone, "info");
    assert.equal(layout.navigationGraph.entries[1]?.navMeta?.attentionLabel, "Orchestration");
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("session endpoint uses the canonical identity path and returns backend-issued session state", async () => {
  const originalFetch = globalThis.fetch;
  let capturedUrl = "";
  let capturedRole = "";
  let capturedActor = "";

  globalThis.fetch = (async (input: URL | RequestInfo, init?: RequestInit) => {
    capturedUrl = String(input);
    capturedRole = String((init?.headers as Record<string, string> | undefined)?.["x-cortex-role"] ?? "");
    capturedActor = String((init?.headers as Record<string, string> | undefined)?.["x-cortex-actor"] ?? "");
    return new Response(
      JSON.stringify({
        schemaVersion: "1.0.0",
        generatedAt: "2026-03-22T00:00:00Z",
        principal: "web-test",
        sessionId: "sess-1",
        identityVerified: false,
        identitySource: "dev_unverified_header",
        authMode: "dev_override",
        grantedRoles: ["viewer", "editor", "operator"],
        activeRole: "operator",
        globalClaims: [],
        spaceGrants: [],
        allowRoleSwitch: true,
        allowUnverifiedRoleHeader: true
      }),
      { status: 200, headers: { "Content-Type": "application/json" } }
    );
  }) as typeof fetch;

  try {
    const session = await workbenchApi.getSession("operator", "web-test");
    assert.ok(capturedUrl.endsWith("/api/system/session"));
    assert.equal(capturedRole, "operator");
    assert.equal(capturedActor, "web-test");
    assert.equal(session.sessionId, "sess-1");
    assert.equal(session.activeRole, "operator");
    assert.deepEqual(session.grantedRoles, ["viewer", "editor", "operator"]);
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("session active-role switch posts to the canonical endpoint", async () => {
  const originalFetch = globalThis.fetch;
  let capturedUrl = "";
  let capturedMethod = "";
  let capturedBody = "";
  let capturedActor = "";

  globalThis.fetch = (async (input: URL | RequestInfo, init?: RequestInit) => {
    capturedUrl = String(input);
    capturedMethod = init?.method ?? "GET";
    capturedBody = String(init?.body ?? "");
    capturedActor = String((init?.headers as Record<string, string> | undefined)?.["x-cortex-actor"] ?? "");
    return new Response(
      JSON.stringify({
        schemaVersion: "1.0.0",
        generatedAt: "2026-03-22T00:00:00Z",
        principal: "web-test",
        sessionId: "sess-1",
        identityVerified: false,
        identitySource: "session_claims",
        authMode: "session_claims",
        grantedRoles: ["viewer", "editor", "operator"],
        activeRole: "viewer",
        globalClaims: [],
        spaceGrants: [],
        allowRoleSwitch: true,
        allowUnverifiedRoleHeader: false
      }),
      { status: 200, headers: { "Content-Type": "application/json" } }
    );
  }) as typeof fetch;

  try {
    const session = await workbenchApi.setActiveRole("viewer", "nostra-governance-v0", "web-test");
    assert.ok(capturedUrl.endsWith("/api/system/session/active-role"));
    assert.equal(capturedMethod, "POST");
    assert.equal(capturedActor, "web-test");
    assert.equal(
      capturedBody,
      JSON.stringify({ role: "viewer", spaceId: "nostra-governance-v0" }),
    );
    assert.equal(session.activeRole, "viewer");
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("capability catalog and space graph endpoints use canonical initiative-130 paths", async () => {
  const originalFetch = globalThis.fetch;
  const capturedUrls: string[] = [];
  globalThis.fetch = (async (input: URL | RequestInfo) => {
    capturedUrls.push(String(input));
    return new Response(
      JSON.stringify({
        schemaVersion: "1.0.0",
        catalogVersion: "v1",
        baseCatalogVersion: "v1",
        baseCatalogHash: "hash",
        nodes: [],
        edges: [],
        spaceId: "nostra-governance-v0",
        updatedAt: "2026-03-03T00:00:00Z",
        updatedBy: "steward"
      }),
      { status: 200, headers: { "Content-Type": "application/json" } }
    );
  }) as typeof fetch;

  try {
    await workbenchApi.getCapabilityCatalog();
    await workbenchApi.getSpaceCapabilityGraph("nostra-governance-v0");
    await workbenchApi.getSpaceNavigationPlan("nostra-governance-v0", {
      actorRole: "operator",
      intent: "navigate",
      density: "comfortable"
    });

    assert.ok(capturedUrls[0]?.endsWith("/api/system/capability-catalog"));
    assert.ok(capturedUrls[1]?.endsWith("/api/spaces/nostra-governance-v0/capability-graph"));
    assert.ok(
      capturedUrls[2]?.endsWith(
        "/api/spaces/nostra-governance-v0/navigation-plan?actor_role=operator&intent=navigate&density=comfortable"
      )
    );
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("space capability graph PUT sends steward headers and serialized graph body", async () => {
  const originalFetch = globalThis.fetch;
  let capturedUrl = "";
  let capturedMethod = "";
  let capturedRole = "";
  let capturedActor = "";
  let capturedBody = "";

  globalThis.fetch = (async (input: URL | RequestInfo, init?: RequestInit) => {
    capturedUrl = String(input);
    capturedMethod = init?.method ?? "GET";
    capturedRole = String((init?.headers as Record<string, string> | undefined)?.["x-cortex-role"] ?? "");
    capturedActor = String((init?.headers as Record<string, string> | undefined)?.["x-cortex-actor"] ?? "");
    capturedBody = String(init?.body ?? "");
    return new Response(JSON.stringify({
      accepted: true,
      spaceId: "nostra-governance-v0",
      capabilityGraphHash: "graph-hash-1",
      capabilityGraphVersion: "v1",
      storedAt: "2026-03-03T00:00:00Z",
    }), {
      status: 200,
      headers: { "Content-Type": "application/json" }
    });
  }) as typeof fetch;

  const payload = {
    schemaVersion: "1.0.0",
    spaceId: "nostra-governance-v0",
    baseCatalogVersion: "v1",
    baseCatalogHash: "hash",
    nodes: [],
    edges: [],
    updatedAt: "2026-03-03T00:00:00Z",
    updatedBy: "steward",
    lineageRef: "decision:130"
  };

  try {
    const response = await workbenchApi.putSpaceCapabilityGraph(
      "nostra-governance-v0",
      payload,
      "steward",
      "web-test"
    );
    assert.ok(capturedUrl.endsWith("/api/spaces/nostra-governance-v0/capability-graph"));
    assert.equal(capturedMethod, "PUT");
    assert.equal(capturedRole, "steward");
    assert.equal(capturedActor, "web-test");
    assert.equal(capturedBody, JSON.stringify(payload));
    assert.equal(response.capabilityGraphHash, "graph-hash-1");
    assert.equal(response.spaceId, "nostra-governance-v0");
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test("space action plan POST sends operator headers and serialized request body", async () => {
  const originalFetch = globalThis.fetch;
  let capturedUrl = "";
  let capturedMethod = "";
  let capturedRole = "";
  let capturedActor = "";
  let capturedBody = "";

  globalThis.fetch = (async (input: URL | RequestInfo, init?: RequestInit) => {
    capturedUrl = String(input);
    capturedMethod = init?.method ?? "GET";
    capturedRole = String((init?.headers as Record<string, string> | undefined)?.["x-cortex-role"] ?? "");
    capturedActor = String((init?.headers as Record<string, string> | undefined)?.["x-cortex-actor"] ?? "");
    capturedBody = String(init?.body ?? "");
    return new Response(
      JSON.stringify({
        schemaVersion: "1.0.0",
        generatedAt: "2026-03-12T00:00:00Z",
        planHash: "hash",
        spaceId: "nostra-governance-v0",
        routeId: "/explore",
        pageType: "heap_board",
        actorRole: "operator",
        zones: [],
      }),
      { status: 200, headers: { "Content-Type": "application/json" } }
    );
  }) as typeof fetch;

    const payload = {
      schemaVersion: "1.0.0",
      spaceId: "nostra-governance-v0",
      actorRole: "operator",
      routeId: "/explore",
      pageType: "heap_board",
      zones: ["heap_page_bar"],
    selection: {
      selectedArtifactIds: [],
      selectedCount: 0,
      selectedBlockTypes: []
    }
  };

  try {
    await workbenchApi.getSpaceActionPlan("nostra-governance-v0", payload, "operator", "web-test");
    assert.ok(capturedUrl.endsWith("/api/spaces/nostra-governance-v0/action-plan"));
    assert.equal(capturedMethod, "POST");
    assert.equal(capturedRole, "operator");
    assert.equal(capturedActor, "web-test");
    assert.equal(capturedBody, JSON.stringify(payload));
  } finally {
    globalThis.fetch = originalFetch;
  }
});
