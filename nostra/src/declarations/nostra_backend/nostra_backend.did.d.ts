import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export type AccessOp = { 'cite' : null } |
  { 'fork' : null } |
  { 'read' : null } |
  { 'traverse' : null };
export interface AccessScope {
  'entityTypes' : Array<string>,
  'operations' : Array<AccessOp>,
  'depth' : bigint,
}
export type ActorID = string;
export type AdoptionMode = { 'pinned' : null } |
  { 'adopted' : null };
export interface AgentConfig {
  'model' : string,
  'capabilities' : Array<string>,
  'temperature' : number,
  'name' : string,
  'agentId' : string,
  'governanceLevel' : GovernanceLevel,
  'systemPrompt' : string,
  'maxTokens' : bigint,
}
export interface AgentSwarmState {
  'master_node' : Principal,
  'routing_table' : Array<[string, IndexRoutingTable]>,
  'version' : bigint,
  'nodes' : Array<[string, Node]>,
  'state_uuid' : string,
}
export type AllowedRole = { 'any' : null } |
  { 'permission' : string } |
  { 'named' : string };
export interface AsyncExternalOp {
  'payloadSchema' : string,
  'timeoutSeconds' : bigint,
  'requiredCapabilities' : Array<string>,
}
export interface Ballot {
  'startedAt' : bigint,
  'votes' : Array<Vote>,
  'proposalId' : string,
  'endsAt' : bigint,
}
export interface Book {
  'id' : BookId,
  'title' : string,
  'workflowIds' : Array<string>,
  'isbn' : [] | [string],
  'structure' : BookStructure,
  'createdAt' : bigint,
  'agentIds' : Array<string>,
  'author' : Principal,
  'coverImage' : [] | [string],
  'updatedAt' : bigint,
  'spaceId' : string,
  'editions' : Array<Edition>,
  'subtitle' : [] | [string],
}
export type BookId = string;
export interface BookStructure { 'nodes' : Array<StructureNode> }
export interface BranchConfig {
  'defaultNext' : [] | [string],
  'conditions' : Array<TransitionGuard>,
}
export interface CanisterStatus {
  'status' : { 'stopped' : null } |
    { 'stopping' : null } |
    { 'running' : null },
  'memory_size' : bigint,
  'name' : string,
  'canister_id' : string,
  'cycles' : bigint,
  'module_hash' : [] | [string],
}
export interface ChatMessage {
  'content' : string,
  'context' : [] | [string],
  'messageType' : ChatMessageType,
  'timestamp' : bigint,
}
export type ChatMessageType = { 'ai' : null } |
  { 'user' : null };
export interface ChronicleEvent {
  'id' : string,
  'libraryId' : [] | [string],
  'metadata' : Array<[string, string]>,
  'description' : string,
  'actorId' : string,
  'affectedEntities' : Array<string>,
  'timestamp' : bigint,
  'eventType' : ChronicleEventType,
}
export type ChronicleEventType = { 'relationship_formed' : null } |
  { 'workflow_failed' : null } |
  { 'entity_updated' : null } |
  { 'role_granted' : null } |
  { 'proposal_approved' : null } |
  { 'workflow_completed' : null } |
  { 'proposal_rejected' : null } |
  { 'proposal_submitted' : null } |
  { 'workflow_started' : null } |
  { 'entity_archived' : null } |
  { 'library_forked' : null } |
  { 'entity_created' : null } |
  { 'library_enabled' : null } |
  { 'library_merged' : null } |
  { 'workflow_step_completed' : null } |
  { 'library_disabled' : null } |
  { 'role_revoked' : null } |
  { 'relationship_removed' : null } |
  { 'library_installed' : null };
export interface ClusterStateDiff {
  'new_version' : bigint,
  'old_version' : bigint,
  'full_state' : [] | [AgentSwarmState],
}
export interface CommonsAdoption {
  'mode' : AdoptionMode,
  'version' : [] | [string],
  'spaceId' : string,
  'commonsId' : string,
}
export type CommonsEnforcementMode = { 'shadow' : null } |
  { 'warnOrBlock' : null };
export interface CommonsRuleset {
  'commonsVersion' : string,
  'commonsId' : string,
  'rules' : Array<IntegrityRule>,
}
export type Constraint = { 'mustExist' : null } |
  { 'minCount' : bigint } |
  { 'noCycles' : null } |
  { 'requiresConstitutionalReference' : null } |
  { 'mustNotExist' : null } |
  { 'noConflicts' : null } |
  { 'maxCount' : bigint };
export type ContributionId = string;
export interface ContributionRef {
  'id' : ContributionId,
  'specificVersion' : [] | [string],
}
export interface CreateInstitutionRequest {
  'parentInstitutionId' : [] | [string],
  'title' : string,
  'lifecyclePhase' : [] | [LifecyclePhase],
  'governanceStrategy' : [] | [GovernanceStrategy],
  'charterRefs' : [] | [Array<string>],
  'scope' : string,
  'summary' : string,
  'spaceId' : string,
  'intent' : string,
  'stewards' : [] | [Array<Principal>],
  'affiliatedSpaces' : [] | [Array<string>],
  'confidence' : [] | [number],
}
export interface DecisionTrace {
  'action' : string,
  'agentId' : string,
  'reasoning' : string,
  'confidenceScore' : number,
  'traceId' : string,
  'timestamp' : bigint,
  'input' : string,
  'outcome' : TraceOutcome,
}
export interface DelayConfig { 'durationSeconds' : bigint, 'nextStep' : string }
export type Direction = { 'incoming' : null } |
  { 'outgoing' : null };
export interface Discussion {
  'id' : string,
  'participants' : Array<Principal>,
  'topic' : string,
  'messages' : Array<Message>,
  'lastActivity' : bigint,
  'gatingPolicy' : GatingPolicy,
  'targetEntityId' : string,
}
export interface EconomicAction {
  'unit' : string,
  'recipient' : [] | [Principal],
  'actionType' : { 'release_escrow' : null } |
    { 'dispute' : null } |
    { 'approve_payout' : null },
  'entityId' : string,
  'amount' : number,
}
export interface EdgeSelector {
  'direction' : Direction,
  'relationType' : string,
}
export interface Edition {
  'id' : EditionId,
  'frozenStructure' : BookStructure,
  'citationHash' : string,
  'name' : string,
  'publishedAt' : bigint,
}
export type EditionId = string;
export interface Entity {
  'id' : string,
  'libraryId' : [] | [string],
  'name' : string,
  'tags' : Array<string>,
  'description' : string,
  'creatorActorId' : [] | [string],
  'logRefs' : [] | [Array<string>],
  'scopeId' : [] | [string],
  'attributes' : Array<[string, string]>,
  'creatorAddress' : [] | [string],
  'timestamp' : bigint,
  'entityType' : EntityType,
}
export type EntityType = { 'protocol' : null } |
  { 'service' : null } |
  { 'cryptography' : null } |
  { 'review' : null } |
  { 'model' : null } |
  { 'component' : null } |
  { 'decision' : null } |
  { 'deliverable' : null } |
  { 'feature' : null } |
  { 'cryptoAsset' : null } |
  { 'economy' : null } |
  { 'pledge' : null } |
  { 'person' : null } |
  { 'library' : null } |
  { 'book' : null } |
  { 'dpub' : null } |
  { 'institution' : null } |
  { 'idea' : null } |
  { 'question' : null } |
  { 'comment' : null } |
  { 'poll' : null } |
  { 'post' : null } |
  { 'artifact' : null } |
  { 'mediaEssay' : null } |
  { 'security' : null } |
  { 'bounty' : null } |
  { 'essay' : null } |
  { 'event' : null } |
  { 'infrastructure' : null } |
  { 'credentialReference' : null } |
  { 'assetReference' : null } |
  { 'observation' : null } |
  { 'proposal' : null } |
  { 'discussion' : null } |
  { 'initiative' : null } |
  { 'milestone' : null } |
  { 'organization' : null } |
  { 'issue' : null } |
  { 'report' : null } |
  { 'chapter' : null } |
  { 'governanceSystem' : null } |
  { 'reflection' : null } |
  { 'developmentTool' : null } |
  { 'project' : null };
export interface ExecutionEvent {
  'stepId' : string,
  'action' : string,
  'user' : [] | [string],
  'timestamp' : bigint,
  'details' : [] | [string],
}
export interface ExternalRequest {
  'stepId' : string,
  'startedAt' : bigint,
  'requestId' : string,
  'instanceId' : string,
  'jurisdiction' : [] | [Jurisdiction],
  'geoLocation' : [] | [GeoLocation],
  'payload' : Array<[string, string]>,
}
export interface ExtractedRef { 'targetText' : string, 'refType' : RefType }
export type FileId = string;
export interface FileMetadata {
  'id' : FileId,
  'provider' : StorageProvider,
  'contentRef' : [] | [string],
  'name' : string,
  'createdAt' : bigint,
  'path' : Path,
  'size' : bigint,
  'mimeType' : string,
  'updatedAt' : bigint,
  'nodeType' : NodeType,
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
  'commit_message' : [] | [string],
  'graph_version' : string,
  'commit_tag' : [] | [string],
  'collapsed_groups' : Array<string>,
}
export interface FlowLayoutInput {
  'handle_positions' : Array<FlowHandlePosition>,
  'workflow_id' : string,
  'node_positions' : Array<FlowNodePosition>,
  'commit_message' : [] | [string],
  'graph_version' : string,
  'commit_tag' : [] | [string],
  'collapsed_groups' : Array<string>,
}
export interface FlowNodePosition {
  'x' : bigint,
  'y' : bigint,
  'node_id' : string,
}
export interface ForkSource {
  'actorId' : ActorID,
  'spaceId' : string,
  'blockHeight' : bigint,
}
export type GatingPolicy = { 'open' : null } |
  { 'member_only' : null } |
  { 'reflection_required' : null };
export interface GeoLocation {
  'latitude' : number,
  'precision' : [] | [number],
  'longitude' : number,
}
export type GovernanceLevel = { 'supervised' : null } |
  { 'autonomous' : null } |
  { 'restricted' : null };
export type GovernanceStrategy = {
    'voting' : {
      'votingSystem' : { 'token_weighted' : null } |
        { 'simple_majority' : null },
      'config' : VotingConfig,
    }
  } |
  { 'owner_dictator' : null };
export interface GraphEdge {
  'id' : string,
  'to_id' : string,
  'type' : string,
  'from_id' : string,
  'origin_canister_id' : Principal,
  'timestamp' : bigint,
  'confidence' : number,
  'scope_id' : [] | [string],
}
export type GuardOperator = { 'contains' : null } |
  { 'not_equals' : null } |
  { 'less_than' : null } |
  { 'greater_than' : null } |
  { 'equals' : null };
export interface HistoryEntry {
  'stepId' : string,
  'action' : string,
  'user' : ActorID,
  'timestamp' : bigint,
}
export interface IndexRoutingTable { 'shards' : Array<[bigint, ShardRouting]> }
export type InstanceStatus = { 'active' : null } |
  { 'waiting_for_external' : ExternalRequest } |
  { 'completed' : null } |
  { 'failed' : string };
export interface Institution {
  'id' : string,
  'parentInstitutionId' : [] | [string],
  'title' : string,
  'lifecyclePhase' : LifecyclePhase,
  'createdAt' : bigint,
  'createdBy' : Principal,
  'governanceStrategy' : [] | [GovernanceStrategy],
  'charterRefs' : Array<string>,
  'scope' : string,
  'version' : bigint,
  'summary' : string,
  'updatedAt' : bigint,
  'spaceId' : string,
  'previousVersionId' : [] | [string],
  'intent' : string,
  'stewards' : Array<Principal>,
  'affiliatedSpaces' : Array<string>,
  'confidence' : number,
}
export interface IntegrityPredicate {
  'relation' : [] | [EdgeSelector],
  'constraint' : Constraint,
  'target' : NodeSelector,
}
export interface IntegrityRule {
  'id' : string,
  'name' : string,
  'description' : string,
  'scope' : IntegrityScope,
  'severity' : Severity,
  'remediationHint' : [] | [string],
  'predicate' : IntegrityPredicate,
}
export type IntegrityScope = { 'space' : string } |
  { 'global' : null } |
  { 'entityType' : string };
export type JoinType = { 'all' : null } |
  { 'any' : null };
export interface Jurisdiction {
  'region' : [] | [string],
  'city' : [] | [string],
  'countryCode' : string,
}
export interface KeyEntry {
  'id' : string,
  'alg' : [] | [string],
  'model' : [] | [string],
  'createdAt' : bigint,
  'scope' : [] | [string],
  'encryptedKey' : Uint8Array | number[],
  'keyLabel' : string,
  'ephemeralPubKey' : [] | [Uint8Array | number[]],
  'keyId' : [] | [string],
  'encVersion' : [] | [bigint],
}
export type KipResult = { 'ok' : [string, [] | [Entity]] } |
  { 'err' : string };
export interface KnowledgeAskCitation {
  'id' : string,
  'score' : number,
  'source_ref' : [] | [string],
}
export interface KnowledgeAskRequest {
  'filters' : [] | [KnowledgeSearchFilters],
  'max_context_chunks' : bigint,
  'question' : string,
  'limit' : bigint,
  'require_provenance' : boolean,
  'retrieval_mode' : string,
}
export interface KnowledgeAskResponse {
  'model' : string,
  'answer' : string,
  'trace_id' : string,
  'citations' : Array<KnowledgeAskCitation>,
}
export type KnowledgeAskResult = { 'ok' : KnowledgeAskResponse } |
  { 'err' : string };
export interface KnowledgeSearchFilters {
  'tags' : Array<string>,
  'source_type' : [] | [string],
  'produced_by_agent' : [] | [string],
  'perspective_scope' : [] | [string],
  'source_version_id' : [] | [string],
  'space_id' : [] | [string],
}
export interface KnowledgeSearchRequest {
  'filters' : [] | [KnowledgeSearchFilters],
  'limit' : bigint,
  'query_text' : string,
  'retrieval_mode' : string,
  'diagnostics' : boolean,
}
export interface KnowledgeSearchResult {
  'id' : string,
  'tags' : Array<string>,
  'score' : number,
  'source_ref' : [] | [string],
  'source_type' : [] | [string],
  'space_id' : [] | [string],
}
export interface Library {
  'id' : string,
  'workflows' : Array<WorkflowDefinition__1>,
  'description' : string,
  'version' : string,
  'entities' : Array<Entity>,
  'dependencies' : Array<string>,
  'relationships' : Array<Relationship>,
}
export interface LibraryManifest {
  'id' : string,
  'description' : string,
  'version' : string,
  'license' : [] | [string],
}
export interface LifecycleChange {
  'institutionId' : string,
  'newPhase' : TargetLifecyclePhase,
}
export type LifecyclePhase = { 'formalized' : null } |
  { 'dormant' : null } |
  { 'emergent' : null } |
  { 'operational' : null } |
  { 'provisional' : null } |
  { 'archived' : null };
export interface LineageNode { 'institution' : Institution, 'depth' : bigint }
export interface LogEntry {
  'id' : string,
  'context' : [] | [Array<[string, string]>],
  'source' : LogSource,
  'level' : LogLevel,
  'message' : string,
  'timestamp' : bigint,
}
export type LogLevel = { 'Error' : null } |
  { 'Info' : null } |
  { 'Warn' : null } |
  { 'Critical' : null };
export type LogSource = { 'Frontend' : null } |
  { 'Agent' : string } |
  { 'Backend' : null };
export interface Member {
  'joinedAt' : bigint,
  'roleIds' : Array<string>,
  'actorId' : ActorID,
}
export interface Message {
  'id' : string,
  'content' : string,
  'sender' : Principal,
  'timestamp' : bigint,
  'reflectionRef' : [] | [string],
}
export type MonitorLevel = { 'Healthy' : null } |
  { 'Critical' : null } |
  { 'Warning' : null };
export interface Node {
  'ephemeral_id' : string,
  'transport_address' : string,
  'name' : string,
  'last_heartbeat' : bigint,
  'attributes' : Array<[string, string]>,
  'roles' : Array<NodeRole>,
}
export type NodeRole = { 'ml' : null } |
  { 'data' : null } |
  { 'ingest' : null } |
  { 'master' : null } |
  { 'remote_cluster_client' : null };
export interface NodeSelector {
  'tags' : [] | [Array<string>],
  'entityType' : [] | [string],
}
export type NodeType = { 'File' : null } |
  { 'Mount' : null } |
  { 'Directory' : null };
export type NodeType__1 = { 'appendix' : null } |
  { 'part' : null } |
  { 'section' : null } |
  { 'sidebar' : null } |
  { 'chapter' : null };
export interface ParallelConfig {
  'forkSteps' : Array<string>,
  'joinStep' : string,
  'joinType' : JoinType,
}
export interface ParallelState {
  'results' : Array<[string, string]>,
  'completedSteps' : Array<string>,
  'activeSteps' : Array<string>,
  'forkId' : string,
}
export type Path = string;
export type Permission = { 'manage_workflow' : null } |
  { 'manage_space' : null } |
  { 'manage_members' : null } |
  { 'create_contribution' : null } |
  { 'view_private' : null } |
  { 'trigger_step' : null };
export interface ProjectMetadata {
  'id' : string,
  'status' : ProjectStatus,
  'owner' : Principal,
  'name' : string,
  'createdAt' : bigint,
  'description' : string,
  'collaborators' : Array<Principal>,
  'schemaId' : string,
  'canisterId' : Principal,
}
export type ProjectStatus = { 'Active' : null } |
  { 'Archived' : null } |
  { 'Deleted' : null };
export interface Proposal {
  'id' : string,
  'status' : ProposalStatus,
  'metadata' : Array<[string, string]>,
  'tallyResult' : [] | [TallyResult],
  'strategy' : GovernanceStrategy,
  'createdAt' : bigint,
  'description' : string,
  'proposalType' : ProposalType,
  'proposer' : Principal,
  'votingSession' : [] | [Ballot],
  'resolvedAt' : [] | [bigint],
}
export type ProposalStatus = { 'expired' : null } |
  { 'pending' : null } |
  { 'approved' : null } |
  { 'rejected' : null };
export type ProposalType = { 'role_assignment' : RoleAssignment } |
  { 'space_config' : SpaceConfigChange } |
  { 'economic_action' : EconomicAction } |
  { 'workflow_approval' : WorkflowApproval } |
  { 'schema_change' : SchemaChange } |
  { 'lifecycle_change' : LifecycleChange };
export type RefType = { 'tag' : null } |
  { 'entity' : null };
export interface Relationship {
  'to' : string,
  'libraryId' : [] | [string],
  'from' : string,
  'type' : string,
  'creatorActorId' : [] | [string],
  'scopeId' : [] | [string],
  'bidirectional' : boolean,
  'creatorAddress' : [] | [string],
  'timestamp' : bigint,
}
export interface RepeatConfig {
  'bodySteps' : Array<string>,
  'maxIterations' : bigint,
  'exitCondition' : TransitionGuard,
}
export interface RepeatState {
  'maxIterations' : bigint,
  'loopId' : string,
  'currentIteration' : bigint,
}
export type Result = { 'ok' : null } |
  { 'err' : string };
export type Result_1 = { 'ok' : string } |
  { 'err' : string };
export type Result_2 = { 'ok' : bigint } |
  { 'err' : string };
export type Role = { 'admin' : null } |
  { 'editor' : null } |
  { 'viewer' : null };
export interface RoleAssignment {
  'action' : { 'revoke' : null } |
    { 'grant' : null },
  'role' : string,
  'targetPrincipal' : Principal,
}
export interface Role__1 {
  'id' : string,
  'permissions' : Array<Permission>,
  'name' : string,
}
export interface SavedUI {
  'id' : string,
  'content' : string,
  'name' : string,
  'updatedAt' : bigint,
}
export interface SchemaChange {
  'changeType' : { 'modify_type' : null } |
    { 'archive_type' : null } |
    { 'add_type' : null },
  'typeId' : string,
  'payload' : string,
}
export type Severity = { 'warning' : null } |
  { 'violation' : null } |
  { 'info' : null } |
  { 'critical' : null };
export interface ShardRouting {
  'node_id' : string,
  'shard_id' : bigint,
  'primary' : boolean,
  'state' : ShardState,
  'relocating_node_id' : [] | [string],
  'index' : string,
}
export type ShardState = { 'started' : null } |
  { 'unassigned' : null } |
  { 'relocating' : null } |
  { 'initializing' : null };
export interface Space {
  'id' : string,
  'members' : Array<Member>,
  'source' : [] | [ForkSource],
  'owner' : ActorID,
  'name' : string,
  'createdAt' : bigint,
  'description' : string,
  'visibility' : Visibility,
  'roles' : Array<Role__1>,
}
export interface SpaceConfigChange {
  'field' : string,
  'newValue' : string,
  'spaceId' : string,
}
export type StepType = { 'branch' : BranchConfig } |
  { 'user_task' : UserTask } |
  { 'repeat_loop' : RepeatConfig } |
  { 'system_op' : SystemOp } |
  { 'async_external_op' : AsyncExternalOp } |
  { 'parallel' : ParallelConfig } |
  { 'sync_skills' : SyncConfig } |
  { 'delay' : DelayConfig } |
  { 'await_event' : string } |
  { 'sequence' : null };
export type StepType__1 = { 'branch' : BranchConfig } |
  { 'user_task' : UserTask } |
  { 'repeat_loop' : RepeatConfig } |
  { 'system_op' : SystemOp__1 } |
  { 'async_external_op' : AsyncExternalOp } |
  { 'parallel' : ParallelConfig } |
  { 'sync_skills' : SyncConfig } |
  { 'delay' : DelayConfig } |
  { 'await_event' : string } |
  { 'sequence' : null };
export type StorageProvider = { 'mcp' : { 'serverName' : string } } |
  { 'internal' : { 'moduleName' : string, 'scopeId' : [] | [string] } } |
  { 'local' : null } |
  { 'canister' : Principal } |
  { 'external' : { 'protocol' : string, 'bucket' : string } };
export interface StructureNode {
  'id' : string,
  'title' : string,
  'slug' : string,
  'reference' : [] | [ContributionRef],
  'children' : Array<StructureNode>,
  'nodeType' : NodeType__1,
}
export interface SyncConfig { 'libraryId' : string, 'strategy' : SyncStrategy }
export type SyncStrategy = { 'overwrite' : null } |
  { 'keep_existing' : null };
export interface SystemOp {
  'method' : string,
  'target' : { 'local' : null } |
    { 'canister' : string },
  'arguments' : string,
}
export interface SystemOp__1 {
  'method' : string,
  'target' : { 'local' : null } |
    { 'canister' : Principal },
  'arguments' : Uint8Array | number[],
}
export interface SystemStatus {
  'status' : MonitorLevel,
  'metrics' : {
    'error_count_24h' : bigint,
    'active_workflows' : bigint,
    'active_users_24h' : bigint,
  },
  'last_updated' : bigint,
  'version' : string,
  'canisters' : Array<CanisterStatus>,
  'uptime_seconds' : bigint,
}
export interface TallyResult {
  'no' : bigint,
  'yes' : bigint,
  'abstain' : bigint,
  'approved' : boolean,
  'totalPower' : bigint,
  'quorumReached' : boolean,
}
export type TargetLifecyclePhase = { 'formalized' : null } |
  { 'dormant' : null } |
  { 'emergent' : null } |
  { 'operational' : null } |
  { 'provisional' : null } |
  { 'archived' : null };
export interface TaskView {
  'startedAt' : bigint,
  'name' : string,
  'description' : string,
  'instanceId' : string,
  'taskId' : string,
}
export type TraceOutcome = { 'failure' : string } |
  { 'success' : null } |
  { 'deferred' : null };
export interface TransitionGuard {
  'field' : string,
  'value' : string,
  'operator' : GuardOperator,
  'targetStep' : string,
}
export interface Treaty {
  'id' : string,
  'status' : TreatyStatus,
  'createdAt' : bigint,
  'grantee' : string,
  'granter' : string,
  'scope' : AccessScope,
  'updatedAt' : bigint,
  'rationale' : string,
  'expiry' : [] | [bigint],
  'revokedAt' : [] | [bigint],
  'revokedReason' : [] | [string],
}
export type TreatyStatus = { 'active' : null } |
  { 'revoked' : null } |
  { 'expired' : null } |
  { 'proposed' : null } |
  { 'suspended' : null };
export type Trigger = { 'on_entity_create' : string } |
  { 'manual' : null };
export type Trigger__1 = { 'on_entity_create' : null } |
  { 'manual' : null };
export interface UpdateInstitutionRequest {
  'id' : string,
  'title' : [] | [string],
  'lifecyclePhase' : [] | [LifecyclePhase],
  'charterRefs' : [] | [Array<string>],
  'scope' : [] | [string],
  'summary' : [] | [string],
  'intent' : [] | [string],
  'stewards' : [] | [Array<Principal>],
  'affiliatedSpaces' : [] | [Array<string>],
  'confidence' : [] | [number],
}
export interface UserProfile {
  'savedUis' : Array<SavedUI>,
  'featureFlags' : Array<string>,
  'labsOptIn' : boolean,
  'jurisdiction' : [] | [Jurisdiction],
  'enabledLibraryIds' : Array<string>,
  'geoLocation' : [] | [GeoLocation],
}
export interface UserTask { 'outputSchema' : string, 'description' : string }
export type VFSResult = { 'ok' : string } |
  { 'err' : string };
export type Visibility = { 'public' : null } |
  { 'member_only' : null } |
  { 'private' : null };
export interface Vote {
  'voter' : Principal,
  'timestamp' : bigint,
  'choice' : VoteChoice,
  'power' : bigint,
}
export type VoteChoice = { 'no' : null } |
  { 'yes' : null } |
  { 'abstain' : null };
export interface VotingConfig {
  'durationSeconds' : bigint,
  'quorum' : number,
  'passingThreshold' : number,
}
export interface WorkerConfig {
  'alg' : [] | [string],
  'workerId' : ActorID,
  'publicKey' : Uint8Array | number[],
  'registeredAt' : bigint,
  'keyId' : [] | [string],
  'encVersion' : [] | [bigint],
}
export interface WorkflowApproval {
  'stepId' : string,
  'decision' : { 'reject' : null } |
    { 'approve' : null },
  'workflowInstanceId' : string,
}
export interface WorkflowDefinition {
  'id' : string,
  'name' : string,
  'steps' : Array<WorkflowStep>,
  'triggers' : Array<Trigger>,
}
export interface WorkflowDefinition__1 {
  'id' : string,
  'name' : string,
  'steps' : Array<WorkflowStep__1>,
  'triggers' : Array<Trigger__1>,
}
export interface WorkflowInstance {
  'id' : string,
  'status' : InstanceStatus,
  'context' : Array<[string, string]>,
  'createdAt' : bigint,
  'history' : Array<HistoryEntry>,
  'definitionId' : string,
  'parallelState' : [] | [ParallelState],
  'jurisdiction' : [] | [Jurisdiction],
  'loopState' : [] | [RepeatState],
  'currentStepId' : string,
  'geoLocation' : [] | [GeoLocation],
}
export interface WorkflowStateView {
  'status' : string,
  'context' : Array<[string, string]>,
  'history' : Array<ExecutionEvent>,
  'instanceId' : string,
  'currentStep' : [] | [WorkflowStep],
  'definitionName' : string,
  'formSchema' : [] | [string],
}
export interface WorkflowStep {
  'id' : string,
  'nextSteps' : Array<string>,
  'name' : string,
  'allowedRoles' : Array<AllowedRole>,
  'stepType' : StepType,
}
export interface WorkflowStep__1 {
  'id' : string,
  'nextSteps' : Array<string>,
  'name' : string,
  'allowedRoles' : Array<AllowedRole>,
  'stepType' : StepType__1,
}
export interface _SERVICE {
  'activateTreaty' : ActorMethod<[string], Result>,
  'addKey' : ActorMethod<
    [string, string, [] | [string], [] | [string], Uint8Array | number[]],
    undefined
  >,
  'addKeyV2' : ActorMethod<
    [
      string,
      string,
      [] | [string],
      [] | [string],
      Uint8Array | number[],
      [] | [Uint8Array | number[]],
      [] | [string],
      [] | [bigint],
      [] | [string],
    ],
    undefined
  >,
  /**
   * / Adopt a Commons for a Space (creates governs:commons:mode edge)
   */
  'adoptCommons' : ActorMethod<[string, string, AdoptionMode], Result>,
  'appendChatMessage' : ActorMethod<[string, string], undefined>,
  'batchIngest' : ActorMethod<[Array<Entity>, Array<Relationship>], string>,
  'bootstrap' : ActorMethod<[], string>,
  'castVote' : ActorMethod<[string, VoteChoice], Result>,
  'checkStatus' : ActorMethod<[], string>,
  'checkTreatyAccess' : ActorMethod<[string, string, string], boolean>,
  'createBook' : ActorMethod<[string, string], string>,
  'createDiscussion' : ActorMethod<[string, string, GatingPolicy], string>,
  /**
   * / Create a new Institution within a Space
   */
  'createInstitution' : ActorMethod<[CreateInstitutionRequest], Result_1>,
  'createProject' : ActorMethod<[string, string], Result_1>,
  'createSpace' : ActorMethod<[string, string, Visibility], Result_1>,
  'createTreaty' : ActorMethod<
    [string, AccessScope, string, [] | [bigint]],
    Result_1
  >,
  'deleteUI' : ActorMethod<[string], undefined>,
  /**
   * / Detach a Commons from a Space (remove adoption edge)
   */
  'detachCommons' : ActorMethod<[string, string], Result>,
  'dev_vector_search' : ActorMethod<[string], Array<string>>,
  'executeKip' : ActorMethod<[string], Result_1>,
  /**
   * / Execute a mutating KIP command (UPSERT, DELETE)
   * / This is a non-query function that can modify state
   * / Execute a mutating KIP command (UPSERT, DELETE)
   */
  'execute_kip_mutation' : ActorMethod<[string], KipResult>,
  'execute_kip_mutation_scoped' : ActorMethod<[string, string], KipResult>,
  'extractReferences' : ActorMethod<[string], Array<ExtractedRef>>,
  /**
   * / Fork an Institution (creates new with lineage)
   */
  'forkInstitution' : ActorMethod<[string, string, string], Result_1>,
  'forkSpace' : ActorMethod<[string], Result_1>,
  /**
   * / Get Affiliated Institutions (traverse affiliated_with edges)
   */
  'getAffiliatedInstitutions' : ActorMethod<[string], Array<Institution>>,
  'getAgentTraces' : ActorMethod<[string], Array<string>>,
  'getAllEntities' : ActorMethod<[], Array<Entity>>,
  'getAllRelationships' : ActorMethod<[], Array<Relationship>>,
  'getAllWorkflowInstances' : ActorMethod<[], Array<WorkflowInstance>>,
  'getBook' : ActorMethod<[string], [] | [Book]>,
  'getChatHistory' : ActorMethod<[], Array<ChatMessage>>,
  'getChronicleEvents' : ActorMethod<
    [[] | [bigint], [] | [bigint], bigint],
    Array<ChronicleEvent>
  >,
  /**
   * / Get adoption details for a Space
   */
  'getCommonsAdoptions' : ActorMethod<[string], Array<CommonsAdoption>>,
  'getCommonsEnforcementMode' : ActorMethod<[], CommonsEnforcementMode>,
  /**
   * / Get all Commons governing a given Space
   */
  'getCommonsForSpace' : ActorMethod<[string], Array<Institution>>,
  'getCommonsRuleset' : ActorMethod<[string], [] | [CommonsRuleset]>,
  'getConstellationEdges' : ActorMethod<[string], Array<GraphEdge>>,
  'getContextObservations' : ActorMethod<
    [Array<string>, Array<string>],
    Array<Entity>
  >,
  'getDiscussion' : ActorMethod<[string], [] | [Discussion]>,
  'getEnabledLibraryIds' : ActorMethod<[], Array<string>>,
  'getEntityEvents' : ActorMethod<[string, bigint], Array<ChronicleEvent>>,
  'getFlowLayout' : ActorMethod<[string, [] | [string]], [] | [FlowLayout]>,
  'getFlowLayoutHistory' : ActorMethod<
    [string, [] | [string], [] | [bigint]],
    Array<FlowLayout>
  >,
  'getGovernanceStrategy' : ActorMethod<[], GovernanceStrategy>,
  'getGovernanceUI' : ActorMethod<[string, [] | [string]], string>,
  'getIncomingConstellationEdges' : ActorMethod<[string], Array<GraphEdge>>,
  'getInstalledLibraries' : ActorMethod<[], Array<LibraryManifest>>,
  /**
   * / Get an Institution by ID
   */
  'getInstitution' : ActorMethod<[string], [] | [Institution]>,
  /**
   * / Get Institution Charters (resolve referenced contributions)
   * / Note: Returns basic metadata or full entities if we access the graph
   * / For MVP, we just return the IDs from the institution object
   */
  'getInstitutionCharters' : ActorMethod<[string], Array<string>>,
  /**
   * / Get Institution lineage (ancestor chain)
   */
  'getInstitutionLineage' : ActorMethod<[string, bigint], Array<LineageNode>>,
  /**
   * / Get Institutions by lifecycle phase
   */
  'getInstitutionsByPhase' : ActorMethod<[LifecyclePhase], Array<Institution>>,
  /**
   * / Get all Institutions in a Space
   */
  'getInstitutionsBySpace' : ActorMethod<[string], Array<Institution>>,
  'getLogs' : ActorMethod<
    [[] | [LogSource], [] | [LogLevel], bigint],
    Array<LogEntry>
  >,
  'getMyKeys' : ActorMethod<[[] | [string]], Array<KeyEntry>>,
  'getMyProfile' : ActorMethod<[], UserProfile>,
  'getPendingProposals' : ActorMethod<[], Array<Proposal>>,
  'getPendingTasks' : ActorMethod<[string], Array<ExternalRequest>>,
  'getPendingTasksV2' : ActorMethod<[string], Array<TaskView>>,
  'getProposal' : ActorMethod<[string], [] | [Proposal]>,
  'getRegistryId' : ActorMethod<[], Principal>,
  'getSchemaExplorerSurface' : ActorMethod<
    [{ 'list' : null } | { 'detail' : string }],
    string
  >,
  'getSchemaRegistryId' : ActorMethod<[], Principal>,
  'getSpace' : ActorMethod<[string], [] | [Space]>,
  'getSystemStatus' : ActorMethod<[], SystemStatus>,
  'getTreaties' : ActorMethod<[], Array<Treaty>>,
  'getTreaty' : ActorMethod<[string], [] | [Treaty]>,
  'getUserKeys' : ActorMethod<[Principal, [] | [string]], Array<KeyEntry>>,
  'getUserRoles' : ActorMethod<[Principal], Array<Role>>,
  'getWorkerConfig' : ActorMethod<[], [] | [WorkerConfig]>,
  'getWorkerKey' : ActorMethod<[], [] | [Uint8Array | number[]]>,
  'getWorkflowStateV2' : ActorMethod<[string], [] | [WorkflowStateView]>,
  'grantRole' : ActorMethod<[Principal, Role], undefined>,
  'installLibrary' : ActorMethod<[Library], undefined>,
  'install_library_kip' : ActorMethod<
    [string, Array<[string, string]>],
    string
  >,
  'knowledgeAsk' : ActorMethod<[KnowledgeAskRequest], KnowledgeAskResult>,
  'knowledgeSearch' : ActorMethod<
    [KnowledgeSearchRequest],
    Array<KnowledgeSearchResult>
  >,
  'linkReferences' : ActorMethod<[string, string], Result_2>,
  /**
   * / List all Commons Institutions in the system
   */
  'listCommons' : ActorMethod<[], Array<Institution>>,
  'listProjects' : ActorMethod<[], Array<ProjectMetadata>>,
  'logAgentTrace' : ActorMethod<[DecisionTrace], Result_1>,
  'nexus_heartbeat' : ActorMethod<
    [string, Node, bigint],
    [] | [ClusterStateDiff]
  >,
  'postMessage' : ActorMethod<[string, string, [] | [string]], Result_1>,
  'processAIQuery' : ActorMethod<[string], string>,
  'publishEdition' : ActorMethod<[string, string], string>,
  'recallObservations' : ActorMethod<
    [string, [] | [string], bigint],
    Array<Entity>
  >,
  'recordLegacyKeyUsage' : ActorMethod<[Principal, string, string], boolean>,
  'recordReflection' : ActorMethod<[string], string>,
  'registerAgent' : ActorMethod<[AgentConfig], Result_1>,
  'registerWorker' : ActorMethod<[Uint8Array | number[]], undefined>,
  'registerWorkerV2' : ActorMethod<
    [Uint8Array | number[], [] | [string], [] | [string], [] | [bigint]],
    undefined
  >,
  'registerWorkflowV2' : ActorMethod<[WorkflowDefinition], undefined>,
  'removeKey' : ActorMethod<[string], undefined>,
  'resolveProposal' : ActorMethod<
    [string, { 'reject' : null } | { 'approve' : null }],
    Result
  >,
  'revokeRole' : ActorMethod<[Principal, Role], undefined>,
  'revokeTreaty' : ActorMethod<[string, string], Result>,
  'runSchemaHygiene' : ActorMethod<[bigint, bigint], string>,
  'saveUI' : ActorMethod<[SavedUI], undefined>,
  'setCommonsEnforcementMode' : ActorMethod<[CommonsEnforcementMode], Result>,
  'setFeatureFlag' : ActorMethod<[string, boolean], undefined>,
  'setFlowLayout' : ActorMethod<[FlowLayoutInput], FlowLayout>,
  'setGovernanceStrategy' : ActorMethod<[GovernanceStrategy], Result>,
  'setLabsOptIn' : ActorMethod<[boolean], undefined>,
  'signalWorkflowV2' : ActorMethod<[string, string, string], Result_1>,
  'startWorkflowV2' : ActorMethod<[string, Array<[string, string]>], Result_1>,
  'submitLog' : ActorMethod<
    [LogSource, LogLevel, string, [] | [Array<[string, string]>]],
    undefined
  >,
  'submitProposal' : ActorMethod<
    [ProposalType, string, Array<[string, string]>, [] | [string]],
    Result_1
  >,
  'submitTaskOutcome' : ActorMethod<[string, string, string], string>,
  'suspendTreaty' : ActorMethod<[string], Result>,
  'toggleLibrary' : ActorMethod<[string, boolean], undefined>,
  'updateBookStructure' : ActorMethod<[string, BookStructure], undefined>,
  /**
   * / Update an existing Institution
   */
  'updateInstitution' : ActorMethod<[UpdateInstitutionRequest], Result>,
  'updateKeyV2ForUser' : ActorMethod<
    [
      Principal,
      string,
      Uint8Array | number[],
      [] | [Uint8Array | number[]],
      [] | [string],
      [] | [bigint],
      [] | [string],
    ],
    boolean
  >,
  'upsertCommonsRuleset' : ActorMethod<[string, CommonsRuleset], Result>,
  'vfs_list' : ActorMethod<[string], Array<FileMetadata>>,
  'vfs_mkdir' : ActorMethod<[string], VFSResult>,
  'vfs_mount' : ActorMethod<[string, Principal], VFSResult>,
  'vfs_read' : ActorMethod<[string], VFSResult>,
  'vfs_write' : ActorMethod<[string, string, string], VFSResult>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
