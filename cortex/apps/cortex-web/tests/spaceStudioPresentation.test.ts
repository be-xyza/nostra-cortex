import assert from "node:assert/strict";
import test from "node:test";

import { resolveSpaceStudioPrimaryActionState } from "../src/components/spaces/spaceStudioPresentation.ts";

test("viewer/operator promotion state submits for steward review and blocks invalid drafts", () => {
  const invalid = resolveSpaceStudioPrimaryActionState({
    canSubmitLiveCreate: false,
    governanceScope: "private",
    draftStatus: "draft",
    validationReady: false,
    confirmLiveCreate: false,
    submitStatus: "idle",
  });
  const submitted = resolveSpaceStudioPrimaryActionState({
    canSubmitLiveCreate: false,
    governanceScope: "public",
    draftStatus: "submitted",
    validationReady: true,
    confirmLiveCreate: false,
    submitStatus: "idle",
  });

  assert.equal(invalid.mode, "steward_review");
  assert.equal(invalid.label, "Submit for steward review");
  assert.equal(invalid.disabled, true);
  assert.equal(submitted.label, "Submitted for steward review");
  assert.equal(submitted.disabled, true);
});

test("steward/admin promotion state requires confirmation before live creation", () => {
  const awaitingConfirmation = resolveSpaceStudioPrimaryActionState({
    canSubmitLiveCreate: true,
    governanceScope: "personal",
    draftStatus: "draft",
    validationReady: true,
    confirmLiveCreate: false,
    submitStatus: "idle",
  });
  const ready = resolveSpaceStudioPrimaryActionState({
    canSubmitLiveCreate: true,
    governanceScope: "personal",
    draftStatus: "draft",
    validationReady: true,
    confirmLiveCreate: true,
    submitStatus: "idle",
  });

  assert.equal(awaitingConfirmation.mode, "live_create");
  assert.equal(awaitingConfirmation.label, "Create personal space");
  assert.equal(awaitingConfirmation.disabled, true);
  assert.equal(ready.disabled, false);
});
