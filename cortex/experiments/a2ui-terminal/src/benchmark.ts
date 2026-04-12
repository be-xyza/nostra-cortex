import { performance } from "node:perf_hooks";

import { resolveInputEnvelope } from "./envelopeLoader.js";
import { FIXTURES } from "./fixtures.js";
import { buildTerminalPlanTree, planSurfaceRender } from "./planner.js";
import { validateTerminalDocument } from "./terminalDocument.js";

const ITERATIONS = Number.parseInt(process.env.A2UI_TERMINAL_BENCH_ITERATIONS || "500", 10);

const args = process.argv.slice(2);
const explicitInput = args.some((arg) => arg.startsWith("--fixture") || arg.startsWith("--payload-file") || arg === "--fixture" || arg === "--payload-file");
const sources = explicitInput
    ? [resolveInputEnvelope(args)]
    : Object.entries(FIXTURES).map(([key, envelope]) => ({ key, envelope }));

for (const source of sources) {
    const startedAt = performance.now();
    for (let index = 0; index < ITERATIONS; index += 1) {
        const plan = planSurfaceRender(source.envelope, {
            cortexWebBaseUrl: process.env.CORTEX_WEB_BASE_URL,
        });
        validateTerminalDocument(
            plan.mode === "terminal_render" && plan.terminalTree
                ? plan.terminalTree
                : buildTerminalPlanTree(plan),
        );
    }
    const finishedAt = performance.now();
    const totalMs = finishedAt - startedAt;
    console.log(JSON.stringify({
        source: source.key,
        iterations: ITERATIONS,
        total_ms: Number(totalMs.toFixed(3)),
        avg_ms: Number((totalMs / ITERATIONS).toFixed(6)),
    }));
}
