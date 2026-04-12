export type A2UIChildCollection = {
    explicitList?: A2UINode[];
};

export type A2UINode = {
    id?: string;
    type?: string;
    componentProperties?: Record<string, unknown>;
    children?: A2UIChildCollection;
};

export type SurfacePayloadType =
    | "a2ui"
    | "rich_text"
    | "note"
    | "media"
    | "structured_data"
    | "pointer"
    | "task"
    | "chart"
    | "telemetry";

export type SurfacePayload = {
    payload_type?: SurfacePayloadType | string;
    text?: string;
    plain_text?: string;
    pointer?: string;
    media?: {
        url?: string;
        mime_type?: string;
    };
    structured_data?: Record<string, unknown>;
    data?: Record<string, unknown>;
    tree?: Record<string, unknown>;
    a2ui?: {
        tree?: Record<string, unknown>;
        [key: string]: unknown;
    };
    meta?: Record<string, unknown>;
    [key: string]: unknown;
};

export type ArtifactSurfaceEnvelope = {
    artifactId?: string;
    title?: string;
    routeHint?: "explore" | "artifacts" | "workflows" | "labs";
    workflowHref?: string;
    surfaceJson: SurfacePayload;
};

export type RenderMode =
    | "terminal_render"
    | "terminal_summary"
    | "terminal_summary_with_handoff"
    | "web_handoff";

export type HandoffTarget = {
    surface: "cortex-web";
    url: string;
    reason: string;
};

export type RenderPlan = {
    mode: RenderMode;
    title: string;
    summaryLines: string[];
    reasons: string[];
    handoff?: HandoffTarget;
    terminalTree?: A2UINode;
};
