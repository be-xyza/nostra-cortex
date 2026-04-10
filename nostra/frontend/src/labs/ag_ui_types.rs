use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum AgComponent {
    Notification(AgNotification),
    Details(AgDetails),
    Input(AgInput),
    Select(AgSelect),
    Action(AgAction),
    // Fallback/Generic container
    Row { children: Vec<AgComponent> },
    Column { children: Vec<AgComponent> },
    Text { text: String },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AgNotification {
    pub variant: String, // primary, success, warning, danger
    pub message: String,
    pub icon: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AgDetails {
    pub summary: String,
    pub children: Vec<AgComponent>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AgInput {
    pub label: String,
    pub name: String,
    pub value: Option<String>,
    pub placeholder: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AgSelect {
    pub label: String,
    pub name: String,
    pub options: Vec<AgSelectOption>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AgSelectOption {
    pub label: String,
    pub value: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AgAction {
    pub label: String,
    pub actionId: String,
    pub variant: Option<String>,
}
