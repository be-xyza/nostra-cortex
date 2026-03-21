import test from "node:test";
import assert from "node:assert/strict";
import { buildNavigationSections } from "../src/components/commons/navSections.ts";

test("buildNavigationSections groups visible entries by navSlot and preserves order", () => {
  const entries = [
    { routeId: "/inbox", label: "Inbox", navSlot: "primary_attention" },
    { routeId: "/heap", label: "Heap", navSlot: "primary_workspace" },
    { routeId: "/logs", label: "Logs", navSlot: "secondary_ops" },
  ];
  const compiledPlan = {
    entries: [
      { routeId: "/inbox", navSlot: "primary_attention" },
      { routeId: "/heap", navSlot: "primary_workspace" },
      { routeId: "/logs", navSlot: "secondary_ops" },
    ],
  };

  const sections = buildNavigationSections(entries as never[], compiledPlan as never);

  assert.deepEqual(
    sections.map((section) => ({
      navSlot: section.slot,
      routes: section.entries.map((entry) => entry.routeId),
    })),
    [
      { navSlot: "primary_attention", routes: ["/inbox"] },
      { navSlot: "primary_workspace", routes: ["/heap"] },
      { navSlot: "secondary_ops", routes: ["/logs"] },
    ]
  );
});
