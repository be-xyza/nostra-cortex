export const idlFactory = ({ IDL }) => {
  const FileMetadata = IDL.Record({
    'mime_type' : IDL.Text,
    'size' : IDL.Nat64,
    'last_modified' : IDL.Nat64,
  });
  const ListEntry = IDL.Tuple(IDL.Text, FileMetadata);
  const Result = IDL.Variant({ 'Ok' : IDL.Null, 'Err' : IDL.Text });
  const ResultBytes = IDL.Variant({
    'Ok' : IDL.Vec(IDL.Nat8),
    'Err' : IDL.Text,
  });
  const ResultText = IDL.Variant({ 'Ok' : IDL.Text, 'Err' : IDL.Text });
  const ResultList = IDL.Variant({ 'Ok' : IDL.Vec(ListEntry), 'Err' : IDL.Text });
  const ContributionVersionRef = IDL.Record({
    'contribution_id' : IDL.Text,
    'version_hash' : IDL.Text,
  });
  const ChapterManifest = IDL.Record({
    'index' : IDL.Nat32,
    'contribution_ref' : ContributionVersionRef,
    'content_hash' : IDL.Text,
    'title' : IDL.Text,
  });
  const EditionMetadata = IDL.Record({ 'license' : IDL.Text });
  const EditionManifest = IDL.Record({
    'edition_id' : IDL.Text,
    'dpub_id' : IDL.Text,
    'version' : IDL.Text,
    'name' : IDL.Opt(IDL.Text),
    'content_root' : IDL.Text,
    'chapters' : IDL.Vec(ChapterManifest),
    'published_at' : IDL.Text,
    'publisher' : IDL.Text,
    'previous_edition' : IDL.Opt(IDL.Text),
    'metadata' : EditionMetadata,
  });
  const ResultEdition = IDL.Variant({
    'Ok' : EditionManifest,
    'Err' : IDL.Text,
  });
  const FlowNode = IDL.Record({
    'id' : IDL.Text,
    'name' : IDL.Text,
    'type' : IDL.Text,
    'schema_ref' : IDL.Opt(IDL.Text),
    'file_ref' : IDL.Opt(IDL.Text),
    'language' : IDL.Opt(IDL.Text),
    'tags' : IDL.Vec(IDL.Text),
    'flows' : IDL.Vec(IDL.Text),
  });
  const FlowEdge = IDL.Record({
    'id' : IDL.Text,
    'source' : IDL.Text,
    'target' : IDL.Text,
    'topic' : IDL.Opt(IDL.Text),
    'variant' : IDL.Text,
    'conditional' : IDL.Opt(IDL.Bool),
  });
  const FlowGraph = IDL.Record({
    'id' : IDL.Text,
    'workflow_id' : IDL.Text,
    'version' : IDL.Text,
    'generated_at' : IDL.Nat64,
    'nodes' : IDL.Vec(FlowNode),
    'edges' : IDL.Vec(FlowEdge),
  });
  const FlowNodePosition = IDL.Record({
    'node_id' : IDL.Text,
    'x' : IDL.Int,
    'y' : IDL.Int,
  });
  const FlowHandlePosition = IDL.Record({
    'handle_id' : IDL.Text,
    'source' : IDL.Text,
    'target' : IDL.Text,
  });
  const FlowLayoutInput = IDL.Record({
    'workflow_id' : IDL.Text,
    'graph_version' : IDL.Text,
    'node_positions' : IDL.Vec(FlowNodePosition),
    'handle_positions' : IDL.Vec(FlowHandlePosition),
    'collapsed_groups' : IDL.Vec(IDL.Text),
  });
  const FlowLayout = IDL.Record({
    'workflow_id' : IDL.Text,
    'graph_version' : IDL.Text,
    'node_positions' : IDL.Vec(FlowNodePosition),
    'handle_positions' : IDL.Vec(FlowHandlePosition),
    'collapsed_groups' : IDL.Vec(IDL.Text),
    'updated_by' : IDL.Principal,
    'updated_at' : IDL.Nat64,
  });
  const ResultFlowGraph = IDL.Variant({
    'Ok' : FlowGraph,
    'Err' : IDL.Text,
  });
  const ResultFlowLayout = IDL.Variant({
    'Ok' : FlowLayout,
    'Err' : IDL.Text,
  });
  const MigrationCounts = IDL.Record({
    'workflows' : IDL.Nat64,
    'vfs_files' : IDL.Nat64,
  });
  const ResultMigrationCounts = IDL.Variant({
    'Ok' : MigrationCounts,
    'Err' : IDL.Text,
  });
  return IDL.Service({
    'start_workflow' : IDL.Func([IDL.Text], [IDL.Text], []),
    'process_message' : IDL.Func([IDL.Text], [IDL.Text], []),
    'get_workflow' : IDL.Func([IDL.Text], [IDL.Opt(IDL.Text)], ['query']),
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
    'set_flow_layout' : IDL.Func([FlowLayoutInput], [ResultFlowLayout], []),
    'write_file' : IDL.Func(
        [IDL.Text, IDL.Vec(IDL.Nat8), IDL.Text],
        [Result],
        [],
      ),
    'read_file' : IDL.Func([IDL.Text], [ResultBytes], ['query']),
    'list_files' : IDL.Func([IDL.Text], [IDL.Vec(ListEntry)], ['query']),
    'read_dpub_file_guarded' : IDL.Func(
        [IDL.Text, IDL.Opt(IDL.Text), IDL.Opt(IDL.Text)],
        [ResultBytes],
        ['query'],
      ),
    'list_dpub_files_guarded' : IDL.Func(
        [IDL.Text, IDL.Opt(IDL.Text), IDL.Opt(IDL.Text)],
        [ResultList],
        ['query'],
      ),
    'read_vfs_guarded' : IDL.Func(
        [IDL.Text, IDL.Opt(IDL.Text), IDL.Opt(IDL.Text)],
        [ResultBytes],
        ['query'],
      ),
    'list_vfs_guarded' : IDL.Func(
        [IDL.Text, IDL.Opt(IDL.Text), IDL.Opt(IDL.Text)],
        [ResultList],
        ['query'],
      ),
    'migrate_legacy_storage' : IDL.Func([], [ResultMigrationCounts], []),
    'migrate_legacy_storage_chunk' : IDL.Func([IDL.Nat64], [ResultMigrationCounts], []),
    'publish_dpub_edition' : IDL.Func(
        [IDL.Text, IDL.Text, IDL.Opt(IDL.Text), IDL.Opt(IDL.Text)],
        [ResultEdition],
        [],
      ),
    'get_dpub_feed' : IDL.Func(
        [IDL.Text, IDL.Nat32, IDL.Opt(IDL.Text), IDL.Opt(IDL.Text)],
        [ResultText],
        ['query'],
      ),
  });
};
export const init = ({ IDL }) => { return []; };
