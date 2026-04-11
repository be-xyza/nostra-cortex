import assert from "node:assert/strict";
import test from "node:test";

import type {
  CompiledNavigationPlan,
  NavigationEntrySpec,
  ShellLayoutSpec,
} from "../src/contracts.ts";
import { resolveShellEntries } from "../src/components/commons/shellNavigationModel.ts";

function layout(entries: NavigationEntrySpec[]): ShellLayoutSpec {
  return {
    layoutId: "cortex.desktop.shell.v1",
    navigationGraph: { entries },
  };
}

function compiledPlan(
  actorRole: string,
  entries: Array<{ routeId: string; rank: number }>,
): CompiledNavigationPlan {
  return {
    schemaVersion: "1.0.0",
    generatedAt: "2026-03-22T00:00:00Z",
    spaceId: "space-1",
    actorRole,
    intent: "navigate",
    density: "comfortable",
    planHash: "plan-hash",
    entries: entries.map((entry) => ({
      capabilityId: `cap:${entry.routeId}`,
      routeId: entry.routeId,
      label: entry.routeId,
      icon: "HP",
      category: "core",
      requiredRole: "viewer",
      navSlot: "primary_workspace",
      navBand: "primary",
      surfacingHeuristic: "primary_core",
      operationalFrequency: "continuous",
      rank: entry.rank,
    })),
    surfacing: {
      primaryCore: entries.map((entry) => entry.routeId),
      secondary: {},
      contextualDeep: [],
      hidden: [],
    },
  };
}

test("resolveShellEntries keeps the base shell navigation when no compiled plan exists", () => {
  const entries = resolveShellEntries({
    layoutSpec: layout([
      {
        routeId: "/explore",
        label: "Explore",
        icon: "EX",
        category: "core",
        requiredRole: "operator",
      },
      {
        routeId: "/labs",
        label: "Labs",
        icon: "LB",
        category: "workbench",
        requiredRole: "viewer",
      },
    ]),
    compiledPlan: null,
    actorRole: "operator",
    playgroundEnabled: true,
  });

  assert.deepEqual(
    entries.map((entry) => entry.routeId),
    ["/explore", "/labs"],
  );
});

test("resolveShellEntries falls back to base entries when compiled plan role mismatches the requested role", () => {
  const entries = resolveShellEntries({
    layoutSpec: layout([
      {
        routeId: "/explore",
        label: "Explore",
        icon: "EX",
        category: "core",
        requiredRole: "operator",
      },
      {
        routeId: "/labs",
        label: "Labs",
        icon: "LB",
        category: "workbench",
        requiredRole: "viewer",
      },
    ]),
    compiledPlan: compiledPlan("viewer", [
      { routeId: "/explore", rank: 1 },
      { routeId: "/labs", rank: 2 },
    ]),
    actorRole: "operator",
    playgroundEnabled: true,
  });

  assert.deepEqual(
    entries.map((entry) => entry.routeId),
    ["/explore", "/labs"],
  );
});

test("resolveShellEntries honors compiled plan ordering when actor roles match", () => {
  const entries = resolveShellEntries({
    layoutSpec: layout([
      {
        routeId: "/explore",
        label: "Explore",
        icon: "EX",
        category: "core",
        requiredRole: "operator",
      },
      {
        routeId: "/labs",
        label: "Labs",
        icon: "LB",
        category: "workbench",
        requiredRole: "viewer",
      },
    ]),
    compiledPlan: compiledPlan("operator", [
      { routeId: "/explore", rank: 1 },
    ]),
    actorRole: "operator",
    playgroundEnabled: true,
  });

  assert.deepEqual(
    entries.map((entry) => entry.routeId),
    ["/explore"],
  );
});

test("resolveShellEntries keeps the base shell available during read-fallback identity mode", () => {
  const entries = resolveShellEntries({
    layoutSpec: layout([
      {
        routeId: "/explore",
        label: "Explore",
        icon: "EX",
        category: "core",
        requiredRole: "operator",
      },
      {
        routeId: "/labs",
        label: "Labs",
        icon: "LB",
        category: "workbench",
        requiredRole: "viewer",
      },
    ]),
    compiledPlan: compiledPlan("viewer", [
      { routeId: "/explore", rank: 1 },
      { routeId: "/labs", rank: 2 },
    ]),
    actorRole: "viewer",
    playgroundEnabled: true,
    preferBaseEntries: true,
  });

  assert.deepEqual(
    entries.map((entry) => entry.routeId),
    ["/explore", "/labs"],
  );
});
