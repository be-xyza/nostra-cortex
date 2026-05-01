import fixture from "./heapBlockCapabilityInventory.fixture.json";

export type HeapBlockCapabilityClass =
  | "read_projection"
  | "local_ui_state"
  | "runtime_read"
  | "runtime_write"
  | "steward_gated_write"
  | "destructive_write"
  | "download"
  | "placeholder_or_disabled"
  | "overlay_interaction";

export type HeapBlockCapabilityInventoryFixture = {
  schema_version: "CortexWebHeapBlockCapabilityInventoryV1";
  snapshot_id: "system:ux:heap-block-capability-inventory";
  inventory_id: string;
  authority_mode: "recommendation_only";
  actions: Array<{
    id: string;
    label: string;
    class: HeapBlockCapabilityClass;
    status: string;
    zones: string[];
    required_observability: string[];
    confirmation_contract?: {
      required: boolean;
      style?: "danger" | "default";
      fallback_enforced?: boolean;
    };
    known_gap?: {
      severity: "low" | "medium" | "high";
      summary: string;
      recommended_action: string;
    };
  }>;
  create_modes: Array<{
    mode: string;
    class: HeapBlockCapabilityClass;
    payload_type: string;
    status: string;
  }>;
  detail_tabs: Array<{
    id: string;
    class: HeapBlockCapabilityClass;
    status: string;
  }>;
  known_gaps: Array<{
    id: string;
    severity: "low" | "medium" | "high";
    summary: string;
    recommended_action: string;
  }>;
  overlay_surfaces?: Array<{
    id: string;
    class: HeapBlockCapabilityClass;
    status: string;
    authority_contract?: {
      persistence: "local_ui_state";
      durable_evidence: false;
      governed_heap_record: false;
      exportable_as_evidence: false;
      recommended_persistence_target: "undecided";
    };
    known_gap?: {
      severity: "low" | "medium" | "high";
      summary: string;
      recommended_action: string;
    };
  }>;
};

export const HEAP_BLOCK_CAPABILITY_INVENTORY_FIXTURE =
  fixture as HeapBlockCapabilityInventoryFixture;

export function buildHeapBlockCapabilityInventoryResponse(): HeapBlockCapabilityInventoryFixture {
  return HEAP_BLOCK_CAPABILITY_INVENTORY_FIXTURE;
}
