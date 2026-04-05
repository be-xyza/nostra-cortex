import assert from "node:assert/strict";
import test from "node:test";

import {
  buildFallbackAuthSession,
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
  assert.equal(whoami.effectiveRole, "viewer");
  assert.equal(whoami.allowUnverifiedRoleHeader, false);
  assert.equal(whoami.generatedAt, "2026-03-20T09:40:00.000Z");
});

test("buildFallbackAuthSession is explicitly degraded instead of optimistic operator auth", () => {
  const session = buildFallbackAuthSession("operator.jo", "operator", "2026-03-20T09:40:00.000Z");

  assert.equal(session.principal, "operator.jo");
  assert.equal(session.activeRole, "viewer");
  assert.equal(session.authMode, "read_fallback");
  assert.equal(session.allowRoleSwitch, false);
  assert.deepEqual(session.grantedRoles, ["viewer"]);
});

test("formatShellBootstrapWarning keeps fallback messaging explicit", () => {
  assert.match(
    formatShellBootstrapWarning(
      "layout",
      "503 Service Unavailable",
      "http://127.0.0.1:3001",
    ),
    /local fallback shell/i,
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
    /local fallback role/i,
  );
});
