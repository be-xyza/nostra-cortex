import { existsSync, readFileSync } from "node:fs";
import { resolve } from "node:path";

import { FIXTURES } from "./fixtures.js";
import { ArtifactSurfaceEnvelope, SurfacePayload } from "./types.js";

export type EnvelopeSource = {
    key: string;
    envelope: ArtifactSurfaceEnvelope;
};

export function resolveInputEnvelope(args: string[]): EnvelopeSource {
    const payloadFile = resolvePayloadFileArg(args);
    if (payloadFile) {
        return {
            key: payloadFile,
            envelope: loadEnvelopeFromFile(payloadFile),
        };
    }

    const fixtureKey = resolveFixtureArg(args);
    return {
        key: fixtureKey,
        envelope: FIXTURES[fixtureKey] ?? FIXTURES["terminal-approval"],
    };
}

export function resolveFixtureArg(args: string[]): string {
    const fixtureFlagIndex = args.findIndex((arg) => arg === "--fixture");
    if (fixtureFlagIndex >= 0 && args[fixtureFlagIndex + 1]) {
        return args[fixtureFlagIndex + 1]!;
    }

    const compactFixtureArg = args.find((arg) => arg.startsWith("--fixture="));
    if (compactFixtureArg) {
        return compactFixtureArg.slice("--fixture=".length);
    }

    return process.env.A2UI_TERMINAL_FIXTURE || "terminal-approval";
}

export function resolvePayloadFileArg(args: string[]): string | null {
    const payloadFileFlagIndex = args.findIndex((arg) => arg === "--payload-file");
    if (payloadFileFlagIndex >= 0 && args[payloadFileFlagIndex + 1]) {
        return args[payloadFileFlagIndex + 1]!;
    }

    const compactPayloadArg = args.find((arg) => arg.startsWith("--payload-file="));
    if (compactPayloadArg) {
        return compactPayloadArg.slice("--payload-file=".length);
    }

    return process.env.A2UI_TERMINAL_PAYLOAD_FILE || null;
}

function loadEnvelopeFromFile(pathValue: string): ArtifactSurfaceEnvelope {
    const resolvedPath = resolvePayloadPath(pathValue);
    const raw = readFileSync(resolvedPath, "utf8");
    const parsed = JSON.parse(raw) as Record<string, unknown>;

    if (parsed.surfaceJson && typeof parsed.surfaceJson === "object" && !Array.isArray(parsed.surfaceJson)) {
        return {
            artifactId: asOptionalString(parsed.artifactId),
            title: asOptionalString(parsed.title),
            routeHint: asRouteHint(parsed.routeHint),
            workflowHref: asOptionalString(parsed.workflowHref),
            surfaceJson: parsed.surfaceJson as SurfacePayload,
        };
    }

    return {
        artifactId: asOptionalString(parsed.artifactId),
        title: asOptionalString(parsed.title),
        routeHint: asRouteHint(parsed.routeHint),
        workflowHref: asOptionalString(parsed.workflowHref),
        surfaceJson: parsed as unknown as SurfacePayload,
    };
}

function resolvePayloadPath(pathValue: string): string {
    const cwdResolved = resolve(process.cwd(), pathValue);
    if (existsSync(cwdResolved)) {
        return cwdResolved;
    }

    const workspaceRoot = process.env.NOSTRA_WORKSPACE_ROOT;
    if (workspaceRoot) {
        const workspaceResolved = resolve(workspaceRoot, pathValue);
        if (existsSync(workspaceResolved)) {
            return workspaceResolved;
        }
    }

    return cwdResolved;
}

function asOptionalString(value: unknown): string | undefined {
    if (typeof value !== "string") {
        return undefined;
    }
    const normalized = value.trim();
    return normalized.length > 0 ? normalized : undefined;
}

function asRouteHint(value: unknown): ArtifactSurfaceEnvelope["routeHint"] {
    switch (value) {
        case "explore":
        case "artifacts":
        case "workflows":
        case "labs":
            return value;
        default:
            return undefined;
    }
}
