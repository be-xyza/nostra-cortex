import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import test from "node:test";

import { resolveBrandVisualState } from "../src/components/commons/brandLogoPolicy.ts";

type CaseFixture = {
  id: string;
  input: {
    mode: "philosophical" | "technical" | "custom";
    authority: "official" | "labs";
    temporal?: string;
    custom_gap_angle?: number;
    custom_stroke_width?: number;
    custom_outer_color?: string;
    custom_inner_color?: string;
    governance_policy?: {
      allow_labs_customizations?: boolean;
      mode_baselines?: {
        philosophical?: {
          gap_degrees?: number;
          stroke_width_delta_px?: number;
          stroke_cap?: "round" | "butt" | "square";
          force_gradient?: boolean;
        };
        technical?: {
          stroke_cap?: "round" | "butt" | "square";
          force_gradient?: boolean;
        };
      };
    };
  };
  expected: {
    effective_mode: "philosophical" | "technical";
    temporal: string;
    gap_angle: number;
    stroke_width: number;
    is_gradient: boolean;
    stroke_cap: string;
    outer_base: string;
    inner_base: string;
  };
};

const CASES_PATH = path.resolve(
  import.meta.dirname,
  "../../../../shared/standards/branding/brand_visual_state_cases_v1.json",
);

test("resolver conforms to canonical visual state fixtures", () => {
  const parsed = JSON.parse(fs.readFileSync(CASES_PATH, "utf8")) as {
    cases: CaseFixture[];
  };

  for (const fixture of parsed.cases) {
    const state = resolveBrandVisualState({
      mode: fixture.input.mode,
      authority: fixture.input.authority,
      temporal: (fixture.input.temporal ?? "none") as "none" | string,
      customGapAngle: fixture.input.custom_gap_angle,
      customStrokeWidth: fixture.input.custom_stroke_width,
      customOuterColor: fixture.input.custom_outer_color,
      customInnerColor: fixture.input.custom_inner_color,
      governancePolicy: fixture.input.governance_policy
        ? {
            allowLabsCustomizations:
              fixture.input.governance_policy.allow_labs_customizations,
            modeBaselines: fixture.input.governance_policy.mode_baselines
              ? {
                  philosophical:
                    fixture.input.governance_policy.mode_baselines.philosophical
                      ? {
                          gapDegrees:
                            fixture.input.governance_policy.mode_baselines
                              .philosophical.gap_degrees,
                          strokeWidthDeltaPx:
                            fixture.input.governance_policy.mode_baselines
                              .philosophical.stroke_width_delta_px,
                          strokeCap:
                            fixture.input.governance_policy.mode_baselines
                              .philosophical.stroke_cap,
                          forceGradient:
                            fixture.input.governance_policy.mode_baselines
                              .philosophical.force_gradient,
                        }
                      : undefined,
                  technical:
                    fixture.input.governance_policy.mode_baselines.technical
                      ? {
                          strokeCap:
                            fixture.input.governance_policy.mode_baselines
                              .technical.stroke_cap,
                          forceGradient:
                            fixture.input.governance_policy.mode_baselines
                              .technical.force_gradient,
                        }
                      : undefined,
                }
              : undefined,
          }
        : undefined,
    });

    assert.equal(state.effectiveMode, fixture.expected.effective_mode, `${fixture.id}: effective_mode`);
    assert.equal(state.temporal, fixture.expected.temporal, `${fixture.id}: temporal`);
    assert.equal(state.gapAngle, fixture.expected.gap_angle, `${fixture.id}: gap_angle`);
    assert.equal(state.strokeWidth, fixture.expected.stroke_width, `${fixture.id}: stroke_width`);
    assert.equal(state.isGradient, fixture.expected.is_gradient, `${fixture.id}: is_gradient`);
    assert.equal(state.strokeCap, fixture.expected.stroke_cap, `${fixture.id}: stroke_cap`);
    assert.equal(state.outerBase, fixture.expected.outer_base, `${fixture.id}: outer_base`);
    assert.equal(state.innerBase, fixture.expected.inner_base, `${fixture.id}: inner_base`);
  }
});

test("policy document input drives kernel geometry", () => {
  const state = resolveBrandVisualState({
    mode: "technical",
    authority: "official",
    policyDocument: {
      kernel: {
        mark_composition: "outer_broken_ring_inner_solid_dot",
        technical_canonical_gap_degrees: 72,
        ring_radius_px: 30,
        dot_radius_px: 10,
        base_stroke_width_px: 6,
        steward_gated: true,
      },
    },
  });

  assert.equal(state.gapAngle, 72);
  assert.equal(state.strokeWidth, 6);
  assert.equal(state.ringRadius, 30);
  assert.equal(state.dotRadius, 10);
});
