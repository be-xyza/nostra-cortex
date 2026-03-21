import { GlobalEvent } from './eventStore';
import { buildSpaceStudioRoute, SPACE_STUDIO_ROUTE } from "../components/spaces/spaceStudioRoutes.ts";
import { 
  PlatformCapabilityGraph, 
  SpaceCapabilityGraph,
  CompiledNavigationPlan, 
  PlatformCapabilityCatalog,
  PlatformCapabilityCatalogNode,
  CompiledNavigationEntry
} from "../contracts";

export const INTRO_SPACE_ID = "01ARZ3NDEKTSV4RRFFQ69G5FAV";

export const SEED_EVENTS: GlobalEvent[] = [
  {
    id: "seed-event-1",
    type: "HeapBlockCreated",
    spaceId: INTRO_SPACE_ID,
    timestamp: "2026-02-24T13:20:41Z",
    payload: {
      artifactId: "heap-demo-1",
      blockType: "widget",
      title: "Heap Demo Card 1",
      surfaceId: "surface:seed-heap-req-1",
      emittedAt: "2026-02-24T13:20:41Z",
      surfaceJson: {
        components: [
          {
            children: [],
            id: "root",
            props: {
              subtitle: "Seeded heap demo block",
              title: "Heap Demo Card 1"
            },
            type: "Card"
          }
        ],
        meta: {
          mentions: ["Project Alpha"],
          tags: ["01ARZ3NDEKTSV4RRFFQ69G5FAX"]
        },
        root: "root",
        surfaceId: "surface:seed-heap-req-1",
        title: "Heap Demo Card 1"
      }
    }
  },
  {
    id: "seed-event-2",
    type: "HeapBlockCreated",
    spaceId: INTRO_SPACE_ID,
    timestamp: "2026-02-24T13:20:42Z",
    payload: {
      artifactId: "heap-demo-2",
      blockType: "note",
      title: "Cortex Architectural Overview",
      surfaceId: "surface:seed-heap-req-2",
      emittedAt: "2026-02-24T13:20:42Z",
        mentionsInline: ["heap-demo-3", "mock-gate-1", "mock-media-1"],
        pageLinks: ["heap-demo-1"],
        surfaceJson: {
          payload_type: "note",
          text: "# Cortex High-Level Design\n\nCortex is a multi-agent orchestration framework designed for **high-throughput** and **low-latency** AI operations.\n\n### Key Components:\n- **Gateway**: Entry point for all requests.\n- **Eudaemon**: Background agent management and monitoring.\n- **Workbench**: Interactive visual environment (this UI).\n\n> [!NOTE]\n> The system utilizes a graph-native database for relational tracking.\n\n- [x] Initial specification approved\n- [x] Prototype implementation complete\n- [ ] Multi-region deployment pending\n\nRefer to `nostra.core.v2` for the schema definitions.",
          meta: {
            mentions: ["Project Beta"],
            tags: ["architecture", "doc"]
          },
          surfaceId: "surface:seed-heap-req-2",
          title: "Cortex Architectural Overview"
        }
      }
    },
  {
    id: "seed-event-3",
    type: "HeapBlockCreated",
    spaceId: INTRO_SPACE_ID,
    timestamp: "2026-03-10T09:30:00Z",
    payload: {
      artifactId: "heap-demo-3",
      blockType: "chart",
      title: "Agent Performance Metrics",
      surfaceId: "surface:seed-heap-req-3",
      emittedAt: "2026-03-10T09:30:00Z",
      surfaceJson: {
        payload_type: "chart",
        tree: {
          chart_type: "line",
          title: "Response Time (ms) by Model Type",
          labels: ["08:00", "08:15", "08:30", "08:45", "09:00", "09:15", "09:30"],
          datasets: [
            {
              label: "nostra-large-v1",
              data: [1200, 1450, 1300, 1600, 1550, 1400, 1350],
              color: "#3b82f6"
            },
            {
              label: "nostra-flash-v2",
              data: [300, 320, 280, 350, 330, 310, 290],
              color: "#10b981"
            }
          ]
        },
        surfaceId: "surface:seed-heap-req-3",
        title: "Agent Performance Metrics"
      }
    }
  },
  {
    id: "seed-event-4",
    type: "HeapBlockCreated",
    spaceId: INTRO_SPACE_ID,
    timestamp: "2026-03-10T08:00:00Z",
    payload: {
      artifactId: "mock-activity-feed-1",
      blockType: "telemetry",
      title: "Deployment Event Log",
      surfaceId: "surface:activity-feed-1",
      emittedAt: "2026-03-10T08:00:00Z",
      surfaceJson: {
        payload_type: "telemetry",
        tree: {
          widget: "ActivityFeed",
          title: "System Events (v1.4.2)",
          items: [
            {
              action: "Deployment Started",
              detail: "Initiating canary rollout to region us-east-1",
              timestamp: "2026-03-10T07:45:00Z"
            },
            {
              action: "Health Check Passed",
              detail: "Canary instances (3/3) reporting healthy state",
              timestamp: "2026-03-10T07:48:00Z"
            },
            {
              action: "Traffic Shift",
              detail: "Routing 10% of production traffic to canary",
              timestamp: "2026-03-10T07:50:00Z"
            },
            {
              action: "Deployment Success",
              detail: "Full rollout completed across all regions",
              timestamp: "2026-03-10T08:00:00Z"
            }
          ]
        }
      }
    }
  },
  {
    id: "seed-event-5",
    type: "HeapBlockCreated",
    spaceId: INTRO_SPACE_ID,
    timestamp: "2026-03-10T07:00:00Z",
    payload: {
      artifactId: "mock-scorecard-1",
      blockType: "widget",
      title: "Security Compliance Audit",
      surfaceId: "surface:siq-scorecard-1",
      emittedAt: "2026-03-10T07:00:00Z",
      surfaceJson: {
        payload_type: "a2ui",
        tree: {
          widget: "SiqScorecard",
          passing: false,
          score: 72,
          violations: [
            {
              node: "CortexGateway.Auth",
              error: "Missing mandatory token rotation for service account 'eudaemon-proxy'"
            },
            {
              node: "Nostra.Storage.V2",
              error: "Permissive CORS policy detected on internal artifact bucket"
            }
          ]
        }
      }
    }
  },
  {
    id: "seed-event-6",
    type: "HeapBlockCreated",
    spaceId: INTRO_SPACE_ID,
    timestamp: "2026-03-10T06:30:00Z",
    payload: {
      artifactId: "mock-gate-1",
      blockType: "structured_data",
      title: "Infrastructure Gate Summary",
      surfaceId: "surface:gate-1",
      emittedAt: "2026-03-10T06:30:00Z",
      surfaceJson: {
        payload_type: "structured_data",
        structured_data: {
          schema_id: "nostra.heap.block.gate_summary.v1",
          overall_verdict: "FAILED",
          counts: { total_tests: 100, passed: 97, failed: 3, skipped: 0 }
        }
      }
    }
  },
  {
    id: "seed-event-7",
    type: "HeapBlockCreated",
    spaceId: INTRO_SPACE_ID,
    timestamp: "2026-03-10T06:00:00Z",
    payload: {
      artifactId: "mock-media-1",
      blockType: "media",
      title: "Architecture Diagram Draft",
      surfaceId: "surface:media-1",
      emittedAt: "2026-03-10T06:00:00Z",
      surfaceJson: {
        payload_type: "media",
        media: {
          url: "https://images.unsplash.com/photo-1518770660439-4636190af475?q=80&w=2000",
          mime_type: "image/jpeg"
        }
      }
    }
  },
  {
    id: "seed-event-8",
    type: "HeapBlockCreated",
    spaceId: INTRO_SPACE_ID,
    timestamp: "2026-03-10T05:30:00Z",
    payload: {
      artifactId: "mock-solicitation-1",
      blockType: "widget",
      title: "Pending Agent Proposal",
      surfaceId: "surface:solicitation-1",
      emittedAt: "2026-03-10T05:30:00Z",
      surfaceJson: {
        payload_type: "a2ui",
        tree: {
          widget: "AgentBenchmarkRecord",
          agent_role: "steward.security",
          rationale: "Restrict egress traffic across the PWA service worker boundary to prevent data exfiltration during offline replay."
        }
      }
    }
  },
  {
    id: "seed-event-9",
    type: "HeapBlockCreated",
    spaceId: INTRO_SPACE_ID,
    timestamp: "2026-03-10T05:00:00Z",
    payload: {
      artifactId: "mock-task-1",
      blockType: "task",
      title: "Release Preparation Checklist",
      surfaceId: "surface:task-1",
      emittedAt: "2026-03-10T05:00:00Z",
      surfaceJson: {
        payload_type: "task",
        text: "### Release Readiness\n- [x] Security audit completed\n- [x] Documentation updated\n- [ ] Final regression tests passing\n- [ ] Steward approval signed"
      }
    }
  }
];

// --- System Snapshots ---

export const MOCK_LAYOUT_SPEC = {
  layoutId: "default",
  navigationGraph: {
    entries: [
      {
        routeId: "/explore",
        label: "Explore",
        icon: "compass",
        category: "execution",
        requiredRole: "operator",
        navSlot: "primary_platform"
      },
      {
        routeId: "/workflows",
        label: "Flows",
        icon: "git-branch",
        category: "bridge",
        requiredRole: "operator",
        navSlot: "primary_execute"
      },
      {
        routeId: "/contributions",
        label: "Contributions",
        icon: "git-merge",
        category: "bridge",
        requiredRole: "operator",
        navSlot: "secondary_ops"
      },
      {
        routeId: "/artifacts",
        label: "Artifacts",
        icon: "file-code",
        category: "bridge",
        requiredRole: "operator",
        navSlot: "secondary_ops"
      },
      {
        routeId: "/logs",
        label: "System Logs",
        icon: "terminal",
        category: "bridge",
        requiredRole: "operator",
        navSlot: "secondary_ops"
      },
      {
        routeId: "/studio",
        label: "Flow Studio",
        icon: "code",
        category: "execution",
        requiredRole: "operator",
        navSlot: "primary_execute"
      },
      {
        routeId: "/labs",
        label: "Labs",
        icon: "flask-conical",
        category: "workbench",
        requiredRole: "operator",
        navSlot: "labs"
      },
      {
        routeId: "/system",
        label: "System",
        icon: "settings",
        category: "system",
        requiredRole: "operator",
        navSlot: "secondary_admin"
      }
    ]
  }
};

export const MOCK_WHOAMI = {
  schemaVersion: "1.0.0",
  generatedAt: new Date().toISOString(),
  principal: "local-user",
  requestedRole: "steward",
  effectiveRole: "steward",
  claims: ["*"],
  identityVerified: true,
  identitySource: "local",
  authzDevMode: true,
  allowUnverifiedRoleHeader: true,
  authzDecisionVersion: "1.0"
};

export const PLATFORM_CAPABILITY_CATALOG: PlatformCapabilityCatalog = {
  schemaVersion: "1.0.0",
  catalogVersion: "1.0.0",
  nodes: [
    { id: "heap-explore", name: "Explore", description: "Navigable graph exploration", intentType: "domain_read", routeId: "/explore", category: "core", requiredRole: "operator", icon: "compass", surfacingHeuristic: "PrimaryCore", operationalFrequency: "Continuous" },
    { id: "platform-spaces", name: "Spaces Management", description: "Manage spaces", intentType: "system", routeId: "/spaces", category: "core", requiredRole: "operator", icon: "grid", surfacingHeuristic: "PrimaryCore", operationalFrequency: "Continuous" },
    { id: "workbench-labs", name: "Labs", description: "Draft and try ideas before they go live", intentType: "system", routeId: "/labs", category: "core", requiredRole: "operator", icon: "flask-conical", surfacingHeuristic: "PrimaryCore", operationalFrequency: "Continuous" },
    { id: "workbench-system", name: "System Control", description: "Global configuration", intentType: "system", routeId: "/system", category: "core", requiredRole: "operator", icon: "settings", surfacingHeuristic: "PrimaryCore", operationalFrequency: "Continuous" },
    { id: "studio-canvas", name: "Flow Studio", description: "Design flows", intentType: "domain_write", routeId: "/studio", category: "core", requiredRole: "operator", icon: "code", surfacingHeuristic: "PrimaryCore", operationalFrequency: "Continuous" },
    { id: "core-notifications", name: "Notifications", description: "System alerts", intentType: "system", routeId: "/notifications", category: "core", requiredRole: "operator", icon: "bell", surfacingHeuristic: "Secondary", operationalFrequency: "AdHoc" },
    { id: "core-metrics", name: "Metrics", description: "System telemetry", intentType: "system", routeId: "/metrics", category: "core", requiredRole: "operator", icon: "bar-chart-3", surfacingHeuristic: "Secondary", operationalFrequency: "AdHoc" },
    { id: "bridge-flows", name: "Flows", description: "Active workflows", intentType: "execution", routeId: "/workflows", category: "bridge", requiredRole: "operator", icon: "git-branch", surfacingHeuristic: "PrimaryCore", operationalFrequency: "Continuous" },
    { id: "bridge-contributions", name: "Contributions", description: "PRs and merges", intentType: "governance", routeId: "/contributions", category: "bridge", requiredRole: "operator", icon: "git-merge", surfacingHeuristic: "PrimaryCore", operationalFrequency: "Continuous" },
    { id: "bridge-artifacts", name: "Artifacts", description: "Static files", intentType: "domain_read", routeId: "/artifacts", category: "bridge", requiredRole: "operator", icon: "file-code", surfacingHeuristic: "PrimaryCore", operationalFrequency: "Continuous" },
    { id: "bridge-vfs", name: "VFS", description: "Virtual File System", intentType: "system", routeId: "/vfs", category: "bridge", requiredRole: "operator", icon: "hard-drive", surfacingHeuristic: "Secondary", operationalFrequency: "AdHoc" },
    { id: "bridge-logs", name: "System Logs", description: "Detailed logs", intentType: "system", routeId: "/logs", category: "bridge", requiredRole: "operator", icon: "terminal", surfacingHeuristic: "PrimaryCore", operationalFrequency: "Continuous" },
    { id: "agents-siq", name: "SIQ", description: "System Intelligence", intentType: "execution", routeId: "/siq", category: "agents", requiredRole: "operator", icon: "cpu", surfacingHeuristic: "Secondary", operationalFrequency: "AdHoc" },
    { id: "agents-discovery", name: "Discovery", description: "Search and index", intentType: "domain_read", routeId: "/discovery", category: "agents", requiredRole: "operator", icon: "compass", surfacingHeuristic: "Secondary", operationalFrequency: "AdHoc" },
    { id: "agents-memory", name: "Memory", description: "Agent context", intentType: "execution", routeId: "/memory", category: "agents", requiredRole: "operator", icon: "database", surfacingHeuristic: "Secondary", operationalFrequency: "AdHoc" },
    { id: "agents-simulation", name: "Simulation", description: "Test scenarios", intentType: "execution", routeId: "/simulation", category: "agents", requiredRole: "operator", icon: "dices", surfacingHeuristic: "Secondary", operationalFrequency: "AdHoc" },
    { id: "admin-institutions", name: "Institutions", description: "Org management", intentType: "governance", routeId: "/institutions", category: "admin", requiredRole: "operator", icon: "building-2", surfacingHeuristic: "Secondary", operationalFrequency: "AdHoc" },
    { id: "admin-testing", name: "Testing", description: "Test automation", intentType: "execution", routeId: "/testing", category: "admin", requiredRole: "operator", icon: "check-square", surfacingHeuristic: "Secondary", operationalFrequency: "AdHoc" }
  ],
  edges: [
    { source: "workbench-spaces", target: "workbench-system", relationship: "configures" },
    { source: "workbench-labs", target: "workbench-system", relationship: "configures" },
    { source: "studio-canvas", target: "bridge-flows", relationship: "deploys" },
    { source: "bridge-flows", target: "bridge-logs", relationship: "emits" },
    { source: "heap-board", target: "bridge-artifacts", relationship: "indexes" },
    { source: "agents-siq", target: "agents-memory", relationship: "reads" },
    { source: "agents-discovery", target: "heap-board", relationship: "searches" }
  ]
};

export const MOCK_CAPABILITY_GRAPH: PlatformCapabilityGraph = {
  schema_version: "1.0.0",
  generated_at: new Date().toISOString(),
  source_of_truth: "PLATFORM_CAPABILITY_CATALOG",
  nodes: PLATFORM_CAPABILITY_CATALOG.nodes.map((n: PlatformCapabilityCatalogNode) => ({
    id: typeof n.id === 'string' ? n.id : n.id[0],
    title: n.name,
    description: n.description,
    intent_type: n.intentType,
    route_id: n.routeId,
    required_role: n.requiredRole,
    domain: (n as any).domain || n.category,
    pattern_id: (n as any).pattern_id || n.category === "admin" ? "Governance" : n.category === "agents" ? "AgentIntelligence" : "Workbench",
    surfacing_heuristic: n.surfacingHeuristic,
    operational_frequency: n.operationalFrequency,
    inspector: {
      surfacing_heuristic: n.surfacingHeuristic,
      operational_frequency: n.operationalFrequency,
      placement_constraint: n.placementConstraint,
    }
  })),
  edges: PLATFORM_CAPABILITY_CATALOG.edges.map((e: any) => ({
    from: typeof e.source === 'string' ? e.source : e.source[0],
    to: typeof e.target === 'string' ? e.target : e.target[0],
    relationship: e.relationship
  })),
  layout_hints: {
    engine: "dagre",
    seed: "cortex-demo",
    cluster_by: "intent_type",
    groups: [
      { key: "monitor", label: "Observability", order: 1, color: "blue" },
      { key: "execute", label: "Execution", order: 2, color: "green" },
      { key: "access", label: "Governance", order: 3, color: "purple" },
      { key: "storage", label: "Persistence", order: 4, color: "orange" }
    ]
  },
  legend: {
    intent_type_colors: {
      "monitor": "#3498db",
      "execute": "#2ecc71",
      "access": "#9b59b6",
      "storage": "#e67e22"
    },
    relationship_styles: {
      "uses": "solid_blue",
      "monitors": "dashed_blue",
      "deploys": "solid_green",
      "persists": "double_orange",
      "controls": "solid_purple"
    },
    lock_semantics: "Pessimistic"
  }
};

/**
 * Space-level capability override mock.
 * Demonstrates hiding "Simulation" and overriding surfacing for "Logs" in a production space.
 */
export const MOCK_SPACE_CAPABILITY_GRAPH: SpaceCapabilityGraph = {
  schemaVersion: "1.0.0",
  spaceId: INTRO_SPACE_ID,
  baseCatalogVersion: "1.0.0",
  baseCatalogHash: "mock-hash",
  nodes: [
    { capabilityId: "agents-simulation", isActive: false, surfacingHeuristic: "Hidden" },
    { capabilityId: "bridge-logs", isActive: true, surfacingHeuristic: "Secondary", operationalFrequency: "AdHoc" },
    { capabilityId: "bridge-vfs", isActive: true, localAlias: "Files", surfacingHeuristic: "ContextualDeep" },
  ],
  edges: [],
  updatedAt: new Date().toISOString(),
  updatedBy: "steward:local",
};

export function compilePreviewNavigationPlan(spaceId: string): CompiledNavigationPlan {
  const activeCapabilities = PLATFORM_CAPABILITY_CATALOG.nodes; // For preview, all are active
  
  const entries = activeCapabilities.map((node: PlatformCapabilityCatalogNode, index: number) => {
    let navSlot = "secondary_ops";
    if (node.id === "core-inbox" || node.id === "core-notifications") navSlot = "primary_focus";
    else if (node.category === "core") navSlot = "primary_platform";
    else if (node.category === "bridge" && node.id !== "bridge-vfs") navSlot = "primary_execute";
    else if (node.category === "agents") navSlot = "secondary_agents";
    else if (node.category === "admin") navSlot = "secondary_admin";

    return {
      capabilityId: typeof node.id === 'string' ? node.id : node.id[0],
      routeId: node.routeId || "/",
      label: node.name,
      icon: node.icon || "circle",
      category: node.category || "core",
      requiredRole: node.requiredRole || "viewer",
      navSlot,
      navBand: "main",
      surfacingHeuristic: node.surfacingHeuristic || "Secondary",
      operationalFrequency: node.operationalFrequency || "AdHoc",
      rank: index + 1
    };
  });

  return {
    schemaVersion: "1.0.0",
    generatedAt: new Date().toISOString(),
    spaceId,
    actorRole: "steward",
    planHash: `preview-hash-${Date.now()}`,
    entries,
    surfacing: {
      primaryCore: entries.filter((e: CompiledNavigationEntry) => e.surfacingHeuristic === "PrimaryCore").map((e: CompiledNavigationEntry) => e.capabilityId),
      secondary: {},
      contextualDeep: [],
      hidden: []
    }
  };
}

// ── Contribution Graph Mock (Ambient Background) ──────────────────────────────
// Provides a rich graph for the ambient background visualization.
// Mirrors the real ContributionGraphV1 schema from nostra-extraction.
// Nodes represent research initiatives; edges represent references/dependencies.

function cgNode(id: string, title: string, layer: string, status: string, portfolioRole: string = "reference", spaceId?: string) {
  return { id, title, kind: "initiative" as const, status, layer, portfolio_role: portfolioRole, space_id: spaceId };
}

function cgEdge(from: string, to: string, kind: string = "references", confidence: number = 0.7) {
  return { from, to, edge_kind: kind, confidence, is_explicit: kind === "depends_on" };
}

export const MOCK_CONTRIBUTION_GRAPH = {
  graph_root_hash: "mock-ambient-graph-v1",
  nodes: [
    // ── Systems Layer (Space: Alpha) ──
    cgNode("000", "DPub Contribution Graph", "Systems", "active", "anchor", "space-alpha"),
    cgNode("046", "System Standards", "Systems", "active", "anchor", "space-alpha"),
    cgNode("125", "System Integrity Quality", "Systems", "active", "anchor", "space-alpha"),
    cgNode("105", "Test Catalog", "Systems", "active", "anchor", "space-alpha"),
    cgNode("118", "Runtime Extraction", "Systems", "active", "anchor", "space-alpha"),
    cgNode("089", "Code Standards", "Systems", "draft", "reference", "space-alpha"),
    cgNode("055", "Compliance Validation", "Systems", "draft", "reference", "space-alpha"),
    cgNode("063", "Testing Methodology", "Systems", "completed", "reference", "space-alpha"),

    // ── Infrastructure Layer (Space: Beta) ──
    cgNode("004", "Architecture Gaps", "infrastructure", "superseded", "reference", "space-beta"),
    cgNode("006", "Sentiment Capture", "infrastructure", "draft", "reference", "space-beta"),
    cgNode("011", "Video Streaming", "infrastructure", "draft", "reference", "space-beta"),
    cgNode("014", "AI Agents on ICP", "infrastructure", "draft", "reference", "space-beta"),
    cgNode("017", "Agent Role Patterns", "infrastructure", "draft", "reference", "space-beta"),
    cgNode("020", "D3 Graph Performance", "infrastructure", "draft", "reference", "space-beta"),
    cgNode("027", "Workflow Builder", "infrastructure", "draft", "reference", "space-beta"),
    cgNode("031", "Production Foundation", "infrastructure", "draft", "reference", "space-beta"),
    cgNode("035", "Auto Analysis", "infrastructure", "draft", "reference", "space-beta"),
    cgNode("042", "Vector Embedding", "infrastructure", "draft", "reference", "space-beta"),
    cgNode("044", "HRM Integration", "infrastructure", "draft", "reference", "space-beta"),
    cgNode("051", "RAG Ingestion", "infrastructure", "draft", "reference", "space-beta"),
    cgNode("085", "File Infrastructure", "infrastructure", "active", "anchor", "space-beta"),

    // ── Protocol Layer (Space: Gamma) ──
    cgNode("007", "Spaces Concept", "protocol", "active", "anchor", "space-gamma"),
    cgNode("008", "Contribution Types", "protocol", "draft", "reference", "space-gamma"),
    cgNode("012", "Bootstrap Protocol", "protocol", "draft", "reference", "space-gamma"),
    cgNode("013", "Workflow Engine", "protocol", "active", "anchor", "space-gamma"),
    cgNode("018", "Library Registry", "protocol", "draft", "reference", "space-gamma"),
    cgNode("019", "Log Registry", "protocol", "active", "anchor", "space-gamma"),
    cgNode("037", "Knowledge Engine", "protocol", "draft", "reference", "space-gamma"),
    cgNode("040", "Schema Standards", "protocol", "draft", "reference", "space-gamma"),
    cgNode("067", "Unified Protocol", "protocol", "draft", "reference", "space-gamma"),
    cgNode("119", "Nostra Commons", "protocol", "active", "anchor", "space-gamma"),

    // ── Runtime Layer (Space: Eudaemon) ──
    cgNode("023", "Flashcards Use Case", "runtime", "draft", "reference", "eudaemon-alpha"),
    cgNode("033", "System Monitor", "runtime", "draft", "reference", "eudaemon-alpha"),
    cgNode("047", "Temporal Architecture", "runtime", "active", "anchor", "eudaemon-alpha"),
    cgNode("122", "Agent Runtime Kernel", "runtime", "active", "anchor", "eudaemon-alpha"),
    cgNode("129", "Simulation Adapter", "runtime", "draft", "reference", "eudaemon-alpha"),

    // ── Cortex Layer (Space: Eudaemon) ──
    cgNode("074", "UI Substrate", "Cortex", "active", "anchor", "eudaemon-alpha"),
    cgNode("117", "UI Standardization", "Cortex", "active", "anchor", "eudaemon-alpha"),
    cgNode("123", "Web Architecture", "Cortex", "active", "anchor", "eudaemon-alpha"),
    cgNode("124", "AGUI Heap Mode", "Cortex", "active", "anchor", "eudaemon-alpha"),
    cgNode("132", "Eudaemon Alpha", "Cortex", "active", "anchor", "eudaemon-alpha"),
  ],
  edges: [
    // Systems cross-references
    cgEdge("125", "105", "depends_on", 0.9),
    cgEdge("125", "046", "references", 0.8),
    cgEdge("105", "063", "references", 0.7),
    cgEdge("046", "089", "depends_on", 0.8),
    cgEdge("000", "118", "references", 0.6),
    cgEdge("055", "046", "references", 0.5),

    // Infrastructure web
    cgEdge("004", "031", "references", 0.5),
    cgEdge("014", "017", "depends_on", 0.8),
    cgEdge("042", "051", "references", 0.7),
    cgEdge("020", "075", "references", 0.6),
    cgEdge("044", "042", "references", 0.5),
    cgEdge("085", "031", "depends_on", 0.7),
    cgEdge("035", "017", "references", 0.4),
    cgEdge("006", "008", "references", 0.5),
    cgEdge("011", "085", "references", 0.4),
    cgEdge("027", "013", "depends_on", 0.8),

    // Protocol backbone
    cgEdge("007", "008", "depends_on", 0.9),
    cgEdge("008", "012", "references", 0.6),
    cgEdge("013", "019", "references", 0.7),
    cgEdge("018", "015", "depends_on", 0.7),
    cgEdge("037", "040", "references", 0.6),
    cgEdge("067", "119", "depends_on", 0.8),
    cgEdge("119", "007", "references", 0.7),
    cgEdge("040", "026", "depends_on", 0.8),
    cgEdge("012", "007", "references", 0.6),

    // Runtime connections
    cgEdge("047", "013", "depends_on", 0.9),
    cgEdge("122", "047", "references", 0.7),
    cgEdge("129", "122", "depends_on", 0.8),
    cgEdge("033", "019", "references", 0.5),
    cgEdge("023", "007", "references", 0.4),

    // Product/UX connections
    cgEdge("109", "005", "supersedes", 0.9),
    cgEdge("109", "074", "depends_on", 0.8),
    cgEdge("005", "028", "references", 0.5),

    // Architectural ties
    cgEdge("026", "040", "references", 0.7),
    cgEdge("080", "000", "depends_on", 0.9),
    cgEdge("056", "067", "references", 0.6),
    cgEdge("097", "118", "depends_on", 0.8),
    cgEdge("015", "018", "references", 0.5),

    // Experimental → production paths
    cgEdge("028", "074", "references", 0.6),
    cgEdge("045", "074", "references", 0.5),
    cgEdge("075", "020", "references", 0.7),

    // Cortex internal
    cgEdge("117", "074", "depends_on", 0.9),
    cgEdge("123", "117", "depends_on", 0.8),
    cgEdge("124", "123", "depends_on", 0.8),
    cgEdge("132", "122", "depends_on", 0.9),
    cgEdge("132", "118", "references", 0.7),
    cgEdge("074", "005", "supersedes", 0.8),

    // Cross-layer strategic connections
    cgEdge("118", "000", "depends_on", 0.9),
    cgEdge("122", "014", "references", 0.6),
    cgEdge("132", "047", "references", 0.7),
    cgEdge("124", "007", "references", 0.5),
    cgEdge("123", "109", "references", 0.6),
    cgEdge("097", "119", "references", 0.7),
    cgEdge("089", "063", "references", 0.6),
    cgEdge("031", "004", "supersedes", 0.5),
    cgEdge("067", "013", "references", 0.6),
    cgEdge("080", "119", "references", 0.7),
    cgEdge("056", "122", "references", 0.5),
    cgEdge("033", "125", "references", 0.6),
    cgEdge("051", "037", "depends_on", 0.7),
    cgEdge("046", "055", "references", 0.5),
    cgEdge("085", "007", "references", 0.4),
    cgEdge("119", "080", "depends_on", 0.8),
    cgEdge("129", "047", "depends_on", 0.7),
    cgEdge("045", "005", "references", 0.4),
    cgEdge("117", "109", "references", 0.6),
  ]
};
export const MOCK_NAVIGATION_PLAN = compilePreviewNavigationPlan(INTRO_SPACE_ID);

export const MOCK_UX_WORKBENCH_MAIN = {
  type: "WorkbenchSurface",
  surfaceId: "surface:workbench-main",
  title: "Mock Workbench",
  components: [
    {
      id: "root-container",
      type: "Column",
      props: { gap: "4", padding: "4" },
      children: ["header", "subtitle", "content-area"]
    },
    {
      id: "header",
      type: "Heading",
      props: { text: "Sovereign Local Host Workbench" }
    },
    {
      id: "subtitle",
      type: "Text",
      props: { text: "Running via PWA Service Worker IDB" }
    },
    {
      id: "content-area",
      type: "Card",
      props: { text: "System Status" },
      children: ["status-text"]
    },
    {
      id: "status-text",
      type: "Text",
      props: { text: "The system is fully operating from local IndexedDB mocks." }
    }
  ],
  meta: {}
};

export const MOCK_UX_WORKBENCH_LABS = {
  type: "WorkbenchSurface",
  surfaceId: "surface:workbench-labs",
  title: "Labs",
  components: [
    {
      id: "root",
      type: "Column",
      props: { gap: "6", padding: "6" },
      children: ["title", "desc", "space-studio-card", "promotion-card", "action-grid"]
    },
    {
      id: "title",
      type: "Heading",
      props: { text: "Labs" }
    },
    {
      id: "desc",
      type: "Text",
      props: { text: "Try ideas here before they become live spaces or templates." }
    },
    {
      id: "space-studio-card",
      type: "Card",
      props: { text: "Space Studio" },
      children: ["space-studio-body"]
    },
    {
      id: "space-studio-body",
      type: "Text",
      props: { text: "Draft a new space, test its shape, and decide later whether it should become a live space or a reusable template." }
    },
    {
      id: "promotion-card",
      type: "Card",
      props: { text: "How it works" },
      children: ["promotion-body"]
    },
    {
      id: "promotion-body",
      type: "Text",
      props: { text: "Drafts stay in Labs. A steward can promote a finished draft into a real space when it is ready." }
    },
    {
      id: "action-grid",
      type: "Grid",
      props: {},
      children: ["btn-start-draft", "btn-open-templates"]
    },
    {
      id: "btn-start-draft",
      type: "Button",
      props: { label: "Start draft", href: SPACE_STUDIO_ROUTE }
    },
    {
      id: "btn-open-templates",
      type: "Button",
      props: { label: "Open templates", href: buildSpaceStudioRoute("templates") }
    }
  ]
};

export const MOCK_UX_WORKBENCH_SYSTEM = {
  type: "WorkbenchSurface",
  surfaceId: "surface:workbench-system",
  title: "System Control",
  components: [
    {
      id: "root",
      type: "Column",
      props: { gap: "4" },
      children: ["title", "cap-map", "matrix"]
    },
    {
      id: "title",
      type: "Heading",
      props: { text: "System Overview" }
    },
    {
      id: "cap-map",
      type: "CapabilityMap",
      props: { dataSourceUrl: "/api/system/capability-graph", graphV2Enabled: "true" }
    },
    {
      id: "matrix",
      type: "RulesMatrixWidget",
      props: {}
    }
  ]
};

export const MOCK_UX_WORKBENCH_SPACES = {
  type: "WorkbenchSurface",
  surfaceId: "surface:workbench-spaces",
  title: "Spaces Management",
  components: [
    {
      id: "root",
      type: "Column",
      props: { gap: "4" },
      children: ["title", "alert", "spaces-metric-grid", "spaces-table"]
    },
    {
      id: "title",
      type: "Heading",
      props: { text: "Managed Spaces" }
    },
    {
      id: "alert",
      type: "AlertBanner",
      props: { severity: "info", title: "Storage Warning", message: "Local storage for space 01ARZ... is at 85% capacity." }
    },
    {
      id: "spaces-metric-grid",
      type: "Grid",
      props: {},
      children: ["total-spaces", "active-contributions"]
    },
    {
      id: "total-spaces",
      type: "MetricCard",
      props: { label: "Total Spaces", value: "3" }
    },
    {
      id: "active-contributions",
      type: "MetricCard",
      props: { label: "Active Contribs", value: "8" }
    },
    {
      id: "spaces-table",
      type: "DataTable",
      props: {
        columns: ["ID", "Name", "Created"],
        rows: [
          { ID: INTRO_SPACE_ID, Name: "Intro Preview", Created: "2026-03-13" },
          { ID: "SPACE-002", Name: "Research Lab", Created: "2026-03-01" },
          { ID: "SPACE-003", Name: "Production Alpha", Created: "2026-03-10" }
        ]
      }
    }
  ]
};

export const MOCK_UX_WORKBENCH_HEAP = {
  type: "WorkbenchSurface",
  surfaceId: "surface:workbench-heap",
  title: "Explore",
  components: [
    {
      id: "root",
      type: "Column",
      props: { gap: "0" }, // Full bleed
      children: ["heap-canvas"]
    },
    {
      id: "heap-canvas",
      type: "Canvas",
      props: {}
    }
  ]
};

export const MOCK_UX_WORKBENCH_STUDIO = {
  type: "WorkbenchSurface",
  surfaceId: "surface:workbench-studio",
  title: "Flow Studio",
  components: [
    {
      id: "root",
      type: "Column",
      props: { gap: "6", padding: "6" },
      children: ["title", "desc", "code-card"]
    },
    {
      id: "title",
      type: "Heading",
      props: { text: "Flow Studio" }
    },
    {
      id: "desc",
      type: "Text",
      props: { text: "Agentic Code Generation & A2UI Live Editing" }
    },
    {
      id: "code-card",
      type: "Card",
      props: { text: "Active Session" },
      children: ["markdown-code"]
    },
    {
      id: "markdown-code",
      type: "Markdown",
      props: { content: "```rust\n// Welcome to Flow Studio\n\nfn main() {\n    println!(\"A2UI Live Editor Online\");\n}\n```" }
    }
  ]
};

export function buildMockActionPlan(spaceId: string, routeId: string): any {
  return {
    schemaVersion: "1.0.0",
    generatedAt: new Date().toISOString(),
    planHash: "mock-action-plan-hash",
    spaceId,
    routeId,
    pageType: "heap_board",
    actorRole: "steward",
    zones: [
      {
        zone: "heap_page_bar",
        layoutHint: "row",
        actions: [
          {
            id: "mock.create",
            capabilityId: "cap.heap.create",
            label: "Create Block",
            icon: "plus",
            kind: "panel_toggle",
            action: "create_block",
            group: "primary",
            enabled: true,
            visible: true
          }
        ]
      }
    ]
  };
}
