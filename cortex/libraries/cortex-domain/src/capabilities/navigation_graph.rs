use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct CapabilityId(pub String);

fn default_catalog_schema_version() -> String {
    "1.0.0".to_string()
}

fn default_catalog_version() -> String {
    "unversioned".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum IntentType {
    /// A capability that visualizes data or architecture
    Visualize,
    /// A capability that edits or mutates state
    Mutate,
    /// A capability that executes a script or workflow
    Execute,
    /// A capability that monitors system health or telemetry
    Monitor,
    /// A capability that configures system settings
    Configure,
    /// Unspecified architectural leaf node
    Unspecified,
}

impl Default for IntentType {
    fn default() -> Self {
        Self::Unspecified
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SurfacingHeuristic {
    /// Placed in the primary top-level navigation.
    PrimaryCore,
    /// Placed in grouped secondary/sub-menu navigation.
    Secondary,
    /// Hidden from top navigation and surfaced contextually.
    ContextualDeep,
    /// Hidden from default surfacing but queryable.
    Hidden,
}

impl Default for SurfacingHeuristic {
    fn default() -> Self {
        Self::Secondary
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OperationalFrequency {
    Continuous,
    Daily,
    AdHoc,
    Rare,
}

impl Default for OperationalFrequency {
    fn default() -> Self {
        Self::AdHoc
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DomainEntityRef {
    pub entity_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entity_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct PlacementConstraint {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preferred_nav_band: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preferred_category: Option<String>,
    #[serde(default)]
    pub allow_contextual_only: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_nav_depth: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapabilityNode {
    pub id: CapabilityId,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "resourceRef")]
    pub resource_ref: Option<String>,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub intent_type: IntentType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub route_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required_role: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_claims: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(default)]
    pub surfacing_heuristic: SurfacingHeuristic,
    #[serde(default)]
    pub operational_frequency: OperationalFrequency,
    #[serde(default)]
    pub domain_entities: Vec<DomainEntityRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub placement_constraint: Option<PlacementConstraint>,
    pub root_path: Option<String>,
    #[serde(default)]
    pub invariant_violations: Vec<crate::integrity::PolicyViolation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EdgeRelationship {
    /// Indicates one capability is a child of another hierarchically
    ChildOf,
    /// Indicates one capability triggers another
    Triggers,
    /// Indicates one capability provides data to another
    ProvidesContextTo,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapabilityEdge {
    pub source: CapabilityId,
    pub target: CapabilityId,
    pub relationship: EdgeRelationship,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PlatformCapabilityCatalog {
    #[serde(default = "default_catalog_schema_version")]
    pub schema_version: String,
    #[serde(default = "default_catalog_version")]
    pub catalog_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub catalog_hash: Option<String>,
    pub nodes: Vec<CapabilityNode>,
    pub edges: Vec<CapabilityEdge>,
}

impl Default for PlatformCapabilityCatalog {
    fn default() -> Self {
        Self::new()
    }
}

impl PlatformCapabilityCatalog {
    pub fn new() -> Self {
        Self {
            schema_version: default_catalog_schema_version(),
            catalog_version: default_catalog_version(),
            catalog_hash: None,
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    pub fn unverified_add_node(&mut self, node: CapabilityNode) {
        if !self.nodes.iter().any(|n| n.id == node.id) {
            self.nodes.push(node);
        }
    }

    pub fn unverified_add_edge(&mut self, edge: CapabilityEdge) {
        if !self.edges.iter().any(|e| {
            e.source == edge.source
                && e.target == edge.target
                && e.relationship == edge.relationship
        }) {
            self.edges.push(edge);
        }
    }
}

/// Compatibility alias maintained during migration to `PlatformCapabilityCatalog`.
pub type PlatformCapabilityGraph = PlatformCapabilityCatalog;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SpaceCapabilityNodeOverride {
    pub capability_id: CapabilityId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local_alias: Option<String>,
    #[serde(default = "default_true")]
    pub is_active: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local_required_role: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub local_additional_required_claims: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub surfacing_heuristic: Option<SurfacingHeuristic>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operational_frequency: Option<OperationalFrequency>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub placement_constraint: Option<PlacementConstraint>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SpaceCapabilityGraph {
    pub schema_version: String,
    pub space_id: String,
    pub base_catalog_version: String,
    pub base_catalog_hash: String,
    #[serde(default)]
    pub nodes: Vec<SpaceCapabilityNodeOverride>,
    #[serde(default)]
    pub edges: Vec<CapabilityEdge>,
    pub updated_at: String,
    pub updated_by: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lineage_ref: Option<String>,
}
