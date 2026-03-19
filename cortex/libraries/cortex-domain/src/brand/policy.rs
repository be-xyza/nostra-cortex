use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const DEFAULT_PHILOSOPHICAL_GAP_DEGREES: f64 = 60.0;
pub const DEFAULT_PHILOSOPHICAL_STROKE_DELTA_PX: f64 = 2.0;
pub const DEFAULT_PHILOSOPHICAL_STROKE_CAP: &str = "round";
pub const DEFAULT_PHILOSOPHICAL_FORCE_GRADIENT: bool = true;
pub const DEFAULT_TECHNICAL_STROKE_CAP: &str = "square";
pub const DEFAULT_TECHNICAL_FORCE_GRADIENT: bool = false;
pub const DEFAULT_TEMPORAL_STATE: &str = "none";
pub const DEFAULT_AUTHORITY_STATE: &str = "official";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct BrandPalette {
    pub outer_base: String,
    pub outer_gradient_to: String,
    pub inner_base: String,
    pub inner_gradient_to: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct LabsBounds {
    pub gap_min_degrees: f64,
    pub gap_max_degrees: f64,
    pub stroke_min_px: f64,
    pub stroke_max_px: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct TemporalVariantPolicy {
    pub force_gradient: bool,
    pub stroke_cap: String,
    pub palette: BrandPalette,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct PhilosophicalMotionPolicy {
    pub container_transition_sec: f64,
    pub ring_transition_sec: f64,
    pub stroke_transition_sec: f64,
    pub ring_animation_duration_sec: f64,
    pub ring_rotation_delta_deg: f64,
    pub ring_stroke_delta_px: f64,
    pub dot_transition_sec: f64,
    pub dot_animation_duration_sec: f64,
    pub dot_pulse_radius_delta_px: f64,
    pub dot_pulse_opacity_min: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct TechnicalMotionPolicy {
    pub container_transition_ms: u64,
    pub ring_transition_ms: u64,
    pub stroke_transition_ms: u64,
    pub dot_transition_ms: u64,
    pub step_count: u64,
    pub ring_step_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct BrandMotionPolicy {
    pub philosophical: PhilosophicalMotionPolicy,
    pub technical: TechnicalMotionPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct PhilosophicalModeBaselinePolicy {
    pub gap_degrees: f64,
    pub stroke_width_delta_px: f64,
    pub stroke_cap: String,
    pub force_gradient: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct TechnicalModeBaselinePolicy {
    pub stroke_cap: String,
    pub force_gradient: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct BrandModeBaselinesPolicy {
    pub philosophical: PhilosophicalModeBaselinePolicy,
    pub technical: TechnicalModeBaselinePolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct BrandHostDefaultsPolicy {
    pub default_temporal_state: String,
    pub default_authority: String,
    pub theme_mode_map: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct BrandStylePolicy {
    pub allow_labs_customizations: bool,
    pub labs_bounds: LabsBounds,
    pub official_palette: BrandPalette,
    #[serde(default)]
    pub mode_baselines: Option<BrandModeBaselinesPolicy>,
    #[serde(default)]
    pub host_defaults: Option<BrandHostDefaultsPolicy>,
    pub temporal_variants: BTreeMap<String, TemporalVariantPolicy>,
    pub motion: BrandMotionPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct BrandKernelPolicy {
    pub mark_composition: String,
    pub technical_canonical_gap_degrees: u64,
    pub ring_radius_px: u64,
    pub dot_radius_px: u64,
    pub base_stroke_width_px: u64,
    pub steward_gated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct TemporalWindow {
    pub state: String,
    pub recurrence: String,
    pub start_month_day: String,
    pub end_month_day: String,
    pub start_time_utc: String,
    pub end_time_utc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct BrandPolicyDocument {
    pub schema_version: String,
    pub policy_id: String,
    pub policy_version: u64,
    pub kernel: BrandKernelPolicy,
    pub style: BrandStylePolicy,
    pub temporal_windows: Vec<TemporalWindow>,
    pub updated_at_ns: u64,
    pub source_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct BrandPolicyMeta {
    pub schema_version: String,
    pub policy_id: String,
    pub policy_version: u64,
    pub updated_at_ns: u64,
    pub source_ref: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BrandMode {
    Philosophical,
    Technical,
    Custom,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuthorityState {
    Official,
    Labs,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct BrandVisualInput {
    pub mode: BrandMode,
    pub authority: AuthorityState,
    pub temporal: Option<String>,
    pub custom_outer_color: Option<String>,
    pub custom_inner_color: Option<String>,
    pub custom_gap_angle: Option<f64>,
    pub custom_stroke_width: Option<f64>,
}

impl Default for BrandVisualInput {
    fn default() -> Self {
        Self {
            mode: BrandMode::Philosophical,
            authority: AuthorityState::Official,
            temporal: Some("none".to_string()),
            custom_outer_color: None,
            custom_inner_color: None,
            custom_gap_angle: None,
            custom_stroke_width: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct ResolvedBrandVisualState {
    pub requested_mode: BrandMode,
    pub effective_mode: BrandMode,
    pub authority: AuthorityState,
    pub temporal: String,
    pub is_gradient: bool,
    pub stroke_cap: String,
    pub gap_angle: f64,
    pub stroke_width: f64,
    pub outer_base: String,
    pub outer_gradient_to: String,
    pub inner_base: String,
    pub inner_gradient_to: String,
    pub ring_radius: f64,
    pub dot_radius: f64,
}

fn clamp(value: f64, min: f64, max: f64) -> f64 {
    value.max(min).min(max)
}

pub fn default_mode_baselines() -> BrandModeBaselinesPolicy {
    BrandModeBaselinesPolicy {
        philosophical: PhilosophicalModeBaselinePolicy {
            gap_degrees: DEFAULT_PHILOSOPHICAL_GAP_DEGREES,
            stroke_width_delta_px: DEFAULT_PHILOSOPHICAL_STROKE_DELTA_PX,
            stroke_cap: DEFAULT_PHILOSOPHICAL_STROKE_CAP.to_string(),
            force_gradient: DEFAULT_PHILOSOPHICAL_FORCE_GRADIENT,
        },
        technical: TechnicalModeBaselinePolicy {
            stroke_cap: DEFAULT_TECHNICAL_STROKE_CAP.to_string(),
            force_gradient: DEFAULT_TECHNICAL_FORCE_GRADIENT,
        },
    }
}

pub fn default_theme_mode_map() -> BTreeMap<String, String> {
    let mut value = BTreeMap::new();
    value.insert("nostra".to_string(), "philosophical".to_string());
    value.insert("cortex".to_string(), "technical".to_string());
    value
}

pub fn default_host_defaults() -> BrandHostDefaultsPolicy {
    BrandHostDefaultsPolicy {
        default_temporal_state: DEFAULT_TEMPORAL_STATE.to_string(),
        default_authority: DEFAULT_AUTHORITY_STATE.to_string(),
        theme_mode_map: default_theme_mode_map(),
    }
}

pub fn normalize_brand_policy_document(
    policy: &BrandPolicyDocument,
) -> (BrandPolicyDocument, bool) {
    let mut normalized = policy.clone();
    let mut defaults_applied = false;

    if normalized.style.mode_baselines.is_none() {
        normalized.style.mode_baselines = Some(default_mode_baselines());
        defaults_applied = true;
    }

    if normalized.style.host_defaults.is_none() {
        normalized.style.host_defaults = Some(default_host_defaults());
        defaults_applied = true;
    }

    (normalized, defaults_applied)
}

pub fn resolve_brand_visual_state(
    policy: &BrandPolicyDocument,
    input: &BrandVisualInput,
) -> ResolvedBrandVisualState {
    let (normalized_policy, _) = normalize_brand_policy_document(policy);
    let policy = &normalized_policy;
    let requested_mode = input.mode;
    let effective_mode = match requested_mode {
        BrandMode::Custom => BrandMode::Technical,
        other => other,
    };
    let temporal = input
        .temporal
        .clone()
        .unwrap_or_else(|| "none".to_string())
        .to_ascii_lowercase();

    let mode_baselines = policy
        .style
        .mode_baselines
        .as_ref()
        .expect("normalized brand policy must include mode baselines");
    let (mut is_gradient, mut stroke_cap, mut gap_angle, mut stroke_width) = match effective_mode {
        BrandMode::Technical => (
            mode_baselines.technical.force_gradient,
            mode_baselines.technical.stroke_cap.clone(),
            policy.kernel.technical_canonical_gap_degrees as f64,
            policy.kernel.base_stroke_width_px as f64,
        ),
        BrandMode::Philosophical => (
            mode_baselines.philosophical.force_gradient,
            mode_baselines.philosophical.stroke_cap.clone(),
            mode_baselines.philosophical.gap_degrees,
            policy.kernel.base_stroke_width_px as f64
                + mode_baselines.philosophical.stroke_width_delta_px,
        ),
        BrandMode::Custom => unreachable!("custom mode is normalized to technical"),
    };

    let mut outer_base = policy.style.official_palette.outer_base.clone();
    let mut outer_gradient_to = policy.style.official_palette.outer_gradient_to.clone();
    let mut inner_base = policy.style.official_palette.inner_base.clone();
    let mut inner_gradient_to = policy.style.official_palette.inner_gradient_to.clone();

    let labs_allowed = policy.style.allow_labs_customizations
        && input.authority == AuthorityState::Labs
        && temporal == "none";

    if labs_allowed {
        if let Some(value) = &input.custom_outer_color {
            outer_base = value.clone();
        }
        if let Some(value) = &input.custom_inner_color {
            inner_base = value.clone();
        }
        if let Some(value) = input.custom_gap_angle {
            gap_angle = clamp(
                value,
                policy.style.labs_bounds.gap_min_degrees,
                policy.style.labs_bounds.gap_max_degrees,
            );
        }
        if let Some(value) = input.custom_stroke_width {
            stroke_width = clamp(
                value,
                policy.style.labs_bounds.stroke_min_px,
                policy.style.labs_bounds.stroke_max_px,
            );
        }
        if requested_mode == BrandMode::Custom {
            is_gradient = mode_baselines.technical.force_gradient;
            stroke_cap = mode_baselines.technical.stroke_cap.clone();
        }
    }

    if temporal != "none" {
        if let Some(variant) = policy.style.temporal_variants.get(&temporal) {
            is_gradient = variant.force_gradient;
            stroke_cap = variant.stroke_cap.clone();
            outer_base = variant.palette.outer_base.clone();
            outer_gradient_to = variant.palette.outer_gradient_to.clone();
            inner_base = variant.palette.inner_base.clone();
            inner_gradient_to = variant.palette.inner_gradient_to.clone();
        }
    }

    ResolvedBrandVisualState {
        requested_mode,
        effective_mode,
        authority: input.authority,
        temporal,
        is_gradient,
        stroke_cap,
        gap_angle,
        stroke_width,
        outer_base,
        outer_gradient_to,
        inner_base,
        inner_gradient_to,
        ring_radius: policy.kernel.ring_radius_px as f64,
        dot_radius: policy.kernel.dot_radius_px as f64,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    const BRAND_POLICY_JSON: &str =
        include_str!("../../../../../shared/standards/branding/brand_policy_v1.json");
    const CASES_JSON: &str =
        include_str!("../../../../../shared/standards/branding/brand_visual_state_cases_v1.json");

    #[derive(Debug, Deserialize)]
    struct CaseSet {
        cases: Vec<CaseFixture>,
    }

    #[derive(Debug, Deserialize)]
    struct CaseFixture {
        id: String,
        input: CaseInput,
        expected: CaseExpected,
    }

    #[derive(Debug, Deserialize)]
    struct CaseInput {
        mode: BrandMode,
        authority: AuthorityState,
        temporal: Option<String>,
        custom_outer_color: Option<String>,
        custom_inner_color: Option<String>,
        custom_gap_angle: Option<f64>,
        custom_stroke_width: Option<f64>,
        governance_policy: Option<GovernancePolicyOverride>,
    }

    #[derive(Debug, Deserialize)]
    struct GovernancePolicyOverride {
        allow_labs_customizations: Option<bool>,
        mode_baselines: Option<ModeBaselinesOverride>,
    }

    #[derive(Debug, Deserialize)]
    struct ModeBaselinesOverride {
        philosophical: Option<PhilosophicalModeBaselineOverride>,
        technical: Option<TechnicalModeBaselineOverride>,
    }

    #[derive(Debug, Deserialize)]
    struct PhilosophicalModeBaselineOverride {
        gap_degrees: Option<f64>,
        stroke_width_delta_px: Option<f64>,
        stroke_cap: Option<String>,
        force_gradient: Option<bool>,
    }

    #[derive(Debug, Deserialize)]
    struct TechnicalModeBaselineOverride {
        stroke_cap: Option<String>,
        force_gradient: Option<bool>,
    }

    #[derive(Debug, Deserialize)]
    struct CaseExpected {
        effective_mode: BrandMode,
        temporal: String,
        gap_angle: f64,
        stroke_width: f64,
        is_gradient: bool,
        stroke_cap: String,
        outer_base: String,
        inner_base: String,
    }

    #[test]
    fn resolver_matches_canonical_brand_vector_cases() {
        let base_policy: BrandPolicyDocument =
            serde_json::from_str(BRAND_POLICY_JSON).expect("valid brand policy fixture");
        let cases: CaseSet = serde_json::from_str(CASES_JSON).expect("valid case fixture");

        for case in cases.cases {
            let (mut policy, _) = normalize_brand_policy_document(&base_policy);
            if let Some(governance) = case.input.governance_policy {
                if let Some(allow) = governance.allow_labs_customizations {
                    policy.style.allow_labs_customizations = allow;
                }
                if let Some(mode_baselines) = governance.mode_baselines {
                    if let Some(ref mut policy_mode_baselines) = policy.style.mode_baselines {
                        if let Some(philosophical) = mode_baselines.philosophical {
                            if let Some(value) = philosophical.gap_degrees {
                                policy_mode_baselines.philosophical.gap_degrees = value;
                            }
                            if let Some(value) = philosophical.stroke_width_delta_px {
                                policy_mode_baselines.philosophical.stroke_width_delta_px = value;
                            }
                            if let Some(value) = philosophical.stroke_cap {
                                policy_mode_baselines.philosophical.stroke_cap = value;
                            }
                            if let Some(value) = philosophical.force_gradient {
                                policy_mode_baselines.philosophical.force_gradient = value;
                            }
                        }
                        if let Some(technical) = mode_baselines.technical {
                            if let Some(value) = technical.stroke_cap {
                                policy_mode_baselines.technical.stroke_cap = value;
                            }
                            if let Some(value) = technical.force_gradient {
                                policy_mode_baselines.technical.force_gradient = value;
                            }
                        }
                    }
                }
            }

            let resolved = resolve_brand_visual_state(
                &policy,
                &BrandVisualInput {
                    mode: case.input.mode,
                    authority: case.input.authority,
                    temporal: case.input.temporal,
                    custom_outer_color: case.input.custom_outer_color,
                    custom_inner_color: case.input.custom_inner_color,
                    custom_gap_angle: case.input.custom_gap_angle,
                    custom_stroke_width: case.input.custom_stroke_width,
                },
            );

            assert_eq!(
                resolved.effective_mode, case.expected.effective_mode,
                "effective_mode mismatch for {}",
                case.id
            );
            assert_eq!(
                resolved.temporal, case.expected.temporal,
                "temporal mismatch for {}",
                case.id
            );
            assert!(
                (resolved.gap_angle - case.expected.gap_angle).abs() < f64::EPSILON,
                "gap_angle mismatch for {}",
                case.id
            );
            assert!(
                (resolved.stroke_width - case.expected.stroke_width).abs() < f64::EPSILON,
                "stroke_width mismatch for {}",
                case.id
            );
            assert_eq!(
                resolved.is_gradient, case.expected.is_gradient,
                "is_gradient mismatch for {}",
                case.id
            );
            assert_eq!(
                resolved.stroke_cap, case.expected.stroke_cap,
                "stroke_cap mismatch for {}",
                case.id
            );
            assert_eq!(
                resolved.outer_base, case.expected.outer_base,
                "outer_base mismatch for {}",
                case.id
            );
            assert_eq!(
                resolved.inner_base, case.expected.inner_base,
                "inner_base mismatch for {}",
                case.id
            );
        }
    }
}
