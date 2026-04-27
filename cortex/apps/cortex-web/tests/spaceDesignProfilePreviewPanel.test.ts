import assert from "node:assert/strict";
import test from "node:test";

import { buildSpaceDesignProfilePreviewPanelModel } from "../src/components/spaces/spaceDesignProfilePreviewModel.ts";
import type { SpaceDesignProfilePreviewFixture } from "../src/store/spaceDesignProfilePreviewContract.ts";
import { SPACE_DESIGN_PROFILE_PREVIEW_SNAPSHOT_ID } from "../src/store/spaceDesignProfilePreviewContract.ts";

function previewFixture(): SpaceDesignProfilePreviewFixture {
  return {
    schema_version: "CortexWebSpaceDesignProfilePreviewFixtureV1",
    snapshot_id: SPACE_DESIGN_PROFILE_PREVIEW_SNAPSHOT_ID,
    source_mode: "fixture",
    generated_from_profile_ref: "research/120-nostra-design-language/prototypes/space-design/SPACE_DESIGN.space-profile.v1.json",
    runtime_binding: "none",
    runtime_policy: {
      recommendation_only: true,
      applies_tokens_to_cortex_web: false,
      runtime_theme_selection: false,
      requires_verified_projection_for_governance: true,
    },
    prohibited_runtime_claims: ["approved"],
    profiles: [
      {
        profile_id: "space-design-profile:space-research-observatory",
        profile_version: "v0.1.0",
        space_id: "space:research-observatory",
        authority_mode: "recommendation_only",
        review_status: "draft",
        approved_by_count: 0,
        lineage_ref: "research/120-nostra-design-language/prototypes/space-design/SPACE_DESIGN.md",
        surface_scope: ["space_shell", "workbench", "artifact_viewer"],
        a2ui_theme_policy: {
          token_version: "ndl-token-v1",
          safe_mode: true,
          theme_allowlist_id: "ndl-space-profile-draft",
          motion_policy: "system",
          contrast_preference: "system",
        },
        preview_status: "metadata_only",
        preview_note: "Cortex Web may display this draft profile as advisory metadata.",
      },
    ],
  };
}

test("Space design preview panel model is draft recommendation metadata only", () => {
  const model = buildSpaceDesignProfilePreviewPanelModel(previewFixture());

  assert.equal(model.visible, true);
  assert.equal(model.statusLabel, "Draft recommendation");
  assert.equal(model.profileId, "space-design-profile:space-research-observatory");
  assert.equal(model.reviewStatus, "draft");
  assert.equal(model.boundaryTone, "metadata_only");
  assert.match(model.boundaryLabel, /Tokens are not applied/i);
  assert.equal(model.exposesDesignTokens, false);
  assert.match(model.surfaceScopeLabel, /workbench/);
});

test("Space design preview panel blocks runtime-style fixture drift", () => {
  const fixture = previewFixture();
  const drifted = {
    ...fixture,
    runtime_policy: {
      ...fixture.runtime_policy,
      runtime_theme_selection: true,
    },
  } as unknown as SpaceDesignProfilePreviewFixture;
  const model = buildSpaceDesignProfilePreviewPanelModel(drifted);

  assert.equal(model.visible, true);
  assert.equal(model.boundaryTone, "blocked");
  assert.match(model.boundaryLabel, /Blocked from runtime theme binding/i);
});
