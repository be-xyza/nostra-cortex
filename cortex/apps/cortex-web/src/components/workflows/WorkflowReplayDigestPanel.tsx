import React from "react";

function renderCompactValue(value: unknown): string {
  if (value === null || value === undefined) return "-";
  if (typeof value === "string") return value;
  if (typeof value === "number" || typeof value === "boolean") return String(value);
  if (Array.isArray(value)) return `${value.length} items`;
  return `${Object.keys(value as Record<string, unknown>).length} fields`;
}

function readSummaryRows(payload: unknown): Array<{ label: string; value: string }> {
  if (!payload || typeof payload !== "object" || Array.isArray(payload)) return [];
  return Object.entries(payload as Record<string, unknown>)
    .slice(0, 6)
    .map(([label, value]) => ({ label, value: renderCompactValue(value) }));
}

export function WorkflowReplayDigestPanel({
  title,
  payload,
}: {
  title: string;
  payload: unknown;
}) {
  const summaryRows = readSummaryRows(payload);
  return (
    <div className="flex flex-col gap-4">
      <div className="flex flex-col gap-1">
        <h3 className="text-sm font-semibold uppercase tracking-[0.18em] text-cortex-ink-muted">
          {title}
        </h3>
        <p className="text-sm text-cortex-ink-muted">
          Durable artifact payload rendered inline for workflow review.
        </p>
      </div>
      {summaryRows.length > 0 ? (
        <div className="grid gap-2 md:grid-cols-2">
          {summaryRows.map((row) => (
            <div
              key={row.label}
              className="rounded-cortex border border-cortex-line bg-cortex-bg px-3 py-2"
            >
              <div className="text-[11px] uppercase tracking-[0.18em] text-cortex-ink-faint">
                {row.label}
              </div>
              <div className="mt-1 text-sm text-cortex-ink">{row.value}</div>
            </div>
          ))}
        </div>
      ) : null}
      <pre className="rounded-cortex border border-cortex-line bg-[#051325] p-4 text-xs text-cortex-ink-muted overflow-auto">
        {JSON.stringify(payload, null, 2)}
      </pre>
    </div>
  );
}
