import React from "react";

import type { WorkflowTraceResponse } from "../../contracts.ts";

function normalizeTraceEntries(trace: unknown): Array<Record<string, unknown>> {
  if (Array.isArray(trace)) {
    return trace.filter((entry) => entry && typeof entry === "object") as Array<
      Record<string, unknown>
    >;
  }
  if (trace && typeof trace === "object") {
    const events = (trace as { events?: unknown }).events;
    if (Array.isArray(events)) {
      return events.filter((entry) => entry && typeof entry === "object") as Array<
        Record<string, unknown>
      >;
    }
  }
  return [];
}

export function WorkflowInstanceTracePanel({
  response,
}: {
  response: WorkflowTraceResponse;
}) {
  const entries = normalizeTraceEntries(response.trace);
  return (
    <div className="flex flex-col gap-4">
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-sm font-semibold uppercase tracking-[0.18em] text-cortex-ink-muted">
            Runtime Trace
          </h3>
          <p className="text-sm text-cortex-ink-muted">
            {entries.length} trace events captured for this workflow instance.
          </p>
        </div>
      </div>
      {entries.length > 0 ? (
        <div className="flex flex-col gap-2">
          {entries.map((entry, index) => {
            const title =
              typeof entry.event_type === "string"
                ? entry.event_type
                : typeof entry.type === "string"
                  ? entry.type
                  : `event_${index + 1}`;
            const timestamp =
              typeof entry.occurred_at === "string"
                ? entry.occurred_at
                : typeof entry.timestamp === "string"
                  ? entry.timestamp
                  : "-";
            return (
              <div
                key={`${title}-${timestamp}-${index}`}
                className="rounded-cortex border border-cortex-line bg-cortex-bg px-3 py-3"
              >
                <div className="flex items-center justify-between gap-3">
                  <div className="text-sm font-medium text-cortex-ink">{title}</div>
                  <div className="text-[11px] uppercase tracking-[0.18em] text-cortex-ink-faint">
                    {timestamp}
                  </div>
                </div>
                <pre className="mt-2 overflow-auto text-xs text-cortex-ink-muted">
                  {JSON.stringify(entry, null, 2)}
                </pre>
              </div>
            );
          })}
        </div>
      ) : (
        <pre className="rounded-cortex border border-cortex-line bg-[#051325] p-4 text-xs text-cortex-ink-muted overflow-auto">
          {JSON.stringify(response.trace, null, 2)}
        </pre>
      )}
    </div>
  );
}
