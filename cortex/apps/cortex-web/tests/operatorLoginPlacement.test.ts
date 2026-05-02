import assert from "node:assert/strict";
import test from "node:test";

import { resolveOperatorLoginPlacement } from "../src/components/commons/operatorLoginPlacement.ts";

test("operator login placement uses the authority lane for expanded trusted read-only sessions", () => {
  assert.equal(
    resolveOperatorLoginPlacement({
      enabled: true,
      session: { authMode: "read_fallback" },
      collapsed: false,
    }),
    "authority_lane",
  );
});

test("operator login placement falls back to observer details when the authority lane is collapsed", () => {
  assert.equal(
    resolveOperatorLoginPlacement({
      enabled: true,
      session: { authMode: "read_fallback" },
      collapsed: true,
    }),
    "observer_details",
  );
});

test("operator login placement stays hidden for verified sessions and disabled deployments", () => {
  assert.equal(
    resolveOperatorLoginPlacement({
      enabled: true,
      session: { authMode: "principal_binding" },
      collapsed: false,
    }),
    "hidden",
  );
  assert.equal(
    resolveOperatorLoginPlacement({
      enabled: false,
      session: { authMode: "read_fallback" },
      collapsed: false,
    }),
    "hidden",
  );
});
