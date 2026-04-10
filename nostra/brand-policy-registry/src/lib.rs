use candid::CandidType;
use ic_cdk_macros::query;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::sync::OnceLock;

const POLICY_JSON: &str = include_str!("../../../shared/standards/branding/brand_policy_v1.json");

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct BrandPalette {
    pub outer_base: String,
    pub outer_gradient_to: String,
    pub inner_base: String,
    pub inner_gradient_to: String,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct LabsBounds {
    pub gap_min_degrees: f64,
    pub gap_max_degrees: f64,
    pub stroke_min_px: f64,
    pub stroke_max_px: f64,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct TemporalVariantPolicy {
    pub force_gradient: bool,
    pub stroke_cap: String,
    pub palette: BrandPalette,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
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

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct TechnicalMotionPolicy {
    pub container_transition_ms: u64,
    pub ring_transition_ms: u64,
    pub stroke_transition_ms: u64,
    pub dot_transition_ms: u64,
    pub step_count: u64,
    pub ring_step_count: u64,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct BrandMotionPolicy {
    pub philosophical: PhilosophicalMotionPolicy,
    pub technical: TechnicalMotionPolicy,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct PhilosophicalModeBaselinePolicy {
    pub gap_degrees: f64,
    pub stroke_width_delta_px: f64,
    pub stroke_cap: String,
    pub force_gradient: bool,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct TechnicalModeBaselinePolicy {
    pub stroke_cap: String,
    pub force_gradient: bool,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct BrandModeBaselinesPolicy {
    pub philosophical: PhilosophicalModeBaselinePolicy,
    pub technical: TechnicalModeBaselinePolicy,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct BrandHostDefaultsPolicy {
    pub default_temporal_state: String,
    pub default_authority: String,
    pub theme_mode_map: BTreeMap<String, String>,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct BrandStylePolicy {
    pub allow_labs_customizations: bool,
    pub labs_bounds: LabsBounds,
    pub official_palette: BrandPalette,
    pub mode_baselines: Option<BrandModeBaselinesPolicy>,
    pub host_defaults: Option<BrandHostDefaultsPolicy>,
    pub temporal_variants: BTreeMap<String, TemporalVariantPolicy>,
    pub motion: BrandMotionPolicy,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct BrandKernelPolicy {
    pub mark_composition: String,
    pub technical_canonical_gap_degrees: u64,
    pub ring_radius_px: u64,
    pub dot_radius_px: u64,
    pub base_stroke_width_px: u64,
    pub steward_gated: bool,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct TemporalWindow {
    pub state: String,
    pub recurrence: String,
    pub start_month_day: String,
    pub end_month_day: String,
    pub start_time_utc: String,
    pub end_time_utc: String,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
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

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct BrandPolicyMeta {
    pub schema_version: String,
    pub policy_id: String,
    pub policy_version: u64,
    pub updated_at_ns: u64,
    pub source_ref: Option<String>,
}

static POLICY_CACHE: OnceLock<BrandPolicyDocument> = OnceLock::new();
static DIGEST_CACHE: OnceLock<String> = OnceLock::new();

fn parsed_policy() -> &'static BrandPolicyDocument {
    POLICY_CACHE.get_or_init(|| {
        serde_json::from_str::<BrandPolicyDocument>(POLICY_JSON)
            .unwrap_or_else(|err| ic_cdk::trap(&format!("invalid embedded brand policy: {err}")))
    })
}

fn policy_digest() -> &'static String {
    DIGEST_CACHE.get_or_init(|| {
        let mut hasher = Sha256::new();
        hasher.update(POLICY_JSON.as_bytes());
        hex::encode(hasher.finalize())
    })
}

#[query]
fn get_brand_policy() -> BrandPolicyDocument {
    parsed_policy().clone()
}

#[query]
fn get_brand_policy_version() -> u64 {
    parsed_policy().policy_version
}

#[query]
fn get_brand_policy_digest() -> String {
    policy_digest().clone()
}

#[query]
fn get_brand_policy_meta() -> BrandPolicyMeta {
    let policy = parsed_policy();
    BrandPolicyMeta {
        schema_version: policy.schema_version.clone(),
        policy_id: policy.policy_id.clone(),
        policy_version: policy.policy_version,
        updated_at_ns: policy.updated_at_ns,
        source_ref: policy.source_ref.clone(),
    }
}

ic_cdk::export_candid!();
