import { ArtifactSurfaceEnvelope } from "./types.js";
import {
    classifyWorkbenchHref,
    isGatewayApiPath,
    normalizeWorkflowHref,
} from "../../../apps/cortex-web/src/components/workflows/artifactRouting.ts";

export function buildWorkbenchHandoffUrl(
    envelope: Pick<ArtifactSurfaceEnvelope, "artifactId" | "routeHint" | "workflowHref">,
    cortexWebBaseUrl?: string,
): string | undefined {
    const baseUrl = trimTrailingSlash(cortexWebBaseUrl || process.env.CORTEX_WEB_BASE_URL || "http://127.0.0.1:4173");
    const workflowHref = readString(envelope.workflowHref);
    if (workflowHref) {
        return buildWorkflowWorkbenchUrl(baseUrl, workflowHref);
    }

    if (envelope.artifactId) {
        const route = envelope.routeHint === "artifacts" ? "/artifacts" : "/explore";
        return `${baseUrl}${route}?artifact_id=${encodeURIComponent(envelope.artifactId)}`;
    }

    if (envelope.routeHint) {
        const route = envelope.routeHint === "explore" ? "/explore" : `/${envelope.routeHint}`;
        return `${baseUrl}${route}`;
    }

    return undefined;
}

export function buildWorkflowWorkbenchUrl(baseUrl: string, workflowHref: string): string {
    const normalized = normalizeWorkflowHref(workflowHref);
    const kind = classifyWorkbenchHref(normalized);
    if (kind === "external") {
        return workflowHref;
    }
    if (kind === "internal_workbench") {
        return `${baseUrl}${normalized.startsWith("/") ? normalized : `/${normalized}`}`;
    }
    if (isGatewayApiPath(normalized)) {
        return `${baseUrl}/workflows?node_id=${encodeURIComponent(normalized)}`;
    }
    return `${baseUrl}${normalized.startsWith("/") ? normalized : `/${normalized}`}`;
}

function trimTrailingSlash(value: string): string {
    return value.endsWith("/") ? value.slice(0, -1) : value;
}

function readString(value: unknown): string | undefined {
    if (typeof value !== "string") {
        return undefined;
    }
    const normalized = value.trim();
    return normalized.length > 0 ? normalized : undefined;
}
