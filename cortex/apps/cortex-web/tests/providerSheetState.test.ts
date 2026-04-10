import assert from "node:assert/strict";
import test from "node:test";

import { createEmptyProviderForm } from "../src/components/system/providerForm.ts";
import {
  serializeProviderFormSnapshot,
  shouldWarnBeforeClosingProviderPanel,
} from "../src/components/system/providerSheetState.ts";

test("serializeProviderFormSnapshot ignores harmless trailing whitespace drift", () => {
  const base = createEmptyProviderForm();
  const changed = {
    ...base,
    name: "OpenRouter x   ",
    metadataJson: "{\n  \n}\n",
  };
  const normalized = {
    ...base,
    name: "OpenRouter x",
    metadataJson: "{\n  \n}",
  };

  assert.equal(
    serializeProviderFormSnapshot(changed),
    serializeProviderFormSnapshot(normalized),
  );
});

test("shouldWarnBeforeClosingProviderPanel only warns for dirty provider sheets", () => {
  assert.equal(
    shouldWarnBeforeClosingProviderPanel({
      panelState: { kind: "provider", providerId: "openrouter_primary" },
      isDirty: true,
      isSubmitting: false,
    }),
    true,
  );
  assert.equal(
    shouldWarnBeforeClosingProviderPanel({
      panelState: { kind: "provider", providerId: "openrouter_primary" },
      isDirty: false,
      isSubmitting: false,
    }),
    false,
  );
  assert.equal(
    shouldWarnBeforeClosingProviderPanel({
      panelState: { kind: "discovery" },
      isDirty: true,
      isSubmitting: false,
    }),
    false,
  );
});
