import React from "react";
import { ArrowUpRight, Orbit, X, FileJson, Wand2, Cpu, Database, Activity } from "lucide-react";

import { openGatewayApiArtifact } from "../../api";
import type {
  WorkflowCheckpointResponse,
  WorkflowDefinitionResponse,
  WorkflowDigestResponse,
  WorkflowProjectionResponse,
  WorkflowTraceResponse,
} from "../../contracts.ts";
import {
  parseWorkflowArtifactPath,
  type WorkflowArtifactPathDescriptor,
} from "./artifactRouting.ts";
import type { WorkflowArtifactInspectorState } from "./WorkflowArtifactInspectorContext.tsx";
import { WorkflowCheckpointPanel } from "./WorkflowCheckpointPanel.tsx";
import { WorkflowDefinitionProjectionTabs } from "./WorkflowDefinitionProjectionTabs.tsx";
import { WorkflowInstanceTracePanel } from "./WorkflowInstanceTracePanel.tsx";
import { WorkflowReplayDigestPanel } from "./WorkflowReplayDigestPanel.tsx";
import { WidgetRegistry } from "../a2ui/WidgetRegistry";
import { ArtifactAssetViewer, displayBlockType } from "../a2ui/ArtifactAssetViewer.tsx";

const formatJsonWithHighlighting = (json: any) => {
  const str = JSON.stringify(json, null, 2);
  if (!str) return null;

  return str.split("\n").map((line, i) => {
    const parts = line.split(/(".*?"|:|\d+)/);
    return (
      <div key={i} className="whitespace-pre">
        {parts.map((part, j) => {
          if (part.startsWith('"') && part.endsWith('"')) {
            const isKey = line.includes(`${part}:`);
            return <span key={j} className={isKey ? "text-purple-400 font-semibold" : "text-emerald-400"}>{part}</span>;
          }
          if (part === ":") return <span key={j} className="text-slate-500 mr-1">:</span>;
          if (/^\d+$/.test(part)) return <span key={j} className="text-orange-400">{part}</span>;
          return <span key={j} className="text-slate-400">{part}</span>;
        })}
      </div>
    );
  });
};

function prettyTitle(descriptor: WorkflowArtifactPathDescriptor | null): string {
  if (!descriptor) return "Workflow Artifact Inspector";
  switch (descriptor.kind) {
    case "proposal_replay":
      return `Proposal Replay · ${descriptor.proposalId}`;
    case "proposal_digest":
      return `Proposal Digest · ${descriptor.proposalId}`;
    case "definition":
      return `Workflow Definition · ${descriptor.definitionId}`;
    case "definition_projection":
      return `Definition Projection · ${descriptor.definitionId}`;
    case "active_definition":
      return `Active Scope · ${descriptor.scopeKey}`;
    case "instance":
      return `Workflow Instance · ${descriptor.instanceId}`;
    case "instance_trace":
      return `Instance Trace · ${descriptor.instanceId}`;
    case "instance_checkpoints":
      return `Instance Checkpoints · ${descriptor.instanceId}`;
    case "instance_outcome":
      return `Instance Outcome · ${descriptor.instanceId}`;
  }
}

function studioArtifactIdFromPath(path: string): string | null {
  const match = path.match(/^\/api\/cortex\/studio\/artifacts\/([^/?#]+)/);
  if (!match) return null;
  try {
    return decodeURIComponent(match[1] ?? "");
  } catch {
    return match[1] ?? null;
  }
}

function prettyInspectorTitle(path: string | null, descriptor: WorkflowArtifactPathDescriptor | null, payload?: unknown): string {
  if (descriptor) {
    return prettyTitle(descriptor);
  }
  if (!path) return "Workflow Artifact Inspector";
  const studioArtifactId = studioArtifactIdFromPath(path);
  if (studioArtifactId) {
    // Prefer readable artifact title when available
    const artifact = payload as Record<string, any> | null;
    const humanTitle = artifact?.title;
    if (typeof humanTitle === "string" && humanTitle.length > 0) {
      return humanTitle;
    }
    return `Studio Artifact · ${studioArtifactId}`;
  }
  return "Workflow Artifact Inspector";
}

function inspectorFailureHint(path: string | null, error: string): string {
  if (path?.startsWith("/api/cortex/studio/artifacts/")) {
    return `The artifact surface could not be loaded for ${path}. Check that the artifact exists and the gateway route is healthy. ${error}`;
  }
  return error;
}

function artifactSurfaceLabel(descriptor: WorkflowArtifactPathDescriptor | null, payload?: unknown): string {
  if (!descriptor) {
    // For studio artifacts, derive surface label from block type
    const artifact = payload as Record<string, any> | null;
    const blockType = artifact?.heapBlockType || artifact?.heap_block_type;
    if (typeof blockType === "string" && blockType.length > 0) {
      return `${displayBlockType(blockType)} artifact`;
    }
    return "Unresolved surface";
  }
  switch (descriptor.kind) {
    case "proposal_replay":
    case "proposal_digest":
      return "Proposal artifact";
    case "definition":
    case "definition_projection":
    case "active_definition":
      return "Definition surface";
    case "instance":
    case "instance_trace":
    case "instance_checkpoints":
    case "instance_outcome":
      return "Runtime surface";
    default:
      return "Artifact surface";
  }
}

function descriptorBadgeLabel(descriptor: WorkflowArtifactPathDescriptor | null, payload?: unknown): string {
  if (!descriptor) {
    // Use human-readable block type from registry for studio artifacts
    const artifact = payload as Record<string, any> | null;
    const blockType = artifact?.heapBlockType || artifact?.heap_block_type;
    if (typeof blockType === "string" && blockType.length > 0) {
      return displayBlockType(blockType);
    }
    return "Unknown";
  }
  switch (descriptor.kind) {
    case "proposal_replay":
      return "Replay";
    case "proposal_digest":
      return "Digest";
    case "definition":
      return "Definition";
    case "definition_projection":
      return "Projection";
    case "active_definition":
      return "Active scope";
    case "instance":
      return "Instance";
    case "instance_trace":
      return "Trace";
    case "instance_checkpoints":
      return "Checkpoints";
    case "instance_outcome":
      return "Outcome";
  }
}

function pathSummary(path: string | null): string {
  if (!path) return "No route selected";
  if (path.startsWith("/api/cortex/studio/artifacts/")) {
    const artifactId = studioArtifactIdFromPath(path);
    return artifactId ? `Studio artifact ${artifactId.slice(0, 8)}…` : "Studio artifact route";
  }
  return path;
}

function resolveInspectorState(state: WorkflowArtifactInspectorState): {
  label: string;
  tone: "cyan" | "violet" | "emerald" | "amber" | "rose";
} {
  if (state.error) {
    return { label: "Failed", tone: "rose" };
  }
  if (state.loading) {
    return { label: "Loading", tone: "amber" };
  }
  if (state.open && state.payload) {
    return { label: "Loaded", tone: "emerald" };
  }
  if (state.open) {
    return { label: "Open", tone: "violet" };
  }
  return { label: "Closed", tone: "cyan" };
}

function toneClasses(tone: "cyan" | "violet" | "emerald" | "amber" | "rose"): string {
  switch (tone) {
    case "cyan":
      return "border-cyan-300/20 bg-cyan-300/10 text-cyan-100";
    case "violet":
      return "border-violet-300/20 bg-violet-300/10 text-violet-100";
    case "emerald":
      return "border-emerald-300/20 bg-emerald-300/10 text-emerald-100";
    case "amber":
      return "border-amber-300/20 bg-amber-300/10 text-amber-100";
    case "rose":
      return "border-rose-300/20 bg-rose-300/10 text-rose-100";
  }
}

import { artifactOverlayClasses, artifactBackdropClasses } from "../artifacts/artifactsPresentation.ts";

function GenericArtifactPanel({ payload }: { payload: unknown }) {
  const artifact = payload as Record<string, any> | null;
  const blockType = artifact?.heapBlockType || artifact?.heap_block_type || "unknown";

  return (
    <div className="flex flex-col md:flex-row items-start md:items-center justify-between gap-4 p-6 rounded-3xl border border-white/10 bg-white/3 backdrop-blur-xl shadow-2xl">
      <div className="relative mb-6">
        <div className="absolute -inset-4 rounded-full bg-cyan-500/20 blur-2xl animate-pulse" />
        <div className="relative flex h-16 w-16 items-center justify-center rounded-2xl bg-linear-to-br from-slate-800 to-slate-900 border border-white/10 shadow-2xl">
           <Database className="h-8 w-8 text-cyan-400/80" />
        </div>
        <div className="absolute -bottom-1 -right-1 flex h-6 w-6 items-center justify-center rounded-lg bg-emerald-500 border-2 border-slate-900 shadow-lg">
          <Activity className="h-3.5 w-3.5 text-white" />
        </div>
      </div>
      
      <h3 className="text-lg font-bold text-white mb-3 tracking-tight">Unregistered Artifact</h3>
      
      <div className="flex items-center gap-2 px-3 py-1.5 rounded-full bg-white/5 border border-white/10 mb-6">
        <Cpu className="w-3.5 h-3.5 text-cyan-400" />
        <span className="text-[10px] font-mono font-bold text-cyan-200 uppercase tracking-widest">
          {displayBlockType(blockType)}
        </span>
      </div>

      <p className="text-sm text-slate-400 max-w-sm leading-relaxed mb-8">
        This artifact type has not been mapped to a dedicated A2UI renderer. 
        You can trigger a dynamic analysis to generate a custom view.
      </p>

      <button 
        onClick={() => console.log("Triggering dynamic artifact resolution workflow...")}
        className="group relative flex items-center gap-2 px-6 py-3 rounded-xl bg-cyan-600 hover:bg-cyan-500 text-white font-semibold text-sm transition-all shadow-lg hover:shadow-cyan-500/25 border border-cyan-400/50"
      >
        <Wand2 className="w-4 h-4 transition-transform group-hover:rotate-12" />
        Process Artifact via Workflow
      </button>
    </div>
  );
}

function ArtifactBody({
  descriptor,
  payload,
}: {
  descriptor: WorkflowArtifactPathDescriptor | null;
  payload: unknown;
}) {
  if (!descriptor) {
    const artifact = payload as Record<string, any> | null;
    const blockType = artifact?.heapBlockType || artifact?.heap_block_type;
    
    if (artifact && blockType && WidgetRegistry[blockType]) {
      const ComponentNode = WidgetRegistry[blockType];
      // Build componentProperties: start with parsed A2UI JSON (if any), then layer root artifact fields as payload
      let componentProps: Record<string, any> = {};
      try {
        const jsonStr = artifact.a2uiInitialUiJson || artifact.aguiInitialUiJson;
        if (jsonStr) {
          componentProps = JSON.parse(jsonStr);
        }
      } catch (e) {
        console.error("Failed to parse A2UI UI JSON state:", e);
      }
      // Always inject the root artifact record as `payload` so widgets can access title, markdownSource, etc.
      if (!componentProps.payload) {
        componentProps.payload = artifact;
      }
      return (
        <div className="flex flex-col gap-4">
          <ArtifactAssetViewer payload={artifact} />
          <div className="p-5 rounded-2xl border border-white/5 bg-white/2 flex flex-col gap-3">
            <ComponentNode id={artifact.artifactId || artifact.id || "inspector"} componentProperties={componentProps} />
          </div>
        </div>
      );
    }
    return (
      <div className="flex flex-col gap-4">
        {artifact && <ArtifactAssetViewer payload={artifact} />}
        <GenericArtifactPanel payload={payload} />
      </div>
    );
  }

  if (descriptor.kind === "definition_projection") {
    return (
      <WorkflowDefinitionProjectionTabs
        definitionId={descriptor.definitionId}
        initialProjectionKind={descriptor.projectionKind}
        initialProjection={payload as WorkflowProjectionResponse}
      />
    );
  }
  if (descriptor.kind === "instance_trace") {
    return <WorkflowInstanceTracePanel response={payload as WorkflowTraceResponse} />;
  }
  if (descriptor.kind === "instance_checkpoints") {
    return <WorkflowCheckpointPanel response={payload as WorkflowCheckpointResponse} />;
  }
  if (descriptor.kind === "proposal_replay") {
    return <WorkflowReplayDigestPanel title="Replay Artifact" payload={payload} />;
  }
  if (descriptor.kind === "proposal_digest") {
    return (
      <WorkflowReplayDigestPanel
        title="Digest Artifact"
        payload={(payload as WorkflowDigestResponse).digest ?? payload}
      />
    );
  }
  if (descriptor.kind === "definition") {
    const definitionResponse = payload as WorkflowDefinitionResponse;
    return <GenericArtifactPanel payload={definitionResponse.definition ?? payload} />;
  }
  return <GenericArtifactPanel payload={payload} />;
}

export function WorkflowArtifactInspector({
  state,
  onClose,
}: {
  state: WorkflowArtifactInspectorState;
  onClose: () => void;
}) {
  const descriptor = state.path ? parseWorkflowArtifactPath(state.path) : null;
  const title = prettyInspectorTitle(state.path, descriptor, state.payload);
  const inspectorState = resolveInspectorState(state);
  const isStudioArtifact = Boolean(state.path?.startsWith("/api/cortex/studio/artifacts/"));
  return (
    <>
      <button
        type="button"
        onClick={onClose}
        aria-label="Close artifact inspector"
        className={artifactBackdropClasses}
      />
      <aside className={artifactOverlayClasses}>
        <div className="shrink-0 border-b border-white/8 bg-[linear-gradient(180deg,rgba(16,24,39,0.96),rgba(9,18,33,0.9))] px-6 py-5">
          <div className="flex items-start justify-between gap-3">
            <div className="min-w-0">
              <div className="inline-flex items-center gap-2 rounded-full border border-cyan-300/20 bg-cyan-300/10 px-3 py-1 text-[10px] font-black uppercase tracking-[0.28em] text-cyan-100">
                <Orbit className="h-3.5 w-3.5" />
                Artifact detail
              </div>
              <h2 className="mt-3 text-[1.5rem] font-black tracking-tight text-white">{title}</h2>
              <div className="mt-4 flex flex-wrap gap-2">
                <span className="rounded-full border border-white/10 bg-white/4 px-3 py-1 text-[10px] font-mono text-slate-300">
                  {artifactSurfaceLabel(descriptor, state.payload)}
                </span>
                <span className="rounded-full border border-white/10 bg-white/4 px-3 py-1 text-[10px] font-mono text-slate-300">
                  {descriptorBadgeLabel(descriptor, state.payload)}
                </span>
                <span className={`rounded-full border px-3 py-1 text-[10px] font-mono ${toneClasses(inspectorState.tone)}`}>
                  {inspectorState.label}
                </span>
                <span className="rounded-full border border-white/10 bg-white/4 px-3 py-1 text-[10px] font-mono text-slate-400">
                  {pathSummary(state.path)}
                </span>
                {isStudioArtifact && (
                  <span className="rounded-full border border-violet-300/20 bg-violet-300/10 px-3 py-1 text-[10px] font-mono text-violet-100">
                    Studio route
                  </span>
                )}
              </div>
            </div>
            <div className="flex shrink-0 gap-2">
              <button
                className="inline-flex items-center gap-2 rounded-full border border-white/10 bg-white/4 px-4 py-2 text-[11px] font-black uppercase tracking-[0.18em] text-white/80 transition hover:border-white/20 hover:bg-white/8 hover:text-white"
                onClick={onClose}
              >
                <X className="h-3.5 w-3.5" />
                Close
              </button>
            </div>
          </div>
        </div>
        <div className="flex flex-1 flex-col gap-4 overflow-y-auto px-6 py-5">
          {!state.open || !state.path ? (
          <div className="rounded-[22px] border border-dashed border-white/10 bg-white/3 px-5 py-6 text-sm leading-6 text-slate-400">
            Select a workflow replay, digest, projection, trace, or checkpoint action from
            the `/workflows` surface to inspect it here.
          </div>
        ) : null}
        {state.loading ? (
          <div className="p-5 rounded-2xl border border-white/5 bg-white/2 flex flex-col justify-between overflow-hidden">
            Loading artifact from {state.path}...
          </div>
        ) : null}
        {state.error ? <div className="error-banner">{inspectorFailureHint(state.path, state.error)}</div> : null}
        {state.payload && !state.loading ? (
          <div className="space-y-4">
            <ArtifactBody descriptor={descriptor} payload={state.payload} />
            
            <details className="mt-6 group">
              <summary className="cursor-pointer list-none rounded-2xl border border-white/8 bg-white/3 p-4 transition-colors hover:bg-white/5">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-3">
                    <span className="flex h-6 w-6 items-center justify-center rounded-full bg-slate-800 text-xs text-slate-400 group-open:bg-cyan-500/20 group-open:text-cyan-400">
                      <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><polyline points="16 18 22 12 16 6"></polyline><polyline points="8 6 2 12 8 18"></polyline></svg>
                    </span>
                    <span className="text-[11px] font-black uppercase tracking-[0.2em] text-slate-300">Raw Artifact JSON</span>
                  </div>
                  <span className="text-xs text-slate-500 group-open:hidden">Show</span>
                  <span className="hidden text-xs text-cyan-400 group-open:block">Hide</span>
                </div>
              </summary>
              <div className="mt-3 overflow-x-auto rounded-[1.35rem] border border-white/8 bg-[linear-gradient(180deg,rgba(15,23,42,0.95),rgba(2,6,23,0.9))] p-5 shadow-inner text-[10px] leading-relaxed font-mono custom-scrollbar">
                {formatJsonWithHighlighting(state.payload)}
              </div>
            </details>
          </div>
        ) : null}
      </div>
      </aside>
    </>
  );
}
