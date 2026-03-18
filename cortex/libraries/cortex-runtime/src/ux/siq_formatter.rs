use chrono::Utc;
use cortex_domain::integrity::invariant::SystemIntegrityQuality;
use serde::Serialize;
use serde_json::Value;

/// Envelope for creating a Polymorphic Heap Block.
/// Matches EMIT_HEAP_BLOCK.schema.json
#[derive(Debug, Clone, Serialize)]
pub struct EmitHeapBlock {
    pub schema_version: String,
    pub mode: String,
    pub workspace_id: String,
    pub source: HeapBlockSource,
    pub block: HeapBlockMeta,
    pub content: HeapBlockContent,
    pub relations: HeapBlockRelations,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crdt_projection: Option<Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HeapBlockSource {
    pub agent_id: String,
    pub session_id: String,
    pub request_id: String,
    pub emitted_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct HeapBlockMeta {
    pub id: String,
    #[serde(rename = "type")]
    pub block_type: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<std::collections::BTreeMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub behaviors: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HeapBlockContent {
    pub payload_type: String,
    pub a2ui: A2uiPayload,
}

#[derive(Debug, Clone, Serialize)]
pub struct A2uiPayload {
    pub surface_id: String,
    pub protocol_version: String,
    pub tree: Value,
    pub data_model: Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct HeapBlockRelations {
    pub tags: Vec<Value>,
    pub mentions: Vec<RelationMention>,
    pub page_links: Vec<Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RelationMention {
    pub to_block_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_path: Option<String>,
}

/// An A2UI V0.8 surface message representing the SIQ Scorecard.
/// This struct can be serialized into JSONL for streaming to A2UI renderers.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SiqSurfaceMessage {
    pub surface_id: String,
    #[serde(flatten)]
    pub content: SiqSurfaceContent,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SiqSurfaceContent {
    BeginRendering {
        root: String,
    },
    SurfaceUpdate {
        components: Vec<A2uiComponent>,
        #[serde(rename = "dataModelUpdate", skip_serializing_if = "Option::is_none")]
        data_model_update: Option<Value>,
    },
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct A2uiComponent {
    pub id: String,
    pub component_properties: Value,
}

/// Converts a `SystemIntegrityQuality` domain result into a Universal Polymorphic Block
/// with an A2UI payload type, ready for persistence in the Cortex workspace.
pub fn format_siq_as_heap_block(
    workspace_id: &str,
    block_id: &str,
    surface_id: &str,
    siq: &SystemIntegrityQuality,
) -> EmitHeapBlock {
    let mut messages = Vec::new();

    // 1. Begin rendering the SIQ surface
    messages.push(SiqSurfaceMessage {
        surface_id: surface_id.to_string(),
        content: SiqSurfaceContent::BeginRendering {
            root: "siq-root".to_string(),
        },
    });

    // 2. Build the component tree
    let mut components = Vec::new();

    components.push(A2uiComponent {
        id: "siq-root".to_string(),
        component_properties: serde_json::json!({
            "Column": {
                "children": {
                    "explicitList": ["siq-header", "siq-status", "siq-violations-list"]
                }
            }
        }),
    });

    let intent = if siq.passing { "success" } else { "error" };
    components.push(A2uiComponent {
        id: "siq-header".to_string(),
        component_properties: serde_json::json!({
            "Heading": {
                "text": format!("System Integrity Quality: {}/100", siq.score),
                "level": 2,
                "meta": { "intent": intent }
            }
        }),
    });

    let status_text = if siq.passing {
        "All invariants passing ✓"
    } else {
        &format!("{} violation(s) detected", siq.violations.len())
    };

    components.push(A2uiComponent {
        id: "siq-status".to_string(),
        component_properties: serde_json::json!({
            "Text": {
                "text": status_text,
                "meta": { "intent": intent, "density": "compact" }
            }
        }),
    });

    let violation_ids: Vec<String> = siq
        .violations
        .iter()
        .enumerate()
        .map(|(i, _)| format!("siq-violation-{}", i))
        .collect();

    components.push(A2uiComponent {
        id: "siq-violations-list".to_string(),
        component_properties: serde_json::json!({
            "List": {
                "children": {
                    "explicitList": violation_ids
                }
            }
        }),
    });

    let mut mentions = Vec::new();

    for (i, violation) in siq.violations.iter().enumerate() {
        // Enriched UX: Failing nodes map to mention graph edges in the polymorphic block!
        for node_id in &violation.affected_nodes {
            mentions.push(RelationMention {
                to_block_id: node_id.clone(),
                label: Some(format!("Failed Invariant: {}", violation.policy_id)),
                source_path: Some(format!("siq-violation-{}", i)),
            });
        }

        components.push(A2uiComponent {
            id: format!("siq-violation-{}", i),
            component_properties: serde_json::json!({
                "Card": {
                    "child": format!("siq-violation-{}-content", i)
                }
            }),
        });

        components.push(A2uiComponent {
            id: format!("siq-violation-{}-content", i),
            component_properties: serde_json::json!({
                "Column": {
                    "children": {
                        "explicitList": [
                            format!("siq-violation-{}-title", i),
                            format!("siq-violation-{}-detail", i),
                            format!("siq-violation-{}-actions", i)
                        ]
                    }
                }
            }),
        });

        components.push(A2uiComponent {
            id: format!("siq-violation-{}-title", i),
            component_properties: serde_json::json!({
                "Heading": {
                    "text": format!("[{}] {}", violation.severity, violation.policy_id),
                    "level": 4,
                    "meta": { "intent": "error" }
                }
            }),
        });

        components.push(A2uiComponent {
            id: format!("siq-violation-{}-detail", i),
            component_properties: serde_json::json!({
                "Text": {
                    "text": violation.message,
                    "meta": { "density": "compact" }
                }
            }),
        });

        components.push(A2uiComponent {
            id: format!("siq-violation-{}-actions", i),
            component_properties: serde_json::json!({
                "Row": {
                    "children": {
                        "explicitList": [
                            format!("siq-violation-{}-fix-btn", i),
                            format!("siq-violation-{}-verify-btn", i)
                        ]
                    }
                }
            }),
        });

        components.push(A2uiComponent {
            id: format!("siq-violation-{}-fix-btn", i),
            component_properties: serde_json::json!({
                "Button": {
                    "label": "Generate Fix",
                    "action": {
                        "type": "dispatch",
                        "name": "generate_remediation",
                        "payload": { "policy_id": violation.policy_id }
                    },
                    "meta": { "intent": "brand" }
                }
            }),
        });

        components.push(A2uiComponent {
            id: format!("siq-violation-{}-verify-btn", i),
            component_properties: serde_json::json!({
                "Button": {
                    "label": "Re-Evaluate",
                    "action": {
                        "type": "dispatch",
                        "name": "re_evaluate_projection",
                        "payload": { "policy_id": violation.policy_id }
                    },
                    "meta": { "intent": "info" }
                }
            }),
        });
    }

    messages.push(SiqSurfaceMessage {
        surface_id: surface_id.to_string(),
        content: SiqSurfaceContent::SurfaceUpdate {
            components,
            data_model_update: Some(serde_json::json!({
                "contents": {
                    "score": siq.score,
                    "passing": siq.passing,
                    "violationCount": siq.violations.len()
                }
            })),
        },
    });

    EmitHeapBlock {
        schema_version: "1.0.0".to_string(),
        mode: "heap".to_string(),
        workspace_id: workspace_id.to_string(),
        source: HeapBlockSource {
            agent_id: "cortex-invariant-engine".to_string(),
            session_id: "system".to_string(),
            request_id: "system".to_string(),
            emitted_at: Utc::now().to_rfc3339(),
        },
        block: HeapBlockMeta {
            id: block_id.to_string(),
            block_type: "siq-scorecard".to_string(),
            title: "System Integrity Scorecard".to_string(),
            icon: Some("shield-check".to_string()),
            color: if siq.passing {
                Some("green".to_string())
            } else {
                Some("red".to_string())
            },
            attributes: Some(std::collections::BTreeMap::from([
                ("component".to_string(), "invariant_engine".to_string()),
                ("score".to_string(), siq.score.to_string()),
            ])),
            behaviors: if !siq.passing {
                Some(vec!["pinned".to_string(), "urgent".to_string()]) // Failing SIQ blocks are pinned and urgent
            } else {
                None // Passing blocks are standard
            },
        },
        content: HeapBlockContent {
            payload_type: "a2ui".to_string(),
            a2ui: A2uiPayload {
                surface_id: surface_id.to_string(),
                protocol_version: "0.8".to_string(),
                tree: serde_json::to_value(messages).unwrap_or(Value::Null),
                data_model: serde_json::json!({}),
            },
        },
        relations: HeapBlockRelations {
            tags: vec![],
            mentions,
            page_links: vec![],
        },
        crdt_projection: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cortex_domain::integrity::invariant::InvariantViolation;

    #[test]
    fn passing_siq_produces_success_scorecard_heap_block() {
        let siq = SystemIntegrityQuality {
            score: 100,
            passing: true,
            violations: vec![],
        };

        let block = format_siq_as_heap_block("ws-1", "block-1", "siq-surface-1", &siq);

        assert_eq!(block.content.payload_type, "a2ui");
        assert_eq!(block.relations.mentions.len(), 0);

        let json = serde_json::to_string_pretty(&block).expect("should serialize");
        assert!(json.contains("success"));
        assert!(json.contains("100/100"));
    }

    #[test]
    fn failing_siq_produces_violation_cards_with_actions_and_mentions() {
        let siq = SystemIntegrityQuality {
            score: 60,
            passing: false,
            violations: vec![
                InvariantViolation {
                    policy_id: "file:no_orphans".to_string(),
                    message: "Initiative 034 missing physical directory".to_string(),
                    severity: "Warning".to_string(),
                    affected_nodes: vec!["initiative-034".to_string()],
                },
                InvariantViolation {
                    policy_id: "dep:no_unpinned".to_string(),
                    message: "Dependency 'serde' is unpinned".to_string(),
                    severity: "Violation".to_string(),
                    affected_nodes: vec!["cargo-serde".to_string()],
                },
            ],
        };

        let block = format_siq_as_heap_block("ws-1", "block-2", "siq-surface-2", &siq);

        assert_eq!(block.content.payload_type, "a2ui");
        assert_eq!(block.relations.mentions.len(), 2);
        assert_eq!(block.relations.mentions[0].to_block_id, "initiative-034");

        let json = serde_json::to_string_pretty(&block).expect("should serialize");
        assert!(json.contains("error"));
        assert!(json.contains("60/100"));
        assert!(json.contains("generate_remediation"));
        assert!(json.contains("re_evaluate_projection"));
        assert!(json.contains("no_orphans"));
        assert!(json.contains("no_unpinned"));
    }
}
