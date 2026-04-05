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

test("buildNavigationSections includes extra custom sections and applies per-section item order", () => {
  const entries = [
    { routeId: "/inbox", label: "Inbox", navSlot: "primary_attention" },
    { routeId: "/heap", label: "Heap", navSlot: "primary_workspace" },
  ];
  const compiledPlan = {
    entries: [
      { routeId: "/inbox", navSlot: "primary_attention" },
      { routeId: "/heap", navSlot: "primary_workspace" },
    ],
  };

  const sections = buildNavigationSections(
    entries as never[],
    compiledPlan as never,
    {
      navItems: {
        custom_views: {
          itemOrder: ["/explore?view=story", "/explore?view=density"],
          hidden: [],
        },
      },
    } as never,
    [
      {
        slot: "custom_views",
        label: "Views",
        entries: [
          {
            routeId: "/explore?view=density",
            label: "Density",
            icon: "bookmark",
            category: "custom",
            requiredRole: "viewer",
          },
          {
            routeId: "/explore?view=story",
            label: "Story",
            icon: "bookmark",
            category: "custom",
            requiredRole: "viewer",
          },
        ] as never[],
      },
    ],
  );

  const custom = sections.find((section) => section.slot === "custom_views");

  assert.equal(custom?.label, "Views");
  assert.deepEqual(custom?.entries.map((entry) => entry.routeId), [
    "/explore?view=story",
    "/explore?view=density",
  ]);
});
