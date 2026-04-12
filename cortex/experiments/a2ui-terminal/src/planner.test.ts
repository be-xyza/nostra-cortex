import test from "node:test";
import assert from "node:assert/strict";

import { FIXTURES } from "./fixtures.js";
import { buildTerminalPlanTree, planSurfaceRender } from "./planner.js";
import { buildWorkbenchHandoffUrl } from "./workbenchRoutes.js";
import { validateTerminalDocument } from "./terminalDocument.js";

test("terminal-safe A2UI stays in terminal render mode", () => {
    const plan = planSurfaceRender(FIXTURES["terminal-approval"], {
        cortexWebBaseUrl: "http://127.0.0.1:4173",
    });

    assert.equal(plan.mode, "terminal_render");
    assert.ok(plan.terminalTree);
    assert.equal(plan.handoff, undefined);
});

test("workflow inspector produces web handoff", () => {
    const plan = planSurfaceRender(FIXTURES["workflow-handoff"], {
        cortexWebBaseUrl: "http://127.0.0.1:4173",
    });

    assert.equal(plan.mode, "web_handoff");
    assert.ok(plan.handoff);
    assert.match(plan.handoff!.url, /\/workflows\?node_id=/);
    assert.match(plan.handoff!.url, /workflow-instances%2Finst_demo_001%2Ftrace/);
});

test("structured data stays terminal-readable while preserving workbench drill-down", () => {
    const plan = planSurfaceRender(FIXTURES["gate-summary"], {
        cortexWebBaseUrl: "http://127.0.0.1:4173",
    });

    assert.equal(plan.mode, "terminal_summary_with_handoff");
    assert.ok(plan.summaryLines.some((line) => line.includes("overall_verdict")));
    assert.match(plan.handoff!.url, /explore\?artifact_id=fixture-gate-summary/);
});

test("unsupported A2UI widgets fail closed into web handoff", () => {
    const plan = planSurfaceRender(FIXTURES["workflow-handoff"], {
        cortexWebBaseUrl: "http://127.0.0.1:4173",
    });

    assert.equal(plan.mode, "web_handoff");
    assert.ok(plan.reasons.some((reason) =>
        reason.includes("web-scoped")
        || reason.includes("not implemented")
        || reason.includes("validation failed")
        || reason.includes("does not expose"),
    ));
});

test("repo-backed note fixture stays terminal-readable", () => {
    const plan = planSurfaceRender(FIXTURES["repo-heap-note"], {
        cortexWebBaseUrl: "http://127.0.0.1:4173",
    });

    assert.equal(plan.mode, "terminal_summary");
    assert.ok(plan.summaryLines.some((line) => line.includes("Cortex")));
});

test("repo-backed structured-data fixture keeps workbench handoff", () => {
    const plan = planSurfaceRender(FIXTURES["repo-gate-summary"], {
        cortexWebBaseUrl: "http://127.0.0.1:4173",
    });

    assert.equal(plan.mode, "terminal_summary_with_handoff");
    assert.match(plan.handoff!.url, /explore\?artifact_id=mock-gate-1/);
});

test("contract-backed workflow trace fixture reuses workbench route semantics", () => {
    const url = buildWorkbenchHandoffUrl(FIXTURES["repo-workflow-trace"], "http://127.0.0.1:4173");
    assert.equal(
        url,
        "http://127.0.0.1:4173/workflows?node_id=%2Fapi%2Fcortex%2Fworkflow-instances%2Fworkflow-instance-alpha%2Ftrace",
    );
});

test("terminal plan tree validates against terminal_document_v1", () => {
    const plan = planSurfaceRender(FIXTURES["repo-gate-summary"], {
        cortexWebBaseUrl: "http://127.0.0.1:4173",
    });

    const validation = validateTerminalDocument(buildTerminalPlanTree(plan));
    assert.equal(validation.valid, true);
});
