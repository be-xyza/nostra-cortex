import assert from "node:assert/strict";
import test from "node:test";

import {
  buildFallbackAuthSession,
  buildFallbackShellLayoutSpec,
  buildFallbackWhoami,
  describeAuthorityMode,
  formatAuthorityStatus,
  formatReadFallbackNotice,
  formatReadOnlyObserverDetailLines,
  formatReadOnlyObserverSummary,
  formatShellBootstrapWarning,
  isPublicObserverGatewayBoundary,
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

test("read fallback notice keeps viewer observability and operator gating explicit", () => {
  const notice = formatReadFallbackNotice("https://eudaemon-alpha-01.taild09100.ts.net");

  assert.match(notice, /Gateway is reachable/i);
  assert.match(notice, /Viewer-scoped heap data remains available/i);
  assert.match(notice, /operator action plans and mutations stay gated/i);
});

test("read-only observer copy separates compact summary from expandable details", () => {
  const details = formatReadOnlyObserverDetailLines("https://eudaemon-alpha-01.taild09100.ts.net");

  assert.equal(formatReadOnlyObserverSummary(), "Read-only observer mode");
  assert.ok(details.some((line) => /Gateway reachable/i.test(line)));
  assert.ok(details.some((line) => /no verified operator identity/i.test(line)));
  assert.ok(details.some((line) => /Heap data is visible in viewer mode/i.test(line)));
  assert.ok(details.some((line) => /Operator action plans and mutations remain gated/i.test(line)));
  assert.ok(details.some((line) => /Operator sign-in is disabled for this deployment/i.test(line)));
  assert.ok(details.some((line) => /eudaemon-alpha-01/i.test(line)));
  assert.equal(
    [formatReadOnlyObserverSummary(), ...details].some((line) => /degraded/i.test(line)),
    false,
  );
});

test("read-only observer copy omits disabled sign-in guidance when operator login is enabled", () => {
  const details = formatReadOnlyObserverDetailLines(
    "http://127.0.0.1:3000",
    "reachable",
    { operatorLoginEnabled: true },
  );

  assert.equal(
    details.some((line) => /Operator sign-in is disabled for this deployment/i.test(line)),
    false,
  );
  assert.ok(details.some((line) => /Gateway reachable/i.test(line)));
});

test("read-only observer copy handles public browser private-gateway boundaries", () => {
  const details = formatReadOnlyObserverDetailLines(
    "https://eudaemon-alpha-01.taild09100.ts.net",
    "public_restricted",
  );

  assert.ok(details.some((line) => /private or browser-restricted/i.test(line)));
  assert.ok(details.some((line) => /trusted operator session/i.test(line)));
  assert.equal(
    isPublicObserverGatewayBoundary(
      "Failed to fetch",
      "https://eudaemon-alpha-01.taild09100.ts.net",
      true,
    ),
    true,
  );
  assert.equal(
    isPublicObserverGatewayBoundary(
      "Failed to fetch",
      "same-origin /api proxy",
      true,
    ),
    false,
  );
  assert.equal(
    isPublicObserverGatewayBoundary(
      "Failed to fetch",
      "https://eudaemon-alpha-01.taild09100.ts.net",
      false,
    ),
    false,
  );
});

test("authority status labels distinguish public viewer, local dev, and verified sessions", () => {
  const readFallback = buildFallbackAuthSession("operator.jo", "operator");
  assert.equal(describeAuthorityMode(readFallback), "Read-only observer mode");
  assert.match(formatAuthorityStatus(readFallback), /read-only observability/i);
  assert.match(formatAuthorityStatus(readFallback), /operator actions are gated/i);

  assert.equal(
    describeAuthorityMode({
      ...readFallback,
      authMode: "dev_override",
      identitySource: "dev_unverified_header",
      activeRole: "operator",
      grantedRoles: ["viewer", "operator"],
      allowRoleSwitch: true,
      allowUnverifiedRoleHeader: true,
    }),
    "Local operator mode",
  );

  assert.equal(
    describeAuthorityMode({
      ...readFallback,
      authMode: "principal_binding",
      identityVerified: true,
      identitySource: "principal_binding",
      activeRole: "operator",
      grantedRoles: ["viewer", "operator"],
      allowRoleSwitch: true,
    }),
    "Verified operator mode",
  );
});
