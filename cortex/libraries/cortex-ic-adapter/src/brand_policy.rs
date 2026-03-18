use crate::dfx::resolve_canister_id_any;
use candid::{CandidType, Decode, Principal};
use cortex_domain::brand::policy::{
    BrandHostDefaultsPolicy, BrandKernelPolicy, BrandModeBaselinesPolicy, BrandMotionPolicy,
    BrandPalette, BrandPolicyDocument, BrandPolicyMeta, BrandStylePolicy, LabsBounds,
    PhilosophicalModeBaselinePolicy, PhilosophicalMotionPolicy, TechnicalModeBaselinePolicy,
    TechnicalMotionPolicy, TemporalVariantPolicy, TemporalWindow,
};
use cortex_runtime::RuntimeError;
use ic_agent::identity::AnonymousIdentity;
use ic_agent::Agent;
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
struct BrandPolicyDocumentCandid {
    schema_version: String,
    policy_id: String,
    policy_version: u64,
    kernel: BrandKernelPolicyCandid,
    style: BrandStylePolicyCandid,
    temporal_windows: Vec<TemporalWindowCandid>,
    updated_at_ns: u64,
    source_ref: Option<String>,
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
struct BrandPolicyMetaCandid {
    schema_version: String,
    policy_id: String,
    policy_version: u64,
    updated_at_ns: u64,
    source_ref: Option<String>,
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
struct BrandKernelPolicyCandid {
    mark_composition: String,
    technical_canonical_gap_degrees: u64,
    ring_radius_px: u64,
    dot_radius_px: u64,
    base_stroke_width_px: u64,
    steward_gated: bool,
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
struct BrandStylePolicyCandid {
    allow_labs_customizations: bool,
    labs_bounds: LabsBoundsCandid,
    official_palette: BrandPaletteCandid,
    mode_baselines: Option<BrandModeBaselinesPolicyCandid>,
    host_defaults: Option<BrandHostDefaultsPolicyCandid>,
    temporal_variants: Vec<TemporalVariantEntryCandid>,
    motion: BrandMotionPolicyCandid,
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
struct BrandModeBaselinesPolicyCandid {
    philosophical: PhilosophicalModeBaselinePolicyCandid,
    technical: TechnicalModeBaselinePolicyCandid,
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
struct PhilosophicalModeBaselinePolicyCandid {
    gap_degrees: f64,
    stroke_width_delta_px: f64,
    stroke_cap: String,
    force_gradient: bool,
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
struct TechnicalModeBaselinePolicyCandid {
    stroke_cap: String,
    force_gradient: bool,
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
struct BrandHostDefaultsPolicyCandid {
    default_temporal_state: String,
    default_authority: String,
    theme_mode_map: Vec<ThemeModeMapEntryCandid>,
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
struct ThemeModeMapEntryCandid {
    key: String,
    value: String,
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
struct TemporalVariantEntryCandid {
    key: String,
    value: TemporalVariantPolicyCandid,
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
struct LabsBoundsCandid {
    gap_min_degrees: f64,
    gap_max_degrees: f64,
    stroke_min_px: f64,
    stroke_max_px: f64,
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
struct BrandPaletteCandid {
    outer_base: String,
    outer_gradient_to: String,
    inner_base: String,
    inner_gradient_to: String,
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
struct TemporalVariantPolicyCandid {
    force_gradient: bool,
    stroke_cap: String,
    palette: BrandPaletteCandid,
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
struct BrandMotionPolicyCandid {
    philosophical: PhilosophicalMotionPolicyCandid,
    technical: TechnicalMotionPolicyCandid,
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
struct PhilosophicalMotionPolicyCandid {
    container_transition_sec: f64,
    ring_transition_sec: f64,
    stroke_transition_sec: f64,
    ring_animation_duration_sec: f64,
    ring_rotation_delta_deg: f64,
    ring_stroke_delta_px: f64,
    dot_transition_sec: f64,
    dot_animation_duration_sec: f64,
    dot_pulse_radius_delta_px: f64,
    dot_pulse_opacity_min: f64,
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
struct TechnicalMotionPolicyCandid {
    container_transition_ms: u64,
    ring_transition_ms: u64,
    stroke_transition_ms: u64,
    dot_transition_ms: u64,
    step_count: u64,
    ring_step_count: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
struct TemporalWindowCandid {
    state: String,
    recurrence: String,
    start_month_day: String,
    end_month_day: String,
    start_time_utc: String,
    end_time_utc: String,
}

#[derive(Clone, Debug)]
pub struct BrandPolicyCanisterClient {
    host: String,
    canister_id: Principal,
}

impl BrandPolicyCanisterClient {
    pub async fn from_env() -> Result<Self, RuntimeError> {
        let host = std::env::var("NOSTRA_IC_HOST")
            .or_else(|_| std::env::var("IC_HOST"))
            .unwrap_or_else(|_| "http://127.0.0.1:4943".to_string());
        let canister_id_text = resolve_canister_id_any(
            &["CANISTER_ID_BRAND_POLICY_REGISTRY"],
            "brand_policy_registry",
        )
        .await
        .map_err(RuntimeError::Network)?;
        let canister_id = Principal::from_text(canister_id_text).map_err(|err| {
            RuntimeError::Network(format!("invalid brand_policy_registry principal: {err}"))
        })?;
        Ok(Self { host, canister_id })
    }

    async fn agent(&self) -> Result<Agent, RuntimeError> {
        let agent = Agent::builder()
            .with_url(self.host.clone())
            .with_identity(AnonymousIdentity)
            .build()
            .map_err(|err| RuntimeError::Network(format!("failed to build ic-agent: {err}")))?;

        if self.host.contains("127.0.0.1") || self.host.contains("localhost") {
            agent
                .fetch_root_key()
                .await
                .map_err(|err| RuntimeError::Network(format!("failed to fetch root key: {err}")))?;
        }

        Ok(agent)
    }

    pub async fn get_brand_policy(&self) -> Result<BrandPolicyDocument, RuntimeError> {
        let agent = self.agent().await?;
        let bytes = agent
            .query(&self.canister_id, "get_brand_policy")
            .call()
            .await
            .map_err(|err| RuntimeError::Network(format!("query failed: {err}")))?;
        let decoded = Decode!(&bytes, BrandPolicyDocumentCandid)
            .map_err(|err| RuntimeError::Serialization(format!("decode failed: {err}")))?;
        Ok(map_brand_policy_document(decoded))
    }

    pub async fn get_brand_policy_version(&self) -> Result<u64, RuntimeError> {
        let agent = self.agent().await?;
        let bytes = agent
            .query(&self.canister_id, "get_brand_policy_version")
            .call()
            .await
            .map_err(|err| RuntimeError::Network(format!("query failed: {err}")))?;
        Decode!(&bytes, u64)
            .map_err(|err| RuntimeError::Serialization(format!("decode failed: {err}")))
    }

    pub async fn get_brand_policy_digest(&self) -> Result<String, RuntimeError> {
        let agent = self.agent().await?;
        let bytes = agent
            .query(&self.canister_id, "get_brand_policy_digest")
            .call()
            .await
            .map_err(|err| RuntimeError::Network(format!("query failed: {err}")))?;
        Decode!(&bytes, String)
            .map_err(|err| RuntimeError::Serialization(format!("decode failed: {err}")))
    }

    pub async fn get_brand_policy_meta(&self) -> Result<BrandPolicyMeta, RuntimeError> {
        let agent = self.agent().await?;
        let bytes = agent
            .query(&self.canister_id, "get_brand_policy_meta")
            .call()
            .await
            .map_err(|err| RuntimeError::Network(format!("query failed: {err}")))?;
        let decoded = Decode!(&bytes, BrandPolicyMetaCandid)
            .map_err(|err| RuntimeError::Serialization(format!("decode failed: {err}")))?;
        Ok(BrandPolicyMeta {
            schema_version: decoded.schema_version,
            policy_id: decoded.policy_id,
            policy_version: decoded.policy_version,
            updated_at_ns: decoded.updated_at_ns,
            source_ref: decoded.source_ref,
        })
    }
}

fn map_brand_policy_document(value: BrandPolicyDocumentCandid) -> BrandPolicyDocument {
    BrandPolicyDocument {
        schema_version: value.schema_version,
        policy_id: value.policy_id,
        policy_version: value.policy_version,
        kernel: BrandKernelPolicy {
            mark_composition: value.kernel.mark_composition,
            technical_canonical_gap_degrees: value.kernel.technical_canonical_gap_degrees,
            ring_radius_px: value.kernel.ring_radius_px,
            dot_radius_px: value.kernel.dot_radius_px,
            base_stroke_width_px: value.kernel.base_stroke_width_px,
            steward_gated: value.kernel.steward_gated,
        },
        style: BrandStylePolicy {
            allow_labs_customizations: value.style.allow_labs_customizations,
            labs_bounds: LabsBounds {
                gap_min_degrees: value.style.labs_bounds.gap_min_degrees,
                gap_max_degrees: value.style.labs_bounds.gap_max_degrees,
                stroke_min_px: value.style.labs_bounds.stroke_min_px,
                stroke_max_px: value.style.labs_bounds.stroke_max_px,
            },
            official_palette: map_palette(value.style.official_palette),
            mode_baselines: value.style.mode_baselines.map(|mode_baselines| {
                BrandModeBaselinesPolicy {
                    philosophical: PhilosophicalModeBaselinePolicy {
                        gap_degrees: mode_baselines.philosophical.gap_degrees,
                        stroke_width_delta_px: mode_baselines.philosophical.stroke_width_delta_px,
                        stroke_cap: mode_baselines.philosophical.stroke_cap,
                        force_gradient: mode_baselines.philosophical.force_gradient,
                    },
                    technical: TechnicalModeBaselinePolicy {
                        stroke_cap: mode_baselines.technical.stroke_cap,
                        force_gradient: mode_baselines.technical.force_gradient,
                    },
                }
            }),
            host_defaults: value
                .style
                .host_defaults
                .map(|host_defaults| BrandHostDefaultsPolicy {
                    default_temporal_state: host_defaults.default_temporal_state,
                    default_authority: host_defaults.default_authority,
                    theme_mode_map: host_defaults
                        .theme_mode_map
                        .into_iter()
                        .map(|entry| (entry.key, entry.value))
                        .collect(),
                }),
            temporal_variants: value
                .style
                .temporal_variants
                .into_iter()
                .map(|entry| {
                    (
                        entry.key,
                        TemporalVariantPolicy {
                            force_gradient: entry.value.force_gradient,
                            stroke_cap: entry.value.stroke_cap,
                            palette: map_palette(entry.value.palette),
                        },
                    )
                })
                .collect::<BTreeMap<_, _>>(),
            motion: BrandMotionPolicy {
                philosophical: PhilosophicalMotionPolicy {
                    container_transition_sec: value
                        .style
                        .motion
                        .philosophical
                        .container_transition_sec,
                    ring_transition_sec: value.style.motion.philosophical.ring_transition_sec,
                    stroke_transition_sec: value.style.motion.philosophical.stroke_transition_sec,
                    ring_animation_duration_sec: value
                        .style
                        .motion
                        .philosophical
                        .ring_animation_duration_sec,
                    ring_rotation_delta_deg: value
                        .style
                        .motion
                        .philosophical
                        .ring_rotation_delta_deg,
                    ring_stroke_delta_px: value.style.motion.philosophical.ring_stroke_delta_px,
                    dot_transition_sec: value.style.motion.philosophical.dot_transition_sec,
                    dot_animation_duration_sec: value
                        .style
                        .motion
                        .philosophical
                        .dot_animation_duration_sec,
                    dot_pulse_radius_delta_px: value
                        .style
                        .motion
                        .philosophical
                        .dot_pulse_radius_delta_px,
                    dot_pulse_opacity_min: value.style.motion.philosophical.dot_pulse_opacity_min,
                },
                technical: TechnicalMotionPolicy {
                    container_transition_ms: value.style.motion.technical.container_transition_ms,
                    ring_transition_ms: value.style.motion.technical.ring_transition_ms,
                    stroke_transition_ms: value.style.motion.technical.stroke_transition_ms,
                    dot_transition_ms: value.style.motion.technical.dot_transition_ms,
                    step_count: value.style.motion.technical.step_count,
                    ring_step_count: value.style.motion.technical.ring_step_count,
                },
            },
        },
        temporal_windows: value
            .temporal_windows
            .into_iter()
            .map(|window| TemporalWindow {
                state: window.state,
                recurrence: window.recurrence,
                start_month_day: window.start_month_day,
                end_month_day: window.end_month_day,
                start_time_utc: window.start_time_utc,
                end_time_utc: window.end_time_utc,
            })
            .collect(),
        updated_at_ns: value.updated_at_ns,
        source_ref: value.source_ref,
    }
}

fn map_palette(value: BrandPaletteCandid) -> BrandPalette {
    BrandPalette {
        outer_base: value.outer_base,
        outer_gradient_to: value.outer_gradient_to,
        inner_base: value.inner_base,
        inner_gradient_to: value.inner_gradient_to,
    }
}
