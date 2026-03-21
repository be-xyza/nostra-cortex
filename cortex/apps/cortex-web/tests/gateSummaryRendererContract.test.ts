import test from "node:test";
import assert from "node:assert/strict";
import React from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { buildGateSummaryRenderModel } from "../src/components/heap/gateSummary.ts";

test("gate summary render model exposes deterministic labels and links", () => {
  const model = buildGateSummaryRenderModel({
    schema_id: "nostra.heap.block.gate_summary.v1",
    kind: "siq",
    generated_at: "2026-03-10T00:00:00Z",
    latest_run_id: "siq_run_20260310",
    overall_verdict: "ready",
    required_gates_pass: true,
    counts: {
      pass: 12,
      fail: 1,
    },
    failures: [
      {
        code: "FM-009",
        message: "dynamic source contract violation",
      },
    ],
    render_hints: {
      primary_route: "/system/siq",
      log_stream_id: "siq_gate_summary_latest",
    },
  });

  const html = renderToStaticMarkup(
    React.createElement(
      "div",
      null,
      React.createElement("h1", null, model.title),
      React.createElement("span", null, model.latestRunId),
      React.createElement("span", null, model.overallVerdict),
      React.createElement("span", null, model.failuresPreview[0]?.code ?? ""),
      React.createElement("a", { href: model.openWorkbenchHref }, "Open Workbench"),
      React.createElement("a", { href: model.openLogsHref ?? "" }, "Open Logs")
    )
  );

  assert.match(html, /SIQ Gate Summary/i);
  assert.match(html, /siq_run_20260310/i);
  assert.match(html, /ready/i);
  assert.match(html, /FM-009/i);
  assert.match(html, /Open Workbench/i);
  assert.match(html, /\/system\/siq/i);
  assert.match(html, /Open Logs/i);
  assert.match(
    html,
    /\/logs\?node_id=log_stream:siq_gate_summary_latest:cursor:0/i
  );
});
