import test from "node:test";
import assert from "node:assert/strict";

import {
  buildSolicitationRenderModel,
  buildStewardFeedbackRenderModel,
} from "../src/components/heap/solicitationRenderModel.ts";

test("buildSolicitationRenderModel normalizes live structured agent solicitation blocks", () => {
  const model = buildSolicitationRenderModel({
    type: "agent_solicitation",
    role: "research-architect",
    requested_agent_role: "research-architect",
    authority_scope: "L1",
    required_capabilities: [
      "graph-validation",
      "analysis",
      "proposal-drafting",
    ],
    budget: {
      currency: "USD",
      max: 20,
    },
    message: "Review the current VPS artifact lineage before any steward action.",
    space_id: "01KM4C04QY37V9RV9H2HH9J1NM",
  });

  assert.ok(model);
  assert.equal(model?.roleLabel, "research-architect");
  assert.equal(model?.requestedRoleLabel, "research-architect");
  assert.equal(model?.authorityScopeLabel, "L1");
  assert.equal(model?.budgetLabel, "USD 20");
  assert.deepEqual(model?.capabilityLabels, [
    "graph-validation",
    "analysis",
    "proposal-drafting",
  ]);
  assert.match(model?.summary ?? "", /artifact lineage/i);
  assert.match(model?.feedbackHint ?? "", /record/i);
});

test("buildSolicitationRenderModel uses neutral fallback language when role is missing", () => {
  const model = buildSolicitationRenderModel({
    type: "agent_solicitation",
    authority_scope: "L2",
    required_capabilities: ["analysis"],
    message: "Review the draft before publication.",
  });

  assert.ok(model);
  assert.equal(model?.roleLabel, "unspecified");
  assert.equal(model?.requestedRoleLabel, null);
  assert.equal(model?.authorityScopeLabel, "L2");
});

test("buildStewardFeedbackRenderModel preserves review lineage and decision summary", () => {
  const model = buildStewardFeedbackRenderModel({
    type: "steward_feedback",
    artifact_id: "feedback_01KM4CDYTP8RD94Z52HJQQNTTD_1",
    parent_artifact_id: "01KM4CDYTP8RD94Z52HJQQNTTD",
    decision: "approved",
    feedback: "Proceed after validating the authority boundary.",
    submitted_by: "operator",
    submitted_at: "2026-03-20T01:06:00Z",
  });

  assert.ok(model);
  assert.equal(model?.decisionLabel, "Approved");
  assert.equal(model?.artifactId, "feedback_01KM4CDYTP8RD94Z52HJQQNTTD_1");
  assert.equal(model?.parentArtifactId, "01KM4CDYTP8RD94Z52HJQQNTTD");
  assert.match(model?.summary ?? "", /authority boundary/i);
  assert.equal(model?.submittedBy, "operator");
  assert.equal(model?.submittedAt, "2026-03-20T01:06:00Z");
});
