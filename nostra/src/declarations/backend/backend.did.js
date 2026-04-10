export const idlFactory = ({ IDL }) => {
  const StructureNode = IDL.Rec();
  const Result = IDL.Variant({ 'ok' : IDL.Null, 'err' : IDL.Text });
  const AdoptionMode = IDL.Variant({
    'pinned' : IDL.Null,
    'adopted' : IDL.Null,
  });
  const EntityType = IDL.Variant({
    'protocol' : IDL.Null,
    'service' : IDL.Null,
    'cryptography' : IDL.Null,
    'review' : IDL.Null,
    'model' : IDL.Null,
    'component' : IDL.Null,
    'decision' : IDL.Null,
    'deliverable' : IDL.Null,
    'feature' : IDL.Null,
    'cryptoAsset' : IDL.Null,
    'economy' : IDL.Null,
    'pledge' : IDL.Null,
    'person' : IDL.Null,
    'library' : IDL.Null,
    'book' : IDL.Null,
    'dpub' : IDL.Null,
    'institution' : IDL.Null,
    'idea' : IDL.Null,
    'question' : IDL.Null,
    'comment' : IDL.Null,
    'poll' : IDL.Null,
    'post' : IDL.Null,
    'artifact' : IDL.Null,
    'mediaEssay' : IDL.Null,
    'security' : IDL.Null,
    'bounty' : IDL.Null,
    'essay' : IDL.Null,
    'event' : IDL.Null,
    'infrastructure' : IDL.Null,
    'credentialReference' : IDL.Null,
    'assetReference' : IDL.Null,
    'observation' : IDL.Null,
    'proposal' : IDL.Null,
    'discussion' : IDL.Null,
    'initiative' : IDL.Null,
    'milestone' : IDL.Null,
    'organization' : IDL.Null,
    'issue' : IDL.Null,
    'report' : IDL.Null,
    'chapter' : IDL.Null,
    'governanceSystem' : IDL.Null,
    'reflection' : IDL.Null,
    'developmentTool' : IDL.Null,
    'project' : IDL.Null,
  });
  const Entity = IDL.Record({
    'id' : IDL.Text,
    'libraryId' : IDL.Opt(IDL.Text),
    'name' : IDL.Text,
    'tags' : IDL.Vec(IDL.Text),
    'description' : IDL.Text,
    'creatorActorId' : IDL.Opt(IDL.Text),
    'logRefs' : IDL.Opt(IDL.Vec(IDL.Text)),
    'scopeId' : IDL.Opt(IDL.Text),
    'attributes' : IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text)),
    'creatorAddress' : IDL.Opt(IDL.Text),
    'timestamp' : IDL.Int,
    'entityType' : EntityType,
  });
  const Relationship = IDL.Record({
    'to' : IDL.Text,
    'libraryId' : IDL.Opt(IDL.Text),
    'from' : IDL.Text,
    'type' : IDL.Text,
    'creatorActorId' : IDL.Opt(IDL.Text),
    'scopeId' : IDL.Opt(IDL.Text),
    'bidirectional' : IDL.Bool,
    'creatorAddress' : IDL.Opt(IDL.Text),
    'timestamp' : IDL.Int,
  });
  const VoteChoice = IDL.Variant({
    'no' : IDL.Null,
    'yes' : IDL.Null,
    'abstain' : IDL.Null,
  });
  const GatingPolicy = IDL.Variant({
    'open' : IDL.Null,
    'member_only' : IDL.Null,
    'reflection_required' : IDL.Null,
  });
  const LifecyclePhase = IDL.Variant({
    'formalized' : IDL.Null,
    'dormant' : IDL.Null,
    'emergent' : IDL.Null,
    'operational' : IDL.Null,
    'provisional' : IDL.Null,
    'archived' : IDL.Null,
  });
  const VotingConfig = IDL.Record({
    'durationSeconds' : IDL.Int,
    'quorum' : IDL.Float64,
    'passingThreshold' : IDL.Float64,
  });
  const GovernanceStrategy = IDL.Variant({
    'voting' : IDL.Record({
      'votingSystem' : IDL.Variant({
        'token_weighted' : IDL.Null,
        'simple_majority' : IDL.Null,
      }),
      'config' : VotingConfig,
    }),
    'owner_dictator' : IDL.Null,
  });
  const CreateInstitutionRequest = IDL.Record({
    'parentInstitutionId' : IDL.Opt(IDL.Text),
    'title' : IDL.Text,
    'lifecyclePhase' : IDL.Opt(LifecyclePhase),
    'governanceStrategy' : IDL.Opt(GovernanceStrategy),
    'charterRefs' : IDL.Opt(IDL.Vec(IDL.Text)),
    'scope' : IDL.Text,
    'summary' : IDL.Text,
    'spaceId' : IDL.Text,
    'intent' : IDL.Text,
    'stewards' : IDL.Opt(IDL.Vec(IDL.Principal)),
    'affiliatedSpaces' : IDL.Opt(IDL.Vec(IDL.Text)),
    'confidence' : IDL.Opt(IDL.Float64),
  });
  const Result_1 = IDL.Variant({ 'ok' : IDL.Text, 'err' : IDL.Text });
  const Visibility = IDL.Variant({
    'public' : IDL.Null,
    'member_only' : IDL.Null,
    'private' : IDL.Null,
  });
  const AccessOp = IDL.Variant({
    'cite' : IDL.Null,
    'fork' : IDL.Null,
    'read' : IDL.Null,
    'traverse' : IDL.Null,
  });
  const AccessScope = IDL.Record({
    'entityTypes' : IDL.Vec(IDL.Text),
    'operations' : IDL.Vec(AccessOp),
    'depth' : IDL.Nat,
  });
  const KipResult = IDL.Variant({
    'ok' : IDL.Tuple(IDL.Text, IDL.Opt(Entity)),
    'err' : IDL.Text,
  });
  const RefType = IDL.Variant({ 'tag' : IDL.Null, 'entity' : IDL.Null });
  const ExtractedRef = IDL.Record({
    'targetText' : IDL.Text,
    'refType' : RefType,
  });
  const Institution = IDL.Record({
    'id' : IDL.Text,
    'parentInstitutionId' : IDL.Opt(IDL.Text),
    'title' : IDL.Text,
    'lifecyclePhase' : LifecyclePhase,
    'createdAt' : IDL.Int,
    'createdBy' : IDL.Principal,
    'governanceStrategy' : IDL.Opt(GovernanceStrategy),
    'charterRefs' : IDL.Vec(IDL.Text),
    'scope' : IDL.Text,
    'version' : IDL.Nat,
    'summary' : IDL.Text,
    'updatedAt' : IDL.Int,
    'spaceId' : IDL.Text,
    'previousVersionId' : IDL.Opt(IDL.Text),
    'intent' : IDL.Text,
    'stewards' : IDL.Vec(IDL.Principal),
    'affiliatedSpaces' : IDL.Vec(IDL.Text),
    'confidence' : IDL.Float64,
  });
  const Jurisdiction = IDL.Record({
    'region' : IDL.Opt(IDL.Text),
    'city' : IDL.Opt(IDL.Text),
    'countryCode' : IDL.Text,
  });
  const GeoLocation = IDL.Record({
    'latitude' : IDL.Float64,
    'precision' : IDL.Opt(IDL.Float64),
    'longitude' : IDL.Float64,
  });
  const ExternalRequest = IDL.Record({
    'stepId' : IDL.Text,
    'startedAt' : IDL.Int,
    'requestId' : IDL.Text,
    'instanceId' : IDL.Text,
    'jurisdiction' : IDL.Opt(Jurisdiction),
    'geoLocation' : IDL.Opt(GeoLocation),
    'payload' : IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text)),
  });
  const InstanceStatus = IDL.Variant({
    'active' : IDL.Null,
    'waiting_for_external' : ExternalRequest,
    'completed' : IDL.Null,
    'failed' : IDL.Text,
  });
  const ActorID = IDL.Text;
  const HistoryEntry = IDL.Record({
    'stepId' : IDL.Text,
    'action' : IDL.Text,
    'user' : ActorID,
    'timestamp' : IDL.Int,
  });
  const ParallelState = IDL.Record({
    'results' : IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text)),
    'completedSteps' : IDL.Vec(IDL.Text),
    'activeSteps' : IDL.Vec(IDL.Text),
    'forkId' : IDL.Text,
  });
  const RepeatState = IDL.Record({
    'maxIterations' : IDL.Nat,
    'loopId' : IDL.Text,
    'currentIteration' : IDL.Nat,
  });
  const WorkflowInstance = IDL.Record({
    'id' : IDL.Text,
    'status' : InstanceStatus,
    'context' : IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text)),
    'createdAt' : IDL.Int,
    'history' : IDL.Vec(HistoryEntry),
    'definitionId' : IDL.Text,
    'parallelState' : IDL.Opt(ParallelState),
    'jurisdiction' : IDL.Opt(Jurisdiction),
    'loopState' : IDL.Opt(RepeatState),
    'currentStepId' : IDL.Text,
    'geoLocation' : IDL.Opt(GeoLocation),
  });
  const BookId = IDL.Text;
  const ContributionId = IDL.Text;
  const ContributionRef = IDL.Record({
    'id' : ContributionId,
    'specificVersion' : IDL.Opt(IDL.Text),
  });
  const NodeType__1 = IDL.Variant({
    'appendix' : IDL.Null,
    'part' : IDL.Null,
    'section' : IDL.Null,
    'sidebar' : IDL.Null,
    'chapter' : IDL.Null,
  });
  StructureNode.fill(
    IDL.Record({
      'id' : IDL.Text,
      'title' : IDL.Text,
      'slug' : IDL.Text,
      'reference' : IDL.Opt(ContributionRef),
      'children' : IDL.Vec(StructureNode),
      'nodeType' : NodeType__1,
    })
  );
  const BookStructure = IDL.Record({ 'nodes' : IDL.Vec(StructureNode) });
  const EditionId = IDL.Text;
  const Edition = IDL.Record({
    'id' : EditionId,
    'frozenStructure' : BookStructure,
    'citationHash' : IDL.Text,
    'name' : IDL.Text,
    'publishedAt' : IDL.Int,
  });
  const Book = IDL.Record({
    'id' : BookId,
    'title' : IDL.Text,
    'workflowIds' : IDL.Vec(IDL.Text),
    'isbn' : IDL.Opt(IDL.Text),
    'structure' : BookStructure,
    'createdAt' : IDL.Int,
    'agentIds' : IDL.Vec(IDL.Text),
    'author' : IDL.Principal,
    'coverImage' : IDL.Opt(IDL.Text),
    'updatedAt' : IDL.Int,
    'spaceId' : IDL.Text,
    'editions' : IDL.Vec(Edition),
    'subtitle' : IDL.Opt(IDL.Text),
  });
  const ChatMessageType = IDL.Variant({ 'ai' : IDL.Null, 'user' : IDL.Null });
  const ChatMessage = IDL.Record({
    'content' : IDL.Text,
    'context' : IDL.Opt(IDL.Text),
    'messageType' : ChatMessageType,
    'timestamp' : IDL.Int,
  });
  const ChronicleEventType = IDL.Variant({
    'relationship_formed' : IDL.Null,
    'workflow_failed' : IDL.Null,
    'entity_updated' : IDL.Null,
    'role_granted' : IDL.Null,
    'proposal_approved' : IDL.Null,
    'workflow_completed' : IDL.Null,
    'proposal_rejected' : IDL.Null,
    'proposal_submitted' : IDL.Null,
    'workflow_started' : IDL.Null,
    'entity_archived' : IDL.Null,
    'library_forked' : IDL.Null,
    'entity_created' : IDL.Null,
    'library_enabled' : IDL.Null,
    'library_merged' : IDL.Null,
    'workflow_step_completed' : IDL.Null,
    'library_disabled' : IDL.Null,
    'role_revoked' : IDL.Null,
    'relationship_removed' : IDL.Null,
    'library_installed' : IDL.Null,
  });
  const ChronicleEvent = IDL.Record({
    'id' : IDL.Text,
    'libraryId' : IDL.Opt(IDL.Text),
    'metadata' : IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text)),
    'description' : IDL.Text,
    'actorId' : IDL.Text,
    'affectedEntities' : IDL.Vec(IDL.Text),
    'timestamp' : IDL.Int,
    'eventType' : ChronicleEventType,
  });
  const CommonsAdoption = IDL.Record({
    'mode' : AdoptionMode,
    'version' : IDL.Opt(IDL.Text),
    'spaceId' : IDL.Text,
    'commonsId' : IDL.Text,
  });
  const CommonsEnforcementMode = IDL.Variant({
    'shadow' : IDL.Null,
    'warnOrBlock' : IDL.Null,
  });
  const IntegrityScope = IDL.Variant({
    'space' : IDL.Text,
    'global' : IDL.Null,
    'entityType' : IDL.Text,
  });
  const Severity = IDL.Variant({
    'warning' : IDL.Null,
    'violation' : IDL.Null,
    'info' : IDL.Null,
    'critical' : IDL.Null,
  });
  const Direction = IDL.Variant({
    'incoming' : IDL.Null,
    'outgoing' : IDL.Null,
  });
  const EdgeSelector = IDL.Record({
    'direction' : Direction,
    'relationType' : IDL.Text,
  });
  const Constraint = IDL.Variant({
    'mustExist' : IDL.Null,
    'minCount' : IDL.Nat,
    'noCycles' : IDL.Null,
    'requiresConstitutionalReference' : IDL.Null,
    'mustNotExist' : IDL.Null,
    'noConflicts' : IDL.Null,
    'maxCount' : IDL.Nat,
  });
  const NodeSelector = IDL.Record({
    'tags' : IDL.Opt(IDL.Vec(IDL.Text)),
    'entityType' : IDL.Opt(IDL.Text),
  });
  const IntegrityPredicate = IDL.Record({
    'relation' : IDL.Opt(EdgeSelector),
    'constraint' : Constraint,
    'target' : NodeSelector,
  });
  const IntegrityRule = IDL.Record({
    'id' : IDL.Text,
    'name' : IDL.Text,
    'description' : IDL.Text,
    'scope' : IntegrityScope,
    'severity' : Severity,
    'remediationHint' : IDL.Opt(IDL.Text),
    'predicate' : IntegrityPredicate,
  });
  const CommonsRuleset = IDL.Record({
    'commonsVersion' : IDL.Text,
    'commonsId' : IDL.Text,
    'rules' : IDL.Vec(IntegrityRule),
  });
  const GraphEdge = IDL.Record({
    'id' : IDL.Text,
    'to_id' : IDL.Text,
    'type' : IDL.Text,
    'from_id' : IDL.Text,
    'origin_canister_id' : IDL.Principal,
    'timestamp' : IDL.Int,
    'confidence' : IDL.Float64,
    'scope_id' : IDL.Opt(IDL.Text),
  });
  const Message = IDL.Record({
    'id' : IDL.Text,
    'content' : IDL.Text,
    'sender' : IDL.Principal,
    'timestamp' : IDL.Int,
    'reflectionRef' : IDL.Opt(IDL.Text),
  });
  const Discussion = IDL.Record({
    'id' : IDL.Text,
    'participants' : IDL.Vec(IDL.Principal),
    'topic' : IDL.Text,
    'messages' : IDL.Vec(Message),
    'lastActivity' : IDL.Int,
    'gatingPolicy' : GatingPolicy,
    'targetEntityId' : IDL.Text,
  });
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
    'commit_message' : IDL.Opt(IDL.Text),
    'graph_version' : IDL.Text,
    'commit_tag' : IDL.Opt(IDL.Text),
    'collapsed_groups' : IDL.Vec(IDL.Text),
  });
  const LibraryManifest = IDL.Record({
    'id' : IDL.Text,
    'description' : IDL.Text,
    'version' : IDL.Text,
    'license' : IDL.Opt(IDL.Text),
  });
  const LineageNode = IDL.Record({
    'institution' : Institution,
    'depth' : IDL.Nat,
  });
  const LogSource = IDL.Variant({
    'Frontend' : IDL.Null,
    'Agent' : IDL.Text,
    'Backend' : IDL.Null,
  });
  const LogLevel = IDL.Variant({
    'Error' : IDL.Null,
    'Info' : IDL.Null,
    'Warn' : IDL.Null,
    'Critical' : IDL.Null,
  });
  const LogEntry = IDL.Record({
    'id' : IDL.Text,
    'context' : IDL.Opt(IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text))),
    'source' : LogSource,
    'level' : LogLevel,
    'message' : IDL.Text,
    'timestamp' : IDL.Int,
  });
  const KeyEntry = IDL.Record({
    'id' : IDL.Text,
    'alg' : IDL.Opt(IDL.Text),
    'model' : IDL.Opt(IDL.Text),
    'createdAt' : IDL.Int,
    'scope' : IDL.Opt(IDL.Text),
    'encryptedKey' : IDL.Vec(IDL.Nat8),
    'keyLabel' : IDL.Text,
    'ephemeralPubKey' : IDL.Opt(IDL.Vec(IDL.Nat8)),
    'keyId' : IDL.Opt(IDL.Text),
    'encVersion' : IDL.Opt(IDL.Nat),
  });
  const SavedUI = IDL.Record({
    'id' : IDL.Text,
    'content' : IDL.Text,
    'name' : IDL.Text,
    'updatedAt' : IDL.Int,
  });
  const UserProfile = IDL.Record({
    'savedUis' : IDL.Vec(SavedUI),
    'featureFlags' : IDL.Vec(IDL.Text),
    'labsOptIn' : IDL.Bool,
    'jurisdiction' : IDL.Opt(Jurisdiction),
    'enabledLibraryIds' : IDL.Vec(IDL.Text),
    'geoLocation' : IDL.Opt(GeoLocation),
  });
  const ProposalStatus = IDL.Variant({
    'expired' : IDL.Null,
    'pending' : IDL.Null,
    'approved' : IDL.Null,
    'rejected' : IDL.Null,
  });
  const TallyResult = IDL.Record({
    'no' : IDL.Nat,
    'yes' : IDL.Nat,
    'abstain' : IDL.Nat,
    'approved' : IDL.Bool,
    'totalPower' : IDL.Nat,
    'quorumReached' : IDL.Bool,
  });
  const RoleAssignment = IDL.Record({
    'action' : IDL.Variant({ 'revoke' : IDL.Null, 'grant' : IDL.Null }),
    'role' : IDL.Text,
    'targetPrincipal' : IDL.Principal,
  });
  const SpaceConfigChange = IDL.Record({
    'field' : IDL.Text,
    'newValue' : IDL.Text,
    'spaceId' : IDL.Text,
  });
  const EconomicAction = IDL.Record({
    'unit' : IDL.Text,
    'recipient' : IDL.Opt(IDL.Principal),
    'actionType' : IDL.Variant({
      'release_escrow' : IDL.Null,
      'dispute' : IDL.Null,
      'approve_payout' : IDL.Null,
    }),
    'entityId' : IDL.Text,
    'amount' : IDL.Float64,
  });
  const WorkflowApproval = IDL.Record({
    'stepId' : IDL.Text,
    'decision' : IDL.Variant({ 'reject' : IDL.Null, 'approve' : IDL.Null }),
    'workflowInstanceId' : IDL.Text,
  });
  const SchemaChange = IDL.Record({
    'changeType' : IDL.Variant({
      'modify_type' : IDL.Null,
      'archive_type' : IDL.Null,
      'add_type' : IDL.Null,
    }),
    'typeId' : IDL.Text,
    'payload' : IDL.Text,
  });
  const TargetLifecyclePhase = IDL.Variant({
    'formalized' : IDL.Null,
    'dormant' : IDL.Null,
    'emergent' : IDL.Null,
    'operational' : IDL.Null,
    'provisional' : IDL.Null,
    'archived' : IDL.Null,
  });
  const LifecycleChange = IDL.Record({
    'institutionId' : IDL.Text,
    'newPhase' : TargetLifecyclePhase,
  });
  const ProposalType = IDL.Variant({
    'role_assignment' : RoleAssignment,
    'space_config' : SpaceConfigChange,
    'economic_action' : EconomicAction,
    'workflow_approval' : WorkflowApproval,
    'schema_change' : SchemaChange,
    'lifecycle_change' : LifecycleChange,
  });
  const Vote = IDL.Record({
    'voter' : IDL.Principal,
    'timestamp' : IDL.Int,
    'choice' : VoteChoice,
    'power' : IDL.Nat,
  });
  const Ballot = IDL.Record({
    'startedAt' : IDL.Int,
    'votes' : IDL.Vec(Vote),
    'proposalId' : IDL.Text,
    'endsAt' : IDL.Int,
  });
  const Proposal = IDL.Record({
    'id' : IDL.Text,
    'status' : ProposalStatus,
    'metadata' : IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text)),
    'tallyResult' : IDL.Opt(TallyResult),
    'strategy' : GovernanceStrategy,
    'createdAt' : IDL.Int,
    'description' : IDL.Text,
    'proposalType' : ProposalType,
    'proposer' : IDL.Principal,
    'votingSession' : IDL.Opt(Ballot),
    'resolvedAt' : IDL.Opt(IDL.Int),
  });
  const TaskView = IDL.Record({
    'startedAt' : IDL.Int,
    'name' : IDL.Text,
    'description' : IDL.Text,
    'instanceId' : IDL.Text,
    'taskId' : IDL.Text,
  });
  const Member = IDL.Record({
    'joinedAt' : IDL.Int,
    'roleIds' : IDL.Vec(IDL.Text),
    'actorId' : ActorID,
  });
  const ForkSource = IDL.Record({
    'actorId' : ActorID,
    'spaceId' : IDL.Text,
    'blockHeight' : IDL.Int,
  });
  const Permission = IDL.Variant({
    'manage_workflow' : IDL.Null,
    'manage_space' : IDL.Null,
    'manage_members' : IDL.Null,
    'create_contribution' : IDL.Null,
    'view_private' : IDL.Null,
    'trigger_step' : IDL.Null,
  });
  const Role__1 = IDL.Record({
    'id' : IDL.Text,
    'permissions' : IDL.Vec(Permission),
    'name' : IDL.Text,
  });
  const Space = IDL.Record({
    'id' : IDL.Text,
    'members' : IDL.Vec(Member),
    'source' : IDL.Opt(ForkSource),
    'owner' : ActorID,
    'name' : IDL.Text,
    'createdAt' : IDL.Int,
    'description' : IDL.Text,
    'visibility' : Visibility,
    'roles' : IDL.Vec(Role__1),
  });
  const MonitorLevel = IDL.Variant({
    'Healthy' : IDL.Null,
    'Critical' : IDL.Null,
    'Warning' : IDL.Null,
  });
  const CanisterStatus = IDL.Record({
    'status' : IDL.Variant({
      'stopped' : IDL.Null,
      'stopping' : IDL.Null,
      'running' : IDL.Null,
    }),
    'memory_size' : IDL.Nat,
    'name' : IDL.Text,
    'canister_id' : IDL.Text,
    'cycles' : IDL.Nat,
    'module_hash' : IDL.Opt(IDL.Text),
  });
  const SystemStatus = IDL.Record({
    'status' : MonitorLevel,
    'metrics' : IDL.Record({
      'error_count_24h' : IDL.Nat,
      'active_workflows' : IDL.Nat,
      'active_users_24h' : IDL.Nat,
    }),
    'last_updated' : IDL.Int,
    'version' : IDL.Text,
    'canisters' : IDL.Vec(CanisterStatus),
    'uptime_seconds' : IDL.Int,
  });
  const TreatyStatus = IDL.Variant({
    'active' : IDL.Null,
    'revoked' : IDL.Null,
    'expired' : IDL.Null,
    'proposed' : IDL.Null,
    'suspended' : IDL.Null,
  });
  const Treaty = IDL.Record({
    'id' : IDL.Text,
    'status' : TreatyStatus,
    'createdAt' : IDL.Int,
    'grantee' : IDL.Text,
    'granter' : IDL.Text,
    'scope' : AccessScope,
    'updatedAt' : IDL.Int,
    'rationale' : IDL.Text,
    'expiry' : IDL.Opt(IDL.Int),
    'revokedAt' : IDL.Opt(IDL.Int),
    'revokedReason' : IDL.Opt(IDL.Text),
  });
  const Role = IDL.Variant({
    'admin' : IDL.Null,
    'editor' : IDL.Null,
    'viewer' : IDL.Null,
  });
  const WorkerConfig = IDL.Record({
    'alg' : IDL.Opt(IDL.Text),
    'workerId' : ActorID,
    'publicKey' : IDL.Vec(IDL.Nat8),
    'registeredAt' : IDL.Int,
    'keyId' : IDL.Opt(IDL.Text),
    'encVersion' : IDL.Opt(IDL.Nat),
  });
  const ExecutionEvent = IDL.Record({
    'stepId' : IDL.Text,
    'action' : IDL.Text,
    'user' : IDL.Opt(IDL.Text),
    'timestamp' : IDL.Int,
    'details' : IDL.Opt(IDL.Text),
  });
  const AllowedRole = IDL.Variant({
    'any' : IDL.Null,
    'permission' : IDL.Text,
    'named' : IDL.Text,
  });
  const GuardOperator = IDL.Variant({
    'contains' : IDL.Null,
    'not_equals' : IDL.Null,
    'less_than' : IDL.Null,
    'greater_than' : IDL.Null,
    'equals' : IDL.Null,
  });
  const TransitionGuard = IDL.Record({
    'field' : IDL.Text,
    'value' : IDL.Text,
    'operator' : GuardOperator,
    'targetStep' : IDL.Text,
  });
  const BranchConfig = IDL.Record({
    'defaultNext' : IDL.Opt(IDL.Text),
    'conditions' : IDL.Vec(TransitionGuard),
  });
  const UserTask = IDL.Record({
    'outputSchema' : IDL.Text,
    'description' : IDL.Text,
  });
  const RepeatConfig = IDL.Record({
    'bodySteps' : IDL.Vec(IDL.Text),
    'maxIterations' : IDL.Nat,
    'exitCondition' : TransitionGuard,
  });
  const SystemOp = IDL.Record({
    'method' : IDL.Text,
    'target' : IDL.Variant({ 'local' : IDL.Null, 'canister' : IDL.Text }),
    'arguments' : IDL.Text,
  });
  const AsyncExternalOp = IDL.Record({
    'payloadSchema' : IDL.Text,
    'timeoutSeconds' : IDL.Nat,
    'requiredCapabilities' : IDL.Vec(IDL.Text),
  });
  const JoinType = IDL.Variant({ 'all' : IDL.Null, 'any' : IDL.Null });
  const ParallelConfig = IDL.Record({
    'forkSteps' : IDL.Vec(IDL.Text),
    'joinStep' : IDL.Text,
    'joinType' : JoinType,
  });
  const SyncStrategy = IDL.Variant({
    'overwrite' : IDL.Null,
    'keep_existing' : IDL.Null,
  });
  const SyncConfig = IDL.Record({
    'libraryId' : IDL.Text,
    'strategy' : SyncStrategy,
  });
  const DelayConfig = IDL.Record({
    'durationSeconds' : IDL.Nat,
    'nextStep' : IDL.Text,
  });
  const StepType = IDL.Variant({
    'branch' : BranchConfig,
    'user_task' : UserTask,
    'repeat_loop' : RepeatConfig,
    'system_op' : SystemOp,
    'async_external_op' : AsyncExternalOp,
    'parallel' : ParallelConfig,
    'sync_skills' : SyncConfig,
    'delay' : DelayConfig,
    'await_event' : IDL.Text,
    'sequence' : IDL.Null,
  });
  const WorkflowStep = IDL.Record({
    'id' : IDL.Text,
    'nextSteps' : IDL.Vec(IDL.Text),
    'name' : IDL.Text,
    'allowedRoles' : IDL.Vec(AllowedRole),
    'stepType' : StepType,
  });
  const WorkflowStateView = IDL.Record({
    'status' : IDL.Text,
    'context' : IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text)),
    'history' : IDL.Vec(ExecutionEvent),
    'instanceId' : IDL.Text,
    'currentStep' : IDL.Opt(WorkflowStep),
    'definitionName' : IDL.Text,
    'formSchema' : IDL.Opt(IDL.Text),
  });
  const SystemOp__1 = IDL.Record({
    'method' : IDL.Text,
    'target' : IDL.Variant({ 'local' : IDL.Null, 'canister' : IDL.Principal }),
    'arguments' : IDL.Vec(IDL.Nat8),
  });
  const StepType__1 = IDL.Variant({
    'branch' : BranchConfig,
    'user_task' : UserTask,
    'repeat_loop' : RepeatConfig,
    'system_op' : SystemOp__1,
    'async_external_op' : AsyncExternalOp,
    'parallel' : ParallelConfig,
    'sync_skills' : SyncConfig,
    'delay' : DelayConfig,
    'await_event' : IDL.Text,
    'sequence' : IDL.Null,
  });
  const WorkflowStep__1 = IDL.Record({
    'id' : IDL.Text,
    'nextSteps' : IDL.Vec(IDL.Text),
    'name' : IDL.Text,
    'allowedRoles' : IDL.Vec(AllowedRole),
    'stepType' : StepType__1,
  });
  const Trigger__1 = IDL.Variant({
    'on_entity_create' : IDL.Null,
    'manual' : IDL.Null,
  });
  const WorkflowDefinition__1 = IDL.Record({
    'id' : IDL.Text,
    'name' : IDL.Text,
    'steps' : IDL.Vec(WorkflowStep__1),
    'triggers' : IDL.Vec(Trigger__1),
  });
  const Library = IDL.Record({
    'id' : IDL.Text,
    'workflows' : IDL.Vec(WorkflowDefinition__1),
    'description' : IDL.Text,
    'version' : IDL.Text,
    'entities' : IDL.Vec(Entity),
    'dependencies' : IDL.Vec(IDL.Text),
    'relationships' : IDL.Vec(Relationship),
  });
  const KnowledgeSearchFilters = IDL.Record({
    'tags' : IDL.Vec(IDL.Text),
    'source_type' : IDL.Opt(IDL.Text),
    'produced_by_agent' : IDL.Opt(IDL.Text),
    'perspective_scope' : IDL.Opt(IDL.Text),
    'source_version_id' : IDL.Opt(IDL.Text),
    'space_id' : IDL.Opt(IDL.Text),
  });
  const KnowledgeAskRequest = IDL.Record({
    'filters' : IDL.Opt(KnowledgeSearchFilters),
    'max_context_chunks' : IDL.Nat,
    'question' : IDL.Text,
    'limit' : IDL.Nat,
    'require_provenance' : IDL.Bool,
    'retrieval_mode' : IDL.Text,
  });
  const KnowledgeAskCitation = IDL.Record({
    'id' : IDL.Text,
    'score' : IDL.Float64,
    'source_ref' : IDL.Opt(IDL.Text),
  });
  const KnowledgeAskResponse = IDL.Record({
    'model' : IDL.Text,
    'answer' : IDL.Text,
    'trace_id' : IDL.Text,
    'citations' : IDL.Vec(KnowledgeAskCitation),
  });
  const KnowledgeAskResult = IDL.Variant({
    'ok' : KnowledgeAskResponse,
    'err' : IDL.Text,
  });
  const KnowledgeSearchRequest = IDL.Record({
    'filters' : IDL.Opt(KnowledgeSearchFilters),
    'limit' : IDL.Nat,
    'query_text' : IDL.Text,
    'retrieval_mode' : IDL.Text,
    'diagnostics' : IDL.Bool,
  });
  const KnowledgeSearchResult = IDL.Record({
    'id' : IDL.Text,
    'tags' : IDL.Vec(IDL.Text),
    'score' : IDL.Float64,
    'source_ref' : IDL.Opt(IDL.Text),
    'source_type' : IDL.Opt(IDL.Text),
    'space_id' : IDL.Opt(IDL.Text),
  });
  const Result_2 = IDL.Variant({ 'ok' : IDL.Nat, 'err' : IDL.Text });
  const ProjectStatus = IDL.Variant({
    'Active' : IDL.Null,
    'Archived' : IDL.Null,
    'Deleted' : IDL.Null,
  });
  const ProjectMetadata = IDL.Record({
    'id' : IDL.Text,
    'status' : ProjectStatus,
    'owner' : IDL.Principal,
    'name' : IDL.Text,
    'createdAt' : IDL.Int,
    'description' : IDL.Text,
    'collaborators' : IDL.Vec(IDL.Principal),
    'schemaId' : IDL.Text,
    'canisterId' : IDL.Principal,
  });
  const TraceOutcome = IDL.Variant({
    'failure' : IDL.Text,
    'success' : IDL.Null,
    'deferred' : IDL.Null,
  });
  const DecisionTrace = IDL.Record({
    'action' : IDL.Text,
    'agentId' : IDL.Text,
    'reasoning' : IDL.Text,
    'confidenceScore' : IDL.Float64,
    'traceId' : IDL.Text,
    'timestamp' : IDL.Int,
    'input' : IDL.Text,
    'outcome' : TraceOutcome,
  });
  const NodeRole = IDL.Variant({
    'ml' : IDL.Null,
    'data' : IDL.Null,
    'ingest' : IDL.Null,
    'master' : IDL.Null,
    'remote_cluster_client' : IDL.Null,
  });
  const Node = IDL.Record({
    'ephemeral_id' : IDL.Text,
    'transport_address' : IDL.Text,
    'name' : IDL.Text,
    'last_heartbeat' : IDL.Int,
    'attributes' : IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text)),
    'roles' : IDL.Vec(NodeRole),
  });
  const ShardState = IDL.Variant({
    'started' : IDL.Null,
    'unassigned' : IDL.Null,
    'relocating' : IDL.Null,
    'initializing' : IDL.Null,
  });
  const ShardRouting = IDL.Record({
    'node_id' : IDL.Text,
    'shard_id' : IDL.Nat,
    'primary' : IDL.Bool,
    'state' : ShardState,
    'relocating_node_id' : IDL.Opt(IDL.Text),
    'index' : IDL.Text,
  });
  const IndexRoutingTable = IDL.Record({
    'shards' : IDL.Vec(IDL.Tuple(IDL.Nat, ShardRouting)),
  });
  const AgentSwarmState = IDL.Record({
    'master_node' : IDL.Principal,
    'routing_table' : IDL.Vec(IDL.Tuple(IDL.Text, IndexRoutingTable)),
    'version' : IDL.Nat,
    'nodes' : IDL.Vec(IDL.Tuple(IDL.Text, Node)),
    'state_uuid' : IDL.Text,
  });
  const ClusterStateDiff = IDL.Record({
    'new_version' : IDL.Nat,
    'old_version' : IDL.Nat,
    'full_state' : IDL.Opt(AgentSwarmState),
  });
  const GovernanceLevel = IDL.Variant({
    'supervised' : IDL.Null,
    'autonomous' : IDL.Null,
    'restricted' : IDL.Null,
  });
  const AgentConfig = IDL.Record({
    'model' : IDL.Text,
    'capabilities' : IDL.Vec(IDL.Text),
    'temperature' : IDL.Float64,
    'name' : IDL.Text,
    'agentId' : IDL.Text,
    'governanceLevel' : GovernanceLevel,
    'systemPrompt' : IDL.Text,
    'maxTokens' : IDL.Nat,
  });
  const Trigger = IDL.Variant({
    'on_entity_create' : IDL.Text,
    'manual' : IDL.Null,
  });
  const WorkflowDefinition = IDL.Record({
    'id' : IDL.Text,
    'name' : IDL.Text,
    'steps' : IDL.Vec(WorkflowStep),
    'triggers' : IDL.Vec(Trigger),
  });
  const FlowLayoutInput = IDL.Record({
    'handle_positions' : IDL.Vec(FlowHandlePosition),
    'workflow_id' : IDL.Text,
    'node_positions' : IDL.Vec(FlowNodePosition),
    'commit_message' : IDL.Opt(IDL.Text),
    'graph_version' : IDL.Text,
    'commit_tag' : IDL.Opt(IDL.Text),
    'collapsed_groups' : IDL.Vec(IDL.Text),
  });
  const UpdateInstitutionRequest = IDL.Record({
    'id' : IDL.Text,
    'title' : IDL.Opt(IDL.Text),
    'lifecyclePhase' : IDL.Opt(LifecyclePhase),
    'charterRefs' : IDL.Opt(IDL.Vec(IDL.Text)),
    'scope' : IDL.Opt(IDL.Text),
    'summary' : IDL.Opt(IDL.Text),
    'intent' : IDL.Opt(IDL.Text),
    'stewards' : IDL.Opt(IDL.Vec(IDL.Principal)),
    'affiliatedSpaces' : IDL.Opt(IDL.Vec(IDL.Text)),
    'confidence' : IDL.Opt(IDL.Float64),
  });
  const FileId = IDL.Text;
  const StorageProvider = IDL.Variant({
    'mcp' : IDL.Record({ 'serverName' : IDL.Text }),
    'internal' : IDL.Record({
      'moduleName' : IDL.Text,
      'scopeId' : IDL.Opt(IDL.Text),
    }),
    'local' : IDL.Null,
    'canister' : IDL.Principal,
    'external' : IDL.Record({ 'protocol' : IDL.Text, 'bucket' : IDL.Text }),
  });
  const Path = IDL.Text;
  const NodeType = IDL.Variant({
    'File' : IDL.Null,
    'Mount' : IDL.Null,
    'Directory' : IDL.Null,
  });
  const FileMetadata = IDL.Record({
    'id' : FileId,
    'provider' : StorageProvider,
    'contentRef' : IDL.Opt(IDL.Text),
    'name' : IDL.Text,
    'createdAt' : IDL.Int,
    'path' : Path,
    'size' : IDL.Nat,
    'mimeType' : IDL.Text,
    'updatedAt' : IDL.Int,
    'nodeType' : NodeType,
  });
  const VFSResult = IDL.Variant({ 'ok' : IDL.Text, 'err' : IDL.Text });
  return IDL.Service({
    'activateTreaty' : IDL.Func([IDL.Text], [Result], []),
    'addKey' : IDL.Func(
        [
          IDL.Text,
          IDL.Text,
          IDL.Opt(IDL.Text),
          IDL.Opt(IDL.Text),
          IDL.Vec(IDL.Nat8),
        ],
        [],
        [],
      ),
    'addKeyV2' : IDL.Func(
        [
          IDL.Text,
          IDL.Text,
          IDL.Opt(IDL.Text),
          IDL.Opt(IDL.Text),
          IDL.Vec(IDL.Nat8),
          IDL.Opt(IDL.Vec(IDL.Nat8)),
          IDL.Opt(IDL.Text),
          IDL.Opt(IDL.Nat),
          IDL.Opt(IDL.Text),
        ],
        [],
        [],
      ),
    'adoptCommons' : IDL.Func([IDL.Text, IDL.Text, AdoptionMode], [Result], []),
    'appendChatMessage' : IDL.Func([IDL.Text, IDL.Text], [], []),
    'batchIngest' : IDL.Func(
        [IDL.Vec(Entity), IDL.Vec(Relationship)],
        [IDL.Text],
        [],
      ),
    'bootstrap' : IDL.Func([], [IDL.Text], []),
    'castVote' : IDL.Func([IDL.Text, VoteChoice], [Result], []),
    'checkStatus' : IDL.Func([], [IDL.Text], ['query']),
    'checkTreatyAccess' : IDL.Func(
        [IDL.Text, IDL.Text, IDL.Text],
        [IDL.Bool],
        ['query'],
      ),
    'createBook' : IDL.Func([IDL.Text, IDL.Text], [IDL.Text], []),
    'createDiscussion' : IDL.Func(
        [IDL.Text, IDL.Text, GatingPolicy],
        [IDL.Text],
        [],
      ),
    'createInstitution' : IDL.Func([CreateInstitutionRequest], [Result_1], []),
    'createProject' : IDL.Func([IDL.Text, IDL.Text], [Result_1], []),
    'createSpace' : IDL.Func([IDL.Text, IDL.Text, Visibility], [Result_1], []),
    'createTreaty' : IDL.Func(
        [IDL.Text, AccessScope, IDL.Text, IDL.Opt(IDL.Int)],
        [Result_1],
        [],
      ),
    'deleteUI' : IDL.Func([IDL.Text], [], []),
    'detachCommons' : IDL.Func([IDL.Text, IDL.Text], [Result], []),
    'dev_vector_search' : IDL.Func([IDL.Text], [IDL.Vec(IDL.Text)], ['query']),
    'executeKip' : IDL.Func([IDL.Text], [Result_1], []),
    'execute_kip_mutation' : IDL.Func([IDL.Text], [KipResult], []),
    'execute_kip_mutation_scoped' : IDL.Func(
        [IDL.Text, IDL.Text],
        [KipResult],
        [],
      ),
    'extractReferences' : IDL.Func(
        [IDL.Text],
        [IDL.Vec(ExtractedRef)],
        ['query'],
      ),
    'forkInstitution' : IDL.Func(
        [IDL.Text, IDL.Text, IDL.Text],
        [Result_1],
        [],
      ),
    'forkSpace' : IDL.Func([IDL.Text], [Result_1], []),
    'getAffiliatedInstitutions' : IDL.Func(
        [IDL.Text],
        [IDL.Vec(Institution)],
        ['query'],
      ),
    'getAgentTraces' : IDL.Func([IDL.Text], [IDL.Vec(IDL.Text)], []),
    'getAllEntities' : IDL.Func([], [IDL.Vec(Entity)], ['query']),
    'getAllRelationships' : IDL.Func([], [IDL.Vec(Relationship)], ['query']),
    'getAllWorkflowInstances' : IDL.Func(
        [],
        [IDL.Vec(WorkflowInstance)],
        ['query'],
      ),
    'getBook' : IDL.Func([IDL.Text], [IDL.Opt(Book)], ['query']),
    'getChatHistory' : IDL.Func([], [IDL.Vec(ChatMessage)], ['query']),
    'getChronicleEvents' : IDL.Func(
        [IDL.Opt(IDL.Int), IDL.Opt(IDL.Int), IDL.Nat],
        [IDL.Vec(ChronicleEvent)],
        ['query'],
      ),
    'getCommonsAdoptions' : IDL.Func(
        [IDL.Text],
        [IDL.Vec(CommonsAdoption)],
        ['query'],
      ),
    'getCommonsEnforcementMode' : IDL.Func(
        [],
        [CommonsEnforcementMode],
        ['query'],
      ),
    'getCommonsForSpace' : IDL.Func(
        [IDL.Text],
        [IDL.Vec(Institution)],
        ['query'],
      ),
    'getCommonsRuleset' : IDL.Func(
        [IDL.Text],
        [IDL.Opt(CommonsRuleset)],
        ['query'],
      ),
    'getConstellationEdges' : IDL.Func(
        [IDL.Text],
        [IDL.Vec(GraphEdge)],
        ['query'],
      ),
    'getContextObservations' : IDL.Func(
        [IDL.Vec(IDL.Text), IDL.Vec(IDL.Text)],
        [IDL.Vec(Entity)],
        ['query'],
      ),
    'getDiscussion' : IDL.Func([IDL.Text], [IDL.Opt(Discussion)], ['query']),
    'getEnabledLibraryIds' : IDL.Func([], [IDL.Vec(IDL.Text)], ['query']),
    'getEntityEvents' : IDL.Func(
        [IDL.Text, IDL.Nat],
        [IDL.Vec(ChronicleEvent)],
        ['query'],
      ),
    'getFlowLayout' : IDL.Func(
        [IDL.Text, IDL.Opt(IDL.Text)],
        [IDL.Opt(FlowLayout)],
        ['query'],
      ),
    'getFlowLayoutHistory' : IDL.Func(
        [IDL.Text, IDL.Opt(IDL.Text), IDL.Opt(IDL.Nat)],
        [IDL.Vec(FlowLayout)],
        ['query'],
      ),
    'getGovernanceStrategy' : IDL.Func([], [GovernanceStrategy], ['query']),
    'getGovernanceUI' : IDL.Func(
        [IDL.Text, IDL.Opt(IDL.Text)],
        [IDL.Text],
        ['query'],
      ),
    'getIncomingConstellationEdges' : IDL.Func(
        [IDL.Text],
        [IDL.Vec(GraphEdge)],
        ['query'],
      ),
    'getInstalledLibraries' : IDL.Func(
        [],
        [IDL.Vec(LibraryManifest)],
        ['query'],
      ),
    'getInstitution' : IDL.Func([IDL.Text], [IDL.Opt(Institution)], ['query']),
    'getInstitutionCharters' : IDL.Func(
        [IDL.Text],
        [IDL.Vec(IDL.Text)],
        ['query'],
      ),
    'getInstitutionLineage' : IDL.Func(
        [IDL.Text, IDL.Nat],
        [IDL.Vec(LineageNode)],
        ['query'],
      ),
    'getInstitutionsByPhase' : IDL.Func(
        [LifecyclePhase],
        [IDL.Vec(Institution)],
        ['query'],
      ),
    'getInstitutionsBySpace' : IDL.Func(
        [IDL.Text],
        [IDL.Vec(Institution)],
        ['query'],
      ),
    'getLogs' : IDL.Func(
        [IDL.Opt(LogSource), IDL.Opt(LogLevel), IDL.Nat],
        [IDL.Vec(LogEntry)],
        ['query'],
      ),
    'getMyKeys' : IDL.Func([IDL.Opt(IDL.Text)], [IDL.Vec(KeyEntry)], ['query']),
    'getMyProfile' : IDL.Func([], [UserProfile], []),
    'getPendingProposals' : IDL.Func([], [IDL.Vec(Proposal)], ['query']),
    'getPendingTasks' : IDL.Func(
        [IDL.Text],
        [IDL.Vec(ExternalRequest)],
        ['query'],
      ),
    'getPendingTasksV2' : IDL.Func([IDL.Text], [IDL.Vec(TaskView)], ['query']),
    'getProposal' : IDL.Func([IDL.Text], [IDL.Opt(Proposal)], ['query']),
    'getRegistryId' : IDL.Func([], [IDL.Principal], ['query']),
    'getSchemaExplorerSurface' : IDL.Func(
        [IDL.Variant({ 'list' : IDL.Null, 'detail' : IDL.Text })],
        [IDL.Text],
        ['query'],
      ),
    'getSchemaRegistryId' : IDL.Func([], [IDL.Principal], ['query']),
    'getSpace' : IDL.Func([IDL.Text], [IDL.Opt(Space)], []),
    'getSystemStatus' : IDL.Func([], [SystemStatus], ['query']),
    'getTreaties' : IDL.Func([], [IDL.Vec(Treaty)], ['query']),
    'getTreaty' : IDL.Func([IDL.Text], [IDL.Opt(Treaty)], ['query']),
    'getUserKeys' : IDL.Func(
        [IDL.Principal, IDL.Opt(IDL.Text)],
        [IDL.Vec(KeyEntry)],
        ['query'],
      ),
    'getUserRoles' : IDL.Func([IDL.Principal], [IDL.Vec(Role)], ['query']),
    'getWorkerConfig' : IDL.Func([], [IDL.Opt(WorkerConfig)], ['query']),
    'getWorkerKey' : IDL.Func([], [IDL.Opt(IDL.Vec(IDL.Nat8))], ['query']),
    'getWorkflowStateV2' : IDL.Func(
        [IDL.Text],
        [IDL.Opt(WorkflowStateView)],
        ['query'],
      ),
    'grantRole' : IDL.Func([IDL.Principal, Role], [], []),
    'installLibrary' : IDL.Func([Library], [], []),
    'install_library_kip' : IDL.Func(
        [IDL.Text, IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text))],
        [IDL.Text],
        [],
      ),
    'knowledgeAsk' : IDL.Func(
        [KnowledgeAskRequest],
        [KnowledgeAskResult],
        ['query'],
      ),
    'knowledgeSearch' : IDL.Func(
        [KnowledgeSearchRequest],
        [IDL.Vec(KnowledgeSearchResult)],
        ['query'],
      ),
    'linkReferences' : IDL.Func([IDL.Text, IDL.Text], [Result_2], []),
    'listCommons' : IDL.Func([], [IDL.Vec(Institution)], ['query']),
    'listProjects' : IDL.Func([], [IDL.Vec(ProjectMetadata)], []),
    'logAgentTrace' : IDL.Func([DecisionTrace], [Result_1], []),
    'nexus_heartbeat' : IDL.Func(
        [IDL.Text, Node, IDL.Nat],
        [IDL.Opt(ClusterStateDiff)],
        [],
      ),
    'postMessage' : IDL.Func(
        [IDL.Text, IDL.Text, IDL.Opt(IDL.Text)],
        [Result_1],
        [],
      ),
    'processAIQuery' : IDL.Func([IDL.Text], [IDL.Text], []),
    'publishEdition' : IDL.Func([IDL.Text, IDL.Text], [IDL.Text], []),
    'recallObservations' : IDL.Func(
        [IDL.Text, IDL.Opt(IDL.Text), IDL.Nat],
        [IDL.Vec(Entity)],
        ['query'],
      ),
    'recordLegacyKeyUsage' : IDL.Func(
        [IDL.Principal, IDL.Text, IDL.Text],
        [IDL.Bool],
        [],
      ),
    'recordReflection' : IDL.Func([IDL.Text], [IDL.Text], []),
    'registerAgent' : IDL.Func([AgentConfig], [Result_1], []),
    'registerWorker' : IDL.Func([IDL.Vec(IDL.Nat8)], [], []),
    'registerWorkerV2' : IDL.Func(
        [
          IDL.Vec(IDL.Nat8),
          IDL.Opt(IDL.Text),
          IDL.Opt(IDL.Text),
          IDL.Opt(IDL.Nat),
        ],
        [],
        [],
      ),
    'registerWorkflowV2' : IDL.Func([WorkflowDefinition], [], []),
    'removeKey' : IDL.Func([IDL.Text], [], []),
    'resolveProposal' : IDL.Func(
        [IDL.Text, IDL.Variant({ 'reject' : IDL.Null, 'approve' : IDL.Null })],
        [Result],
        [],
      ),
    'revokeRole' : IDL.Func([IDL.Principal, Role], [], []),
    'revokeTreaty' : IDL.Func([IDL.Text, IDL.Text], [Result], []),
    'runSchemaHygiene' : IDL.Func([IDL.Nat, IDL.Nat], [IDL.Text], []),
    'saveUI' : IDL.Func([SavedUI], [], []),
    'setCommonsEnforcementMode' : IDL.Func(
        [CommonsEnforcementMode],
        [Result],
        [],
      ),
    'setFeatureFlag' : IDL.Func([IDL.Text, IDL.Bool], [], []),
    'setFlowLayout' : IDL.Func([FlowLayoutInput], [FlowLayout], []),
    'setGovernanceStrategy' : IDL.Func([GovernanceStrategy], [Result], []),
    'setLabsOptIn' : IDL.Func([IDL.Bool], [], []),
    'signalWorkflowV2' : IDL.Func(
        [IDL.Text, IDL.Text, IDL.Text],
        [Result_1],
        [],
      ),
    'startWorkflowV2' : IDL.Func(
        [IDL.Text, IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text))],
        [Result_1],
        [],
      ),
    'submitLog' : IDL.Func(
        [
          LogSource,
          LogLevel,
          IDL.Text,
          IDL.Opt(IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text))),
        ],
        [],
        [],
      ),
    'submitProposal' : IDL.Func(
        [
          ProposalType,
          IDL.Text,
          IDL.Vec(IDL.Tuple(IDL.Text, IDL.Text)),
          IDL.Opt(IDL.Text),
        ],
        [Result_1],
        [],
      ),
    'submitTaskOutcome' : IDL.Func(
        [IDL.Text, IDL.Text, IDL.Text],
        [IDL.Text],
        [],
      ),
    'suspendTreaty' : IDL.Func([IDL.Text], [Result], []),
    'toggleLibrary' : IDL.Func([IDL.Text, IDL.Bool], [], []),
    'updateBookStructure' : IDL.Func([IDL.Text, BookStructure], [], []),
    'updateInstitution' : IDL.Func([UpdateInstitutionRequest], [Result], []),
    'updateKeyV2ForUser' : IDL.Func(
        [
          IDL.Principal,
          IDL.Text,
          IDL.Vec(IDL.Nat8),
          IDL.Opt(IDL.Vec(IDL.Nat8)),
          IDL.Opt(IDL.Text),
          IDL.Opt(IDL.Nat),
          IDL.Opt(IDL.Text),
        ],
        [IDL.Bool],
        [],
      ),
    'upsertCommonsRuleset' : IDL.Func([IDL.Text, CommonsRuleset], [Result], []),
    'vfs_list' : IDL.Func([IDL.Text], [IDL.Vec(FileMetadata)], ['query']),
    'vfs_mkdir' : IDL.Func([IDL.Text], [VFSResult], []),
    'vfs_mount' : IDL.Func([IDL.Text, IDL.Principal], [VFSResult], []),
    'vfs_read' : IDL.Func([IDL.Text], [VFSResult], []),
    'vfs_write' : IDL.Func([IDL.Text, IDL.Text, IDL.Text], [VFSResult], []),
  });
};
export const init = ({ IDL }) => { return []; };
