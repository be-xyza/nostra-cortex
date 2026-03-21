import assert from "node:assert/strict";
import test from "node:test";

import {
  buildFallbackShellLayoutSpec,
  buildFallbackWhoami,
  formatShellBootstrapWarning,
} from "../src/components/commons/shellBootstrapFallback.ts";

test("buildFallbackShellLayoutSpec exposes a usable local navigation shell", () => {
  const layout = buildFallbackShellLayoutSpec();

  assert.equal(layout.layoutId, "default");
  assert.ok(layout.navigationGraph.entries.length > 0);
  assert.ok(
    layout.navigationGraph.entries.some((entry) => entry.routeId === "/explore"),
  );
  assert.ok(
    layout.navigationGraph.entries.some((entry) => entry.routeId === "/labs"),
  );
});

test("buildFallbackWhoami preserves the requested local identity context", () => {
  const whoami = buildFallbackWhoami(
    "operator.jo",
    "operator",
    "2026-03-20T09:40:00.000Z",
  );

  assert.equal(whoami.principal, "operator.jo");
  assert.equal(whoami.requestedRole, "operator");
  assert.equal(whoami.effectiveRole, "operator");
  assert.equal(whoami.allowUnverifiedRoleHeader, true);
  assert.equal(whoami.generatedAt, "2026-03-20T09:40:00.000Z");
});

test("formatShellBootstrapWarning keeps fallback messaging explicit", () => {
  assert.match(
    formatShellBootstrapWarning(
      "layout",
      "503 Service Unavailable",
      "http://127.0.0.1:3001",
    ),
    /local preview shell/i,
  );
  assert.match(
    formatShellBootstrapWarning(
      "layout",
      "503 Service Unavailable",
      "http://127.0.0.1:3001",
    ),
    /127\.0\.0\.1:3001/,
  );
  assert.match(
    formatShellBootstrapWarning("identity", "connection reset"),
    /local preview role/i,
  );
});
