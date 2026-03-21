import assert from "node:assert/strict";
import test from "node:test";

import { workbenchApi } from "../src/api.ts";

test("getBrandPolicy targets canonical endpoint", async () => {
  const originalFetch = globalThis.fetch;
  let capturedUrl = "";
  let capturedInit: RequestInit | undefined;

  globalThis.fetch = (async (input: URL | RequestInfo, init?: RequestInit) => {
    capturedUrl = String(input);
    capturedInit = init;
    return new Response(
      JSON.stringify({
        policy: {
          schema_version: "brand-policy/v1",
          policy_id: "nostra_cortex_master_brand",
          policy_version: 1,
          kernel: {
            mark_composition: "outer_broken_ring_inner_solid_dot",
            technical_canonical_gap_degrees: 45,
            ring_radius_px: 35,
            dot_radius_px: 14,
            base_stroke_width_px: 8,
            steward_gated: true,
          },
          style: {
            allow_labs_customizations: true,
            labs_bounds: {
              gap_min_degrees: 12,
              gap_max_degrees: 160,
              stroke_min_px: 4,
              stroke_max_px: 20,
            },
            official_palette: {
              outer_base: "#E63946",
              outer_gradient_to: "#F4A261",
              inner_base: "#1D3557",
              inner_gradient_to: "#00B4D8",
            },
            temporal_variants: {},
            motion: {
              philosophical: {},
              technical: {},
            },
          },
          temporal_windows: [],
          updated_at_ns: 0,
        },
        policyVersion: 1,
        policyDigest: "digest",
        activeTemporalState: "none",
        serverTimeUtc: "2026-02-24T00:00:00Z",
        sourceOfTruth: "canister",
      }),
      { status: 200, headers: { "Content-Type": "application/json" } },
    );
  }) as typeof fetch;

  try {
    const response = await workbenchApi.getBrandPolicy();
    assert.equal(response.sourceOfTruth, "canister");
    assert.equal(response.policyVersion, 1);
  } finally {
    globalThis.fetch = originalFetch;
  }

  assert.ok(capturedUrl.endsWith("/api/system/brand-policy"));
  assert.equal(capturedInit?.method, undefined);
});
