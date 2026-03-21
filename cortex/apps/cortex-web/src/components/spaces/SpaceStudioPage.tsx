import { useEffect, useMemo, useState } from "react";
import { Link, useNavigate, useSearchParams } from "react-router-dom";
import { Copy, FilePlus2, Layers, ArrowRight, Trash2 } from "lucide-react";

import { resolveWorkbenchSpaceId, workbenchApi } from "../../api";
import { useUiStore } from "../../store/uiStore";
import { useActiveSpaceContext } from "../../store/spacesRegistry";
import {
  getSpaceStudioTemplatesForArchetype,
  resolveSpaceArchetypeProfile,
  resolveSpaceStudioTemplate,
  SPACE_ARCHETYPE_PROFILES,
  SPACE_STUDIO_TEMPLATES,
} from "./spaceArchetypeProfiles";
import {
  buildDraftGovernanceRows,
  buildDraftSnapshotHeapBlock,
  buildPromotionHandoffHeapBlock,
  buildPromotionReceiptHeapBlock,
  buildPromotionValidationResult,
  canPromoteDraftToLive,
  parseDraftSnapshotHeapBlock,
  resolveDraftReviewLane,
  type DraftSpaceGovernanceScope,
  type DraftSpaceSource,
  type DraftSpaceV1,
  useSpaceStudioDrafts,
} from "./spaceStudioStore";
import { resolveSpaceStudioPrimaryActionState } from "./spaceStudioPresentation";

function SectionCard({
  title,
  children,
}: {
  title: string;
  children: React.ReactNode;
}) {
  return (
    <section className="rounded-2xl border border-white/8 bg-white/[0.03] p-5">
      <h2 className="text-sm font-semibold text-white/90">{title}</h2>
      <div className="mt-4">{children}</div>
    </section>
  );
}

function Field({
  label,
  children,
}: {
  label: string;
  children: React.ReactNode;
}) {
  return (
    <label className="flex flex-col gap-2">
      <span className="text-xs font-medium text-white/55">{label}</span>
      {children}
    </label>
  );
}

function textInputClassName() {
  return "w-full rounded-xl border border-white/8 bg-[#11151f] px-3 py-2.5 text-sm text-white/90 outline-none transition focus:border-white/20";
}

function statusChipClassName(status: DraftSpaceV1["status"]): string {
  switch (status) {
    case "submitted":
      return "border-amber-400/25 bg-amber-400/10 text-amber-200";
    case "promoted":
      return "border-emerald-400/25 bg-emerald-400/10 text-emerald-200";
    default:
      return "border-white/10 bg-white/[0.05] text-white/55";
  }
}

function buildDraftSeedFromSession(
  sessionUser: { actorId: string; role: string } | null,
): Partial<DraftSpaceV1> {
  return {
    owner: sessionUser?.actorId || "",
    requestedByActorId: sessionUser?.actorId || undefined,
    requestedByRole: sessionUser?.role || undefined,
    governanceScope: "personal",
    archetype: resolveSpaceArchetypeProfile().label,
  };
}

function describeGovernanceScope(
  scope: DraftSpaceGovernanceScope,
): { title: string; summary: string } {
  if (scope === "personal") {
    return {
      title: "Personal",
      summary: "For your own work. It stays owner-scoped unless you later widen access.",
    };
  }
  if (scope === "public") {
    return {
      title: "Public",
      summary: "For broader shared access. It should go through steward review before it becomes live.",
    };
  }
  return {
    title: "Private",
    summary: "For invited people only. It stays shared, but not broadly visible.",
  };
}

export function SpaceStudioPage() {
  const navigate = useNavigate();
  const [searchParams, setSearchParams] = useSearchParams();
  const view = searchParams.get("view") === "templates" ? "templates" : "draft";
  const selectedDraftId = searchParams.get("draft");
  const sessionUser = useUiStore((state) => state.sessionUser);
  const setActiveSpaceIds = useUiStore((state) => state.setActiveSpaceIds);
  const activeSpaceId = useActiveSpaceContext();
  const {
    drafts,
    createDraft,
    createTemplateDraft,
    updateDraft,
    markDraftSaved,
    markDraftSubmitted,
    markDraftPromoted,
    mergeDrafts,
    removeDraft,
  } = useSpaceStudioDrafts();
  const [copyState, setCopyState] = useState<"idle" | "copied" | "failed">("idle");
  const [confirmLiveCreate, setConfirmLiveCreate] = useState(false);
  const [snapshotState, setSnapshotState] = useState<{
    status: "idle" | "saving" | "success" | "error";
    message?: string;
  }>({ status: "idle" });
  const [submitState, setSubmitState] = useState<{
    status: "idle" | "submitting" | "success" | "error";
    message?: string;
  }>({ status: "idle" });

  const selectedDraft = useMemo(() => {
    if (selectedDraftId) {
      return drafts.find((draft) => draft.draftId === selectedDraftId) ?? drafts[0] ?? null;
    }
    return drafts[0] ?? null;
  }, [drafts, selectedDraftId]);

  const promotionValidation = useMemo(
    () =>
      selectedDraft
        ? buildPromotionValidationResult(selectedDraft)
        : { request: null, issues: [], ready: false },
    [selectedDraft],
  );
  const promotionRequest = promotionValidation.request;
  const canSubmitLiveCreate = canPromoteDraftToLive(
    sessionUser?.role,
    selectedDraft,
    sessionUser?.actorId,
  );
  const handoffSpaceId = useMemo(() => {
    if (activeSpaceId === "meta") {
      return resolveWorkbenchSpaceId(undefined);
    }
    return resolveWorkbenchSpaceId(activeSpaceId);
  }, [activeSpaceId]);
  const draftTemplates = useMemo(
    () => getSpaceStudioTemplatesForArchetype(selectedDraft?.archetype),
    [selectedDraft?.archetype],
  );

  useEffect(() => {
    if (!selectedDraft && drafts.length === 0) {
      return;
    }
    if (!selectedDraftId && drafts[0]) {
      setSearchParams((current) => {
        const next = new URLSearchParams(current);
        next.set("draft", drafts[0]!.draftId);
        return next;
      });
    }
  }, [drafts, selectedDraft, selectedDraftId, setSearchParams]);

  useEffect(() => {
    setConfirmLiveCreate(false);
    setSubmitState({ status: "idle" });
    setSnapshotState({ status: "idle" });
    setCopyState("idle");
  }, [selectedDraftId]);

  useEffect(() => {
    if (!sessionUser?.actorId) {
      return;
    }

    let cancelled = false;
    workbenchApi
      .getHeapBlocks({
        spaceId: handoffSpaceId,
        blockType: "space_draft_snapshot",
        limit: 50,
      })
      .then((response) => {
        if (cancelled) {
          return;
        }
        const snapshots = response.items
          .map((item) => parseDraftSnapshotHeapBlock(item))
          .filter((draft): draft is DraftSpaceV1 => Boolean(draft))
          .filter((draft) => draft.requestedByActorId === sessionUser.actorId);
        mergeDrafts(snapshots);
      })
      .catch(() => {
        // Draft resume should fail quietly and leave local drafts untouched.
      });

    return () => {
      cancelled = true;
    };
  }, [handoffSpaceId, mergeDrafts, sessionUser?.actorId]);

  const setView = (nextView: "draft" | "templates") => {
    setSearchParams((current) => {
      const next = new URLSearchParams(current);
      if (nextView === "templates") {
        next.set("view", "templates");
      } else {
        next.delete("view");
      }
      return next;
    });
  };

  const selectDraft = (draftId: string) => {
    setSearchParams((current) => {
      const next = new URLSearchParams(current);
      next.set("draft", draftId);
      next.delete("view");
      return next;
    });
  };

  const createNewDraft = () => {
    const draft = createDraft(buildDraftSeedFromSession(sessionUser));
    selectDraft(draft.draftId);
  };

  const startFromTemplate = (templateId: string) => {
    const template = resolveSpaceStudioTemplate(templateId);
    const draft = createTemplateDraft(templateId, {
      ...buildDraftSeedFromSession(sessionUser),
      archetype: template
        ? resolveSpaceArchetypeProfile(template.archetypeId).label
        : resolveSpaceArchetypeProfile().label,
    });
    selectDraft(draft.draftId);
  };

  const updateSelectedDraft = (
    patch: Partial<DraftSpaceV1> & { sourceMode?: DraftSpaceSource },
  ) => {
    if (!selectedDraft) return;
    updateDraft(selectedDraft.draftId, patch);
  };

  const copyPromotionRequest = async () => {
    if (!promotionRequest || !promotionValidation.ready) return;
    try {
      await navigator.clipboard.writeText(JSON.stringify(promotionRequest, null, 2));
      setCopyState("copied");
    } catch {
      setCopyState("failed");
    }
    window.setTimeout(() => setCopyState("idle"), 1500);
  };

  const submitForStewardReview = async () => {
    if (!selectedDraft || !promotionValidation.ready || canSubmitLiveCreate) {
      return;
    }
    if (selectedDraft.status === "submitted" || selectedDraft.status === "promoted") {
      return;
    }

    setSubmitState({ status: "submitting" });
    try {
      const actorId = sessionUser?.actorId || "cortex-web";
      const actorRole = sessionUser?.role || "operator";
      const reviewLane = resolveDraftReviewLane(selectedDraft.governanceScope);
      const payload = buildPromotionHandoffHeapBlock(
        selectedDraft,
        handoffSpaceId,
        actorId,
        actorRole,
      );
      const response = await workbenchApi.emitHeapBlock(payload, actorRole, actorId);
      markDraftSubmitted(
        selectedDraft.draftId,
        response.artifactId,
        actorId,
        actorRole,
        reviewLane,
      );
      const submittedSnapshot: DraftSpaceV1 = {
        ...selectedDraft,
        status: "submitted",
        submittedAt: new Date().toISOString(),
        submittedArtifactId: response.artifactId,
        reviewLane,
        requestedByActorId: actorId,
        requestedByRole: actorRole,
      };
      const snapshotPayload = buildDraftSnapshotHeapBlock(
        submittedSnapshot,
        handoffSpaceId,
        actorId,
        actorRole,
      );
      const snapshotResponse = await workbenchApi.emitHeapBlock(
        snapshotPayload,
        actorRole,
        actorId,
      );
      markDraftSaved(selectedDraft.draftId, snapshotResponse.artifactId);
      setSubmitState({
        status: "success",
        message: `Submitted to the ${reviewLane === "public_review" ? "public" : "private"} steward queue in ${handoffSpaceId}.`,
      });
    } catch (error) {
      setSubmitState({
        status: "error",
        message: error instanceof Error ? error.message : String(error),
      });
    }
  };

  const submitLiveCreate = async () => {
    if (
      !selectedDraft ||
      !promotionRequest ||
      !promotionValidation.ready ||
      !canSubmitLiveCreate ||
      !confirmLiveCreate ||
      selectedDraft.status === "promoted"
    ) {
      return;
    }

    setSubmitState({ status: "submitting" });
    const actorRole = sessionUser?.role || "steward";
    const actorId = sessionUser?.actorId || "cortex-web";

    try {
      const response = await workbenchApi.createSpace(
        promotionRequest,
        actorRole,
        actorId,
      );

      let receiptArtifactId: string | undefined;
      let nextStateMessage =
        response.message || `Live space ${response.space_id} created successfully.`;

      try {
        const receiptPayload = buildPromotionReceiptHeapBlock(
          selectedDraft,
          response,
          actorId,
        );
        const receiptResponse = await workbenchApi.emitHeapBlock(
          receiptPayload,
          actorRole,
          actorId,
        );
        receiptArtifactId = receiptResponse.artifactId;
      } catch (receiptError) {
        nextStateMessage = `Live space ${response.space_id} created, but the receipt artifact could not be emitted: ${
          receiptError instanceof Error ? receiptError.message : String(receiptError)
        }`;
      }

      markDraftPromoted(selectedDraft.draftId, response.space_id, receiptArtifactId);
      const promotedSnapshot: DraftSpaceV1 = {
        ...selectedDraft,
        status: "promoted",
        promotedAt: new Date().toISOString(),
        promotedSpaceId: response.space_id,
        promotionReceiptArtifactId: receiptArtifactId,
        requestedByActorId: actorId,
        requestedByRole: actorRole,
      };
      try {
        const snapshotPayload = buildDraftSnapshotHeapBlock(
          promotedSnapshot,
          handoffSpaceId,
          actorId,
          actorRole,
        );
        const snapshotResponse = await workbenchApi.emitHeapBlock(
          snapshotPayload,
          actorRole,
          actorId,
        );
        markDraftSaved(selectedDraft.draftId, snapshotResponse.artifactId);
      } catch {
        // Promotion already succeeded; keeping the draft snapshot in sync is best-effort.
      }
      setConfirmLiveCreate(false);
      setSubmitState({ status: "success", message: nextStateMessage });
      navigate(`/spaces/${response.space_id}`);
    } catch (error) {
      setSubmitState({
        status: "error",
        message: error instanceof Error ? error.message : String(error),
      });
    }
  };

  const saveDraftForLater = async () => {
    if (!selectedDraft || draftLocked) {
      return;
    }

    setSnapshotState({ status: "saving" });
    const actorRole = sessionUser?.role || "operator";
    const actorId = sessionUser?.actorId || "cortex-web";

    try {
      const savedDraft: DraftSpaceV1 = {
        ...selectedDraft,
        requestedByActorId: actorId,
        requestedByRole: actorRole,
        updatedAt: new Date().toISOString(),
      };
      const payload = buildDraftSnapshotHeapBlock(
        savedDraft,
        handoffSpaceId,
        actorId,
        actorRole,
      );
      const response = await workbenchApi.emitHeapBlock(payload, actorRole, actorId);
      markDraftSaved(selectedDraft.draftId, response.artifactId);
      setSnapshotState({
        status: "success",
        message: `Saved for later in ${handoffSpaceId}.`,
      });
    } catch (error) {
      setSnapshotState({
        status: "error",
        message: error instanceof Error ? error.message : String(error),
      });
    }
  };

  const openGovernedHistoryItem = (href?: string) => {
    if (!href) {
      return;
    }
    if (href.startsWith("/explore")) {
      setActiveSpaceIds([handoffSpaceId]);
    }
    navigate(href);
  };

  const primaryAction = resolveSpaceStudioPrimaryActionState({
    canSubmitLiveCreate,
    governanceScope: selectedDraft?.governanceScope,
    draftStatus: selectedDraft?.status,
    validationReady: promotionValidation.ready,
    confirmLiveCreate,
    submitStatus: submitState.status,
  });
  const draftLocked = selectedDraft?.status === "promoted";
  const governedHistory = useMemo(
    () => (selectedDraft ? buildDraftGovernanceRows(selectedDraft) : []),
    [selectedDraft],
  );
  const governanceScopeCopy = describeGovernanceScope(
    selectedDraft?.governanceScope || "personal",
  );

  return (
    <div className="mx-auto flex w-full max-w-6xl flex-col gap-6 px-6 py-8">
      <div className="flex flex-wrap items-start justify-between gap-4">
        <div>
          <div className="text-[11px] font-semibold uppercase tracking-[0.24em] text-white/35">
            Labs
          </div>
          <h1 className="mt-2 text-3xl font-semibold tracking-tight text-white/95">
            Space Studio
          </h1>
          <p className="mt-2 max-w-2xl text-sm text-white/55">
            Draft a space here first. When it is ready, submit a steward handoff
            or use a steward-approved live creation step.
          </p>
        </div>
        <div className="flex items-center gap-3">
          <Link
            to="/spaces"
            className="rounded-xl border border-white/8 px-4 py-2 text-sm text-white/70 transition hover:border-white/15 hover:text-white"
          >
            Back to Spaces
          </Link>
          <button
            onClick={createNewDraft}
            className="rounded-xl bg-white px-4 py-2 text-sm font-medium text-[#0c1118] transition hover:bg-white/90"
          >
            New draft
          </button>
        </div>
      </div>

      <div className="flex gap-2">
        <button
          onClick={() => setView("draft")}
          className={`rounded-full px-4 py-2 text-sm transition ${
            view === "draft"
              ? "bg-white text-[#0c1118]"
              : "border border-white/8 text-white/65 hover:border-white/15 hover:text-white"
          }`}
        >
          Drafts
        </button>
        <button
          onClick={() => setView("templates")}
          className={`rounded-full px-4 py-2 text-sm transition ${
            view === "templates"
              ? "bg-white text-[#0c1118]"
              : "border border-white/8 text-white/65 hover:border-white/15 hover:text-white"
          }`}
        >
          Templates
        </button>
      </div>

      {view === "templates" ? (
        <div className="grid gap-4 md:grid-cols-2">
          {SPACE_STUDIO_TEMPLATES.map((template) => {
            const profile = resolveSpaceArchetypeProfile(template.archetypeId);
            return (
              <SectionCard key={template.id} title={template.name}>
                <div className="inline-flex rounded-full border border-white/10 bg-white/[0.04] px-2.5 py-1 text-[11px] uppercase tracking-[0.18em] text-white/45">
                  {profile.label}
                </div>
                <p className="mt-4 text-sm text-white/60">{template.summary}</p>
                <div className="mt-4 text-xs text-white/45">
                  <div>Purpose: {template.purpose}</div>
                  <div className="mt-1">Access: {template.accessSummary}</div>
                </div>
                <button
                  onClick={() => startFromTemplate(template.id)}
                  className="mt-5 inline-flex items-center gap-2 rounded-xl border border-white/10 px-4 py-2 text-sm text-white/80 transition hover:border-white/20 hover:text-white"
                >
                  Use template
                  <ArrowRight className="h-4 w-4" />
                </button>
              </SectionCard>
            );
          })}
        </div>
      ) : (
        <div className="grid gap-6 lg:grid-cols-[300px_minmax(0,1fr)]">
          <SectionCard title="Drafts">
            <div className="space-y-3">
              {drafts.length === 0 ? (
                <div className="rounded-xl border border-dashed border-white/10 px-4 py-5 text-sm text-white/45">
                  No drafts yet. Start one here or open a template first.
                </div>
              ) : (
                drafts.map((draft) => {
                  const selected = selectedDraft?.draftId === draft.draftId;
                  return (
                    <button
                      key={draft.draftId}
                      onClick={() => selectDraft(draft.draftId)}
                      className={`w-full rounded-xl border px-4 py-3 text-left transition ${
                        selected
                          ? "border-white/25 bg-white/8"
                          : "border-white/8 bg-white/[0.02] hover:border-white/15 hover:bg-white/[0.04]"
                      }`}
                    >
                      <div className="flex items-center justify-between gap-3">
                        <div className="text-sm font-medium text-white/90">{draft.title}</div>
                        <span
                          className={`rounded-full border px-2 py-0.5 text-[10px] uppercase tracking-[0.16em] ${statusChipClassName(
                            draft.status,
                          )}`}
                        >
                          {draft.status}
                        </span>
                      </div>
                      <div className="mt-1 text-xs text-white/45">
                        {draft.proposedSpaceId || "Choose a live space id later"}
                      </div>
                    </button>
                  );
                })
              )}
            </div>
          </SectionCard>

          {selectedDraft ? (
            <div className="space-y-6">
              <SectionCard title="Draft details">
                <div className="grid gap-4 md:grid-cols-2">
                  <Field label="Working title">
                    <input
                      className={textInputClassName()}
                      value={selectedDraft.title}
                      disabled={draftLocked}
                      onChange={(event) =>
                        updateSelectedDraft({ title: event.target.value })
                      }
                    />
                  </Field>
                  <Field label="Proposed live space id">
                    <input
                      className={textInputClassName()}
                      value={selectedDraft.proposedSpaceId}
                      disabled={draftLocked}
                      onChange={(event) =>
                        updateSelectedDraft({ proposedSpaceId: event.target.value })
                      }
                    />
                  </Field>
                  <Field label="Owner">
                    <input
                      className={textInputClassName()}
                      value={selectedDraft.owner}
                      placeholder="Optional owner id"
                      disabled={draftLocked}
                      onChange={(event) =>
                        updateSelectedDraft({ owner: event.target.value })
                      }
                    />
                  </Field>
                  <Field label="Archetype">
                    <select
                      className={textInputClassName()}
                      value={selectedDraft.archetype || resolveSpaceArchetypeProfile().label}
                      disabled={draftLocked}
                      onChange={(event) =>
                        updateSelectedDraft({ archetype: event.target.value })
                      }
                    >
                      {SPACE_ARCHETYPE_PROFILES.filter((profile) => profile.id !== "meta").map(
                        (profile) => (
                          <option key={profile.id} value={profile.label}>
                            {profile.label}
                          </option>
                        ),
                      )}
                    </select>
                  </Field>
                  <Field label="Start from">
                    <select
                      className={textInputClassName()}
                      value={selectedDraft.sourceMode}
                      disabled={draftLocked}
                      onChange={(event) =>
                        updateSelectedDraft({
                          sourceMode: event.target.value as DraftSpaceSource,
                        })
                      }
                    >
                      <option value="blank">Blank</option>
                      <option value="template">Template</option>
                      <option value="reference">Reference</option>
                      </select>
                  </Field>
                  <Field label="Space type">
                    <select
                      className={textInputClassName()}
                      value={selectedDraft.governanceScope}
                      disabled={draftLocked}
                      onChange={(event) =>
                        updateSelectedDraft({
                          governanceScope: event.target.value as DraftSpaceGovernanceScope,
                        })
                      }
                    >
                      <option value="personal">Personal</option>
                      <option value="private">Private</option>
                      <option value="public">Public</option>
                    </select>
                  </Field>
                </div>

                <div className="mt-4 grid gap-4">
                  <div className="rounded-xl border border-white/8 bg-white/[0.03] px-4 py-3 text-sm text-white/65">
                    <div className="font-medium text-white/90">{governanceScopeCopy.title}</div>
                    <div className="mt-1">{governanceScopeCopy.summary}</div>
                  </div>
                  <Field label="Purpose">
                    <textarea
                      className={`${textInputClassName()} min-h-[96px]`}
                      value={selectedDraft.purpose}
                      disabled={draftLocked}
                      onChange={(event) =>
                        updateSelectedDraft({ purpose: event.target.value })
                      }
                    />
                  </Field>

                  <Field label="Who can work here">
                    <textarea
                      className={`${textInputClassName()} min-h-[80px]`}
                      value={selectedDraft.accessSummary}
                      disabled={draftLocked}
                      onChange={(event) =>
                        updateSelectedDraft({ accessSummary: event.target.value })
                      }
                    />
                  </Field>

                  {selectedDraft.sourceMode === "template" ? (
                    <Field label="Template id">
                      <select
                        className={textInputClassName()}
                        value={selectedDraft.templateId || ""}
                        disabled={draftLocked}
                        onChange={(event) => {
                          const template = resolveSpaceStudioTemplate(event.target.value);
                          updateSelectedDraft({
                            templateId: event.target.value,
                            archetype: template
                              ? resolveSpaceArchetypeProfile(template.archetypeId).label
                              : selectedDraft.archetype,
                          });
                        }}
                      >
                        <option value="">Choose a template</option>
                        {draftTemplates.map((template) => (
                          <option key={template.id} value={template.templateId}>
                            {template.name}
                          </option>
                        ))}
                      </select>
                    </Field>
                  ) : null}

                  {selectedDraft.sourceMode === "reference" ? (
                    <Field label="Reference source">
                      <input
                        className={textInputClassName()}
                        value={selectedDraft.referenceUri || ""}
                        placeholder="nostra://reference/..."
                        disabled={draftLocked}
                        onChange={(event) =>
                          updateSelectedDraft({ referenceUri: event.target.value })
                        }
                      />
                    </Field>
                  ) : null}

                  <Field label="Lineage note">
                    <input
                      className={textInputClassName()}
                      value={selectedDraft.lineageNote || ""}
                      disabled={draftLocked}
                      onChange={(event) =>
                        updateSelectedDraft({ lineageNote: event.target.value })
                      }
                    />
                  </Field>
                </div>
              </SectionCard>

              <SectionCard title="Promotion handoff">
                <p className="text-sm text-white/60">
                  {canSubmitLiveCreate
                    ? selectedDraft.governanceScope === "personal"
                      ? "Your current role can create this personal space directly."
                      : "Your current role can promote a valid draft directly into a live space."
                    : `This ${selectedDraft.governanceScope} space will be submitted to ${handoffSpaceId} for steward review before it becomes live.`}
                </p>

                {promotionValidation.issues.length > 0 ? (
                  <div className="mt-4 rounded-xl border border-amber-400/20 bg-amber-400/5 p-4">
                    <div className="text-sm font-medium text-amber-100">
                      Promotion needs a little more information
                    </div>
                    <ul className="mt-2 space-y-1 text-sm text-amber-200/80">
                      {promotionValidation.issues.map((issue) => (
                        <li key={`${issue.field}:${issue.code}`}>{issue.message}</li>
                      ))}
                    </ul>
                  </div>
                ) : null}

                <div className="mt-4 rounded-xl border border-white/8 bg-[#10141d] p-4">
                  <div className="flex flex-wrap items-center gap-2 text-xs text-white/45">
                    <FilePlus2 className="h-4 w-4" />
                    Promotion request preview
                  </div>
                  <pre className="mt-3 overflow-x-auto whitespace-pre-wrap text-sm text-white/80">
                    {promotionRequest
                      ? JSON.stringify(promotionRequest, null, 2)
                      : "Complete the draft details above to prepare a live creation request."}
                  </pre>
                </div>

                <div className="mt-4 rounded-xl border border-white/8 bg-white/[0.02] p-4">
                  <div className="flex items-center justify-between gap-3">
                    <div>
                      <div className="text-sm font-medium text-white/90">Draft status</div>
                      <p className="mt-1 text-sm text-white/55">
                        {selectedDraft.status === "promoted"
                          ? `This draft already produced live space ${selectedDraft.promotedSpaceId || "unknown"}.`
                          : selectedDraft.status === "submitted"
                            ? `This draft has already been submitted for steward review${selectedDraft.submittedArtifactId ? ` as ${selectedDraft.submittedArtifactId}` : ""}.`
                            : selectedDraft.governanceScope === "personal"
                              ? "This draft is still local to Labs and can become a personal space when you are ready."
                              : "This draft is still local to Labs."}
                      </p>
                    </div>
                    <span
                      className={`rounded-full border px-2.5 py-1 text-[10px] uppercase tracking-[0.18em] ${statusChipClassName(
                        selectedDraft.status,
                      )}`}
                    >
                      {selectedDraft.status}
                    </span>
                  </div>

                  {canSubmitLiveCreate ? (
                    <label className="mt-4 flex items-start gap-3 text-sm text-white/70">
                      <input
                        type="checkbox"
                        className="mt-1"
                        checked={confirmLiveCreate}
                        onChange={(event) => setConfirmLiveCreate(event.target.checked)}
                        disabled={!promotionValidation.ready || selectedDraft.status === "promoted"}
                      />
                      <span>I understand this creates a live space, not just another draft.</span>
                    </label>
                  ) : null}

                  {submitState.message ? (
                    <div
                      className={`mt-3 text-sm ${
                        submitState.status === "error"
                          ? "text-red-300"
                          : submitState.status === "success"
                            ? "text-emerald-300"
                            : "text-white/55"
                      }`}
                    >
                      {submitState.message}
                    </div>
                  ) : null}
                  {snapshotState.message ? (
                    <div
                      className={`mt-3 text-sm ${
                        snapshotState.status === "error"
                          ? "text-red-300"
                          : snapshotState.status === "success"
                            ? "text-emerald-300"
                            : "text-white/55"
                      }`}
                    >
                      {snapshotState.message}
                    </div>
                  ) : null}
                </div>

                <div className="mt-4 flex flex-wrap gap-3">
                  <button
                    onClick={saveDraftForLater}
                    disabled={snapshotState.status === "saving" || draftLocked}
                    className="inline-flex items-center gap-2 rounded-xl border border-white/10 px-4 py-2 text-sm text-white/80 transition hover:border-white/20 hover:text-white disabled:cursor-not-allowed disabled:opacity-50"
                  >
                    <Layers className="h-4 w-4" />
                    {snapshotState.status === "saving" ? "Saving" : "Save for later"}
                  </button>
                  <button
                    onClick={copyPromotionRequest}
                    disabled={!promotionValidation.ready}
                    className="inline-flex items-center gap-2 rounded-xl border border-white/10 px-4 py-2 text-sm text-white/80 transition hover:border-white/20 hover:text-white disabled:cursor-not-allowed disabled:opacity-50"
                  >
                    <Copy className="h-4 w-4" />
                    {copyState === "copied"
                      ? "Copied"
                      : copyState === "failed"
                        ? "Copy failed"
                        : "Copy request JSON"}
                  </button>
                  <button
                    onClick={canSubmitLiveCreate ? submitLiveCreate : submitForStewardReview}
                    disabled={primaryAction.disabled}
                    className="inline-flex items-center gap-2 rounded-xl bg-white px-4 py-2 text-sm font-medium text-[#0c1118] transition hover:bg-white/90 disabled:cursor-not-allowed disabled:opacity-50"
                  >
                    <FilePlus2 className="h-4 w-4" />
                    {primaryAction.label}
                  </button>
                  <button
                    onClick={() => removeDraft(selectedDraft.draftId)}
                    className="inline-flex items-center gap-2 rounded-xl border border-red-400/15 px-4 py-2 text-sm text-red-300/80 transition hover:border-red-400/25 hover:text-red-200"
                  >
                    <Trash2 className="h-4 w-4" />
                    Remove draft
                  </button>
                  <button
                    onClick={() => navigate("/spaces")}
                    className="inline-flex items-center gap-2 rounded-xl bg-white px-4 py-2 text-sm font-medium text-[#0c1118] transition hover:bg-white/90"
                  >
                    Open spaces
                    <ArrowRight className="h-4 w-4" />
                  </button>
                </div>
              </SectionCard>

              <SectionCard title="Governed history">
                {governedHistory.length === 0 ? (
                  <p className="text-sm text-white/55">
                    This draft is still local to Labs. Save it, send it for review, or create the live space to start its governed history.
                  </p>
                ) : (
                  <div className="space-y-3">
                    {governedHistory.map((row) => (
                      <div
                        key={`${row.label}:${row.value}`}
                        className="rounded-xl border border-white/8 bg-[#10141d] px-4 py-4"
                      >
                        <div className="flex items-start justify-between gap-4">
                          <div className="min-w-0">
                            <div className="text-xs font-semibold uppercase tracking-[0.18em] text-white/45">
                              {row.label}
                            </div>
                            <p className="mt-2 text-sm leading-6 text-white/78">{row.value}</p>
                          </div>
                          {row.href ? (
                            <button
                              type="button"
                              onClick={() => openGovernedHistoryItem(row.href)}
                              className="shrink-0 rounded-full border border-white/10 bg-white/[0.04] px-3 py-1.5 text-sm font-medium text-white/78 transition hover:border-white/20 hover:bg-white/[0.08] hover:text-white"
                            >
                              Open
                            </button>
                          ) : null}
                        </div>
                      </div>
                    ))}
                  </div>
                )}
              </SectionCard>
            </div>
          ) : (
            <SectionCard title="Space Studio">
              <div className="flex flex-col items-start gap-4 text-sm text-white/55">
                <p>Create a draft or start from a template to continue.</p>
                <div className="flex gap-3">
                  <button
                    onClick={createNewDraft}
                    className="rounded-xl bg-white px-4 py-2 font-medium text-[#0c1118] transition hover:bg-white/90"
                  >
                    New draft
                  </button>
                  <button
                    onClick={() => setView("templates")}
                    className="inline-flex items-center gap-2 rounded-xl border border-white/10 px-4 py-2 text-white/80 transition hover:border-white/20 hover:text-white"
                  >
                    <Layers className="h-4 w-4" />
                    Browse templates
                  </button>
                </div>
              </div>
            </SectionCard>
          )}
        </div>
      )}
    </div>
  );
}
