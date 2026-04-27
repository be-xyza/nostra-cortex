export const SPACE_DESIGN_PROFILE_PREVIEW_SNAPSHOT_ID = "system:ux:space-design-profiles";

export type SpaceDesignProfilePreviewSummary = {
  profile_id: string;
  profile_version: string;
  space_id: string;
  authority_mode: "recommendation_only";
  review_status: string;
  approved_by_count: number;
  lineage_ref: string;
  surface_scope: string[];
  a2ui_theme_policy: {
    token_version: string;
    safe_mode: boolean;
    theme_allowlist_id: string;
    motion_policy: string;
    contrast_preference: string;
  };
  preview_status: "metadata_only";
  preview_note: string;
};

export type SpaceDesignProfilePreviewFixture = {
  schema_version: "CortexWebSpaceDesignProfilePreviewFixtureV1";
  snapshot_id: typeof SPACE_DESIGN_PROFILE_PREVIEW_SNAPSHOT_ID;
  source_mode: "fixture";
  generated_from_profile_ref: string;
  runtime_binding: "none";
  runtime_policy: {
    recommendation_only: true;
    applies_tokens_to_cortex_web: false;
    runtime_theme_selection: false;
    requires_verified_projection_for_governance: true;
  };
  prohibited_runtime_claims: string[];
  profiles: SpaceDesignProfilePreviewSummary[];
};

export function isSpaceDesignProfilePreviewMetadataOnly(
  preview: SpaceDesignProfilePreviewFixture,
): boolean {
  return (
    preview.runtime_binding === "none" &&
    preview.runtime_policy.recommendation_only === true &&
    preview.runtime_policy.applies_tokens_to_cortex_web === false &&
    preview.runtime_policy.runtime_theme_selection === false &&
    preview.profiles.every((profile) => profile.authority_mode === "recommendation_only" && profile.preview_status === "metadata_only")
  );
}

export function spaceDesignProfilePreviewThemeBindingState(
  preview: SpaceDesignProfilePreviewFixture,
): "metadata_only" | "blocked" {
  return isSpaceDesignProfilePreviewMetadataOnly(preview) ? "metadata_only" : "blocked";
}
