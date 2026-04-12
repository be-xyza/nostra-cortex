import { ArtifactSurfaceEnvelope, A2UINode, HandoffTarget, RenderPlan, SurfacePayload } from "./types.js";
export { resolveFixtureArg } from "./envelopeLoader.js";
import { buildWorkbenchHandoffUrl } from "./workbenchRoutes.js";
import { summarizeTerminalDocumentFailure, validateTerminalDocument } from "./terminalDocument.js";

export function planSurfaceRender(
    envelope: ArtifactSurfaceEnvelope,
    options: { cortexWebBaseUrl?: string } = {},
): RenderPlan {
    const payload = envelope.surfaceJson ?? {};
    const payloadType = normalizePayloadType(payload);
    const title = envelope.title || readString(payload.title) || envelope.artifactId || "Untitled Surface";
    const handoff = buildHandoff(envelope, options.cortexWebBaseUrl);

    if (payloadType === "a2ui") {
        const a2uiTree = extractA2UITree(payload);
        const validation = a2uiTree ? validateTerminalDocument(a2uiTree) : null;
        if (a2uiTree && validation?.valid) {
            return {
                mode: "terminal_render",
                title,
                summaryLines: [
                    "Terminal-safe A2UI subset detected.",
                    "This surface can stay inside the terminal without losing the interaction model.",
                ],
                reasons: [
                    "All declared widgets are supported by the terminal adapter.",
                    "The terminal document passes the strict terminal_document_v1 validation gate.",
                ],
                terminalTree: a2uiTree,
            };
        }

        const a2uiReason =
            a2uiTree
                ? summarizeTerminalDocumentFailure(a2uiTree)
                : "A2UI payload does not expose a terminal-safe component tree.";
        return {
            mode: "web_handoff",
            title,
            summaryLines: [
                "This A2UI surface exceeds the current terminal widget subset.",
                summarizeArtifactIntent(envelope),
            ],
            reasons: [a2uiReason],
            handoff: withFallbackHandoff(handoff, "Open in cortex-web for full A2UI fidelity."),
        };
    }

    if (payloadType === "rich_text" || payloadType === "note" || payloadType === "task") {
        return {
            mode: "terminal_summary",
            title,
            summaryLines: summarizeTextPayload(payload),
            reasons: ["Textual payloads map cleanly to terminal-first reading and review."],
        };
    }

    if (payloadType === "structured_data" || payloadType === "pointer") {
        return {
            mode: handoff ? "terminal_summary_with_handoff" : "terminal_summary",
            title,
            summaryLines: summarizeStructuredPayload(payload, payloadType),
            reasons: [
                payloadType === "structured_data"
                    ? "Structured data can be summarized in terminal while keeping richer drill-down in cortex-web."
                    : "Pointers can be previewed in terminal and opened in cortex-web when deeper navigation is needed.",
            ],
            handoff,
        };
    }

    if (payloadType === "chart" || payloadType === "telemetry" || payloadType === "media") {
        return {
            mode: "terminal_summary_with_handoff",
            title,
            summaryLines: summarizeRichSurface(payload, payloadType),
            reasons: [
                `${payloadType} surfaces are better served by cortex-web's richer renderer and interaction patterns.`,
            ],
            handoff: withFallbackHandoff(handoff, `Open in cortex-web for ${payloadType} rendering.`),
        };
    }

    return {
        mode: handoff ? "terminal_summary_with_handoff" : "terminal_summary",
        title,
        summaryLines: [
            `Payload type '${payloadType}' is not explicitly modeled yet.`,
            "Terminal can provide a compact summary while cortex-web remains the richer fallback surface.",
        ],
        reasons: ["Unknown payloads should degrade to summary-first, not implicit rendering."],
        handoff,
    };
}

export function buildTerminalPlanTree(plan: RenderPlan): A2UINode {
    const children: A2UINode[] = [
        textNode("header", `${plan.title}`, "cyan"),
        spacerNode("space-1"),
        boxNode("mode-box", [
            textNode("mode", `Mode: ${plan.mode}`),
            ...plan.summaryLines.map((line, index) => textNode(`summary-${index}`, line)),
        ]),
    ];

    if (plan.reasons.length > 0) {
        children.push(spacerNode("space-2"));
        children.push(
            boxNode(
                "reason-box",
                [textNode("reason-title", "Why this route?"), ...plan.reasons.map((line, index) => textNode(`reason-${index}`, `- ${line}`))],
                "gray",
            ),
        );
    }

    if (plan.handoff) {
        children.push(spacerNode("space-3"));
        children.push(
            boxNode("handoff-box", [
                textNode("handoff-title", "Web handoff available", "cyan"),
                textNode("handoff-reason", plan.handoff.reason),
                textNode("handoff-url", plan.handoff.url),
            ]),
        );
        children.push(spacerNode("space-4"));
        children.push(
            selectNode("handoff-actions", [
                {
                    value: `open_web:${plan.handoff.url}`,
                    label: "Open in cortex-web",
                    description: "Print the handoff URL for the richer workbench surface",
                },
                {
                    value: "stay_terminal",
                    label: "Stay in terminal",
                    description: "Keep the compact terminal summary",
                },
            ]),
        );
        return {
            id: "terminal-plan-root",
            type: "Container",
            componentProperties: {},
            children: { explicitList: children },
        };
    }

    if (plan.mode !== "terminal_render") {
        children.push(spacerNode("space-5"));
        children.push(
            selectNode("summary-actions", [
                {
                    value: "complete",
                    label: "Acknowledge Summary",
                    description: "Exit after reviewing the terminal summary",
                },
            ]),
        );
    }

    return {
        id: "terminal-plan-root",
        type: "Container",
        componentProperties: {},
        children: { explicitList: children },
    };
}

export function hasPlanOnlyFlag(args: string[]): boolean {
    return args.includes("--plan-only");
}

function extractA2UITree(payload: SurfacePayload): A2UINode | null {
    const directTree = payload as unknown as A2UINode;
    if (looksLikeNodeTree(directTree)) {
        return directTree;
    }
    const nestedTree = payload.a2ui?.tree;
    if (nestedTree && looksLikeNodeTree(nestedTree as A2UINode)) {
        return nestedTree as A2UINode;
    }
    const tree = payload.tree;
    if (tree && looksLikeNodeTree(tree as A2UINode)) {
        return tree as A2UINode;
    }
    return null;
}

function looksLikeNodeTree(value: A2UINode | null | undefined): boolean {
    return Boolean(value && typeof value === "object" && typeof value.type === "string");
}

function buildHandoff(
    envelope: ArtifactSurfaceEnvelope,
    cortexWebBaseUrl?: string,
): HandoffTarget | undefined {
    const url = buildWorkbenchHandoffUrl(envelope, cortexWebBaseUrl);
    if (!url) {
        return undefined;
    }
    return {
        surface: "cortex-web",
        url,
        reason: envelope.workflowHref
            ? "Workflow or artifact navigation requires the richer cortex-web surface."
            : "Artifact drill-down is available through the web workbench.",
    };
}

function withFallbackHandoff(
    handoff: HandoffTarget | undefined,
    reason: string,
): HandoffTarget | undefined {
    if (handoff) {
        return handoff;
    }
    const baseUrl = trimTrailingSlash(process.env.CORTEX_WEB_BASE_URL || "http://127.0.0.1:4173");
    return {
        surface: "cortex-web",
        url: `${baseUrl}/explore`,
        reason,
    };
}

function trimTrailingSlash(value: string): string {
    return value.endsWith("/") ? value.slice(0, -1) : value;
}

function normalizePayloadType(payload: SurfacePayload): string {
    const candidate = readString(payload.payload_type)?.toLowerCase();
    if (candidate) {
        return candidate;
    }
    if (payload.a2ui || looksLikeNodeTree(payload as unknown as A2UINode)) {
        return "a2ui";
    }
    if (payload.media) {
        return "media";
    }
    if (payload.structured_data || payload.data) {
        return "structured_data";
    }
    if (payload.pointer) {
        return "pointer";
    }
    if (payload.text || payload.plain_text) {
        return "rich_text";
    }
    return "unknown";
}

function summarizeArtifactIntent(envelope: ArtifactSurfaceEnvelope): string {
    if (envelope.routeHint === "workflows") {
        return "Workflow inspection and projection tabs belong in cortex-web.";
    }
    if (envelope.routeHint === "artifacts") {
        return "Artifact inspection belongs in cortex-web when terminal loses fidelity.";
    }
    return "Terminal can summarize the surface, but cortex-web owns richer navigation and inspection.";
}

function summarizeTextPayload(payload: SurfacePayload): string[] {
    const text = readString(payload.text) || readString(payload.plain_text) || "";
    const lines = text
        .split(/\r?\n/)
        .map((line) => line.trim())
        .filter((line) => line.length > 0)
        .slice(0, 6);
    if (lines.length > 0) {
        return lines;
    }
    return ["No textual preview was available."];
}

function summarizeStructuredPayload(payload: SurfacePayload, payloadType: string): string[] {
    if (payloadType === "pointer") {
        return [readString(payload.pointer) || "Pointer target unavailable."];
    }
    const data = asRecord(payload.structured_data) || asRecord(payload.data) || {};
    const entries = Object.entries(data).slice(0, 6);
    if (entries.length === 0) {
        return ["Structured payload present, but no preview fields were found."];
    }
    return entries.map(([key, value]) => `${key}: ${stringifyScalar(value)}`);
}

function summarizeRichSurface(payload: SurfacePayload, payloadType: string): string[] {
    if (payloadType === "media") {
        return [
            `Media URL: ${readString(payload.media?.url) || "n/a"}`,
            `MIME type: ${readString(payload.media?.mime_type) || "n/a"}`,
        ];
    }

    const tree = asRecord(payload.tree) || {};
    const entries = Object.entries(tree).slice(0, 5);
    if (entries.length > 0) {
        return entries.map(([key, value]) => `${key}: ${stringifyScalar(value)}`);
    }
    return [`${payloadType} payload detected; open cortex-web for the full renderer.`];
}

function asRecord(value: unknown): Record<string, unknown> | null {
    if (value && typeof value === "object" && !Array.isArray(value)) {
        return value as Record<string, unknown>;
    }
    return null;
}

function stringifyScalar(value: unknown): string {
    if (typeof value === "string") {
        return value;
    }
    if (typeof value === "number" || typeof value === "boolean") {
        return String(value);
    }
    if (Array.isArray(value)) {
        return `Array(${value.length})`;
    }
    if (value && typeof value === "object") {
        return `Object(${Object.keys(value).length})`;
    }
    return "n/a";
}

function readString(value: unknown): string | undefined {
    if (typeof value !== "string") {
        return undefined;
    }
    const normalized = value.trim();
    return normalized.length > 0 ? normalized : undefined;
}

function textNode(id: string, content: string, color?: string): A2UINode {
    return {
        id,
        type: "Text",
        componentProperties: {
            content,
            ...(color ? { color } : {}),
        },
    };
}

function spacerNode(id: string): A2UINode {
    return {
        id,
        type: "Spacer",
        componentProperties: { lines: 1 },
    };
}

function boxNode(id: string, children: A2UINode[], bg?: string): A2UINode {
    return {
        id,
        type: "Box",
        componentProperties: {
            paddingX: 2,
            paddingY: 1,
            ...(bg ? { bg } : {}),
        },
        children: { explicitList: children },
    };
}

function selectNode(
    id: string,
    items: Array<{ value: string; label: string; description: string }>,
): A2UINode {
    return {
        id,
        type: "SelectList",
        componentProperties: {
            items,
            maxVisible: items.length,
        },
    };
}
