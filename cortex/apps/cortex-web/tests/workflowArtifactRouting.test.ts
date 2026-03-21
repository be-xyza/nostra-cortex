import assert from "node:assert/strict";
import test from "node:test";

import {
  classifyWorkbenchHref,
  isGatewayApiPath,
  parseWorkflowArtifactPath,
} from "../src/components/workflows/artifactRouting.ts";

test("workflow artifact routing distinguishes workbench, gateway api, and external hrefs", () => {
  assert.equal(classifyWorkbenchHref("/workflows?node_id=workflow_definition:def-1"), "internal_workbench");
  assert.equal(classifyWorkbenchHref("/api/cortex/workflow-instances/instance-1/trace"), "gateway_api");
  assert.equal(classifyWorkbenchHref("https://example.com/docs"), "external");
});

test("isGatewayApiPath only matches workflow gateway artifact routes", () => {
  assert.equal(isGatewayApiPath("/api/cortex/workflow-drafts/proposals/proposal-1/replay"), true);
  assert.equal(isGatewayApiPath("/api/cortex/workflow-definitions/definition-1/projections/flow_graph_v1"), true);
  assert.equal(isGatewayApiPath("/api/system/ux/workbench?route=%2Fworkflows"), false);
  assert.equal(isGatewayApiPath("/workflows?node_id=workflow_definition:def-1"), false);
});

test("parseWorkflowArtifactPath resolves typed workflow artifact targets", () => {
  assert.deepEqual(
    parseWorkflowArtifactPath("/api/cortex/workflow-drafts/proposals/proposal-1/replay"),
    {
      kind: "proposal_replay",
      proposalId: "proposal-1",
      path: "/api/cortex/workflow-drafts/proposals/proposal-1/replay",
    }
  );

  assert.deepEqual(
    parseWorkflowArtifactPath("/api/cortex/workflow-definitions/definition-1/projections/serverless_workflow_v0_8"),
    {
      kind: "definition_projection",
      definitionId: "definition-1",
      projectionKind: "serverless_workflow_v0_8",
      path: "/api/cortex/workflow-definitions/definition-1/projections/serverless_workflow_v0_8",
    }
  );

  assert.deepEqual(
    parseWorkflowArtifactPath("/api/cortex/workflow-instances/instance-1/checkpoints"),
    {
      kind: "instance_checkpoints",
      instanceId: "instance-1",
      path: "/api/cortex/workflow-instances/instance-1/checkpoints",
    }
  );
});
