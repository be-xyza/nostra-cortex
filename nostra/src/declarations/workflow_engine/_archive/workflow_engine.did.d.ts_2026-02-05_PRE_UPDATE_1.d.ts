import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';
import type { Principal } from '@dfinity/principal';

export interface FileMetadata {
  'mime_type' : string,
  'size' : bigint,
  'last_modified' : bigint,
}
export type ListEntry = [string, FileMetadata];
export type Result = { 'Ok' : null } |
  { 'Err' : string };
export type ResultBytes = { 'Ok' : Uint8Array | number[] } |
  { 'Err' : string };
export type ResultText = { 'Ok' : string } |
  { 'Err' : string };
export type ResultList = { 'Ok' : Array<ListEntry> } |
  { 'Err' : string };
export interface ContributionVersionRef {
  'contribution_id' : string,
  'version_hash' : string,
}
export interface ChapterManifest {
  'index' : number,
  'contribution_ref' : ContributionVersionRef,
  'content_hash' : string,
  'title' : string,
}
export interface EditionMetadata { 'license' : string }
export interface EditionManifest {
  'edition_id' : string,
  'dpub_id' : string,
  'version' : string,
  'name' : [] | [string],
  'content_root' : string,
  'chapters' : Array<ChapterManifest>,
  'published_at' : string,
  'publisher' : string,
  'previous_edition' : [] | [string],
  'metadata' : EditionMetadata,
}
export type ResultEdition = { 'Ok' : EditionManifest } |
  { 'Err' : string };
export interface FlowNode {
  'id' : string,
  'name' : string,
  'type' : string,
  'schema_ref' : [] | [string],
  'file_ref' : [] | [string],
  'language' : [] | [string],
  'tags' : Array<string>,
  'flows' : Array<string>,
}
export interface FlowEdge {
  'id' : string,
  'source' : string,
  'target' : string,
  'topic' : [] | [string],
  'variant' : string,
  'conditional' : [] | [boolean],
}
export interface FlowGraph {
  'id' : string,
  'workflow_id' : string,
  'version' : string,
  'generated_at' : bigint,
  'nodes' : Array<FlowNode>,
  'edges' : Array<FlowEdge>,
}
export interface FlowNodePosition {
  'node_id' : string,
  'x' : bigint,
  'y' : bigint,
}
export interface FlowHandlePosition {
  'handle_id' : string,
  'source' : string,
  'target' : string,
}
export interface FlowLayoutInput {
  'workflow_id' : string,
  'graph_version' : string,
  'node_positions' : Array<FlowNodePosition>,
  'handle_positions' : Array<FlowHandlePosition>,
  'collapsed_groups' : Array<string>,
}
export interface FlowLayout {
  'workflow_id' : string,
  'graph_version' : string,
  'node_positions' : Array<FlowNodePosition>,
  'handle_positions' : Array<FlowHandlePosition>,
  'collapsed_groups' : Array<string>,
  'updated_by' : Principal,
  'updated_at' : bigint,
}
export type ResultFlowGraph = { 'Ok' : FlowGraph } |
  { 'Err' : string };
export type ResultFlowLayout = { 'Ok' : FlowLayout } |
  { 'Err' : string };
export interface MigrationCounts {
  'workflows' : bigint,
  'vfs_files' : bigint,
}
export type ResultMigrationCounts = { 'Ok' : MigrationCounts } |
  { 'Err' : string };
export interface _SERVICE {
  'start_workflow' : ActorMethod<[string], string>,
  'process_message' : ActorMethod<[string], string>,
  'get_workflow' : ActorMethod<[string], [] | [string]>,
  'get_flow_graph' : ActorMethod<[string, [] | [string]], ResultFlowGraph>,
  'get_flow_layout' : ActorMethod<[string, [] | [string]], ResultFlowLayout>,
  'set_flow_layout' : ActorMethod<[FlowLayoutInput], ResultFlowLayout>,
  'write_file' : ActorMethod<[string, Uint8Array | number[], string], Result>,
  'read_file' : ActorMethod<[string], ResultBytes>,
  'list_files' : ActorMethod<[string], Array<ListEntry>>,
  'read_dpub_file_guarded' : ActorMethod<[string, [] | [string], [] | [string]], ResultBytes>,
  'list_dpub_files_guarded' : ActorMethod<[string, [] | [string], [] | [string]], ResultList>,
  'read_vfs_guarded' : ActorMethod<[string, [] | [string], [] | [string]], ResultBytes>,
  'list_vfs_guarded' : ActorMethod<[string, [] | [string], [] | [string]], ResultList>,
  'migrate_legacy_storage' : ActorMethod<[], ResultMigrationCounts>,
  'migrate_legacy_storage_chunk' : ActorMethod<[bigint], ResultMigrationCounts>,
  'publish_dpub_edition' : ActorMethod<[string, string, [] | [string], [] | [string]], ResultEdition>,
  'get_dpub_feed' : ActorMethod<[string, number, [] | [string], [] | [string]], ResultText>,
}
export declare const idlFactory: IDL.InterfaceFactory;
