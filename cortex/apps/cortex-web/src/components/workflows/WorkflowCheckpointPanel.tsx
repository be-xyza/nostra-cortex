import React from "react";

import type { WorkflowCheckpointResponse } from "../../contracts.ts";

export function WorkflowCheckpointPanel({
  response,
}: {
  response: WorkflowCheckpointResponse;
}) {
  const checkpoints = Array.isArray(response.checkpoints) ? response.checkpoints : [];
  return (
    <div className="flex flex-col gap-4">
      <div>
        <h3 className="text-sm font-semibold uppercase tracking-[0.18em] text-cortex-ink-muted">
          Checkpoints
        </h3>
        <p className="text-sm text-cortex-ink-muted">
          {checkpoints.length} operator or runtime checkpoints are currently attached.
        </p>
      </div>
      {checkpoints.length > 0 ? (
        <div className="flex flex-col gap-2">
          {checkpoints.map((checkpoint, index) => {
            const record =
              checkpoint && typeof checkpoint === "object"
                ? (checkpoint as Record<string, unknown>)
                : { value: checkpoint };
            const title =
              typeof record.checkpoint_id === "string"
                ? record.checkpoint_id
                : typeof record.title === "string"
                  ? record.title
                  : `checkpoint_${index + 1}`;
            return (
              <div
                key={`${title}-${index}`}
                className="rounded-cortex border border-cortex-line bg-cortex-bg px-3 py-3"
              >
                <div className="text-sm font-medium text-cortex-ink">{title}</div>
                <pre className="mt-2 overflow-auto text-xs text-cortex-ink-muted">
                  {JSON.stringify(record, null, 2)}
                </pre>
              </div>
            );
          })}
        </div>
      ) : (
        <div className="rounded-cortex border border-dashed border-cortex-line bg-cortex-bg px-3 py-4 text-sm text-cortex-ink-faint">
          No checkpoints captured yet.
        </div>
      )}
    </div>
  );
}
