import { ArtifactSurfaceEnvelope } from "./types.js";
import { REPO_AUTHORITY_FIXTURES } from "./repoAuthorityFixtures.js";

const BASE_FIXTURES: Record<string, ArtifactSurfaceEnvelope> = {
    "terminal-approval": {
        artifactId: "fixture-terminal-approval",
        title: "Terminal Approval Surface",
        routeHint: "explore",
        surfaceJson: {
            payload_type: "a2ui",
            a2ui: {
                tree: {
                    id: "root-container",
                    type: "Container",
                    componentProperties: {},
                    children: {
                        explicitList: [
                            {
                                id: "welcome-text",
                                type: "Text",
                                componentProperties: {
                                    content: "System Initialized: Cortex Eudaemon Linked.\nExecuting terminal-safe approval surface...",
                                    color: "cyan",
                                },
                            },
                            {
                                id: "spacer-1",
                                type: "Spacer",
                                componentProperties: { lines: 1 },
                            },
                            {
                                id: "agent-status-box",
                                type: "Box",
                                componentProperties: {
                                    bg: "gray",
                                    paddingX: 2,
                                    paddingY: 1,
                                },
                                children: {
                                    explicitList: [
                                        {
                                            id: "agent-status-text",
                                            type: "Text",
                                            componentProperties: {
                                                content: "Active Agent: Space Topology Mapper\nStatus: Awaiting steward action",
                                            },
                                        },
                                    ],
                                },
                            },
                            {
                                id: "spacer-2",
                                type: "Spacer",
                                componentProperties: { lines: 1 },
                            },
                            {
                                id: "action-list",
                                type: "SelectList",
                                componentProperties: {
                                    items: [
                                        { value: "approve", label: "Approve Plan", description: "Proceed with graph synthesis" },
                                        { value: "reject", label: "Reject Plan", description: "Halt operations and retain artifact only" },
                                        { value: "edit", label: "Modify Params", description: "Steer the agent logic" },
                                    ],
                                    maxVisible: 3,
                                },
                            },
                        ],
                    },
                },
            },
        },
    },
    "workflow-handoff": {
        artifactId: "fixture-workflow-handoff",
        title: "Workflow Artifact Inspector",
        routeHint: "workflows",
        workflowHref: "/api/cortex/workflow-instances/inst_demo_001/trace",
        surfaceJson: {
            payload_type: "a2ui",
            tree: {
                widget: "WorkflowInstanceTimeline",
                run_id: "inst_demo_001",
                projection_kind: "trace",
            },
        },
    },
    "media-review": {
        artifactId: "fixture-media-review",
        title: "Architecture Diagram Draft",
        routeHint: "artifacts",
        surfaceJson: {
            payload_type: "media",
            media: {
                url: "https://images.unsplash.com/photo-1518770660439-4636190af475?q=80&w=2000",
                mime_type: "image/jpeg",
            },
        },
    },
    "gate-summary": {
        artifactId: "fixture-gate-summary",
        title: "Infrastructure Gate Summary",
        routeHint: "explore",
        surfaceJson: {
            payload_type: "structured_data",
            structured_data: {
                schema_id: "nostra.heap.block.gate_summary.v1",
                overall_verdict: "FAILED",
                counts: { total_tests: 100, passed: 97, failed: 3, skipped: 0 },
            },
        },
    },
};

export const FIXTURES: Record<string, ArtifactSurfaceEnvelope> = {
    ...BASE_FIXTURES,
    ...REPO_AUTHORITY_FIXTURES,
};
