import React from "react";
import type { HeapStewardGateValidateResponse, Json, SuggestedEnrichment } from "../../contracts";
import { A2UIInterpreter, type A2UINode } from "../a2ui/A2UIInterpreter";
import "./heap.css";

interface StewardGateModalProps {
  artifactId: string;
  gate: HeapStewardGateValidateResponse;
  applyingId?: string | null;
  publishing?: boolean;
  onClose: () => void;
  onApply: (enrichmentId: string) => Promise<void> | void;
  onPublish: () => Promise<void> | void;
  onRevalidate: () => Promise<void> | void;
}

function toSurfaceNode(surface: Json | undefined, fallbackSummary: string): A2UINode | null {
  if (!surface || typeof surface !== "object") {
    return null;
  }
  const componentLike = surface as Record<string, unknown>;
  const title = String(componentLike.title ?? "Steward Gate");
  const summary = String(
    ((componentLike.components as Array<Record<string, unknown>> | undefined)?.[0]?.props as Record<string, unknown> | undefined)
      ?.description ?? fallbackSummary
  );

  const actions =
    (((componentLike.meta as Record<string, unknown> | undefined)?.actions as Array<Record<string, unknown>> | undefined) ?? [])
      .map((action): A2UINode | null => {
        const props = (action.props as Record<string, unknown> | undefined) ?? {};
        const enrichmentId = props.enrichmentId;
        if (typeof enrichmentId !== "string" || enrichmentId.trim().length === 0) {
          return null;
        }
        return {
          id: String(action.id ?? `steward_action_${enrichmentId}`),
          type: "Button",
          componentProperties: {
            Button: {
              label: String(props.label ?? `Apply ${enrichmentId}`),
              action: `stewardGateApply?enrichmentId=${enrichmentId}`,
              enrichmentId,
            }
          },
          children: { explicitList: [] },
        };
      })
      .filter((node): node is A2UINode => node !== null);

  return {
    id: "steward_gate_surface",
    type: "Card",
    componentProperties: {
      Card: {
        text: title,
      }
    },
    children: {
      explicitList: [
        {
          id: "steward_gate_summary",
          type: "Text",
          componentProperties: {
            Text: {
              text: summary,
            }
          },
          children: { explicitList: [] },
        },
        ...actions,
      ]
    }
  };
}

function enrichmentKey(enrichment: SuggestedEnrichment): string {
  return enrichment.enrichmentId;
}

export function StewardGateModal({
  artifactId,
  gate,
  applyingId,
  publishing,
  onClose,
  onApply,
  onPublish,
  onRevalidate,
}: StewardGateModalProps) {
  const summary = gate.outcome.shouldBlock
    ? "Publish is blocked until critical Commons violations are resolved."
    : "Review and optionally apply suggested enrichments before publish.";
  const surfaceNode = toSurfaceNode(gate.surface, summary);

  return (
    <div className="heap-modal-backdrop" onClick={onClose}>
      <div className="heap-modal-content" onClick={(event) => event.stopPropagation()}>
        <div className="heap-modal__header">
          <div style={{ flex: 1 }}>
            <div style={{ display: "flex", alignItems: "center", gap: "0.5rem", marginBottom: "0.5rem" }}>
              <span className={`heap-badge heap-badge--outline heap-badge--${gate.outcome.shouldBlock ? "red" : "yellow"}`}>
                {gate.outcome.shouldBlock ? "blocking" : "action required"}
              </span>
            </div>
            <h2 className="heap-modal__title">Steward Gate · {artifactId}</h2>
            <p className="heap-modal__meta">{summary}</p>
          </div>
          <button className="heap-modal__close-btn" onClick={onClose}>✕</button>
        </div>

        <div className="heap-modal__body heap-scroll">
          {surfaceNode ? (
            <div className="heap-steward-gate-surface">
              <A2UIInterpreter node={surfaceNode} />
            </div>
          ) : null}

          {(gate.outcome.violations?.length ?? 0) > 0 && (
            <div style={{ marginTop: "1rem" }}>
              <h3 className="heap-modal__section-label">Violations</h3>
              <div className="heap-steward-violation-list">
                {gate.outcome.violations.map((violation, index) => (
                  <div className="heap-steward-violation" key={`${violation.rule_id}-${index}`}>
                    <span className="heap-steward-violation__rule">{violation.rule_id}</span>
                    <span>{violation.explanation}</span>
                  </div>
                ))}
              </div>
            </div>
          )}

          {(gate.outcome.suggestedEnrichments?.length ?? 0) > 0 && (
            <div style={{ marginTop: "1rem" }}>
              <h3 className="heap-modal__section-label">Suggested Enrichments</h3>
              <div className="heap-steward-enrichment-list">
                {gate.outcome.suggestedEnrichments.map((enrichment) => (
                  <div className="heap-steward-enrichment" key={enrichmentKey(enrichment)}>
                    <div>
                      <div className="heap-steward-enrichment__title">{enrichment.displayLabel}</div>
                      <div className="heap-steward-enrichment__meta">{enrichment.matchedText}</div>
                    </div>
                    <button
                      className="heap-modal__footer-btn heap-modal__footer-btn--accent"
                      disabled={applyingId === enrichment.enrichmentId}
                      onClick={() => {
                        void onApply(enrichment.enrichmentId);
                      }}
                    >
                      {applyingId === enrichment.enrichmentId ? "Applying..." : "Apply"}
                    </button>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>

        <div className="heap-modal__footer">
          <button className="heap-modal__footer-btn" onClick={() => { void onRevalidate(); }}>
            Revalidate
          </button>
          <button
            className="heap-modal__footer-btn heap-modal__footer-btn--accent"
            disabled={gate.outcome.shouldBlock || publishing}
            onClick={() => { void onPublish(); }}
          >
            {publishing ? "Publishing..." : "Publish"}
          </button>
        </div>
      </div>
    </div>
  );
}
