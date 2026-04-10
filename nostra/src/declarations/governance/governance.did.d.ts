import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface ActionScopeEvaluation {
  'policy_ref' : [] | [string],
  'allowed' : boolean,
  'required_actions' : Array<string>,
  'gate_decision' : string,
  'requires_review' : boolean,
  'effective_weight' : number,
  'reason' : string,
}
export interface AttributionWeightPolicy {
  'weight' : number,
  'updated_at' : bigint,
  'domain_mode' : string,
  'rationale' : string,
  'allow_binding' : boolean,
}
export type ComplianceResult = { 'RequiresReview' : string } |
  { 'Fail' : string } |
  { 'Pass' : null };
export interface Constitution { 'id' : string, 'policies' : Array<Policy> }
export type DecisionClass = { 'merge' : null } |
  { 'governance' : null } |
  { 'high_impact' : null } |
  { 'standard' : null };
export type EpistemicMode = { 'soft_gate' : null } |
  { 'observe' : null } |
  { 'hard_gate' : null };
export interface EpistemicPolicy {
  'block_on_soft' : boolean,
  'max_fork_pressure' : number,
  'min_alternatives' : number,
  'enforced_decision_classes' : Array<DecisionClass>,
  'mode' : EpistemicMode,
  'min_evidence' : number,
  'simulation_ttl_days' : number,
  'min_robustness' : number,
  'max_correction_density' : number,
  'max_confidence_drift' : number,
}
export interface Policy {
  'id' : string,
  'rule_type' : string,
  'parameters' : string,
}
export type Result = { 'Ok' : null } |
  { 'Err' : string };
export interface _SERVICE {
  'check_compliance' : ActorMethod<[string, string], ComplianceResult>,
  'evaluate_action_scope' : ActorMethod<
    [string, string, string],
    ActionScopeEvaluation
  >,
  'evaluate_action_scope_with_gate' : ActorMethod<
    [string, string, string, string],
    ActionScopeEvaluation
  >,
  'get_attribution_weight_policy' : ActorMethod<
    [string],
    [] | [AttributionWeightPolicy]
  >,
  'get_epistemic_policy' : ActorMethod<[], [] | [EpistemicPolicy]>,
  'get_workflow_engine_target' : ActorMethod<[], [] | [Principal]>,
  'register_constitution' : ActorMethod<[Constitution], undefined>,
  'set_attribution_weight_policy' : ActorMethod<
    [string, AttributionWeightPolicy],
    Result
  >,
  'set_epistemic_policy' : ActorMethod<[EpistemicPolicy], Result>,
  'set_workflow_engine_target' : ActorMethod<[Principal], Result>,
  'sync_epistemic_policy_to_workflow_engine' : ActorMethod<[], Result>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
