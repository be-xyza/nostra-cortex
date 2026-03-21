import assert from "node:assert/strict";
import test from "node:test";

import {
  projectWorkflowInstanceTimeline,
  projectWorkflowProjectionPreview,
  projectWorkflowStatusBadge,
  projectWorkflowSummaryStrip,
} from "../src/components/workflows/workflowWidgetProjection.ts";
import { resolveWorkflowProjectionTabs } from "../src/components/workflows/workflowProjectionTabs.ts";

test("workflow summary strip projects typed metrics", () => {
  const projected = projectWorkflowSummaryStrip({
    WorkflowSummaryStrip: {
      eyebrow: "Governance Summary",
      title: "Workflow Orchestration",
      description: "Governed runtime posture.",
      metrics: [
        { label: "Drafts", value: "2", tone: "default", href: "/workflows?node_id=workflow_draft:alpha" },
        { label: "Blocked", value: "1", tone: "warning" },
      ],
    },
  });

  assert.equal(projected.eyebrow, "Governance Summary");
  assert.equal(projected.title, "Workflow Orchestration");
  assert.equal(projected.metrics.length, 2);
  assert.equal(projected.metrics[0]?.href, "/workflows?node_id=workflow_draft:alpha");
  assert.equal(projected.metrics[1]?.tone, "warning");
});

test("workflow status badge projects label and emphasis", () => {
  const projected = projectWorkflowStatusBadge({
    WorkflowStatusBadge: {
      label: "Proposal",
      status: "ratified",
      emphasis: "success",
      href: "/workflows?node_id=workflow_proposal:proposal-alpha",
    },
  });

  assert.deepEqual(projected, {
    label: "Proposal",
    status: "ratified",
    emphasis: "success",
    href: "/workflows?node_id=workflow_proposal:proposal-alpha",
  });
});

test("workflow projection preview projects definition metadata", () => {
  const projected = projectWorkflowProjectionPreview({
    WorkflowProjectionPreview: {
      eyebrow: "Definition Lens",
      definitionId: "workflow-definition-alpha",
      definitionHref: "/workflows?node_id=workflow_definition:workflow-definition-alpha",
      motif: "parallel_compare",
      digest: "digest-alpha",
      nodeCount: "5",
      projections: [
        { label: "Graph", kind: "flow_graph_v1", href: "/api/cortex/workflow-definitions/workflow-definition-alpha/projections/flow_graph_v1" },
        { label: "A2UI", kind: "a2ui_surface_v1" },
      ],
    },
  });

  assert.equal(projected.eyebrow, "Definition Lens");
  assert.equal(projected.definitionId, "workflow-definition-alpha");
  assert.equal(
    projected.definitionHref,
    "/workflows?node_id=workflow_definition:workflow-definition-alpha"
  );
  assert.equal(
    projected.projections[0]?.href,
    "/api/cortex/workflow-definitions/workflow-definition-alpha/projections/flow_graph_v1"
  );
  assert.equal(projected.projections[0]?.kind, "flow_graph_v1");
});

test("workflow instance timeline projects runtime entries", () => {
  const projected = projectWorkflowInstanceTimeline({
    WorkflowInstanceTimeline: {
      eyebrow: "Runtime Lens",
      title: "Recent Instances",
      entries: [
        {
          instanceId: "workflow-instance-alpha",
          status: "waitingcheckpoint",
          updatedAt: "2026-03-11T10:10:00Z",
          checkpoints: "1",
          outcome: "-",
          href: "/api/cortex/workflow-instances/workflow-instance-alpha/trace",
        },
      ],
    },
  });

  assert.equal(projected.eyebrow, "Runtime Lens");
  assert.equal(projected.title, "Recent Instances");
  assert.equal(projected.entries[0]?.instanceId, "workflow-instance-alpha");
  assert.equal(
    projected.entries[0]?.href,
    "/api/cortex/workflow-instances/workflow-instance-alpha/trace"
  );
});

test("workflow projection tabs prefer server-emitted descriptors", () => {
  const tabs = resolveWorkflowProjectionTabs(
    {
      schema_version: "1.0.0",
      generated_at: "2026-03-11T00:00:00Z",
      projection_kind: "flow_graph_v1",
      projection: {},
      available_projections: [
        { kind: "flow_graph_v1", label: "Graph Lens" },
        { kind: "a2ui_surface_v1", label: "A2UI Surface" },
      ],
    },
    "flow_graph_v1"
  );

  assert.deepEqual(tabs, [
    { key: "flow_graph_v1", label: "Graph Lens" },
    { key: "a2ui_surface_v1", label: "A2UI Surface" },
  ]);
});

test("workflow projection tabs fall back to canonical defaults", () => {
  const tabs = resolveWorkflowProjectionTabs(null, "normalized_graph_v1");

  assert.equal(tabs[0]?.key, "flow_graph_v1");
  assert.equal(tabs[3]?.key, "normalized_graph_v1");
});
