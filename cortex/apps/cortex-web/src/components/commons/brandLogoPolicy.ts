export type BrandMode = "philosophical" | "technical" | "custom";
export type AuthorityState = "official" | "labs";
export type TemporalState = "none" | (string & {});

export interface BrandPalette {
    outer_base: string;
    outer_gradient_to: string;
    inner_base: string;
    inner_gradient_to: string;
}

export interface LabsBounds {
    gap_min_degrees: number;
    gap_max_degrees: number;
    stroke_min_px: number;
    stroke_max_px: number;
}

export interface TemporalVariantPolicy {
    force_gradient: boolean;
    stroke_cap: "round" | "butt" | "square" | string;
    palette: BrandPalette;
}

interface PhilosophicalMotionPolicy {
    container_transition_sec: number;
    ring_transition_sec: number;
    stroke_transition_sec: number;
    ring_animation_duration_sec: number;
    ring_rotation_delta_deg: number;
    ring_stroke_delta_px: number;
    dot_transition_sec: number;
    dot_animation_duration_sec: number;
    dot_pulse_radius_delta_px: number;
    dot_pulse_opacity_min: number;
}

interface TechnicalMotionPolicy {
    container_transition_ms: number;
    ring_transition_ms: number;
    stroke_transition_ms: number;
    dot_transition_ms: number;
    step_count: number;
    ring_step_count: number;
}

interface BrandMotionPolicy {
    philosophical: PhilosophicalMotionPolicy;
    technical: TechnicalMotionPolicy;
}

export interface PhilosophicalModeBaselinePolicy {
    gap_degrees: number;
    stroke_width_delta_px: number;
    stroke_cap: "round" | "butt" | "square" | string;
    force_gradient: boolean;
}

export interface TechnicalModeBaselinePolicy {
    stroke_cap: "round" | "butt" | "square" | string;
    force_gradient: boolean;
}

export interface BrandModeBaselinesPolicy {
    philosophical: PhilosophicalModeBaselinePolicy;
    technical: TechnicalModeBaselinePolicy;
}

export interface BrandHostDefaultsPolicy {
    default_temporal_state: string;
    default_authority: AuthorityState | string;
    theme_mode_map: Record<string, "philosophical" | "technical" | "custom" | string>;
}

export interface BrandStylePolicy {
    allow_labs_customizations: boolean;
    labs_bounds: LabsBounds;
    official_palette: BrandPalette;
    mode_baselines?: BrandModeBaselinesPolicy;
    host_defaults?: BrandHostDefaultsPolicy;
    temporal_variants: Record<string, TemporalVariantPolicy>;
    motion: BrandMotionPolicy;
}

export interface BrandKernelPolicy {
    mark_composition: string;
    technical_canonical_gap_degrees: number;
    ring_radius_px: number;
    dot_radius_px: number;
    base_stroke_width_px: number;
    steward_gated: boolean;
}

export interface TemporalWindow {
    state: string;
    recurrence: string;
    start_month_day: string;
    end_month_day: string;
    start_time_utc: string;
    end_time_utc: string;
}

export interface BrandPolicyDocument {
    schema_version: string;
    policy_id: string;
    policy_version: number;
    kernel: BrandKernelPolicy;
    style: BrandStylePolicy;
    temporal_windows: TemporalWindow[];
    updated_at_ns: number;
    source_ref?: string;
}

export interface BrandGovernancePolicyInput {
    allowLabsCustomizations?: boolean;
    labsBounds?: Partial<{ gapMinDegrees: number; gapMaxDegrees: number; strokeMinPx: number; strokeMaxPx: number }>;
    officialPalette?: Partial<{ outerBase: string; outerGradientTo: string; innerBase: string; innerGradientTo: string }>;
    modeBaselines?: {
        philosophical?: Partial<{
            gapDegrees: number;
            strokeWidthDeltaPx: number;
            strokeCap: "round" | "butt" | "square" | string;
            forceGradient: boolean;
        }>;
        technical?: Partial<{
            strokeCap: "round" | "butt" | "square" | string;
            forceGradient: boolean;
        }>;
    };
    hostDefaults?: Partial<{
        defaultTemporalState: string;
        defaultAuthority: AuthorityState;
        themeModeMap: Record<string, "philosophical" | "technical" | "custom">;
    }>;
    temporalVariants?: Partial<Record<string, Partial<{ forceGradient: boolean; strokeCap: string; palette: Partial<{ outerBase: string; outerGradientTo: string; innerBase: string; innerGradientTo: string }> }>>>;
    motion?: {
        philosophical?: Partial<{
            containerTransitionSec: number;
            ringTransitionSec: number;
            strokeTransitionSec: number;
            ringAnimationDurationSec: number;
            ringRotationDeltaDeg: number;
            ringStrokeDeltaPx: number;
            dotTransitionSec: number;
            dotAnimationDurationSec: number;
            dotPulseRadiusDeltaPx: number;
            dotPulseOpacityMin: number;
        }>;
        technical?: Partial<{
            containerTransitionMs: number;
            ringTransitionMs: number;
            strokeTransitionMs: number;
            dotTransitionMs: number;
            stepCount: number;
            ringStepCount: number;
        }>;
    };
}

export interface BrandLogoProps {
    size?: number;
    mode?: BrandMode;
    authority?: AuthorityState;
    temporal?: TemporalState;
    animating?: boolean;
    customOuterColor?: string;
    customInnerColor?: string;
    customGapAngle?: number;
    customStrokeWidth?: number;
    governancePolicy?: BrandGovernancePolicyInput;
    policyDocument?: Partial<BrandPolicyDocument>;
}

export interface ResolvedBrandVisualState {
    requestedMode: BrandMode;
    effectiveMode: "philosophical" | "technical";
    authority: AuthorityState;
    temporal: TemporalState;
    isGradient: boolean;
    strokeCap: "round" | "butt" | "square";
    gapAngle: number;
    strokeWidth: number;
    outerBase: string;
    outerGradientTo: string;
    innerBase: string;
    innerGradientTo: string;
    ringRadius: number;
    dotRadius: number;
    motion: {
        philosophical: {
            containerTransitionSec: number;
            ringTransitionSec: number;
            strokeTransitionSec: number;
            ringAnimationDurationSec: number;
            ringRotationDeltaDeg: number;
            ringStrokeDeltaPx: number;
            dotTransitionSec: number;
            dotAnimationDurationSec: number;
            dotPulseRadiusDeltaPx: number;
            dotPulseOpacityMin: number;
        };
        technical: {
            containerTransitionMs: number;
            ringTransitionMs: number;
            strokeTransitionMs: number;
            dotTransitionMs: number;
            stepCount: number;
            ringStepCount: number;
        };
    };
}

export const DEFAULT_BRAND_POLICY_DOCUMENT: BrandPolicyDocument = {
    schema_version: "brand-policy/v1",
    policy_id: "nostra-cortex-master-brand",
    policy_version: 2,
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
        mode_baselines: {
            philosophical: {
                gap_degrees: 60,
                stroke_width_delta_px: 2,
                stroke_cap: "round",
                force_gradient: true,
            },
            technical: {
                stroke_cap: "square",
                force_gradient: false,
            },
        },
        host_defaults: {
            default_temporal_state: "none",
            default_authority: "official",
            theme_mode_map: {
                nostra: "philosophical",
                cortex: "technical",
            },
        },
        temporal_variants: {
            christmas: {
                force_gradient: true,
                stroke_cap: "round",
                palette: {
                    outer_base: "#D42A38",
                    outer_gradient_to: "#0F523A",
                    inner_base: "#FFD700",
                    inner_gradient_to: "#FFF5E1",
                },
            },
        },
        motion: {
            philosophical: {
                container_transition_sec: 0.4,
                ring_transition_sec: 0.5,
                stroke_transition_sec: 0.3,
                ring_animation_duration_sec: 4,
                ring_rotation_delta_deg: 5,
                ring_stroke_delta_px: 2,
                dot_transition_sec: 0.5,
                dot_animation_duration_sec: 2,
                dot_pulse_radius_delta_px: 1.5,
                dot_pulse_opacity_min: 0.85,
            },
            technical: {
                container_transition_ms: 180,
                ring_transition_ms: 200,
                stroke_transition_ms: 200,
                dot_transition_ms: 200,
                step_count: 2,
                ring_step_count: 4,
            },
        },
    },
    temporal_windows: [
        {
            state: "christmas",
            recurrence: "annual",
            start_month_day: "12-15",
            end_month_day: "12-26",
            start_time_utc: "00:00:00",
            end_time_utc: "23:59:59",
        },
    ],
    updated_at_ns: 0,
    source_ref: "DEC-2026-02-24-018",
};

type TemporalVariantOverride = {
    force_gradient?: boolean;
    stroke_cap?: string;
    palette?: Partial<BrandPalette>;
};

type ModeBaselineOverride = {
    philosophical?: Partial<PhilosophicalModeBaselinePolicy>;
    technical?: Partial<TechnicalModeBaselinePolicy>;
};

type HostDefaultsOverride = Partial<BrandHostDefaultsPolicy>;

function clamp(value: number, min: number, max: number): number {
    return Math.min(max, Math.max(min, value));
}

function mergeTemporalVariants(
    base: Record<string, TemporalVariantPolicy>,
    incoming?: Record<string, TemporalVariantOverride>,
    fallbackPalette: BrandPalette = DEFAULT_BRAND_POLICY_DOCUMENT.style.official_palette,
    technicalBaseline: TechnicalModeBaselinePolicy =
        DEFAULT_BRAND_POLICY_DOCUMENT.style.mode_baselines!.technical,
): Record<string, TemporalVariantPolicy> {
    if (!incoming) {
        return { ...base };
    }

    const merged: Record<string, TemporalVariantPolicy> = { ...base };
    for (const [key, partial] of Object.entries(incoming)) {
        const current = merged[key] ?? {
            force_gradient: technicalBaseline.force_gradient,
            stroke_cap: technicalBaseline.stroke_cap,
            palette: {
                outer_base: fallbackPalette.outer_base,
                outer_gradient_to: fallbackPalette.outer_gradient_to,
                inner_base: fallbackPalette.inner_base,
                inner_gradient_to: fallbackPalette.inner_gradient_to,
            },
        };
        merged[key] = {
            force_gradient: partial.force_gradient ?? current.force_gradient,
            stroke_cap: partial.stroke_cap ?? current.stroke_cap,
            palette: {
                outer_base: partial.palette?.outer_base ?? current.palette.outer_base,
                outer_gradient_to: partial.palette?.outer_gradient_to ?? current.palette.outer_gradient_to,
                inner_base: partial.palette?.inner_base ?? current.palette.inner_base,
                inner_gradient_to: partial.palette?.inner_gradient_to ?? current.palette.inner_gradient_to,
            },
        };
    }
    return merged;
}

function mergeModeBaselines(
    base: BrandModeBaselinesPolicy,
    incoming?: ModeBaselineOverride,
): BrandModeBaselinesPolicy {
    if (!incoming) {
        return {
            philosophical: { ...base.philosophical },
            technical: { ...base.technical },
        };
    }

    return {
        philosophical: {
            ...base.philosophical,
            ...(incoming.philosophical ?? {}),
        },
        technical: {
            ...base.technical,
            ...(incoming.technical ?? {}),
        },
    };
}

function mergeHostDefaults(
    base: BrandHostDefaultsPolicy,
    incoming?: HostDefaultsOverride,
): BrandHostDefaultsPolicy {
    if (!incoming) {
        return {
            ...base,
            theme_mode_map: { ...base.theme_mode_map },
        };
    }

    return {
        default_temporal_state: incoming.default_temporal_state ?? base.default_temporal_state,
        default_authority: incoming.default_authority ?? base.default_authority,
        theme_mode_map: {
            ...base.theme_mode_map,
            ...(incoming.theme_mode_map ?? {}),
        },
    };
}

function mergePolicy(
    policyDocument?: Partial<BrandPolicyDocument>,
    governancePolicy?: BrandGovernancePolicyInput,
): BrandPolicyDocument {
    const base = DEFAULT_BRAND_POLICY_DOCUMENT;
    const policy = {
        ...base,
        ...policyDocument,
        kernel: {
            ...base.kernel,
            ...(policyDocument?.kernel ?? {}),
        },
        style: {
            ...base.style,
            ...(policyDocument?.style ?? {}),
            labs_bounds: {
                ...base.style.labs_bounds,
                ...(policyDocument?.style?.labs_bounds ?? {}),
            },
            official_palette: {
                ...base.style.official_palette,
                ...(policyDocument?.style?.official_palette ?? {}),
            },
            mode_baselines: mergeModeBaselines(
                base.style.mode_baselines ?? DEFAULT_BRAND_POLICY_DOCUMENT.style.mode_baselines!,
                policyDocument?.style?.mode_baselines,
            ),
            host_defaults: mergeHostDefaults(
                base.style.host_defaults ?? DEFAULT_BRAND_POLICY_DOCUMENT.style.host_defaults!,
                policyDocument?.style?.host_defaults,
            ),
            temporal_variants: mergeTemporalVariants(
                base.style.temporal_variants,
                policyDocument?.style?.temporal_variants,
                {
                    ...base.style.official_palette,
                    ...(policyDocument?.style?.official_palette ?? {}),
                },
                mergeModeBaselines(
                    base.style.mode_baselines ?? DEFAULT_BRAND_POLICY_DOCUMENT.style.mode_baselines!,
                    policyDocument?.style?.mode_baselines,
                ).technical,
            ),
            motion: {
                philosophical: {
                    ...base.style.motion.philosophical,
                    ...(policyDocument?.style?.motion?.philosophical ?? {}),
                },
                technical: {
                    ...base.style.motion.technical,
                    ...(policyDocument?.style?.motion?.technical ?? {}),
                },
            },
        },
    } satisfies BrandPolicyDocument;

    if (!governancePolicy) {
        return policy;
    }

    const temporalVariantsFromGovernance = governancePolicy.temporalVariants
        ? Object.fromEntries(
              Object.entries(governancePolicy.temporalVariants).map(([key, value]) => [
                  key,
                  {
                      force_gradient: value?.forceGradient,
                      stroke_cap: value?.strokeCap,
                      palette: {
                          outer_base: value?.palette?.outerBase,
                          outer_gradient_to: value?.palette?.outerGradientTo,
                          inner_base: value?.palette?.innerBase,
                          inner_gradient_to: value?.palette?.innerGradientTo,
                      },
                  },
              ]),
          )
        : undefined;

    const modeBaselinesFromGovernance = governancePolicy.modeBaselines
        ? {
              philosophical: governancePolicy.modeBaselines.philosophical
                  ? {
                        gap_degrees: governancePolicy.modeBaselines.philosophical.gapDegrees,
                        stroke_width_delta_px:
                            governancePolicy.modeBaselines.philosophical.strokeWidthDeltaPx,
                        stroke_cap: governancePolicy.modeBaselines.philosophical.strokeCap,
                        force_gradient: governancePolicy.modeBaselines.philosophical.forceGradient,
                    }
                  : undefined,
              technical: governancePolicy.modeBaselines.technical
                  ? {
                        stroke_cap: governancePolicy.modeBaselines.technical.strokeCap,
                        force_gradient: governancePolicy.modeBaselines.technical.forceGradient,
                    }
                  : undefined,
          }
        : undefined;

    const hostDefaultsFromGovernance = governancePolicy.hostDefaults
        ? {
              default_temporal_state: governancePolicy.hostDefaults.defaultTemporalState,
              default_authority: governancePolicy.hostDefaults.defaultAuthority,
              theme_mode_map: governancePolicy.hostDefaults.themeModeMap,
          }
        : undefined;

    return {
        ...policy,
        style: {
            ...policy.style,
            allow_labs_customizations:
                governancePolicy.allowLabsCustomizations ?? policy.style.allow_labs_customizations,
            labs_bounds: {
                gap_min_degrees:
                    governancePolicy.labsBounds?.gapMinDegrees ?? policy.style.labs_bounds.gap_min_degrees,
                gap_max_degrees:
                    governancePolicy.labsBounds?.gapMaxDegrees ?? policy.style.labs_bounds.gap_max_degrees,
                stroke_min_px:
                    governancePolicy.labsBounds?.strokeMinPx ?? policy.style.labs_bounds.stroke_min_px,
                stroke_max_px:
                    governancePolicy.labsBounds?.strokeMaxPx ?? policy.style.labs_bounds.stroke_max_px,
            },
            official_palette: {
                outer_base: governancePolicy.officialPalette?.outerBase ?? policy.style.official_palette.outer_base,
                outer_gradient_to:
                    governancePolicy.officialPalette?.outerGradientTo ?? policy.style.official_palette.outer_gradient_to,
                inner_base: governancePolicy.officialPalette?.innerBase ?? policy.style.official_palette.inner_base,
                inner_gradient_to:
                    governancePolicy.officialPalette?.innerGradientTo ?? policy.style.official_palette.inner_gradient_to,
            },
            mode_baselines: mergeModeBaselines(
                policy.style.mode_baselines ?? DEFAULT_BRAND_POLICY_DOCUMENT.style.mode_baselines!,
                modeBaselinesFromGovernance,
            ),
            host_defaults: mergeHostDefaults(
                policy.style.host_defaults ?? DEFAULT_BRAND_POLICY_DOCUMENT.style.host_defaults!,
                hostDefaultsFromGovernance,
            ),
            temporal_variants: mergeTemporalVariants(
                policy.style.temporal_variants,
                temporalVariantsFromGovernance,
                policy.style.official_palette,
                (policy.style.mode_baselines ?? DEFAULT_BRAND_POLICY_DOCUMENT.style.mode_baselines!)
                    .technical,
            ),
            motion: {
                philosophical: {
                    container_transition_sec:
                        governancePolicy.motion?.philosophical?.containerTransitionSec ??
                        policy.style.motion.philosophical.container_transition_sec,
                    ring_transition_sec:
                        governancePolicy.motion?.philosophical?.ringTransitionSec ??
                        policy.style.motion.philosophical.ring_transition_sec,
                    stroke_transition_sec:
                        governancePolicy.motion?.philosophical?.strokeTransitionSec ??
                        policy.style.motion.philosophical.stroke_transition_sec,
                    ring_animation_duration_sec:
                        governancePolicy.motion?.philosophical?.ringAnimationDurationSec ??
                        policy.style.motion.philosophical.ring_animation_duration_sec,
                    ring_rotation_delta_deg:
                        governancePolicy.motion?.philosophical?.ringRotationDeltaDeg ??
                        policy.style.motion.philosophical.ring_rotation_delta_deg,
                    ring_stroke_delta_px:
                        governancePolicy.motion?.philosophical?.ringStrokeDeltaPx ??
                        policy.style.motion.philosophical.ring_stroke_delta_px,
                    dot_transition_sec:
                        governancePolicy.motion?.philosophical?.dotTransitionSec ??
                        policy.style.motion.philosophical.dot_transition_sec,
                    dot_animation_duration_sec:
                        governancePolicy.motion?.philosophical?.dotAnimationDurationSec ??
                        policy.style.motion.philosophical.dot_animation_duration_sec,
                    dot_pulse_radius_delta_px:
                        governancePolicy.motion?.philosophical?.dotPulseRadiusDeltaPx ??
                        policy.style.motion.philosophical.dot_pulse_radius_delta_px,
                    dot_pulse_opacity_min:
                        governancePolicy.motion?.philosophical?.dotPulseOpacityMin ??
                        policy.style.motion.philosophical.dot_pulse_opacity_min,
                },
                technical: {
                    container_transition_ms:
                        governancePolicy.motion?.technical?.containerTransitionMs ??
                        policy.style.motion.technical.container_transition_ms,
                    ring_transition_ms:
                        governancePolicy.motion?.technical?.ringTransitionMs ??
                        policy.style.motion.technical.ring_transition_ms,
                    stroke_transition_ms:
                        governancePolicy.motion?.technical?.strokeTransitionMs ??
                        policy.style.motion.technical.stroke_transition_ms,
                    dot_transition_ms:
                        governancePolicy.motion?.technical?.dotTransitionMs ??
                        policy.style.motion.technical.dot_transition_ms,
                    step_count:
                        governancePolicy.motion?.technical?.stepCount ??
                        policy.style.motion.technical.step_count,
                    ring_step_count:
                        governancePolicy.motion?.technical?.ringStepCount ??
                        policy.style.motion.technical.ring_step_count,
                },
            },
        },
    };
}

function baseStateForMode(mode: "philosophical" | "technical", policy: BrandPolicyDocument) {
    const modeBaselines =
        policy.style.mode_baselines ?? DEFAULT_BRAND_POLICY_DOCUMENT.style.mode_baselines!;

    if (mode === "technical") {
        return {
            isGradient: modeBaselines.technical.force_gradient,
            strokeCap: modeBaselines.technical.stroke_cap as "round" | "butt" | "square",
            gapAngle: policy.kernel.technical_canonical_gap_degrees,
            strokeWidth: policy.kernel.base_stroke_width_px,
        };
    }

    return {
        isGradient: modeBaselines.philosophical.force_gradient,
        strokeCap: modeBaselines.philosophical.stroke_cap as "round" | "butt" | "square",
        gapAngle: modeBaselines.philosophical.gap_degrees,
        strokeWidth: policy.kernel.base_stroke_width_px + modeBaselines.philosophical.stroke_width_delta_px,
    };
}

export function resolveBrandVisualState(props: BrandLogoProps = {}): ResolvedBrandVisualState {
    const policy = mergePolicy(props.policyDocument, props.governancePolicy);
    const requestedMode = props.mode ?? "philosophical";
    const effectiveMode = requestedMode === "custom" ? "technical" : requestedMode;
    const authority = props.authority ?? "official";
    const temporal = (props.temporal ?? "none") as TemporalState;
    const modeBaselines =
        policy.style.mode_baselines ?? DEFAULT_BRAND_POLICY_DOCUMENT.style.mode_baselines!;

    const base = baseStateForMode(effectiveMode, policy);

    let isGradient = base.isGradient;
    let strokeCap: "round" | "butt" | "square" = base.strokeCap;
    let gapAngle = base.gapAngle;
    let strokeWidth = base.strokeWidth;

    let outerBase = policy.style.official_palette.outer_base;
    let outerGradientTo = policy.style.official_palette.outer_gradient_to;
    let innerBase = policy.style.official_palette.inner_base;
    let innerGradientTo = policy.style.official_palette.inner_gradient_to;

    const labsAllowed =
        policy.style.allow_labs_customizations && authority === "labs" && temporal === "none";

    if (labsAllowed) {
        if (props.customOuterColor) outerBase = props.customOuterColor;
        if (props.customInnerColor) innerBase = props.customInnerColor;
        if (props.customGapAngle !== undefined) {
            gapAngle = clamp(
                props.customGapAngle,
                policy.style.labs_bounds.gap_min_degrees,
                policy.style.labs_bounds.gap_max_degrees,
            );
        }
        if (props.customStrokeWidth !== undefined) {
            strokeWidth = clamp(
                props.customStrokeWidth,
                policy.style.labs_bounds.stroke_min_px,
                policy.style.labs_bounds.stroke_max_px,
            );
        }

        if (requestedMode === "custom") {
            isGradient = modeBaselines.technical.force_gradient;
            strokeCap = modeBaselines.technical.stroke_cap as "round" | "butt" | "square";
        }
    }

    if (temporal !== "none") {
        const temporalPolicy = policy.style.temporal_variants[temporal];
        if (temporalPolicy) {
            isGradient = temporalPolicy.force_gradient;
            strokeCap = temporalPolicy.stroke_cap as "round" | "butt" | "square";
            outerBase = temporalPolicy.palette.outer_base;
            outerGradientTo = temporalPolicy.palette.outer_gradient_to;
            innerBase = temporalPolicy.palette.inner_base;
            innerGradientTo = temporalPolicy.palette.inner_gradient_to;
        }
    }

    return {
        requestedMode,
        effectiveMode,
        authority,
        temporal,
        isGradient,
        strokeCap,
        gapAngle,
        strokeWidth,
        outerBase,
        outerGradientTo,
        innerBase,
        innerGradientTo,
        ringRadius: policy.kernel.ring_radius_px,
        dotRadius: policy.kernel.dot_radius_px,
        motion: {
            philosophical: {
                containerTransitionSec: policy.style.motion.philosophical.container_transition_sec,
                ringTransitionSec: policy.style.motion.philosophical.ring_transition_sec,
                strokeTransitionSec: policy.style.motion.philosophical.stroke_transition_sec,
                ringAnimationDurationSec: policy.style.motion.philosophical.ring_animation_duration_sec,
                ringRotationDeltaDeg: policy.style.motion.philosophical.ring_rotation_delta_deg,
                ringStrokeDeltaPx: policy.style.motion.philosophical.ring_stroke_delta_px,
                dotTransitionSec: policy.style.motion.philosophical.dot_transition_sec,
                dotAnimationDurationSec: policy.style.motion.philosophical.dot_animation_duration_sec,
                dotPulseRadiusDeltaPx: policy.style.motion.philosophical.dot_pulse_radius_delta_px,
                dotPulseOpacityMin: policy.style.motion.philosophical.dot_pulse_opacity_min,
            },
            technical: {
                containerTransitionMs: policy.style.motion.technical.container_transition_ms,
                ringTransitionMs: policy.style.motion.technical.ring_transition_ms,
                strokeTransitionMs: policy.style.motion.technical.stroke_transition_ms,
                dotTransitionMs: policy.style.motion.technical.dot_transition_ms,
                stepCount: policy.style.motion.technical.step_count,
                ringStepCount: policy.style.motion.technical.ring_step_count,
            },
        },
    };
}
