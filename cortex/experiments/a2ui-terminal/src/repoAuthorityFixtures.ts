import { ArtifactSurfaceEnvelope } from "./types.js";
import { SEED_EVENTS } from "../../../apps/cortex-web/src/store/seedData.ts";

function buildSeedEnvelope(artifactId: string): ArtifactSurfaceEnvelope {
    const event = SEED_EVENTS.find((candidate) => candidate.payload?.artifactId === artifactId);
    if (!event) {
        throw new Error(`Seed event not found for artifact '${artifactId}'`);
    }

    const payload = event.payload as Record<string, unknown>;
    const surfaceJson = payload.surfaceJson;
    if (!surfaceJson || typeof surfaceJson !== "object" || Array.isArray(surfaceJson)) {
        throw new Error(`Seed event '${artifactId}' is missing a surfaceJson payload`);
    }

    return {
        artifactId,
        title: typeof payload.title === "string" ? payload.title : artifactId,
        routeHint: "explore",
        surfaceJson: surfaceJson as ArtifactSurfaceEnvelope["surfaceJson"],
    };
}

export const REPO_AUTHORITY_FIXTURES: Record<string, ArtifactSurfaceEnvelope> = {
    "repo-heap-note": buildSeedEnvelope("heap-demo-2"),
    "repo-gate-summary": buildSeedEnvelope("mock-gate-1"),
    "repo-workflow-trace": {
        artifactId: "workflow-instance-alpha",
        title: "Workflow Runtime Lens",
        routeHint: "workflows",
        workflowHref: "/api/cortex/workflow-instances/workflow-instance-alpha/trace",
        surfaceJson: {
            payload_type: "structured_data",
            structured_data: {
                schema_id: "cortex.workflow.instance.timeline.v1",
                instance_id: "workflow-instance-alpha",
                status: "waitingcheckpoint",
                updated_at: "2026-03-11T10:10:00Z",
                checkpoints: 1,
            },
        },
    },
};
