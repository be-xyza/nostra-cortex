import React, { useState, useEffect, useMemo } from "react";
import { FileDown, Layers, FileImage, ExternalLink } from "lucide-react";
import { gatewayBaseUrl } from "../../api";
import type { HeapBlockProjection } from "../../contracts";

/**
 * Mapping from raw internal `blockType` identifiers to clean, human-readable labels.
 * Exported so that HeapBlockCard and other table/grid surfaces can also
 * display the same friendly names for filtering and navigation.
 */
export const BLOCK_TYPE_DISPLAY_NAMES: Record<string, string> = {
  note: "Note",
  upload: "Upload",
  file: "File",
  document: "Document",
  image: "Image",
  video: "Video",
  dpub: "Publication",
  media: "Media",
  task: "Task",
  checklist: "Checklist",
  scorecard: "Scorecard",
  chart: "Chart",
  pointer: "Pointer",
  a2ui: "A2UI Surface",
  gate_summary: "Gate Summary",
  telemetry: "Telemetry",
  usage_report: "Activity Summary",
  self_optimization_proposal: "Suggested Improvement",
  agent_execution_record: "Agent Work Update",
  eudaemon_evidence_note: "Evidence Note",
  capability: "Capability",
  pattern: "Pattern",
  agent_solicitation: "Review Request",
  action_plan: "Action Plan",
  compiled_plan: "Compiled Plan",
  chat_thread: "Chat Thread",
  workspace_agent_trace: "Agent Trace",
  conversation_message: "Conversation",
  evaluation: "Evaluation",
  rule_set: "Rule Set",
  agent_trace: "Agent Trace",
};

/** Returns a human-readable label for the given blockType, falling back to title-casing the raw key. */
export function displayBlockType(blockType: string): string {
  return BLOCK_TYPE_DISPLAY_NAMES[blockType] ?? blockType
    .split("_")
    .map(w => w.charAt(0).toUpperCase() + w.slice(1))
    .join(" ");
}

/** Extract upload_id from a cortex://upload?id=XXXX resource_ref string */
function parseUploadIdFromRef(ref: string): string | null {
  if (ref.startsWith("cortex://upload")) {
    try {
      const url = new URL(ref);
      return url.searchParams.get("id");
    } catch {
      const m = ref.match(/[?&]id=([a-zA-Z0-9_-]+)/);
      return m ? m[1] : null;
    }
  }
  return null;
}

interface ArtifactAssetViewerProps {
  /** The full artifact payload record from the inspector, including projection and surfaceJson */
  payload: Record<string, any>;
}

/**
 * ArtifactAssetViewer renders inline file previews (PDF, images, dpub iframes)
 * for heap blocks that reference uploaded assets.
 *
 * Resolution strategy (ordered by priority):
 *  1. `projection.attributes.resource_ref` → parse cortex://upload?id=XXXX to get upload_id
 *  2. `projection.attributes.upload_id` → direct upload ID
 *  3. For flat ArtifactDocumentV2 payloads: async fetch heap block export to resolve upload_id
 *
 * Streams via: GET /api/cortex/studio/uploads/:upload_id/blob
 */
export const ArtifactAssetViewer: React.FC<ArtifactAssetViewerProps> = ({ payload }) => {
  const [resolvedUploadId, setResolvedUploadId] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const projection = payload.projection as HeapBlockProjection | undefined;
  const attributes = (projection?.attributes ?? payload.attributes ?? {}) as Record<string, string>;
  const surfaceJson = (payload.surfaceJson ?? payload) as Record<string, any>;
  const artifactId = projection?.artifactId ?? payload.artifactId ?? payload.artifact_id;
  const blockType = (projection?.blockType ?? payload.heapBlockType ?? payload.heap_block_type ?? "").toLowerCase();

  /**
   * Synchronous resolve: try to extract upload_id from projection attributes
   */
  const syncUploadId = useMemo((): string | null => {
    const refsToTry = [
      attributes.resource_ref,
      attributes.resourceRef,
      payload.resource_ref,
      payload.resourceRef,
      surfaceJson.resource_ref,
      surfaceJson.resourceRef
    ];

    for (const ref of refsToTry) {
      if (typeof ref === "string" && ref.length > 6) {
        const id = parseUploadIdFromRef(ref);
        if (id) return id;
        const pathMatch = ref.match(/\/uploads\/([a-zA-Z0-9_-]+)\//);
        if (pathMatch) return pathMatch[1];
      }
    }

    // 2. Direct upload_id from attributes
    const directId = attributes.upload_id ?? payload.upload_id ?? payload.file_id ?? payload.asset_id ?? surfaceJson.upload_id ?? surfaceJson.uploadId;
    if (typeof directId === "string" && directId.length > 6) return directId;

    return null;
  }, [attributes, payload, surfaceJson]);

  /**
   * Async fallback: fetch projection via heap block export endpoint
   * when we have artifactId + upload blockType but no upload_id yet
   */
  useEffect(() => {
    if (syncUploadId) {
      setResolvedUploadId(syncUploadId);
      return;
    }

    const isUploadLike = blockType === "upload" || blockType === "file" || blockType === "document";
    if (!isUploadLike || !artifactId) return;

    let cancelled = false;
    setLoading(true);

    (async () => {
      try {
        const exportPath = `/api/cortex/studio/heap/blocks/${artifactId}/export?format=json`;
        const response = await fetch(`${gatewayBaseUrl()}${exportPath}`, {
          credentials: "include",
          headers: { "Content-Type": "application/json" },
        });
        if (!response.ok) throw new Error(`Export fetch failed: ${response.status}`);
        const result = await response.json() as Record<string, any>;
        if (cancelled) return;

        // The export returns { surface_json, ... } — surface_json contains the original emit payload
        const surface = result?.surface_json ?? result;
        const content = surface?.content ?? {};
        const exportAttrs = surface?.attributes ?? {};

        // Try resource_ref from export
        if (typeof exportAttrs.resource_ref === "string") {
          const id = parseUploadIdFromRef(exportAttrs.resource_ref);
          if (id) { setResolvedUploadId(id); return; }
        }
        // Try upload_id from export attributes
        if (typeof exportAttrs.upload_id === "string") {
          setResolvedUploadId(exportAttrs.upload_id);
          return;
        }
        // Try content.pointer or surface-level pointer which has cortex://upload?id=XXX format
        const pointer = content.pointer ?? surface.pointer;
        if (typeof pointer === "string") {
          const id = parseUploadIdFromRef(pointer);
          if (id) { setResolvedUploadId(id); return; }
        }
      } catch (err) {
        console.warn("[ArtifactAssetViewer] Failed to resolve upload from export:", err);
      } finally {
        if (!cancelled) setLoading(false);
      }
    })();

    return () => { cancelled = true; };
  }, [syncUploadId, blockType, artifactId]);

  const uploadId = resolvedUploadId;

  // Determine MIME classification for render mode
  const mode = useMemo((): "pdf" | "image" | "dpub" | "unknown" => {
    const mime = (attributes.mime_type ?? surfaceJson.mime_type ?? surfaceJson.mimeType ?? payload.mime_type ?? payload.mimeType ?? "").toLowerCase();
    const filename = (attributes.file_name ?? surfaceJson.file_name ?? surfaceJson.fileName ?? payload.file_name ?? payload.fileName ?? payload.title ?? projection?.title ?? "").toLowerCase();

    if (mime === "application/pdf" || filename.endsWith(".pdf") || (blockType === "document" && filename.includes("pdf"))) return "pdf";
    if (mime.startsWith("image/") || /\.(png|jpe?g|gif|webp|svg)$/.test(filename) || blockType === "image") return "image";
    if (mime === "application/dpub" || filename.endsWith(".dpub") || blockType === "dpub") return "dpub";

    // Heuristic: check title or text for file extension hints
    const title = (projection?.title ?? payload.title ?? "").toLowerCase();
    if (title.endsWith(".pdf") || (blockType === "document" && title.includes("pdf"))) return "pdf";
    if (/\.(png|jpe?g|gif|webp|svg)$/.test(title)) return "image";
    if (title.endsWith(".dpub")) return "dpub";

    return "unknown";
  }, [attributes, surfaceJson, projection, payload, blockType]);

  // Still loading
  if (loading) {
    return (
      <div className="w-full mt-4 mb-2 flex items-center gap-3 px-4 py-3 rounded-xl border border-white/5 bg-slate-900/40 text-slate-500 text-xs">
        <div className="h-4 w-4 rounded-full border-2 border-slate-600 border-t-cyan-400 animate-spin" />
        Resolving asset reference…
      </div>
    );
  }

  if (!uploadId || mode === "unknown") {
    return null;
  }

  const streamPath = `/api/cortex/studio/uploads/${uploadId}/blob`;
  const fullUrl = `${gatewayBaseUrl()}${streamPath}`;

  return (
    <div className="w-full mt-4 mb-2 animate-in fade-in slide-in-from-bottom-3 duration-500">
      <div className="flex items-center justify-between mb-3">
        <div className="flex items-center gap-2 text-slate-400">
          {mode === "pdf" && <FileDown className="w-4 h-4 text-rose-400" />}
          {mode === "image" && <FileImage className="w-4 h-4 text-emerald-400" />}
          {mode === "dpub" && <Layers className="w-4 h-4 text-indigo-400" />}
          <span className="text-xs font-semibold tracking-wider uppercase">
            {mode === "pdf" ? "Document Preview" : mode === "image" ? "Attached Image" : "Publication Preview"}
          </span>
        </div>
        <a
          href={fullUrl}
          target="_blank"
          rel="noopener noreferrer"
          className="inline-flex items-center gap-1.5 text-[10px] font-semibold tracking-wide uppercase text-slate-500 hover:text-white transition-colors"
        >
          <ExternalLink className="w-3 h-3" />
          Open in new tab
        </a>
      </div>

      <div className="w-full overflow-hidden rounded-xl border border-white/10 bg-black/40 shadow-2xl backdrop-blur-md">
        {mode === "pdf" && (
          <iframe
            src={fullUrl}
            className="w-full h-[600px] rounded-b-xl"
            title="PDF Document Viewer"
          />
        )}

        {mode === "image" && (
          <div className="w-full bg-slate-900/50 flex justify-center p-4">
            <img
              src={fullUrl}
              alt="Artifact Attachment"
              className="max-h-[700px] w-auto object-contain rounded-lg shadow-xl"
              loading="lazy"
            />
          </div>
        )}

        {mode === "dpub" && (
          <iframe
            src={fullUrl}
            className="w-full h-[700px] bg-slate-50"
            sandbox="allow-scripts allow-same-origin"
            title="Decentralized Runtime Canvas"
          />
        )}
      </div>
    </div>
  );
};
