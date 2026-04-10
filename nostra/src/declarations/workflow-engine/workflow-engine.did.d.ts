import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface AttributionDomain {
  'id' : string,
  'reattachment_policy' : string,
  'auditability_level' : string,
  'updated_at' : bigint,
  'updated_by' : Principal,
  'weight_policy_ref' : [] | [string],
  'mode' : AttributionMode,
  'governance_visibility' : string,
}
export type AttributionMode = { 'delayed' : null } |
  { 'pseudonymous' : null } |
  { 'anonymous' : null } |
  { 'attributed' : null };
export interface ChapterManifest {
  'title' : string,
  'content_hash' : string,
  'index' : number,
  'contribution_ref' : ContributionVersionRef,
}
export type ConsensusMode = { 'ReplicatedConsensus' : null } |
  { 'DelegatedConsensus' : null } |
  { 'NoneLocal' : null };
export interface ContributionAttributionBinding {
  'bound_at' : bigint,
  'bound_by' : Principal,
  'domain_id' : string,
  'space_id' : string,
  'contribution_id' : string,
}
export interface ContributionVersionRef {
  'contribution_id' : string,
  'version_hash' : string,
}
export type DecisionClass = { 'merge' : null } |
  { 'governance' : null } |
  { 'high_impact' : null } |
  { 'standard' : null };
export interface EditionManifest {
  'metadata' : EditionMetadata,
  'publisher' : string,
  'name' : [] | [string],
  'previous_edition' : [] | [string],
  'content_root' : string,
  'published_at' : string,
  'version' : string,
  'chapters' : Array<ChapterManifest>,
  'dpub_id' : string,
  'edition_id' : string,
}
export interface EditionMetadata { 'license' : string }
export interface EpistemicAssessment {
  'source_reliability' : number,
  'mutation_id' : string,
  'reasons' : Array<string>,
  'workflow_id' : string,
  'robustness_score' : number,
  'regret_risk' : number,
  'created_at' : bigint,
  'voi_score' : number,
  'evidence_count' : number,
  'decision_class' : DecisionClass,
  'assumption_count' : number,
  'gate_outcome' : GateOutcome,
  'alternative_count' : number,
  'confidence_score' : number,
  'assessment_id' : string,
}
/**
 * Blackwell Guardrails
 */
export type EpistemicMode = { 'soft_gate' : null } |
  { 'observe' : null } |
  { 'hard_gate' : null };
export interface EpistemicOverrideAck {
  'mutation_id' : string,
  'justification' : string,
  'workflow_id' : string,
  'approved_at' : bigint,
  'approved_by' : Principal,
  'assessment_id' : string,
}
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
export type EpistemicPolicyAuthority = { 'LocalAdmin' : null } |
  { 'GovernanceCanister' : Principal };
export interface ExecutionProfile {
  'updated_at' : bigint,
  'updated_by' : Principal,
  'execution_topology' : ExecutionTopology,
  'consensus_mode' : ConsensusMode,
  'trust_boundary' : TrustBoundary,
}
export type ExecutionTopology = { 'Networked' : null } |
  { 'LocalOnly' : null } |
  { 'Hybrid' : null };
export interface FileMetadata {
  'size' : bigint,
  'mime_type' : string,
  'last_modified' : bigint,
}
export interface FlowEdge {
  'id' : string,
  'topic' : [] | [string],
  'source' : string,
  'conditional' : [] | [boolean],
  'target' : string,
  'variant' : string,
}
export interface FlowGraph {
  'id' : string,
  'workflow_id' : string,
  'generated_at' : bigint,
  'edges' : Array<FlowEdge>,
  'version' : string,
  'nodes' : Array<FlowNode>,
}
export interface FlowHandlePosition {
  'source' : string,
  'target' : string,
  'handle_id' : string,
}
export interface FlowLayout {
  'handle_positions' : Array<FlowHandlePosition>,
  'updated_at' : bigint,
  'updated_by' : Principal,
  'workflow_id' : string,
  'node_positions' : Array<FlowNodePosition>,
  'graph_version' : string,
  'collapsed_groups' : Array<string>,
}
export interface FlowLayoutInput {
  'handle_positions' : Array<FlowHandlePosition>,
  'workflow_id' : string,
  'node_positions' : Array<FlowNodePosition>,
  'graph_version' : string,
  'collapsed_groups' : Array<string>,
}
/**
 * Flow Graph + Workbench
 */
export interface FlowNode {
  'id' : string,
  'flows' : Array<string>,
  'schema_ref' : [] | [string],
  'name' : string,
  'tags' : Array<string>,
  'type' : string,
  'file_ref' : [] | [string],
  'language' : [] | [string],
}
export interface FlowNodePosition {
  'x' : bigint,
  'y' : bigint,
  'node_id' : string,
}
export type GateOutcome = { 'require_simulation' : null } |
  { 'pass' : null } |
  { 'warn' : null } |
  { 'block' : null } |
  { 'require_review' : null };
export type ListEntry = [string, FileMetadata];
export interface OfflineConflictSummary {
  'status' : string,
  'mutation_id' : string,
  'workflow_id' : string,
  'source' : [] | [string],
  'kind' : string,
  'error' : string,
}
export interface PublishContext {
  'source_ref' : [] | [string],
  'commit_hash' : string,
}
export interface ReplayContract {
  'mutation_id' : string,
  'evidence_refs' : Array<string>,
  'workflow_id' : string,
  'action_target' : string,
  'adapter_set_ref' : string,
  'deterministic_input_hash' : string,
  'execution_profile_ref' : string,
  'attribution_domain_ref' : string,
  'policy_snapshot_ref' : [] | [string],
  'captured_at' : bigint,
  'lineage_id' : [] | [string],
}
export type Result = { 'Ok' : null } |
  { 'Err' : string };
export type ResultBytes = { 'Ok' : Uint8Array | number[] } |
  { 'Err' : string };
export type ResultEdition = { 'Ok' : EditionManifest } |
  { 'Err' : string };
export type ResultEpistemicAssessment = { 'Ok' : EpistemicAssessment } |
  { 'Err' : string };
export type ResultEpistemicPolicy = { 'Ok' : EpistemicPolicy } |
  { 'Err' : string };
export type ResultFlowGraph = { 'Ok' : FlowGraph } |
  { 'Err' : string };
export type ResultFlowLayout = { 'Ok' : FlowLayout } |
  { 'Err' : string };
export type ResultList = { 'Ok' : Array<ListEntry> } |
  { 'Err' : string };
export type ResultText = { 'Ok' : string } |
  { 'Err' : string };
export type TrustBoundary = { 'PrivacyPreferred' : null } |
  { 'MixedAttribution' : null } |
  { 'AttributedDefault' : null };
export interface _SERVICE {
  'ack_epistemic_override' : ActorMethod<[EpistemicOverrideAck], Result>,
  'bind_contribution_attribution_domain' : ActorMethod<
    [string, string, string],
    Result
  >,
  /**
   * Blackwell Guardrails
   */
  'evaluate_epistemic_gate' : ActorMethod<
    [string, string, [] | [string]],
    ResultEpistemicAssessment
  >,
  'get_attribution_domains' : ActorMethod<[string], Array<AttributionDomain>>,
  'get_contribution_attribution_binding' : ActorMethod<
    [string],
    [] | [ContributionAttributionBinding]
  >,
  'get_dpub_feed' : ActorMethod<
    [string, number, [] | [string], [] | [string]],
    ResultText
  >,
  'get_epistemic_assessment' : ActorMethod<
    [string],
    [] | [EpistemicAssessment]
  >,
  'get_epistemic_assessment_by_mutation' : ActorMethod<
    [string],
    [] | [EpistemicAssessment]
  >,
  'get_epistemic_policy' : ActorMethod<[], EpistemicPolicy>,
  'get_epistemic_policy_authority' : ActorMethod<[], EpistemicPolicyAuthority>,
  /**
   * Flow Graph
   */
  'get_flow_graph' : ActorMethod<[string, [] | [string]], ResultFlowGraph>,
  'get_flow_layout' : ActorMethod<[string, [] | [string]], ResultFlowLayout>,
  'get_replay_contract' : ActorMethod<[string], [] | [ReplayContract]>,
  'get_space_execution_profile' : ActorMethod<
    [string],
    [] | [ExecutionProfile]
  >,
  'get_workflow' : ActorMethod<[string], [] | [string]>,
  'list_dpub_files_guarded' : ActorMethod<
    [string, [] | [string], [] | [string]],
    ResultList
  >,
  'list_epistemic_assessments' : ActorMethod<
    [string, number],
    Array<EpistemicAssessment>
  >,
  'list_files' : ActorMethod<[string], Array<ListEntry>>,
  'list_offline_conflicts' : ActorMethod<[], Array<OfflineConflictSummary>>,
  'list_space_replay_contracts' : ActorMethod<
    [string, number],
    Array<ReplayContract>
  >,
  'list_vfs_guarded' : ActorMethod<
    [string, [] | [string], [] | [string]],
    ResultList
  >,
  'process_message' : ActorMethod<[string], string>,
  /**
   * dPub V1
   * Deprecated for new clients; use publish_dpub_edition_v2.
   */
  'publish_dpub_edition' : ActorMethod<
    [string, string, [] | [string], [] | [string]],
    ResultEdition
  >,
  'publish_dpub_edition_v2' : ActorMethod<
    [string, string, [] | [string], [] | [string], PublishContext],
    ResultEdition
  >,
  'read_dpub_file_guarded' : ActorMethod<
    [string, [] | [string], [] | [string]],
    ResultBytes
  >,
  'read_file' : ActorMethod<[string], ResultBytes>,
  'read_vfs_guarded' : ActorMethod<
    [string, [] | [string], [] | [string]],
    ResultBytes
  >,
  'set_epistemic_policy' : ActorMethod<[EpistemicPolicy], Result>,
  'set_epistemic_policy_authority' : ActorMethod<
    [EpistemicPolicyAuthority],
    Result
  >,
  'set_flow_layout' : ActorMethod<[FlowLayoutInput], ResultFlowLayout>,
  /**
   * Execution + Attribution decision surfaces
   */
  'set_space_execution_profile' : ActorMethod<
    [string, ExecutionProfile],
    Result
  >,
  'start_workflow' : ActorMethod<[string], string>,
  'upsert_attribution_domain' : ActorMethod<
    [string, AttributionDomain],
    Result
  >,
  /**
   * VFS
   */
  'write_file' : ActorMethod<[string, Uint8Array | number[], string], Result>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
