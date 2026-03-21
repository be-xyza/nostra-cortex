import assert from "node:assert/strict";
import test from "node:test";

import { resolveSpaceSelectorTriggerState } from "../src/components/commons/spaceSelectorPresentation.ts";

test("shows loading copy while live registry is unresolved and no active space has hydrated", () => {
  assert.deepEqual(
    resolveSpaceSelectorTriggerState({
      isMeta: false,
      isMulti: false,
      activeSpaceCount: 0,
      registryResolved: false,
    }),
    {
      title: "Loading Spaces...",
      subtitle: "Syncing live registry",
    },
  );
});

test("keeps meta copy when the meta workbench is active", () => {
  assert.deepEqual(
    resolveSpaceSelectorTriggerState({
      isMeta: true,
      isMulti: false,
      activeSpaceCount: 1,
      activeSpaceName: "Research · 01KM4C04",
      registryResolved: true,
    }),
    {
      title: "Meta Workbench",
      subtitle: "Aggregated Session",
    },
  );
});

test("shows the hydrated space name once the registry has resolved", () => {
  assert.deepEqual(
    resolveSpaceSelectorTriggerState({
      isMeta: false,
      isMulti: false,
      activeSpaceCount: 1,
      activeSpaceName: "Research · 01KM4C04",
      registryResolved: true,
    }),
    {
      title: "Research · 01KM4C04",
      subtitle: "Active Space",
    },
  );
});
