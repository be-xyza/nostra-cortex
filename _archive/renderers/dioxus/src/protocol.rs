use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Message {
    BeginRendering(BeginRendering),
    SurfaceUpdate(SurfaceUpdate),
    DataModelUpdate(DataModelUpdate),
    DeleteSurface(DeleteSurface),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BeginRendering {
    pub surface_id: String,
    pub root: String, // Component ID
    #[serde(default)]
    pub catalog_id: Option<String>,
    #[serde(default)]
    pub styles: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SurfaceUpdate {
    pub surface_id: String,
    pub components: Vec<ComponentNode>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ComponentNode {
    pub id: String,
    #[serde(default)]
    pub weight: Option<f64>,
    pub component: crate::components::ComponentWrapper,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DataModelUpdate {
    pub surface_id: String,
    #[serde(default)]
    pub path: Option<String>,
    pub contents: Vec<DataModelEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct DataModelEntry {
    pub key: String,
    #[serde(flatten)]
    pub value: DataValue,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum DataValue {
    ValueString(String),
    ValueNumber(f64),
    ValueBoolean(bool),
    ValueMap(Vec<DataModelEntry>),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DeleteSurface {
    pub surface_id: String,
}
