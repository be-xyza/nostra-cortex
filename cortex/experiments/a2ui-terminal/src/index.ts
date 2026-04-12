import { ProcessTerminal, TUI } from "@mariozechner/pi-tui";
import { resolveInputEnvelope } from "./envelopeLoader.js";
import { buildTerminalPlanTree, hasPlanOnlyFlag, planSurfaceRender } from "./planner.js";
import { createTUIComponent } from "./A2UITerminalInterpreter.js";
import { validateTerminalDocument } from "./terminalDocument.js";

const terminal = new ProcessTerminal();
const tui = new TUI(terminal);
const source = resolveInputEnvelope(process.argv.slice(2));
const plan = planSurfaceRender(source.envelope, {
    cortexWebBaseUrl: process.env.CORTEX_WEB_BASE_URL,
});
const rootTree = plan.mode === "terminal_render" && plan.terminalTree
    ? plan.terminalTree
    : buildTerminalPlanTree(plan);
const terminalDocumentValidation = validateTerminalDocument(rootTree);

if (hasPlanOnlyFlag(process.argv.slice(2))) {
    console.log(JSON.stringify({
        fixture: source.key,
        mode: plan.mode,
        title: plan.title,
        reasons: plan.reasons,
        summaryLines: plan.summaryLines,
        handoff: plan.handoff ?? null,
        terminalDocument: rootTree,
        terminalDocumentValidation,
    }, null, 2));
    process.exit(0);
}

if (!terminalDocumentValidation.valid) {
    console.error(JSON.stringify({
        error: "terminal_document_validation_failed",
        details: terminalDocumentValidation.errors,
    }, null, 2));
    process.exit(1);
}

const rootComponent = createTUIComponent(rootTree, tui);

if (rootComponent) {
    tui.addChild(rootComponent);
}

// Global debug key handler (Shift+Ctrl+D)
tui.onDebug = () => console.log("Debug triggered");

tui.start();
