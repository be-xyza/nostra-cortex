import type { SpaceDesignProfilePreviewFixture } from "../../store/spaceDesignProfilePreviewContract.ts";
import {
  isSpaceDesignProfilePreviewMetadataOnly,
  spaceDesignProfilePreviewThemeBindingState,
} from "../../store/spaceDesignProfilePreviewContract.ts";

export type SpaceDesignProfilePreviewPanelModel = {
  visible: boolean;
  title: string;
  statusLabel: string;
  profileId: string;
  profileVersion: string;
  reviewStatus: string;
  surfaceScopeLabel: string;
  boundaryLabel: string;
  boundaryTone: "metadata_only" | "blocked";
  note: string;
  exposesDesignTokens: boolean;
};

export function buildSpaceDesignProfilePreviewPanelModel(
  preview: SpaceDesignProfilePreviewFixture | null,
): SpaceDesignProfilePreviewPanelModel {
  const profile = preview?.profiles[0] ?? null;
  if (!preview || !profile) {
    return {
      visible: false,
      title: "Space Design Profile",
      statusLabel: "Unavailable",
      profileId: "",
      profileVersion: "",
      reviewStatus: "",
      surfaceScopeLabel: "",
      boundaryLabel: "Metadata unavailable",
      boundaryTone: "blocked",
      note: "",
      exposesDesignTokens: false,
    };
  }

  const metadataOnly = isSpaceDesignProfilePreviewMetadataOnly(preview);
  const bindingState = spaceDesignProfilePreviewThemeBindingState(preview);

  return {
    visible: true,
    title: "Space Design Profile",
    statusLabel: profile.authority_mode === "recommendation_only" ? "Draft recommendation" : "Blocked",
    profileId: profile.profile_id,
    profileVersion: profile.profile_version,
    reviewStatus: profile.review_status,
    surfaceScopeLabel: profile.surface_scope.join(", "),
    boundaryLabel: metadataOnly
      ? "Metadata only. Tokens are not applied to Cortex Web."
      : "Blocked from runtime theme binding.",
    boundaryTone: bindingState,
    note: profile.preview_note,
    exposesDesignTokens: Object.prototype.hasOwnProperty.call(profile, "design_tokens"),
  };
}
