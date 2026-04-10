use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// 2.1 Backend -> Client (Stream)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum A2UIMessage {
    #[serde(rename = "RenderSurface")]
    RenderSurface {
        #[serde(rename = "surfaceId")]
        surface_id: String,
        title: String,
        #[serde(rename = "root", skip_serializing_if = "Option::is_none")]
        root: Option<String>,
        components: Vec<Component>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        meta: Option<A2UIMeta>,
    },
    #[serde(rename = "UpdateData")]
    UpdateData {
        patch: serde_json::Value, // Using Value for generic JSON Patch
    },
    #[serde(rename = "Navigation")]
    Navigation { route: String },
    #[serde(rename = "Error")]
    Error { message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct A2UIMeta {
    #[serde(default)]
    pub theme: Option<String>,
    #[serde(default)]
    pub tone: Option<String>,
    #[serde(default)]
    pub context: Option<String>,
    #[serde(default)]
    pub density: Option<String>,
    #[serde(default)]
    pub priority: Option<String>,
    #[serde(default)]
    pub intent: Option<String>,
    #[serde(default)]
    pub severity: Option<String>,
    #[serde(default)]
    pub workflow_id: Option<String>,
    #[serde(default)]
    pub mutation_id: Option<String>,
    #[serde(default)]
    pub space_id: Option<String>,
    #[serde(default)]
    pub execution_profile_ref: Option<String>,
    #[serde(default)]
    pub attribution_domain_ref: Option<String>,
    #[serde(default)]
    pub gate_level: Option<String>,
    #[serde(default)]
    pub gate_status: Option<String>,
    #[serde(default)]
    pub decision_gate_id: Option<String>,
    #[serde(default)]
    pub replay_contract_ref: Option<String>,
    #[serde(default)]
    pub action_target_ref: Option<String>,
    #[serde(default)]
    pub actor_ref: Option<String>,
    #[serde(default)]
    pub policy_ref: Option<String>,
    #[serde(default)]
    pub lineage_id: Option<String>,
    #[serde(default)]
    pub source_of_truth: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub timestamp: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    pub id: String,
    #[serde(rename = "type")]
    pub component_type: ComponentType,
    #[serde(default)]
    pub props: HashMap<String, serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub a11y: Option<A11yProperties>,
    #[serde(default)]
    pub children: Vec<String>, // IDs of children
    #[serde(rename = "dataBind", skip_serializing_if = "Option::is_none")]
    pub data_bind: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct A11yProperties {
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub live: Option<String>,
    #[serde(default)]
    pub atomic: Option<bool>,
    #[serde(default)]
    pub hidden: Option<bool>,
    #[serde(default)]
    pub decorative: Option<bool>,
    #[serde(default)]
    pub required: Option<bool>,
    #[serde(default)]
    pub invalid: Option<bool>,
    #[serde(default)]
    pub expanded: Option<bool>,
    #[serde(default)]
    pub pressed: Option<bool>,
    #[serde(default)]
    pub selected: Option<bool>,
    #[serde(default)]
    pub checked: Option<bool>,
    #[serde(default, rename = "level")]
    pub level: Option<u8>,
    #[serde(default, rename = "valueMin")]
    pub value_min: Option<f64>,
    #[serde(default, rename = "valueMax")]
    pub value_max: Option<f64>,
    #[serde(default, rename = "valueNow")]
    pub value_now: Option<f64>,
}

impl A11yProperties {
    pub fn with_label(label: impl Into<String>) -> Self {
        Self {
            label: Some(label.into()),
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComponentType {
    // Layout
    Container,
    Card,
    Row,
    Column,
    Tabs,
    Modal,
    Divider,
    // Input
    TextField,
    TextArea,
    Select,
    #[serde(alias = "CheckBox")]
    Checkbox,
    Slider,
    DateTimeInput,
    MultipleChoice,
    // Display
    Text,
    Heading,
    Markdown,
    CodeBlock,
    DataTable,
    StatusBadge,
    Image,
    Video,
    AudioPlayer,
    // Action
    Button,
}

// 2.2 Client -> Backend (Action)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientAction {
    #[serde(rename = "Submit")]
    Submit {
        #[serde(rename = "surfaceId")]
        surface_id: String,
        data: HashMap<String, serde_json::Value>,
    },
    #[serde(rename = "Click")]
    Click {
        #[serde(rename = "componentId")]
        component_id: String,
        action: String,
    },
    #[serde(rename = "Cancel")]
    Cancel {
        #[serde(rename = "surfaceId")]
        surface_id: String,
    },
}
