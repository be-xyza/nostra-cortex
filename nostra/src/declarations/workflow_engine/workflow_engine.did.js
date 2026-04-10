export const idlFactory = ({ IDL }) => {
  const EpistemicOverrideAck = IDL.Record({
    'mutation_id' : IDL.Text,
    'justification' : IDL.Text,
    'workflow_id' : IDL.Text,
    'approved_at' : IDL.Nat64,
    'approved_by' : IDL.Principal,
    'assessment_id' : IDL.Text,
  });
  const Result = IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text });
  const DecisionClass = IDL.Variant({
    'merge' : IDL.Null,
    'governance' : IDL.Null,
    'high_impact' : IDL.Null,
    'standard' : IDL.Null,
  });
  const GateOutcome = IDL.Variant({
    'require_simulation' : IDL.Null,
    'pass' : IDL.Null,
    'warn' : IDL.Null,
    'block' : IDL.Null,
    'require_review' : IDL.Null,
  });
  const EpistemicAssessment = IDL.Record({
    'source_reliability' : IDL.Float64,
    'mutation_id' : IDL.Text,
    'reasons' : IDL.Vec(IDL.Text),
    'workflow_id' : IDL.Text,
    'robustness_score' : IDL.Float64,
    'regret_risk' : IDL.Float64,
    'created_at' : IDL.Nat64,
    'voi_score' : IDL.Float64,
    'evidence_count' : IDL.Nat32,
    'decision_class' : DecisionClass,
    'assumption_count' : IDL.Nat32,
    'gate_outcome' : GateOutcome,
    'alternative_count' : IDL.Nat32,
    'confidence_score' : IDL.Float64,
    'assessment_id' : IDL.Text,
  });
  const ResultEpistemicAssessment = IDL.Variant({
    'Ok' : EpistemicAssessment,
    'Err' : IDL.Text,
  });
  const AttributionMode = IDL.Variant({
    'delayed' : IDL.Null,
    'pseudonymous' : IDL.Null,
    'anonymous' : IDL.Null,
    'attributed' : IDL.Null,
  });
  const AttributionDomain = IDL.Record({
    'id' : IDL.Text,
    'reattachment_policy' : IDL.Text,
    'auditability_level' : IDL.Text,
    'updated_at' : IDL.Nat64,
    'updated_by' : IDL.Principal,
    'weight_policy_ref' : IDL.Opt(IDL.Text),
    'mode' : AttributionMode,
    'governance_visibility' : IDL.Text,
  });
  const ContributionAttributionBinding = IDL.Record({
    'bound_at' : IDL.Nat64,
    'bound_by' : IDL.Principal,
    'domain_id' : IDL.Text,
    'space_id' : IDL.Text,
    'contribution_id' : IDL.Text,
  });
  const ResultText = IDL.Variant({ 'Ok' : IDL.Text, 'Err' : IDL.Text });
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
  const EpistemicPolicyAuthority = IDL.Variant({
    'LocalAdmin' : IDL.Null,
    'GovernanceCanister' : IDL.Principal,
  });
  const FlowEdge = IDL.Record({
    'id' : IDL.Text,
    'topic' : IDL.Opt(IDL.Text),
    'source' : IDL.Text,
    'conditional' : IDL.Opt(IDL.Bool),
    'target' : IDL.Text,
    'variant' : IDL.Text,
  });
  const FlowNode = IDL.Record({
    'id' : IDL.Text,
    'flows' : IDL.Vec(IDL.Text),
    'schema_ref' : IDL.Opt(IDL.Text),
    'name' : IDL.Text,
    'tags' : IDL.Vec(IDL.Text),
    'type' : IDL.Text,
    'file_ref' : IDL.Opt(IDL.Text),
    'language' : IDL.Opt(IDL.Text),
  });
  const FlowGraph = IDL.Record({
    'id' : IDL.Text,
    'workflow_id' : IDL.Text,
    'generated_at' : IDL.Nat64,
    'edges' : IDL.Vec(FlowEdge),
    'version' : IDL.Text,
    'nodes' : IDL.Vec(FlowNode),
  });
  const ResultFlowGraph = IDL.Variant({ 'Ok' : FlowGraph, 'Err' : IDL.Text });
  const FlowHandlePosition = IDL.Record({
    'source' : IDL.Text,
    'target' : IDL.Text,
    'handle_id' : IDL.Text,
  });
  const FlowNodePosition = IDL.Record({
    'x' : IDL.Int,
    'y' : IDL.Int,
    'node_id' : IDL.Text,
  });
  const FlowLayout = IDL.Record({
    'handle_positions' : IDL.Vec(FlowHandlePosition),
    'updated_at' : IDL.Nat64,
    'updated_by' : IDL.Principal,
    'workflow_id' : IDL.Text,
    'node_positions' : IDL.Vec(FlowNodePosition),
    'graph_version' : IDL.Text,
    'collapsed_groups' : IDL.Vec(IDL.Text),
  });
  const ResultFlowLayout = IDL.Variant({ 'Ok' : FlowLayout, 'Err' : IDL.Text });
  const ReplayContract = IDL.Record({
    'mutation_id' : IDL.Text,
    'evidence_refs' : IDL.Vec(IDL.Text),
    'workflow_id' : IDL.Text,
    'action_target' : IDL.Text,
    'adapter_set_ref' : IDL.Text,
    'deterministic_input_hash' : IDL.Text,
    'execution_profile_ref' : IDL.Text,
    'attribution_domain_ref' : IDL.Text,
    'policy_snapshot_ref' : IDL.Opt(IDL.Text),
    'captured_at' : IDL.Nat64,
    'lineage_id' : IDL.Opt(IDL.Text),
  });
  const ExecutionTopology = IDL.Variant({
    'Networked' : IDL.Null,
    'LocalOnly' : IDL.Null,
    'Hybrid' : IDL.Null,
  });
  const ConsensusMode = IDL.Variant({
    'ReplicatedConsensus' : IDL.Null,
    'DelegatedConsensus' : IDL.Null,
    'NoneLocal' : IDL.Null,
  });
  const TrustBoundary = IDL.Variant({
    'PrivacyPreferred' : IDL.Null,
    'MixedAttribution' : IDL.Null,
    'AttributedDefault' : IDL.Null,
  });
  const ExecutionProfile = IDL.Record({
    'updated_at' : IDL.Nat64,
    'updated_by' : IDL.Principal,
    'execution_topology' : ExecutionTopology,
    'consensus_mode' : ConsensusMode,
    'trust_boundary' : TrustBoundary,
  });
  const FileMetadata = IDL.Record({
    'size' : IDL.Nat64,
    'mime_type' : IDL.Text,
    'last_modified' : IDL.Nat64,
  });
  const ListEntry = IDL.Tuple(IDL.Text, FileMetadata);
  const ResultList = IDL.Variant({
    'Ok' : IDL.Vec(ListEntry),
    'Err' : IDL.Text,
  });
  const OfflineConflictSummary = IDL.Record({
    'status' : IDL.Text,
    'mutation_id' : IDL.Text,
    'workflow_id' : IDL.Text,
    'source' : IDL.Opt(IDL.Text),
    'kind' : IDL.Text,
    'error' : IDL.Text,
  });
  const EditionMetadata = IDL.Record({ 'license' : IDL.Text });
  const ContributionVersionRef = IDL.Record({
    'contribution_id' : IDL.Text,
    'version_hash' : IDL.Text,
  });
  const ChapterManifest = IDL.Record({
    'title' : IDL.Text,
    'content_hash' : IDL.Text,
    'index' : IDL.Nat32,
    'contribution_ref' : ContributionVersionRef,
  });
  const EditionManifest = IDL.Record({
    'metadata' : EditionMetadata,
    'publisher' : IDL.Text,
    'name' : IDL.Opt(IDL.Text),
    'previous_edition' : IDL.Opt(IDL.Text),
    'content_root' : IDL.Text,
    'published_at' : IDL.Text,
    'version' : IDL.Text,
    'chapters' : IDL.Vec(ChapterManifest),
    'dpub_id' : IDL.Text,
    'edition_id' : IDL.Text,
  });
  const ResultEdition = IDL.Variant({
    'Ok' : EditionManifest,
    'Err' : IDL.Text,
  });
  const PublishContext = IDL.Record({
    'source_ref' : IDL.Opt(IDL.Text),
    'commit_hash' : IDL.Text,
  });
  const ResultBytes = IDL.Variant({
    'Ok' : IDL.Vec(IDL.Nat8),
    'Err' : IDL.Text,
  });
  const FlowLayoutInput = IDL.Record({
    'handle_positions' : IDL.Vec(FlowHandlePosition),
    'workflow_id' : IDL.Text,
    'node_positions' : IDL.Vec(FlowNodePosition),
    'graph_version' : IDL.Text,
    'collapsed_groups' : IDL.Vec(IDL.Text),
  });
  return IDL.Service({
    'ack_epistemic_override' : IDL.Func([EpistemicOverrideAck], [Result], []),
    'bind_contribution_attribution_domain' : IDL.Func(
        [IDL.Text, IDL.Text, IDL.Text],
        [Result],
        [],
      ),
    'evaluate_epistemic_gate' : IDL.Func(
        [IDL.Text, IDL.Text, IDL.Opt(IDL.Text)],
        [ResultEpistemicAssessment],
        [],
      ),
    'get_attribution_domains' : IDL.Func(
        [IDL.Text],
        [IDL.Vec(AttributionDomain)],
        ['query'],
      ),
    'get_contribution_attribution_binding' : IDL.Func(
        [IDL.Text],
        [IDL.Opt(ContributionAttributionBinding)],
        ['query'],
      ),
    'get_dpub_feed' : IDL.Func(
        [IDL.Text, IDL.Nat32, IDL.Opt(IDL.Text), IDL.Opt(IDL.Text)],
        [ResultText],
        ['query'],
      ),
    'get_epistemic_assessment' : IDL.Func(
        [IDL.Text],
        [IDL.Opt(EpistemicAssessment)],
        ['query'],
      ),
    'get_epistemic_assessment_by_mutation' : IDL.Func(
        [IDL.Text],
        [IDL.Opt(EpistemicAssessment)],
        ['query'],
      ),
    'get_epistemic_policy' : IDL.Func([], [EpistemicPolicy], ['query']),
    'get_epistemic_policy_authority' : IDL.Func(
        [],
        [EpistemicPolicyAuthority],
        ['query'],
      ),
    'get_flow_graph' : IDL.Func(
        [IDL.Text, IDL.Opt(IDL.Text)],
        [ResultFlowGraph],
        ['query'],
      ),
    'get_flow_layout' : IDL.Func(
        [IDL.Text, IDL.Opt(IDL.Text)],
        [ResultFlowLayout],
        ['query'],
      ),
    'get_replay_contract' : IDL.Func(
        [IDL.Text],
        [IDL.Opt(ReplayContract)],
        ['query'],
      ),
    'get_space_execution_profile' : IDL.Func(
        [IDL.Text],
        [IDL.Opt(ExecutionProfile)],
        ['query'],
      ),
    'get_workflow' : IDL.Func([IDL.Text], [IDL.Opt(IDL.Text)], ['query']),
    'list_dpub_files_guarded' : IDL.Func(
        [IDL.Text, IDL.Opt(IDL.Text), IDL.Opt(IDL.Text)],
        [ResultList],
        ['query'],
      ),
    'list_epistemic_assessments' : IDL.Func(
        [IDL.Text, IDL.Nat32],
        [IDL.Vec(EpistemicAssessment)],
        ['query'],
      ),
    'list_files' : IDL.Func([IDL.Text], [IDL.Vec(ListEntry)], ['query']),
    'list_offline_conflicts' : IDL.Func(
        [],
        [IDL.Vec(OfflineConflictSummary)],
        ['query'],
      ),
    'list_space_replay_contracts' : IDL.Func(
        [IDL.Text, IDL.Nat32],
        [IDL.Vec(ReplayContract)],
        ['query'],
      ),
    'list_vfs_guarded' : IDL.Func(
        [IDL.Text, IDL.Opt(IDL.Text), IDL.Opt(IDL.Text)],
        [ResultList],
        ['query'],
      ),
    'process_message' : IDL.Func([IDL.Text], [IDL.Text], []),
    'publish_dpub_edition' : IDL.Func(
        [IDL.Text, IDL.Text, IDL.Opt(IDL.Text), IDL.Opt(IDL.Text)],
        [ResultEdition],
        [],
      ),
    'publish_dpub_edition_v2' : IDL.Func(
        [
          IDL.Text,
          IDL.Text,
          IDL.Opt(IDL.Text),
          IDL.Opt(IDL.Text),
          PublishContext,
        ],
        [ResultEdition],
        [],
      ),
    'read_dpub_file_guarded' : IDL.Func(
        [IDL.Text, IDL.Opt(IDL.Text), IDL.Opt(IDL.Text)],
        [ResultBytes],
        ['query'],
      ),
    'read_file' : IDL.Func([IDL.Text], [ResultBytes], ['query']),
    'read_vfs_guarded' : IDL.Func(
        [IDL.Text, IDL.Opt(IDL.Text), IDL.Opt(IDL.Text)],
        [ResultBytes],
        ['query'],
      ),
    'set_epistemic_policy' : IDL.Func([EpistemicPolicy], [Result], []),
    'set_epistemic_policy_authority' : IDL.Func(
        [EpistemicPolicyAuthority],
        [Result],
        [],
      ),
    'set_flow_layout' : IDL.Func([FlowLayoutInput], [ResultFlowLayout], []),
    'set_space_execution_profile' : IDL.Func(
        [IDL.Text, ExecutionProfile],
        [Result],
        [],
      ),
    'start_workflow' : IDL.Func([IDL.Text], [IDL.Text], []),
    'upsert_attribution_domain' : IDL.Func(
        [IDL.Text, AttributionDomain],
        [Result],
        [],
      ),
    'write_file' : IDL.Func(
        [IDL.Text, IDL.Vec(IDL.Nat8), IDL.Text],
        [Result],
        [],
      ),
  });
};
export const init = ({ IDL }) => { return []; };
