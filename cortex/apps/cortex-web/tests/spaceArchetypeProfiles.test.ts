import assert from "node:assert/strict";
import test from "node:test";

import {
  resolveExploreSurfacePolicy,
} from "../src/components/heap/exploreSurfacePolicy.ts";
import {
  getSpaceStudioTemplatesForArchetype,
  resolveSpaceArchetypeProfile,
} from "../src/components/spaces/spaceArchetypeProfiles.ts";

test("shared archetype profiles align Explore policy, Space visuals, and Space Studio templates", () => {
  const researchProfile = resolveSpaceArchetypeProfile("Research");
  const governanceProfile = resolveSpaceArchetypeProfile("Governance");
  const introProfile = resolveSpaceArchetypeProfile("Intro");

  assert.equal(researchProfile.id, "research");
  assert.equal(researchProfile.explorePolicyId, "explore.list.density.v1");
  assert.ok(researchProfile.visuals.gradient.includes("purple"));
  assert.equal(governanceProfile.explorePolicyId, "explore.list.lineage.v1");
  assert.equal(introProfile.explorePolicyId, "explore.list.story.v1");

  const explorePolicy = resolveExploreSurfacePolicy({ spaceArchetype: "Research" });
  assert.equal(explorePolicy.policyId, researchProfile.explorePolicyId);

  const researchTemplates = getSpaceStudioTemplatesForArchetype("Research");
  assert.ok(researchTemplates.some((template) => template.templateId === "tpl_research_starter_v1"));
});

test("unknown archetypes resolve to a stable default profile", () => {
  const profile = resolveSpaceArchetypeProfile("Unknown");

  assert.equal(profile.id, "default");
  assert.equal(profile.explorePolicyId, "explore.list.default.v1");
  assert.ok(profile.spaceStudioTemplates.length >= 1);
});
