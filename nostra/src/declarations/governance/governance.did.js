export const idlFactory = ({ IDL }) => {
  const ComplianceResult = IDL.Variant({
    'RequiresReview' : IDL.Text,
    'Fail' : IDL.Text,
    'Pass' : IDL.Null,
  });
  const ActionScopeEvaluation = IDL.Record({
    'policy_ref' : IDL.Opt(IDL.Text),
    'allowed' : IDL.Bool,
    'required_actions' : IDL.Vec(IDL.Text),
    'gate_decision' : IDL.Text,
    'requires_review' : IDL.Bool,
    'effective_weight' : IDL.Float64,
    'reason' : IDL.Text,
  });
  const AttributionWeightPolicy = IDL.Record({
    'weight' : IDL.Float64,
    'updated_at' : IDL.Nat64,
    'domain_mode' : IDL.Text,
    'rationale' : IDL.Text,
    'allow_binding' : IDL.Bool,
  });
  const DecisionClass = IDL.Variant({
    'merge' : IDL.Null,
    'governance' : IDL.Null,
    'high_impact' : IDL.Null,
    'standard' : IDL.Null,
  });
  const EpistemicMode = IDL.Variant({
    'soft_gate' : IDL.Null,
    'observe' : IDL.Null,
    'hard_gate' : IDL.Null,
  });
  const EpistemicPolicy = IDL.Record({
    'block_on_soft' : IDL.Bool,
    'max_fork_pressure' : IDL.Float64,
    'min_alternatives' : IDL.Nat32,
    'enforced_decision_classes' : IDL.Vec(DecisionClass),
    'mode' : EpistemicMode,
    'min_evidence' : IDL.Nat32,
    'simulation_ttl_days' : IDL.Nat32,
    'min_robustness' : IDL.Float64,
    'max_correction_density' : IDL.Float64,
    'max_confidence_drift' : IDL.Float64,
  });
  const Policy = IDL.Record({
    'id' : IDL.Text,
    'rule_type' : IDL.Text,
    'parameters' : IDL.Text,
  });
  const Constitution = IDL.Record({
    'id' : IDL.Text,
    'policies' : IDL.Vec(Policy),
  });
  const Result = IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text });
  return IDL.Service({
    'check_compliance' : IDL.Func(
        [IDL.Text, IDL.Text],
        [ComplianceResult],
        ['query'],
      ),
    'evaluate_action_scope' : IDL.Func(
        [IDL.Text, IDL.Text, IDL.Text],
        [ActionScopeEvaluation],
        ['query'],
      ),
    'evaluate_action_scope_with_gate' : IDL.Func(
        [IDL.Text, IDL.Text, IDL.Text, IDL.Text],
        [ActionScopeEvaluation],
        ['query'],
      ),
    'get_attribution_weight_policy' : IDL.Func(
        [IDL.Text],
        [IDL.Opt(AttributionWeightPolicy)],
        ['query'],
      ),
    'get_epistemic_policy' : IDL.Func(
        [],
        [IDL.Opt(EpistemicPolicy)],
        ['query'],
      ),
    'get_workflow_engine_target' : IDL.Func(
        [],
        [IDL.Opt(IDL.Principal)],
        ['query'],
      ),
    'register_constitution' : IDL.Func([Constitution], [], []),
    'set_attribution_weight_policy' : IDL.Func(
        [IDL.Text, AttributionWeightPolicy],
        [Result],
        [],
      ),
    'set_epistemic_policy' : IDL.Func([EpistemicPolicy], [Result], []),
    'set_workflow_engine_target' : IDL.Func([IDL.Principal], [Result], []),
    'sync_epistemic_policy_to_workflow_engine' : IDL.Func([], [Result], []),
  });
};
export const init = ({ IDL }) => { return []; };
