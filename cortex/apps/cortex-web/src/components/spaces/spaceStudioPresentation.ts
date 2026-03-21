import type {
  DraftSpaceGovernanceScope,
  DraftSpaceStatus,
} from "./spaceStudioStore.ts";

export interface SpaceStudioPrimaryActionState {
  mode: "live_create" | "steward_review";
  label: string;
  disabled: boolean;
}

export function resolveSpaceStudioPrimaryActionState(params: {
  canSubmitLiveCreate: boolean;
  governanceScope?: DraftSpaceGovernanceScope;
  draftStatus?: DraftSpaceStatus;
  validationReady: boolean;
  confirmLiveCreate: boolean;
  submitStatus: "idle" | "submitting" | "success" | "error";
}): SpaceStudioPrimaryActionState {
  if (params.canSubmitLiveCreate) {
    return {
      mode: "live_create",
      label:
        params.submitStatus === "submitting"
          ? params.governanceScope === "personal"
            ? "Creating personal space..."
            : "Creating live space..."
          : params.draftStatus === "promoted"
            ? "Live space created"
            : params.governanceScope === "personal"
              ? "Create personal space"
              : "Create live space",
      disabled:
        !params.validationReady ||
        !params.confirmLiveCreate ||
        params.submitStatus === "submitting" ||
        params.draftStatus === "promoted",
    };
  }

  return {
    mode: "steward_review",
    label:
      params.submitStatus === "submitting"
        ? "Submitting for steward review..."
        : params.draftStatus === "submitted"
          ? "Submitted for steward review"
          : params.draftStatus === "promoted"
            ? "Live space created"
            : "Submit for steward review",
    disabled:
      !params.validationReady ||
      params.submitStatus === "submitting" ||
      params.draftStatus === "submitted" ||
      params.draftStatus === "promoted",
  };
}
