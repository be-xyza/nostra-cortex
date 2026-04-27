import fixture from "./spaceDesignProfilePreview.fixture.json";
import type { SpaceDesignProfilePreviewFixture } from "./spaceDesignProfilePreviewContract.ts";

export const SPACE_DESIGN_PROFILE_PREVIEW_FIXTURE = fixture as SpaceDesignProfilePreviewFixture;

export function buildSpaceDesignProfilePreviewResponse(): SpaceDesignProfilePreviewFixture {
  return SPACE_DESIGN_PROFILE_PREVIEW_FIXTURE;
}
