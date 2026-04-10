use candid::{CandidType, Deserialize};
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, CandidType, Deserialize, Serialize)]
pub struct PopupConfig {
    pub version: String,
    pub id: String,
    pub title: String,
    pub body_markdown: String,
    pub primary_action: Option<ActionConfig>,
    pub secondary_action: Option<ActionConfig>,
    pub dismissible: bool,
    pub size: PopupSize,
}

#[derive(Clone, Debug, PartialEq, CandidType, Deserialize, Serialize)]
pub struct ActionConfig {
    pub label: String,
    pub action_id: String,
    pub style: ActionStyle,
}

#[derive(Clone, Copy, Debug, PartialEq, CandidType, Deserialize, Serialize)]
pub enum PopupSize {
    Small,
    Medium,
    Large,
}

#[derive(Clone, Copy, Debug, PartialEq, CandidType, Deserialize, Serialize)]
pub enum ActionStyle {
    Primary,
    Secondary,
    Destructive,
    Outline,
}

impl Default for PopupConfig {
    fn default() -> Self {
        Self {
            version: "1.0.0".to_string(),
            id: "default-popup".to_string(),
            title: "Popup Title".to_string(),
            body_markdown: "Popup content goes here.".to_string(),
            primary_action: Some(ActionConfig {
                label: "Confirm".to_string(),
                action_id: "confirm".to_string(),
                style: ActionStyle::Primary,
            }),
            secondary_action: Some(ActionConfig {
                label: "Cancel".to_string(),
                action_id: "cancel".to_string(),
                style: ActionStyle::Outline,
            }),
            dismissible: true,
            size: PopupSize::Medium,
        }
    }
}
