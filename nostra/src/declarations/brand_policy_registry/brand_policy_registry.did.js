export const idlFactory = ({ IDL }) => {
  const BrandPalette = IDL.Record({
    'outer_base' : IDL.Text,
    'outer_gradient_to' : IDL.Text,
    'inner_base' : IDL.Text,
    'inner_gradient_to' : IDL.Text,
  });
  const LabsBounds = IDL.Record({
    'gap_min_degrees' : IDL.Float64,
    'gap_max_degrees' : IDL.Float64,
    'stroke_min_px' : IDL.Float64,
    'stroke_max_px' : IDL.Float64,
  });
  const TemporalVariantPolicy = IDL.Record({
    'force_gradient' : IDL.Bool,
    'stroke_cap' : IDL.Text,
    'palette' : BrandPalette,
  });
  const PhilosophicalMotionPolicy = IDL.Record({
    'container_transition_sec' : IDL.Float64,
    'ring_transition_sec' : IDL.Float64,
    'stroke_transition_sec' : IDL.Float64,
    'ring_animation_duration_sec' : IDL.Float64,
    'ring_rotation_delta_deg' : IDL.Float64,
    'ring_stroke_delta_px' : IDL.Float64,
    'dot_transition_sec' : IDL.Float64,
    'dot_animation_duration_sec' : IDL.Float64,
    'dot_pulse_radius_delta_px' : IDL.Float64,
    'dot_pulse_opacity_min' : IDL.Float64,
  });
  const TechnicalMotionPolicy = IDL.Record({
    'container_transition_ms' : IDL.Nat64,
    'ring_transition_ms' : IDL.Nat64,
    'stroke_transition_ms' : IDL.Nat64,
    'dot_transition_ms' : IDL.Nat64,
    'step_count' : IDL.Nat64,
    'ring_step_count' : IDL.Nat64,
  });
  const BrandMotionPolicy = IDL.Record({
    'philosophical' : PhilosophicalMotionPolicy,
    'technical' : TechnicalMotionPolicy,
  });
  const PhilosophicalModeBaselinePolicy = IDL.Record({
    'gap_degrees' : IDL.Float64,
    'stroke_width_delta_px' : IDL.Float64,
    'stroke_cap' : IDL.Text,
    'force_gradient' : IDL.Bool,
  });
  const TechnicalModeBaselinePolicy = IDL.Record({
    'stroke_cap' : IDL.Text,
    'force_gradient' : IDL.Bool,
  });
  const BrandModeBaselinesPolicy = IDL.Record({
    'philosophical' : PhilosophicalModeBaselinePolicy,
    'technical' : TechnicalModeBaselinePolicy,
  });
  const ThemeModeMapEntry = IDL.Record({
    'key' : IDL.Text,
    'value' : IDL.Text,
  });
  const BrandHostDefaultsPolicy = IDL.Record({
    'default_temporal_state' : IDL.Text,
    'default_authority' : IDL.Text,
    'theme_mode_map' : IDL.Vec(ThemeModeMapEntry),
  });
  const TemporalVariantEntry = IDL.Record({
    'key' : IDL.Text,
    'value' : TemporalVariantPolicy,
  });
  const BrandStylePolicy = IDL.Record({
    'allow_labs_customizations' : IDL.Bool,
    'labs_bounds' : LabsBounds,
    'official_palette' : BrandPalette,
    'mode_baselines' : IDL.Opt(BrandModeBaselinesPolicy),
    'host_defaults' : IDL.Opt(BrandHostDefaultsPolicy),
    'temporal_variants' : IDL.Vec(TemporalVariantEntry),
    'motion' : BrandMotionPolicy,
  });
  const BrandKernelPolicy = IDL.Record({
    'mark_composition' : IDL.Text,
    'technical_canonical_gap_degrees' : IDL.Nat64,
    'ring_radius_px' : IDL.Nat64,
    'dot_radius_px' : IDL.Nat64,
    'base_stroke_width_px' : IDL.Nat64,
    'steward_gated' : IDL.Bool,
  });
  const TemporalWindow = IDL.Record({
    'state' : IDL.Text,
    'recurrence' : IDL.Text,
    'start_month_day' : IDL.Text,
    'end_month_day' : IDL.Text,
    'start_time_utc' : IDL.Text,
    'end_time_utc' : IDL.Text,
  });
  const BrandPolicyDocument = IDL.Record({
    'schema_version' : IDL.Text,
    'policy_id' : IDL.Text,
    'policy_version' : IDL.Nat64,
    'kernel' : BrandKernelPolicy,
    'style' : BrandStylePolicy,
    'temporal_windows' : IDL.Vec(TemporalWindow),
    'updated_at_ns' : IDL.Nat64,
    'source_ref' : IDL.Opt(IDL.Text),
  });
  const BrandPolicyMeta = IDL.Record({
    'schema_version' : IDL.Text,
    'policy_id' : IDL.Text,
    'policy_version' : IDL.Nat64,
    'updated_at_ns' : IDL.Nat64,
    'source_ref' : IDL.Opt(IDL.Text),
  });

  return IDL.Service({
    'get_brand_policy' : IDL.Func([], [BrandPolicyDocument], ['query']),
    'get_brand_policy_version' : IDL.Func([], [IDL.Nat64], ['query']),
    'get_brand_policy_digest' : IDL.Func([], [IDL.Text], ['query']),
    'get_brand_policy_meta' : IDL.Func([], [BrandPolicyMeta], ['query']),
  });
};

export const init = ({ IDL }) => {
  return [];
};
