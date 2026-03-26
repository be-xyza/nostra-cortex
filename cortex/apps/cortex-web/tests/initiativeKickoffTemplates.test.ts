import assert from "node:assert/strict";
import test from "node:test";

import { GENERATED_INITIATIVE_KICKOFF_SOURCES } from "../src/components/heap/generatedInitiativeKickoffRegistry.ts";
import {
  buildInitiative078KickoffTemplate,
  buildInitiativeKickoffCatalogSpec,
  buildInitiativeKickoffEmitRequest,
  buildInitiativeKickoffTemplate,
  buildTaskRoutingContext,
  buildGuidedTaskBody,
  canLaunchInitiativeKickoff,
  filterInitiativeKickoffTemplates,
  INITIATIVE_KICKOFF_REGISTRY_DIAGNOSTICS,
  INITIATIVE_KICKOFF_TEMPLATES,
  resolveInitiativeKickoffRegistry,
  resolveInitiativeKickoffTemplate,
  type InitiativeKickoffSource,
} from "../src/components/heap/initiativeKickoffTemplates.ts";

test("generated kickoff registry exposes the governed 078 and 132 sources", () => {
  assert.deepEqual(
    GENERATED_INITIATIVE_KICKOFF_SOURCES.map((entry) => entry.plan?.id),
    ["078", "132"],
  );
});

test("initiative kickoff registry resolves the governed 078 and 132 entries without diagnostics", () => {
  assert.deepEqual(
    INITIATIVE_KICKOFF_TEMPLATES.map((entry) => entry.template.initiativeId),
    ["078", "132"],
  );
  assert.deepEqual(INITIATIVE_KICKOFF_TEMPLATES.map((entry) => entry.label), [
    "078 kickoff approval",
    "132 kickoff approval",
  ]);
  assert.deepEqual(INITIATIVE_KICKOFF_REGISTRY_DIAGNOSTICS, []);
});

test("initiative 078 kickoff template includes the source-of-truth files and staged work items", () => {
  const template = buildInitiative078KickoffTemplate();

  assert.equal(template.id, "initiative-078-kickoff");
  assert.equal(template.initiativeId, "078");
  assert.equal(template.title, "Knowledge Graphs — Schema, Ontology & Graph Strategy");
  assert.equal(template.blockType, "agent_solicitation");
  assert.equal(template.agentRole, "research-architect");
  assert.deepEqual(template.requiredCapabilities, [
    "research-analysis",
    "schema-registry",
    "ontology-design",
    "graph-query-contracts",
    "retrieval-governance",
  ]);

  for (const path of [
    "research/078-knowledge-graphs/PLAN.md",
    "research/078-knowledge-graphs/DECISIONS.md",
    "research/078-knowledge-graphs/CHECKLIST.md",
    "research/reference/analysis/trustgraph.md",
    "research/reference/analysis/trustgraph_capability_matrix.md",
  ]) {
    assert.ok(template.referencePaths.includes(path), `expected reference path ${path}`);
    assert.ok(template.body.includes(path), `expected body to mention ${path}`);
  }

  for (const phrase of [
    "Objective:",
    "Audit existing JSON schema locations",
    "Draft the M21 schema registry index",
    "Define the minimal ontology scope",
    "Success criteria:",
    "Bottleneck signals:",
    "Error signals:",
    "Fallback routes:",
    "Routing options:",
    "Direct agent assignment",
    "Generate proposal or workflow draft",
    "Escalate to steward",
    "Specify the `KnowledgeBundle` manifest fields",
    "Prototype the read-only triple facade",
    "Keep GraphRAG gated",
  ]) {
    assert.ok(template.body.includes(phrase), `expected body to mention ${phrase}`);
  }
});

test("initiative 132 kickoff template proves the pattern is reusable for a second initiative", () => {
  const template = resolveInitiativeKickoffTemplate("initiative-132-kickoff");

  assert.ok(template);
  assert.equal(template?.initiativeId, "132");
  assert.equal(template?.agentRole, "systems-architect");
  assert.ok(template?.body.includes("heap for exploratory work"));
  assert.ok(template?.body.includes("workflow-backed promotion paths"));
  assert.ok(template?.referencePaths.includes("research/132-eudaemon-alpha-initiative/PLAN.md"));
});

test("guided task routing context is reusable and serializable", () => {
  const context = buildTaskRoutingContext({
    id: "initiative-xyz-kickoff",
    initiativeId: "xyz",
    title: "Initiative XYZ Kickoff",
    objective: "Keep the launch explicit and recoverable.",
    agentRole: "research-architect",
    requiredCapabilities: ["analysis"],
    referencePaths: ["research/xyz/PLAN.md"],
    requiredTasks: ["Review the plan."],
    successCriteria: ["The router can classify the task."],
    bottleneckSignals: ["Unknown dependencies"],
    errorSignals: ["Missing references"],
    fallbackRoutes: ["Escalate to proposal review"],
  });

  const body = buildGuidedTaskBody(context);

  assert.equal(context.version, "1.0.0");
  assert.equal(context.initiative_id, "xyz");
  assert.equal(context.decision_mode, "auto_if_clear_else_proposal");
  assert.ok(body.includes("Objective:"));
  assert.ok(body.includes("Keep the launch explicit and recoverable."));
  assert.ok(body.includes("Bottleneck signals:"));
  assert.ok(body.includes("Error signals:"));
  assert.ok(body.includes("Fallback routes:"));
  assert.ok(body.includes("Routing options:"));
});

test("generic kickoff template builder is reusable for other initiatives", () => {
  const template = buildInitiativeKickoffTemplate({
    id: "initiative-xyz-kickoff",
    initiativeId: "xyz",
    title: "Initiative XYZ Kickoff",
    agentRole: "research-architect",
    requiredCapabilities: ["analysis"],
    referencePaths: ["research/xyz/PLAN.md"],
    requiredTasks: ["Review the plan."],
  });

  assert.equal(template.id, "initiative-xyz-kickoff");
  assert.equal(template.title, "Initiative XYZ Kickoff");
  assert.equal(template.blockType, "agent_solicitation");
  assert.ok(template.body.includes("Review the plan."));
  assert.ok(template.body.includes("research/xyz/PLAN.md"));
  assert.equal(
    template.approvalSummary,
    "Steward-backed kickoff approval for initiative xyz before any live kickoff task is emitted.",
  );
  assert.ok(template.approvalRationale.includes("should not bypass stewardship"));
  assert.ok(template.approvalMessage.includes("record steward approval"));
});

test("kickoff templates can override approval framing per initiative without leaking 078 rationale", () => {
  const template = buildInitiativeKickoffTemplate({
    id: "initiative-abc-kickoff",
    initiativeId: "abc",
    title: "Initiative ABC Kickoff",
    agentRole: "workflow-architect",
    requiredCapabilities: ["workflow-design"],
    referencePaths: ["research/abc/PLAN.md"],
    requiredTasks: ["Draft the kickoff packet."],
    approvalSummary: "Steward-backed kickoff approval for initiative abc before the workflow-authority pilot starts.",
    approvalRationale: "Approval confirms the workflow-authority scope, dependencies, and rollback plan are explicit before execution.",
    approvalMessage: "Review the Initiative ABC kickoff packet and approve it before emitting any runnable task.",
  });

  const request = buildInitiativeKickoffEmitRequest(
    template,
    "space-abc",
    "2026-03-26T00:00:00Z",
  );

  assert.equal(
    request.content.structured_data?.summary,
    "Steward-backed kickoff approval for initiative abc before the workflow-authority pilot starts.",
  );
  assert.equal(
    request.content.structured_data?.rationale,
    "Approval confirms the workflow-authority scope, dependencies, and rollback plan are explicit before execution.",
  );
  assert.equal(
    request.content.structured_data?.message,
    "Review the Initiative ABC kickoff packet and approve it before emitting any runnable task.",
  );
});

test("registry builder rejects kickoff sources without kickoff metadata", () => {
  const resolved = resolveInitiativeKickoffRegistry([
    {
      directory: "999-missing-kickoff",
      plan: {
        id: "999",
        title: "Missing Kickoff",
        status: "active",
        primarySteward: "Systems Steward",
        planPath: "research/999-missing-kickoff/PLAN.md",
      },
      kickoff: null,
    },
  ]);

  assert.equal(resolved.entries.length, 0);
  assert.equal(resolved.diagnostics[0]?.code, "missing_kickoff");
});

test("registry builder rejects inactive initiatives", () => {
  const source: InitiativeKickoffSource = {
    directory: "xyz-inactive",
    plan: {
      id: "xyz",
      title: "Inactive Initiative",
      status: "hold",
      primarySteward: "Systems Steward",
      planPath: "research/xyz/PLAN.md",
    },
    kickoff: {
      enabled: true,
      templateId: "initiative-xyz-kickoff",
      label: "XYZ kickoff approval",
      description: "Load xyz.",
      agentRole: "research-architect",
      requiredCapabilities: ["analysis"],
      referencePaths: ["research/xyz/PLAN.md"],
      requiredTasks: ["Review the plan."],
    },
  };

  const resolved = buildInitiativeKickoffCatalogSpec(source);
  assert.equal(resolved.spec, null);
  assert.equal(resolved.diagnostics[0]?.code, "inactive_plan");
});

test("registry builder rejects initiative id mismatches between kickoff metadata and plan metadata", () => {
  const source: InitiativeKickoffSource = {
    directory: "xyz-mismatch",
    plan: {
      id: "xyz",
      title: "Mismatched Initiative",
      status: "active",
      primarySteward: "Systems Steward",
      planPath: "research/xyz/PLAN.md",
    },
    kickoff: {
      enabled: true,
      initiativeId: "abc",
      templateId: "initiative-xyz-kickoff",
      label: "XYZ kickoff approval",
      description: "Load xyz.",
      agentRole: "research-architect",
      requiredCapabilities: ["analysis"],
      referencePaths: ["research/xyz/PLAN.md"],
      requiredTasks: ["Review the plan."],
    },
  };

  const resolved = buildInitiativeKickoffCatalogSpec(source);
  assert.equal(resolved.spec, null);
  assert.equal(resolved.diagnostics[0]?.code, "initiative_id_mismatch");
});

test("registry builder rejects initiatives without steward metadata", () => {
  const source: InitiativeKickoffSource = {
    directory: "xyz-no-steward",
    plan: {
      id: "xyz",
      title: "No Steward",
      status: "active",
      primarySteward: null,
      planPath: "research/xyz/PLAN.md",
    },
    kickoff: {
      enabled: true,
      templateId: "initiative-xyz-kickoff",
      label: "XYZ kickoff approval",
      description: "Load xyz.",
      agentRole: "research-architect",
      requiredCapabilities: ["analysis"],
      referencePaths: ["research/xyz/PLAN.md"],
      requiredTasks: ["Review the plan."],
    },
  };

  const resolved = buildInitiativeKickoffCatalogSpec(source);
  assert.equal(resolved.spec, null);
  assert.equal(resolved.diagnostics[0]?.code, "missing_steward");
});

test("registry builder rejects kickoff metadata without required routing fields", () => {
  const source: InitiativeKickoffSource = {
    directory: "xyz-incomplete",
    plan: {
      id: "xyz",
      title: "Incomplete Kickoff",
      status: "active",
      primarySteward: "Systems Steward",
      planPath: "research/xyz/PLAN.md",
    },
    kickoff: {
      enabled: true,
      templateId: "initiative-xyz-kickoff",
      label: "XYZ kickoff approval",
      description: "Load xyz.",
      agentRole: "",
      requiredCapabilities: [],
      referencePaths: [],
      requiredTasks: [],
    },
  };

  const resolved = buildInitiativeKickoffCatalogSpec(source);
  assert.equal(resolved.spec, null);
  assert.deepEqual(
    resolved.diagnostics.map((entry) => entry.code),
    [
      "missing_agent_role",
      "missing_required_capabilities",
      "missing_required_tasks",
    ],
  );
});

test("catalog filtering still hides dismissed kickoff templates", () => {
  const visible = filterInitiativeKickoffTemplates(
    INITIATIVE_KICKOFF_TEMPLATES,
    ["initiative-078-kickoff"],
  );

  assert.deepEqual(
    visible.map((entry) => entry.template.id),
    ["initiative-132-kickoff"],
  );
});

test("initiative kickoff emit requests materialize a steward-backed approval request", async () => {
  const mod: any = await import("../src/components/heap/initiativeKickoffTemplates.ts");
  const template = mod.buildInitiative078KickoffTemplate();
  const request = mod.buildInitiativeKickoffEmitRequest(
    template,
    "01ARZ3NDEKTSV4RRFFQ69G5FAV",
    "2026-03-23T00:00:00Z",
  );

  assert.equal(request.block.type, "agent_solicitation");
  assert.equal(request.block.title, "Knowledge Graphs — Schema, Ontology & Graph Strategy Approval");
  assert.deepEqual(request.block.behaviors, ["awaiting_approval"]);
  assert.equal(request.content.payload_type, "structured_data");
  assert.equal(request.block.attributes?.initiative_id, "078");
  assert.equal(request.block.attributes?.initiative_kickoff_template_id, "initiative-078-kickoff");
  assert.equal(request.block.attributes?.requested_agent_role, "research-architect");
  assert.equal(request.block.attributes?.task_context_version, "1.0.0");
  assert.equal(request.block.attributes?.routing_decision_mode, "auto_if_clear_else_proposal");
  assert.equal(request.block.attributes?.approval_required, "steward");
  assert.equal(request.block.attributes?.review_lane, "private_review");
  assert.ok(request.block.attributes?.task_context);
  assert.equal(request.content.structured_data?.type, "agent_solicitation");
  assert.equal(request.content.structured_data?.solicitation_kind, "initiative_kickoff_approval");
  assert.equal(request.content.structured_data?.initiative_id, "078");
  assert.equal(request.content.structured_data?.requested_agent_role, "research-architect");
  assert.equal(request.content.structured_data?.authority_scope, "steward_gate");
  assert.ok(String(request.content.structured_data?.message ?? "").includes("record steward approval"));
  assert.ok(String(request.content.structured_data?.summary ?? "").includes("staged M21-M25"));
  assert.ok(String(request.content.structured_data?.rationale ?? "").includes("GraphRAG gating"));
  assert.ok(String(request.content.structured_data?.description ?? "").includes("Routing options:"));

  const taskContext = JSON.parse(String(request.block.attributes?.task_context ?? "{}")) as Record<string, unknown>;
  assert.equal(taskContext["initiative_id"], "078");
  assert.equal(taskContext["agent_role"], "research-architect");
  assert.equal(taskContext["decision_mode"], "auto_if_clear_else_proposal");
  assert.ok(Array.isArray(taskContext["bottleneck_signals"]));
  assert.ok(Array.isArray(taskContext["error_signals"]));
  assert.ok(Array.isArray(taskContext["fallback_routes"]));
  assert.ok(Array.isArray(taskContext["routing_options"]));
});

test("initiative 132 emit requests reuse the same approval-first primitive", () => {
  const template = resolveInitiativeKickoffTemplate("initiative-132-kickoff");
  assert.ok(template);

  const request = buildInitiativeKickoffEmitRequest(
    template!,
    "01ARZ3NDEKTSV4RRFFQ69G5FAV",
    "2026-03-26T00:00:00Z",
  );

  assert.equal(request.block.type, "agent_solicitation");
  assert.equal(request.block.attributes?.initiative_id, "132");
  assert.equal(request.block.attributes?.approval_kind, "initiative_kickoff");
  assert.equal(request.content.structured_data?.solicitation_kind, "initiative_kickoff_approval");
  assert.equal(request.content.structured_data?.requested_agent_role, "systems-architect");
});

test("initiative kickoff launch requires an operator-capable role", () => {
  assert.equal(canLaunchInitiativeKickoff("viewer"), false);
  assert.equal(canLaunchInitiativeKickoff("editor"), false);
  assert.equal(canLaunchInitiativeKickoff("operator"), true);
  assert.equal(canLaunchInitiativeKickoff("steward"), true);
  assert.equal(canLaunchInitiativeKickoff("admin"), true);
});
