export interface GateSummaryFailure {
    code: string;
    message: string;
}

export interface GateSummaryRenderModel {
    kind: string;
    title: string;
    generatedAt: string;
    latestRunId: string;
    overallVerdict: string;
    requiredGatesPass: boolean;
    counts: Record<string, unknown>;
    failuresPreview: GateSummaryFailure[];
    failuresOverflow: number;
    openWorkbenchHref: string;
    openLogsHref: string | null;
}

function asRecord(value: unknown): Record<string, unknown> | null {
    if (!value || typeof value !== "object" || Array.isArray(value)) return null;
    return value as Record<string, unknown>;
}

function asString(value: unknown): string | null {
    if (typeof value !== "string") return null;
    const trimmed = value.trim();
    return trimmed.length > 0 ? trimmed : null;
}

function asBoolean(value: unknown): boolean | null {
    return typeof value === "boolean" ? value : null;
}

export function buildGateSummaryRenderModel(data: Record<string, unknown>): GateSummaryRenderModel {
    const kind = asString(data.kind) || "siq";
    const generatedAt = asString(data.generated_at) || "unknown";
    const latestRunId = asString(data.latest_run_id) || "unknown";
    const overallVerdict = asString(data.overall_verdict) || "unknown";
    const requiredGatesPass = asBoolean(data.required_gates_pass) ?? false;
    const counts = asRecord(data.counts) || {};
    const failures = Array.isArray(data.failures) ? data.failures : [];
    const failuresPreview = failures.slice(0, 25).map((row) => {
        const mapped = asRecord(row) || {};
        return {
            code: asString(mapped.code) || "unknown",
            message: asString(mapped.message) || JSON.stringify(row),
        };
    });
    const hints = asRecord(data.render_hints) || {};
    const openWorkbenchHref =
        asString(hints.primary_route) || (kind === "testing" ? "/testing" : "/system/siq");
    const logStreamId = asString(hints.log_stream_id);

    return {
        kind,
        title: kind === "testing" ? "Testing Gate Summary" : "SIQ Gate Summary",
        generatedAt,
        latestRunId,
        overallVerdict,
        requiredGatesPass,
        counts,
        failuresPreview,
        failuresOverflow: Math.max(0, failures.length - failuresPreview.length),
        openWorkbenchHref,
        openLogsHref: logStreamId
            ? `/logs?node_id=log_stream:${encodeURIComponent(logStreamId)}:cursor:0`
            : null,
    };
}
