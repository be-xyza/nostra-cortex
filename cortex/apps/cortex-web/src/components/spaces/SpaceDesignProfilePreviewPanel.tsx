import React from "react";
import { Palette, ShieldCheck } from "lucide-react";

import type { SpaceDesignProfilePreviewFixture } from "../../store/spaceDesignProfilePreviewContract.ts";
import {
  buildSpaceDesignProfilePreviewPanelModel,
  type SpaceDesignProfilePreviewPanelModel,
} from "./spaceDesignProfilePreviewModel.ts";

export function SpaceDesignProfilePreviewPanel({
  preview,
  loading = false,
}: {
  preview: SpaceDesignProfilePreviewFixture | null;
  loading?: boolean;
}) {
  const model = buildSpaceDesignProfilePreviewPanelModel(preview);
  if (!model.visible && !loading) {
    return null;
  }

  return (
    <section
      className="mb-6 rounded-xl border border-white/8 bg-white/3 px-4 py-3"
      aria-label="Space design profile preview"
    >
      <div className="flex flex-col gap-3 md:flex-row md:items-start md:justify-between">
        <div className="flex min-w-0 items-start gap-3">
          <div className="mt-0.5 flex h-8 w-8 shrink-0 items-center justify-center rounded-lg border border-white/10 bg-white/5 text-white/45">
            <Palette className="h-4 w-4" />
          </div>
          <div className="min-w-0">
            <div className="flex flex-wrap items-center gap-2">
              <h2 className="text-xs font-semibold text-white/80">{model.title}</h2>
              <BoundaryPill model={model} loading={loading} />
            </div>
            <p className="mt-1 text-[11px] leading-5 text-white/45">
              {loading ? "Loading draft Space design metadata..." : model.note}
            </p>
          </div>
        </div>
        {model.visible && (
          <dl className="grid min-w-0 grid-cols-2 gap-x-4 gap-y-2 text-[10px] text-white/45 md:min-w-[360px]">
            <PreviewTerm label="Profile" value={model.profileId} />
            <PreviewTerm label="Version" value={model.profileVersion} />
            <PreviewTerm label="Review" value={model.reviewStatus} />
            <PreviewTerm label="Scope" value={model.surfaceScopeLabel} wide />
          </dl>
        )}
      </div>
    </section>
  );
}

function BoundaryPill({ model, loading }: { model: SpaceDesignProfilePreviewPanelModel; loading: boolean }) {
  const blocked = model.boundaryTone === "blocked" || model.exposesDesignTokens;
  return (
    <span
      className={`inline-flex items-center gap-1 rounded-md border px-2 py-1 text-[9px] font-bold uppercase tracking-[0.16em] ${
        blocked
          ? "border-amber-400/20 bg-amber-400/10 text-amber-100/70"
          : "border-emerald-400/15 bg-emerald-400/8 text-emerald-100/65"
      }`}
      title={loading ? "Loading Space design profile metadata" : model.boundaryLabel}
    >
      <ShieldCheck className="h-3 w-3" />
      {loading ? "Loading" : model.statusLabel}
    </span>
  );
}

function PreviewTerm({ label, value, wide = false }: { label: string; value: string; wide?: boolean }) {
  return (
    <div className={wide ? "col-span-2 min-w-0" : "min-w-0"}>
      <dt className="uppercase tracking-[0.18em] text-white/25">{label}</dt>
      <dd className="mt-0.5 truncate text-white/65" title={value}>
        {value}
      </dd>
    </div>
  );
}
