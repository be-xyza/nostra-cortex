use crate::a2ui_types::{A11yProperties, A2UIMessage, Component, ComponentType};
use nostra_workflow_core::{Action, Step};
use std::collections::HashMap;

pub fn generate_a2ui_form(step: &Step, instance_id: &str) -> Option<A2UIMessage> {
    if let Action::UserTask {
        a2ui_schema,
        description,
        ..
    } = &step.action
    {
        if let Some(schema_json) = a2ui_schema {
            // Parse the schema string into components
            // Return None if JSON is invalid (should prob log this)
            // Accept either a raw component list or a full RenderSurface message
            let components: Vec<Component> = match serde_json::from_str(schema_json) {
                Ok(list) => list,
                Err(_) => {
                    if let Ok(msg) = serde_json::from_str::<A2UIMessage>(schema_json) {
                        if let A2UIMessage::RenderSurface { components, .. } = msg {
                            components
                        } else {
                            return None;
                        }
                    } else {
                        return None;
                    }
                }
            };

            return Some(A2UIMessage::RenderSurface {
                surface_id: format!("{}_{}", step.id, instance_id),
                title: step.description.clone(),
                root: None,
                components,
                meta: None,
            });
        } else {
            // Fallback: Generate a simple text display with the description and a "Continue" button
            let mut card_props = HashMap::new();
            card_props.insert(
                "description".to_string(),
                serde_json::Value::String(description.clone()),
            );

            let mut btn_props = HashMap::new();
            btn_props.insert(
                "label".to_string(),
                serde_json::Value::String("Complete Task".into()),
            );
            btn_props.insert(
                "variant".to_string(),
                serde_json::Value::String("primary".into()),
            );

            let components = vec![
                Component {
                    id: "instruction_card".into(),
                    component_type: ComponentType::Card,
                    props: card_props,
                    a11y: None,
                    children: vec!["complete_btn".into()],
                    data_bind: None,
                },
                Component {
                    id: "complete_btn".into(),
                    component_type: ComponentType::Button,
                    props: btn_props,
                    a11y: Some(A11yProperties::with_label("Complete Task")),
                    children: vec![],
                    data_bind: None,
                },
            ];

            return Some(A2UIMessage::RenderSurface {
                surface_id: format!("{}_{}", step.id, instance_id),
                title: step.description.clone(),
                root: None,
                components,
                meta: None,
            });
        }
    }
    None
}
