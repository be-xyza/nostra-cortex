import assert from "node:assert/strict";
import test from "node:test";

import { summarizeWorkflowDefinition } from "../src/components/workflows/workflowDefinitionSummary.ts";

test("workflow definition summary reads typed motif and digest fields", () => {
  const summary = summarizeWorkflowDefinition({
    schema_version: "1.0.0",
    generated_at: "2026-03-11T00:00:00Z",
    definition: {
      schemaVersion: "1.0.0",
      definitionId: "definition-1",
      scope: { spaceId: "space-alpha" },
      intent: "Typed definition metadata",
      motifKind: "sequential",
      constraints: [],
      graph: { nodes: [], edges: [] },
      contextContract: { allowedSections: ["inputs"] },
      confidence: { score: 0.9, rationale: "fixture" },
      lineage: { mergeRefs: [] },
      policy: {
        recommendationOnly: false,
        requireReview: true,
        allowShadowExecution: false,
      },
      provenance: {
        createdBy: "tester",
        createdAt: "2026-03-11T00:00:00Z",
        sourceMode: "fixture",
      },
      digest: "sha256:typed-definition",
    },
  });

  assert.equal(summary.motifKind, "sequential");
  assert.equal(summary.digest, "sha256:typed-definition");
});
