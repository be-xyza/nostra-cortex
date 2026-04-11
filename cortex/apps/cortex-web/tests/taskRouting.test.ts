import assert from "node:assert/strict";
import test from "node:test";

import {
  buildTaskRoutePrompt,
  buildTaskRouteLineageSnapshot,
  buildTaskRouteExecutionResult,
  buildTaskRouteSummaryEmitRequest,
  buildTaskRouteSourceStampEmitRequest,
  decideTaskRoute,
  inferWorkflowMotifKind,
  parseTaskRoutingContextFromAttributes,
} from "../src/components/heap/taskRouting.ts";

test("parses task routing context from serialized heap attributes", () => {
  const context = parseTaskRoutingContextFromAttributes({
    task_context: JSON.stringify({
      version: "1.0.0",
      initiative_id: "078",
      title: "Initiative 078 Kickoff",
      objective: "Load the kickoff task.",
      decision_mode: "auto_if_clear_else_proposal",
      agent_role: "research-architect",
      required_capabilities: ["research-analysis", "ontology-design"],
      required_tasks: ["Review the plan."],
      success_criteria: ["The router can classify the task."],
      bottleneck_signals: [],
      error_signals: [],
      fallback_routes: [],
      routing_options: [],
      reference_paths: ["research/078-knowledge-graphs/PLAN.md"],
    }),
  });

  assert.ok(context);
  assert.equal(context?.initiative_id, "078");
  assert.equal(context?.agent_role, "research-architect");
  assert.deepEqual(context?.reference_paths, [
    "research/078-knowledge-graphs/PLAN.md",
  ]);
});

test("routes a clear kickoff task to direct agent assignment", () => {
  const decision = decideTaskRoute({
    version: "1.0.0",
    initiative_id: "078",
    title: "Initiative 078 Kickoff",
    objective: "Load the initiative 078 graph kickoff as a reusable task.",
    decision_mode: "auto_if_clear_else_proposal",
    agent_role: "research-architect",
    required_capabilities: [
      "research-analysis",
      "schema-registry",
      "ontology-design",
      "graph-query-contracts",
      "retrieval-governance",
    ],
    required_tasks: [
      "Read the initiative plan.",
      "Audit schema locations.",
      "Draft the schema registry index.",
    ],
    success_criteria: ["The router can classify the task."],
    bottleneck_signals: [],
    error_signals: [],
    fallback_routes: [],
    routing_options: [],
    reference_paths: ["research/078-knowledge-graphs/PLAN.md"],
  });

  assert.equal(decision.route_id, "direct_agent_assignment");
  assert.equal(decision.surface_mode, "chat");
  assert.equal(decision.confidence_hint, "high");
  assert.ok(decision.prompt.includes("Initiative 078 Kickoff"));
  assert.ok(decision.prompt.includes("Read the initiative plan."));
});

test("routes a workflow draft request to proposal generation", () => {
  const decision = decideTaskRoute({
    version: "1.0.0",
    initiative_id: "initiative-xyz-kickoff",
    title: "Initiative XYZ Kickoff",
    objective: "Draft a workflow proposal for review and ratification.",
    decision_mode: "auto_if_clear_else_proposal",
    agent_role: "research-architect",
    required_capabilities: ["workflow-design"],
    required_tasks: ["Draft the workflow.", "Prepare the proposal."],
    success_criteria: ["A proposal artifact exists."],
    bottleneck_signals: [],
    error_signals: [],
    fallback_routes: [],
    routing_options: [],
    reference_paths: ["research/xyz/PLAN.md"],
  });

  assert.equal(decision.route_id, "proposal_generation");
  assert.equal(decision.surface_mode, "generate");
  assert.equal(decision.confidence_hint, "medium");
  assert.ok(decision.prompt.includes("workflow proposal"));
});

test("routes ambiguous or risky tasks to steward escalation", () => {
  const decision = decideTaskRoute({
    version: "1.0.0",
    initiative_id: "initiative-risk",
    title: "Risky Kickoff",
    objective: "Do the risky thing.",
    decision_mode: "auto_if_clear_else_proposal",
    agent_role: "research-architect",
    required_capabilities: ["analysis"],
    required_tasks: ["Wait for the missing registry ownership decision."],
    success_criteria: ["The steward confirms the next step."],
    bottleneck_signals: ["Missing registry ownership"],
    error_signals: ["Policy ambiguity"],
    fallback_routes: ["Escalate to steward"],
    routing_options: [],
    reference_paths: ["research/risk/PLAN.md"],
  });

  assert.equal(decision.route_id, "steward_escalation");
  assert.equal(decision.surface_mode, "chat");
  assert.equal(decision.confidence_hint, "low");
  assert.ok(decision.prompt.includes("Missing registry ownership"));
});

test("builds a route prompt that preserves the routing context", () => {
  const prompt = buildTaskRoutePrompt({
    version: "1.0.0",
    initiative_id: "078",
    title: "Initiative 078 Kickoff",
    objective: "Load the initiative 078 graph kickoff.",
    decision_mode: "auto_if_clear_else_proposal",
    agent_role: "research-architect",
    required_capabilities: ["research-analysis"],
    required_tasks: ["Read the plan."],
    success_criteria: ["The router can classify the task."],
    bottleneck_signals: [],
    error_signals: [],
    fallback_routes: [],
    routing_options: [],
    reference_paths: ["research/078-knowledge-graphs/PLAN.md"],
  });

  assert.ok(prompt.includes("Initiative 078 Kickoff"));
  assert.ok(prompt.includes("Read the plan."));
  assert.ok(prompt.includes("research/078-knowledge-graphs/PLAN.md"));
});

test("builds a route summary artifact for direct agent assignment without re-embedding task context", () => {
  const context = {
    version: "1.0.0",
    initiative_id: "078",
    title: "Initiative 078 Kickoff",
    objective: "Load the initiative 078 graph kickoff as a reusable task.",
    decision_mode: "auto_if_clear_else_proposal",
    agent_role: "research-architect",
    required_capabilities: [
      "research-analysis",
      "schema-registry",
      "ontology-design",
    ],
    required_tasks: ["Review the plan."],
    success_criteria: ["The router can classify the task."],
    bottleneck_signals: [],
    error_signals: [],
    fallback_routes: [],
    routing_options: [],
    reference_paths: ["research/078-knowledge-graphs/PLAN.md"],
  } as const;

  const decision = decideTaskRoute(context);
  const request = buildTaskRouteSummaryEmitRequest(context, {
    route_id: decision.route_id,
    decision,
    source_task_artifact_id: "artifact-078-task",
    space_id: "space-078",
    routed_at: "2026-03-23T00:00:00Z",
  });

  assert.equal(request.block.type, "agent_solicitation");
  assert.equal(request.content.payload_type, "structured_data");
  assert.equal(request.block.attributes?.source_task_artifact_id, "artifact-078-task");
  assert.equal(request.block.attributes?.route_id, "direct_agent_assignment");
  assert.equal(request.block.attributes?.task_context, undefined);
  assert.equal(request.content.structured_data?.type, "agent_solicitation");
  assert.ok(String(request.content.structured_data?.message ?? "").includes("Direct agent assignment"));
  assert.ok(String(request.content.structured_data?.message ?? "").includes("research/078-knowledge-graphs/PLAN.md"));
});

test("infers a compare-heavy workflow motif for proposal generation", () => {
  const motif = inferWorkflowMotifKind({
    version: "1.0.0",
    initiative_id: "initiative-xyz",
    title: "Workflow Draft Proposal",
    objective: "Draft a proposal and compare candidate workflows before ratification.",
    decision_mode: "auto_if_clear_else_proposal",
    agent_role: "workflow-architect",
    required_capabilities: ["workflow-design"],
    required_tasks: ["Compare candidate drafts.", "Stage the best workflow."],
    success_criteria: ["A workflow proposal exists."],
    bottleneck_signals: [],
    error_signals: [],
    fallback_routes: [],
    routing_options: [],
    reference_paths: [],
  });

  assert.equal(motif, "parallel_compare");
});

test("builds a workflow proposal summary artifact with proposal lineage metadata", () => {
  const context = {
    version: "1.0.0",
    initiative_id: "initiative-xyz",
    title: "Workflow Draft Proposal",
    objective: "Draft a proposal and compare candidate workflows before ratification.",
    decision_mode: "auto_if_clear_else_proposal",
    agent_role: "workflow-architect",
    required_capabilities: ["workflow-design"],
    required_tasks: ["Compare candidate drafts.", "Stage the best workflow."],
    success_criteria: ["A workflow proposal exists."],
    bottleneck_signals: [],
    error_signals: [],
    fallback_routes: [],
    routing_options: [],
    reference_paths: ["research/134-hybrid-workflow-authority-and-execution/PLAN.md"],
  } as const;

  const decision = decideTaskRoute(context);
  const request = buildTaskRouteSummaryEmitRequest(context, {
    route_id: "proposal_generation",
    decision,
    source_task_artifact_id: "artifact-workflow-task",
    space_id: "space-workflow",
    routed_at: "2026-03-23T00:00:00Z",
    workflow: {
      workflow_intent_id: "workflow_intent_1",
      candidate_set_id: "workflow_set_1",
      workflow_draft_id: "workflow_draft_1",
      proposal_id: "workflow_proposal_1",
      definition_id: "workflow_def_1",
      scope_key: "space:space-workflow",
      motif_kind: "parallel_compare",
      generation_mode: "motif_hybrid",
      proposal_digest_path: "/api/cortex/workflow-drafts/proposals/workflow_proposal_1/digest",
    },
  });

  assert.equal(request.block.type, "agent_solicitation");
  assert.equal(request.content.payload_type, "structured_data");
  assert.equal(request.block.attributes?.workflow_proposal_id, "workflow_proposal_1");
  assert.equal(request.block.attributes?.workflow_draft_id, "workflow_draft_1");
  assert.equal(request.block.attributes?.workflow_candidate_set_id, "workflow_set_1");
  assert.equal(request.block.attributes?.task_context, undefined);
  assert.equal(request.content.structured_data?.type, "agent_solicitation");
  assert.ok(String(request.content.structured_data?.message ?? "").includes("workflow_proposal_1"));
  assert.ok(String(request.content.structured_data?.message ?? "").includes("/api/cortex/workflow-drafts/proposals/workflow_proposal_1/digest"));
});

test("builds a route stamp request that persists the chosen route onto the source task", () => {
  const context = {
    version: "1.0.0",
    initiative_id: "078",
    title: "Initiative 078 Kickoff",
    objective: "Load the initiative 078 graph kickoff as a reusable task.",
    decision_mode: "auto_if_clear_else_proposal",
    agent_role: "research-architect",
    required_capabilities: ["research-analysis", "schema-registry"],
    required_tasks: ["Read the initiative plan."],
    success_criteria: ["The router can classify the task."],
    bottleneck_signals: [],
    error_signals: [],
    fallback_routes: [],
    routing_options: [],
    reference_paths: ["research/078-knowledge-graphs/PLAN.md"],
  } as const;

  const decision = decideTaskRoute(context);
  const request = buildTaskRouteSourceStampEmitRequest(context, {
    source_block: {
      projection: {
        artifactId: "artifact-078-task",
        spaceId: "space-078",
        blockType: "task",
        title: "Initiative 078 Kickoff",
        updatedAt: "2026-03-23T00:00:00Z",
        tags: ["initiative"],
        mentionsInline: ["reference-1"],
        pageLinks: ["page-1"],
        attributes: {
          initiative_id: "078",
          task_context: JSON.stringify(context),
        },
      },
      surfaceJson: {
        payload_type: "task",
        task: "Load the initiative 078 graph kickoff as a reusable task.",
      },
    },
    route_id: decision.route_id,
    decision,
    summary_artifact_id: "artifact-078-route-summary",
    routed_at: "2026-03-23T00:00:00Z",
  });

  assert.equal(request.block.id, "artifact-078-task");
  assert.equal(request.block.type, "task");
  assert.equal(request.block.attributes?.task_route_id, "direct_agent_assignment");
  assert.equal(request.block.attributes?.task_route_summary_artifact_id, "artifact-078-route-summary");
  assert.equal(request.content.payload_type, "task");
  assert.equal(request.content.task, "Load the initiative 078 graph kickoff as a reusable task.");
  assert.ok(request.relations?.mentions?.some((mention) => mention.to_block_id === "artifact-078-route-summary"));
  assert.ok(request.relations?.mentions?.some((mention) => mention.to_block_id === "reference-1"));
  assert.ok(String(request.block.attributes?.task_route_summary ?? "").includes("Route the task directly"));
});

test("builds a route lineage snapshot from stamped task attributes and relations", () => {
  const block = {
    projection: {
      artifactId: "artifact-078-task",
      spaceId: "space-078",
      blockType: "task",
      title: "Initiative 078 Kickoff",
      updatedAt: "2026-03-23T00:00:00Z",
      tags: [],
      mentionsInline: [],
      pageLinks: [],
      attributes: {
        initiative_id: "078",
        task_context: JSON.stringify({
          version: "1.0.0",
          initiative_id: "078",
          title: "Initiative 078 Kickoff",
          objective: "Load the initiative 078 graph kickoff as a reusable task.",
          decision_mode: "auto_if_clear_else_proposal",
          agent_role: "research-architect",
          required_capabilities: ["research-analysis"],
          required_tasks: ["Read the initiative plan."],
          success_criteria: ["The router can classify the task."],
          bottleneck_signals: [],
          error_signals: [],
          fallback_routes: [],
          routing_options: [],
          reference_paths: ["research/078-knowledge-graphs/PLAN.md"],
        }),
        task_route_id: "direct_agent_assignment",
        task_route_decision_mode: "auto_if_clear_else_proposal",
        task_route_confidence_hint: "high",
        task_route_summary: "Route the task directly to the most relevant specialist agent.",
        task_route_rationale: "The task is bounded enough to hand to a specialist agent without escalation.",
        task_route_summary_artifact_id: "artifact-078-route-summary",
        workflow_proposal_id: "workflow_proposal_1",
        workflow_draft_id: "workflow_draft_1",
        workflow_intent_id: "workflow_intent_1",
        workflow_candidate_set_id: "workflow_set_1",
        workflow_definition_id: "workflow_def_1",
        workflow_scope_key: "space:space-078",
        workflow_motif_kind: "parallel_compare",
        workflow_generation_mode: "motif_hybrid",
        workflow_proposal_digest_path: "/api/cortex/workflow-drafts/proposals/workflow_proposal_1/digest",
      },
    },
    surfaceJson: {
      payload_type: "task",
      task: "Load the initiative 078 graph kickoff as a reusable task.",
    },
  };

  const lineage = buildTaskRouteLineageSnapshot(block, {
    outboundLinks: [],
    outboundMentions: [
      {
        id: "artifact-078-route-summary",
        title: "Initiative 078 Route Summary",
        isNavigable: true,
      },
    ],
    backlinks: [],
    tagNeighbors: [],
    semanticLineage: [],
  });

  assert.ok(lineage);
  assert.equal(lineage?.route_id, "direct_agent_assignment");
  assert.equal(lineage?.summary_artifact_title, "Initiative 078 Route Summary");
  assert.equal(lineage?.summary_artifact_linked, true);
  assert.equal(lineage?.workflow?.proposal_id, "workflow_proposal_1");
  assert.ok(String(lineage?.route_summary ?? "").includes("specialist agent"));
});

test("builds a standardized execution result for a successful route application", () => {
  const result = buildTaskRouteExecutionResult({
    route_id: "proposal_generation",
    route_label: "Generate proposal or workflow draft",
    confidence_hint: "medium",
    source_task_artifact_id: "artifact-078-task",
    summary_artifact_id: "artifact-078-route-summary",
    stamped_source_artifact_id: "artifact-078-task",
    routed_at: "2026-03-23T00:00:00Z",
    success: true,
    workflow: {
      workflow_intent_id: "workflow_intent_1",
      candidate_set_id: "workflow_set_1",
      workflow_draft_id: "workflow_draft_1",
      proposal_id: "workflow_proposal_1",
      definition_id: "workflow_def_1",
      scope_key: "space:space-078",
      motif_kind: "parallel_compare",
      generation_mode: "motif_hybrid",
    },
  });

  assert.equal(result.success, true);
  assert.equal(result.summary_artifact_id, "artifact-078-route-summary");
  assert.equal(result.stamped_source_artifact_id, "artifact-078-task");
  assert.equal(result.workflow?.proposal_id, "workflow_proposal_1");
});

test("builds a standardized execution result for a failed route application", () => {
  const result = buildTaskRouteExecutionResult({
    route_id: "steward_escalation",
    route_label: "Escalate to steward",
    confidence_hint: "low",
    source_task_artifact_id: "artifact-078-task",
    routed_at: "2026-03-23T00:00:00Z",
    success: false,
    failure_reason: "Missing registry ownership decision.",
  });

  assert.equal(result.success, false);
  assert.equal(result.failure_reason, "Missing registry ownership decision.");
  assert.equal(result.summary_artifact_id, undefined);
});
