import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface BrandPalette {
  'outer_base' : string,
  'outer_gradient_to' : string,
  'inner_base' : string,
  'inner_gradient_to' : string,
}
export interface LabsBounds {
  'gap_min_degrees' : number,
  'gap_max_degrees' : number,
  'stroke_min_px' : number,
  'stroke_max_px' : number,
}
export interface TemporalVariantPolicy {
  'force_gradient' : boolean,
  'stroke_cap' : string,
  'palette' : BrandPalette,
}
export interface PhilosophicalMotionPolicy {
  'container_transition_sec' : number,
  'ring_transition_sec' : number,
  'stroke_transition_sec' : number,
  'ring_animation_duration_sec' : number,
  'ring_rotation_delta_deg' : number,
  'ring_stroke_delta_px' : number,
  'dot_transition_sec' : number,
  'dot_animation_duration_sec' : number,
  'dot_pulse_radius_delta_px' : number,
  'dot_pulse_opacity_min' : number,
}
export interface TechnicalMotionPolicy {
  'container_transition_ms' : bigint,
  'ring_transition_ms' : bigint,
  'stroke_transition_ms' : bigint,
  'dot_transition_ms' : bigint,
  'step_count' : bigint,
  'ring_step_count' : bigint,
}
export interface BrandMotionPolicy {
  'philosophical' : PhilosophicalMotionPolicy,
  'technical' : TechnicalMotionPolicy,
}
export interface PhilosophicalModeBaselinePolicy {
  'gap_degrees' : number,
  'stroke_width_delta_px' : number,
  'stroke_cap' : string,
  'force_gradient' : boolean,
}
export interface TechnicalModeBaselinePolicy {
  'stroke_cap' : string,
  'force_gradient' : boolean,
}
export interface BrandModeBaselinesPolicy {
  'philosophical' : PhilosophicalModeBaselinePolicy,
  'technical' : TechnicalModeBaselinePolicy,
}
export interface ThemeModeMapEntry { 'key' : string, 'value' : string }
export interface BrandHostDefaultsPolicy {
  'default_temporal_state' : string,
  'default_authority' : string,
  'theme_mode_map' : Array<ThemeModeMapEntry>,
}
export interface TemporalVariantEntry {
  'key' : string,
  'value' : TemporalVariantPolicy,
}
export interface BrandStylePolicy {
  'allow_labs_customizations' : boolean,
  'labs_bounds' : LabsBounds,
  'official_palette' : BrandPalette,
  'mode_baselines' : [] | [BrandModeBaselinesPolicy],
  'host_defaults' : [] | [BrandHostDefaultsPolicy],
  'temporal_variants' : Array<TemporalVariantEntry>,
  'motion' : BrandMotionPolicy,
}
export interface BrandKernelPolicy {
  'mark_composition' : string,
  'technical_canonical_gap_degrees' : bigint,
  'ring_radius_px' : bigint,
  'dot_radius_px' : bigint,
  'base_stroke_width_px' : bigint,
  'steward_gated' : boolean,
}
export interface TemporalWindow {
  'state' : string,
  'recurrence' : string,
  'start_month_day' : string,
  'end_month_day' : string,
  'start_time_utc' : string,
  'end_time_utc' : string,
}
export interface BrandPolicyDocument {
  'schema_version' : string,
  'policy_id' : string,
  'policy_version' : bigint,
  'kernel' : BrandKernelPolicy,
  'style' : BrandStylePolicy,
  'temporal_windows' : Array<TemporalWindow>,
  'updated_at_ns' : bigint,
  'source_ref' : [] | [string],
}
export interface BrandPolicyMeta {
  'schema_version' : string,
  'policy_id' : string,
  'policy_version' : bigint,
  'updated_at_ns' : bigint,
  'source_ref' : [] | [string],
}

export interface _SERVICE {
  'get_brand_policy' : ActorMethod<[], BrandPolicyDocument>,
  'get_brand_policy_version' : ActorMethod<[], bigint>,
  'get_brand_policy_digest' : ActorMethod<[], string>,
  'get_brand_policy_meta' : ActorMethod<[], BrandPolicyMeta>,
}

export declare const idlFactory: IDL.InterfaceFactory;
