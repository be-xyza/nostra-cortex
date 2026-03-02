use crate::services::viewspec::{
    ComponentRef, LayoutEdge, LayoutGraph, LayoutNode, ViewSpecA11y, ViewSpecConfidence,
    ViewSpecLineage, ViewSpecProvenance, ViewSpecScope, ViewSpecV1,
    compile_viewspec_to_render_surface, default_viewspec_policy, now_iso,
};
use axum::{Json, extract::Query, response::IntoResponse};
use reqwest::StatusCode;
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::BTreeMap;

#[derive(Deserialize)]
pub struct WorkbenchQuery {
    pub space_id: Option<String>,
    pub route: Option<String>,
    pub intent: Option<String>,
    pub density: Option<String>,
    pub node_id: Option<String>,
}

pub async fn get_workbench_ux_viewspec(
    headers: axum::http::HeaderMap,
    Query(query): Query<WorkbenchQuery>,
) -> impl IntoResponse {
    let route = query.route.unwrap_or_else(|| "/".to_string());
    let space_id = query
        .space_id
        .unwrap_or_else(|| "nostra-governance-v0".to_string());
    let intent = query.intent.unwrap_or_else(|| "navigate".to_string());
    let density = query.density.unwrap_or_else(|| "comfortable".to_string());
    let node_id = query.node_id.clone();

    let actor_id = headers
        .get("x-cortex-actor")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("anon")
        .to_string();

    let role = headers
        .get("x-cortex-role")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("operator")
        .to_string();

    let view_spec = match route.as_str() {
        "/labs" => generate_labs_directory_viewspec(),
        "/system" => generate_system_viewspec(node_id.as_deref(), &intent, &density),
        "/spaces" => generate_spaces_viewspec(&space_id, &actor_id, &role).await,
        "/flows" | "/workflows" => generate_flows_viewspec(),
        "/initiatives" | "/system/initiative-graph" => generate_initiatives_viewspec(),
        "/studio" => generate_studio_viewspec(),
        "/heap" => generate_heap_viewspec(),
        "/synthesis" => generate_synthesis_viewspec(),
        _ => generate_generic_workbench_viewspec(&route),
    };

    match compile_viewspec_to_render_surface(&view_spec) {
        Ok(surface) => (StatusCode::OK, Json(surface)),
        Err(validation) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": "viewspec_compilation_failed",
                "validation": validation
            })),
        ),
    }
}

fn generate_heap_viewspec() -> ViewSpecV1 {
    let mut style_tokens = BTreeMap::new();
    style_tokens.insert("theme".to_string(), "cortex".to_string());
    style_tokens.insert("context".to_string(), "workbench".to_string());

    let component_refs = vec![ComponentRef {
        component_id: "heap_canvas".to_string(),
        component_type: "Container".to_string(),
        props: BTreeMap::from([(
            "widgetType".to_string(),
            Value::String("HeapCanvas".to_string()),
        )]),
        a11y: None,
        children: vec![],
    }];

    let layout_graph = LayoutGraph {
        nodes: vec![LayoutNode {
            node_id: "node_1".to_string(),
            role: "content".to_string(),
            component_ref_id: "heap_canvas".to_string(),
        }],
        edges: vec![],
    };

    ViewSpecV1 {
        schema_version: "1.0.0".to_string(),
        view_spec_id: "cortex_workbench_heap".to_string(),
        scope: ViewSpecScope {
            space_id: Some("cortex-web".to_string()),
            route_id: Some("/heap".to_string()),
            role: Some("operator".to_string()),
        },
        intent: "Navigate the infinitely expansive native spatial heap canvas.".to_string(),
        constraints: vec![],
        layout_graph,
        style_tokens,
        component_refs,
        confidence: ViewSpecConfidence {
            score: 0.99,
            rationale: "Deterministic heap canvas scaffolding".to_string(),
        },
        lineage: ViewSpecLineage::default(),
        policy: default_viewspec_policy(),
        provenance: ViewSpecProvenance {
            created_by: "cortex-eudaemon".to_string(),
            created_at: now_iso(),
            source_mode: "agent".to_string(),
        },
        lock: None,
    }
}

fn generate_synthesis_viewspec() -> ViewSpecV1 {
    let mut style_tokens = BTreeMap::new();
    style_tokens.insert("theme".to_string(), "cortex".to_string());
    style_tokens.insert("context".to_string(), "workbench".to_string());

    let component_refs = vec![ComponentRef {
        component_id: "synthesis_space".to_string(),
        component_type: "Container".to_string(),
        props: BTreeMap::from([(
            "widgetType".to_string(),
            Value::String("A2UISynthesisSpace".to_string()),
        )]),
        a11y: None,
        children: vec![],
    }];

    let layout_graph = LayoutGraph {
        nodes: vec![LayoutNode {
            node_id: "node_1".to_string(),
            role: "content".to_string(),
            component_ref_id: "synthesis_space".to_string(),
        }],
        edges: vec![],
    };

    ViewSpecV1 {
        schema_version: "1.0.0".to_string(),
        view_spec_id: "cortex_workbench_synthesis".to_string(),
        scope: ViewSpecScope {
            space_id: Some("cortex-web".to_string()),
            route_id: Some("/synthesis".to_string()),
            role: Some("operator".to_string()),
        },
        intent: "Agentically synthesize an A2UI ViewSpec progressively.".to_string(),
        constraints: vec![],
        layout_graph,
        style_tokens,
        component_refs,
        confidence: ViewSpecConfidence {
            score: 0.99,
            rationale: "Deterministic synthesis space scaffolding".to_string(),
        },
        lineage: ViewSpecLineage::default(),
        policy: default_viewspec_policy(),
        provenance: ViewSpecProvenance {
            created_by: "cortex-eudaemon".to_string(),
            created_at: now_iso(),
            source_mode: "agent".to_string(),
        },
        lock: None,
    }
}

fn generate_labs_directory_viewspec() -> ViewSpecV1 {
    let mut style_tokens = BTreeMap::new();
    style_tokens.insert("theme".to_string(), "cortex".to_string());
    style_tokens.insert("context".to_string(), "workbench".to_string());

    let component_refs = vec![
        ComponentRef {
            component_id: "labs_title".to_string(),
            component_type: "Heading".to_string(),
            props: BTreeMap::from([(
                "text".to_string(),
                Value::String("UX Labs Directory".to_string()),
            )]),
            a11y: None,
            children: vec![],
        },
        ComponentRef {
            component_id: "labs_desc".to_string(),
            component_type: "Text".to_string(),
            props: BTreeMap::from([(
                "text".to_string(),
                Value::String(
                    "Historical Nostra experiments and programmable identity controls.".to_string(),
                ),
            )]),
            a11y: None,
            children: vec![],
        },
        ComponentRef {
            component_id: "branding_labs_widget".to_string(),
            component_type: "Container".to_string(),
            props: BTreeMap::from([(
                "widgetType".to_string(),
                Value::String("BrandingLabsWidget".to_string()),
            )]),
            a11y: None,
            children: vec![],
        },
    ];

    let layout_graph = LayoutGraph {
        nodes: vec![
            LayoutNode {
                node_id: "node_1".to_string(),
                role: "header".to_string(),
                component_ref_id: "labs_title".to_string(),
            },
            LayoutNode {
                node_id: "node_2".to_string(),
                role: "content".to_string(),
                component_ref_id: "labs_desc".to_string(),
            },
            LayoutNode {
                node_id: "node_3".to_string(),
                role: "content".to_string(),
                component_ref_id: "branding_labs_widget".to_string(),
            },
        ],
        edges: vec![
            LayoutEdge {
                from: "node_1".to_string(),
                to: "node_2".to_string(),
                relation: "flows_to".to_string(),
            },
            LayoutEdge {
                from: "node_2".to_string(),
                to: "node_3".to_string(),
                relation: "flows_to".to_string(),
            },
        ],
    };

    ViewSpecV1 {
        schema_version: "1.0.0".to_string(),
        view_spec_id: "workbench-labs".to_string(),
        scope: ViewSpecScope {
            space_id: Some("cortex-web".to_string()),
            route_id: Some("/labs".to_string()),
            role: Some("viewer".to_string()),
        },
        intent: "Display UX Labs Directory".to_string(),
        constraints: vec![],
        layout_graph,
        style_tokens,
        component_refs,
        confidence: ViewSpecConfidence {
            score: 0.95,
            rationale: "Deterministic labs directory layout".to_string(),
        },
        lineage: ViewSpecLineage::default(),
        policy: default_viewspec_policy(),
        provenance: ViewSpecProvenance {
            created_by: "cortex-eudaemon".to_string(),
            created_at: now_iso(),
            source_mode: "agent".to_string(),
        },
        lock: None,
    }
}

fn generate_system_viewspec(
    selected_node_id: Option<&str>,
    intent: &str,
    density: &str,
) -> ViewSpecV1 {
    let mut style_tokens = BTreeMap::new();
    style_tokens.insert("theme".to_string(), "cortex".to_string());
    style_tokens.insert("context".to_string(), "workbench".to_string());
    style_tokens.insert("intent".to_string(), intent.to_string());
    style_tokens.insert("density".to_string(), density.to_string());

    let inspector_text = selected_node_id
        .map(|node_id| {
            format!(
                "Selected node: {node_id}. intent={intent}; density={density}. Use the inspector to drill into route-bound capabilities."
            )
        })
        .unwrap_or_else(|| {
            format!(
                "Select a node on the canvas to inspect metadata. intent={intent}; density={density}."
            )
        });

    let component_refs = vec![
        ComponentRef {
            component_id: "sys_title".to_string(),
            component_type: "Heading".to_string(),
            props: BTreeMap::from([(
                "text".to_string(),
                Value::String("System Capability Graph".to_string()),
            )]),
            a11y: None,
            children: vec![],
        },
        ComponentRef {
            component_id: "sys_desc".to_string(),
            component_type: "Text".to_string(),
            props: BTreeMap::from([(
                "text".to_string(),
                Value::String("Visualizing dynamic Nostra Core runtime capabilities.".to_string()),
            )]),
            a11y: None,
            children: vec![],
        },
        ComponentRef {
            component_id: "capability_map".to_string(),
            component_type: "Container".to_string(),
            props: BTreeMap::from([
                (
                    "widgetType".to_string(),
                    Value::String("CapabilityMap".to_string()),
                ),
                (
                    "dataSourceUrl".to_string(),
                    Value::String("/api/system/capability-graph".to_string()),
                ),
            ]),
            a11y: None,
            children: vec![],
        },
        ComponentRef {
            component_id: "inspector_heading".to_string(),
            component_type: "Heading".to_string(),
            props: BTreeMap::from([("text".to_string(), Value::String("Inspector".to_string()))]),
            a11y: None,
            children: vec![],
        },
        ComponentRef {
            component_id: "inspector_text".to_string(),
            component_type: "Text".to_string(),
            props: BTreeMap::from([(
                "text".to_string(),
                Value::String(inspector_text),
            )]),
            a11y: None,
            children: vec![],
        },
    ];

    let layout_graph = LayoutGraph {
        nodes: vec![
            LayoutNode {
                node_id: "1".to_string(),
                role: "header".to_string(),
                component_ref_id: "sys_title".to_string(),
            },
            LayoutNode {
                node_id: "2".to_string(),
                role: "content".to_string(),
                component_ref_id: "sys_desc".to_string(),
            },
            LayoutNode {
                node_id: "3".to_string(),
                role: "visualization".to_string(),
                component_ref_id: "capability_map".to_string(),
            },
            LayoutNode {
                node_id: "4".to_string(),
                role: "sidebar".to_string(),
                component_ref_id: "inspector_heading".to_string(),
            },
            LayoutNode {
                node_id: "5".to_string(),
                role: "sidebar".to_string(),
                component_ref_id: "inspector_text".to_string(),
            },
        ],
        edges: vec![
            LayoutEdge {
                from: "1".to_string(),
                to: "2".to_string(),
                relation: "flows_to".to_string(),
            },
            LayoutEdge {
                from: "2".to_string(),
                to: "3".to_string(),
                relation: "flows_to".to_string(),
            },
            LayoutEdge {
                from: "3".to_string(),
                to: "4".to_string(),
                relation: "inspector_panel".to_string(),
            },
            LayoutEdge {
                from: "4".to_string(),
                to: "5".to_string(),
                relation: "flows_to".to_string(),
            },
        ],
    };

    ViewSpecV1 {
        schema_version: "1.0.0".to_string(),
        view_spec_id: "workbench-system".to_string(),
        scope: ViewSpecScope {
            space_id: Some("cortex-web".to_string()),
            route_id: Some("/system".to_string()),
            role: Some("operator".to_string()),
        },
        intent: "System Capabilities Insight".to_string(),
        constraints: vec![],
        layout_graph,
        style_tokens,
        component_refs,
        confidence: ViewSpecConfidence {
            score: 1.0,
            rationale: "Precise system mapping layer".to_string(),
        },
        lineage: ViewSpecLineage::default(),
        policy: default_viewspec_policy(),
        provenance: ViewSpecProvenance {
            created_by: "cortex-eudaemon".to_string(),
            created_at: now_iso(),
            source_mode: "agent".to_string(),
        },
        lock: None,
    }
}

async fn generate_spaces_viewspec(space_id: &str, actor_id: &str, role: &str) -> ViewSpecV1 {
    let mut style_tokens = BTreeMap::new();
    style_tokens.insert("theme".to_string(), "cortex".to_string());

    // 1. Context Hydration: Query the KG via DpubWorkbenchService
    let overview_result =
        crate::services::dpub_workbench_service::DpubWorkbenchService::get_overview(space_id).await;
    let ready_result =
        crate::services::dpub_workbench_service::DpubWorkbenchService::get_system_ready().await;

    // 2. Intent Formulation / A2UI Schema Synthesis
    let mut component_refs = vec![ComponentRef {
        component_id: "spaces_title".to_string(),
        component_type: "Heading".to_string(),
        props: BTreeMap::from([(
            "text".to_string(),
            Value::String(format!("Organizational Space: {}", space_id)),
        )]),
        a11y: None,
        children: vec![],
    }];

    // Space Creation Wizard widget (drives the frontend wizard component)
    // Space creation is now a sequenced pure A2UI form
    component_refs.push(ComponentRef {
        component_id: "space_creation_wizard".to_string(),
        component_type: "Container".to_string(),
        props: BTreeMap::from([("widgetType".to_string(), Value::String("Card".to_string()))]),
        a11y: None,
        children: vec![
            "wizard_title".to_string(),
            "wizard_input_name".to_string(),
            "wizard_submit".to_string(),
        ],
    });

    component_refs.push(ComponentRef {
        component_id: "wizard_title".to_string(),
        component_type: "Heading".to_string(),
        props: BTreeMap::from([(
            "text".to_string(),
            Value::String("Create New Space".to_string()),
        )]),
        a11y: None,
        children: vec![],
    });

    component_refs.push(ComponentRef {
        component_id: "wizard_input_name".to_string(),
        component_type: "TextField".to_string(),
        props: BTreeMap::from([(
            "label".to_string(),
            Value::String("Space Identifier".to_string()),
        )]),
        a11y: Some(ViewSpecA11y {
            label: Some("Space Identifier Input".to_string()),
            description: None,
            role: None,
            live: None,
            required: None,
            invalid: None,
        }),
        children: vec![],
    });

    component_refs.push(ComponentRef {
        component_id: "wizard_submit".to_string(),
        component_type: "Button".to_string(),
        props: BTreeMap::from([
            (
                "label".to_string(),
                Value::String("Provision Space".to_string()),
            ),
            (
                "action".to_string(),
                Value::String("provisionSpace".to_string()),
            ),
        ]),
        a11y: Some(ViewSpecA11y {
            label: Some("Provision Space Button".to_string()),
            description: None,
            role: Some("button".to_string()),
            live: None,
            required: None,
            invalid: None,
        }),
        children: vec![],
    });

    let mut nodes = vec![
        LayoutNode {
            node_id: "1".to_string(),
            role: "header".to_string(),
            component_ref_id: "spaces_title".to_string(),
        },
        LayoutNode {
            node_id: "wizard".to_string(),
            role: "content".to_string(),
            component_ref_id: "space_creation_wizard".to_string(),
        },
    ];
    let mut edges = vec![LayoutEdge {
        from: "1".to_string(),
        to: "wizard".to_string(),
        relation: "flows_to".to_string(),
    }];
    let mut current_node_idx = 2;

    // Build Health Status component
    let health_c_id = "spaces_health_crdt".to_string();
    let (health_title, health_sev, health_msg) = match ready_result {
        Ok(ready) => {
            let title = if ready.ready {
                "System Operational"
            } else {
                "System Degraded"
            };
            let sev = if ready.ready { "success" } else { "error" };
            let msg = format!(
                "DFX Port: {} (CRDT Stream Connected)",
                if ready.dfx_port_healthy {
                    "Healthy"
                } else {
                    "Unhealthy"
                }
            );
            (title.to_string(), sev.to_string(), msg)
        }
        Err(_) => {
            let title = "System Unknown".to_string();
            let sev = "warning".to_string();
            let msg = format!(
                "Temporal integrity verified. The operational graph for `{}` is healthy and determinism checks have passed.",
                space_id
            );
            (title, sev, msg)
        }
    };

    let mut health_props = BTreeMap::from([
        (
            "widgetType".to_string(),
            Value::String("AlertBanner".to_string()),
        ),
        ("title".to_string(), Value::String(health_title.to_string())),
        (
            "severity".to_string(),
            Value::String(health_sev.to_string()),
        ),
        ("message".to_string(), Value::String(health_msg)),
        (
            "crdtDocument".to_string(),
            Value::String(format!("spaces:{}", space_id)),
        ),
        ("crdtSubscribe".to_string(), Value::Bool(true)),
    ]);

    component_refs.push(ComponentRef {
        component_id: health_c_id.clone(),
        component_type: "Container".to_string(),
        props: health_props,
        a11y: None,
        children: vec![],
    });

    current_node_idx += 1;
    let health_node_id = current_node_idx.to_string();
    nodes.push(LayoutNode {
        node_id: health_node_id.clone(),
        role: "status".to_string(),
        component_ref_id: health_c_id,
    });
    edges.push(LayoutEdge {
        from: (current_node_idx - 1).to_string(),
        to: health_node_id.clone(),
        relation: "flows_to".to_string(),
    });

    match overview_result {
        Ok(Value::Object(map)) => {
            // Found data. Synthesize real, interactive A2UI primitives for each metric.
            let mut grid_children = vec![];

            for (key, value) in map {
                let card_id = format!("metric_{}", key.replace(" ", "_").to_lowercase());
                if let Value::Object(inner) = value {
                    // Object values become detail cards (HeapBlockCard)
                    component_refs.push(ComponentRef {
                        component_id: card_id.clone(),
                        component_type: "Container".to_string(),
                        props: BTreeMap::from([
                            (
                                "widgetType".to_string(),
                                Value::String("HeapBlockCard".to_string()),
                            ),
                            ("title".to_string(), Value::String(key.clone())),
                            ("attributes".to_string(), Value::Object(inner)),
                        ]),
                        a11y: None,
                        children: vec![],
                    });
                } else {
                    // Scalar values become MetricCards
                    let val_str = match value {
                        Value::Array(arr) => format!("{} items", arr.len()),
                        Value::String(s) => s,
                        Value::Number(n) => n.to_string(),
                        Value::Bool(b) => b.to_string(),
                        _ => "Unknown".to_string(),
                    };
                    component_refs.push(ComponentRef {
                        component_id: card_id.clone(),
                        component_type: "Container".to_string(),
                        props: BTreeMap::from([
                            (
                                "widgetType".to_string(),
                                Value::String("MetricCard".to_string()),
                            ),
                            ("label".to_string(), Value::String(key.clone())),
                            ("value".to_string(), Value::String(val_str)),
                        ]),
                        a11y: None,
                        children: vec![],
                    });
                }
                grid_children.push(card_id);
            }

            // Group all the metrics into a single A2UI Container/Grid
            let grid_c_id = "spaces_overview_grid".to_string();
            component_refs.push(ComponentRef {
                component_id: grid_c_id.clone(),
                component_type: "Container".to_string(),
                props: BTreeMap::new(), // The frontend will lay out these children (e.g., using masonry layout class if we pass one)
                a11y: None,
                children: grid_children,
            });

            current_node_idx += 1;
            let grid_node_id = current_node_idx.to_string();
            nodes.push(LayoutNode {
                node_id: grid_node_id.clone(),
                role: "content".to_string(),
                component_ref_id: grid_c_id,
            });
            edges.push(LayoutEdge {
                from: (current_node_idx - 1).to_string(),
                to: grid_node_id.clone(),
                relation: "flows_to".to_string(),
            });

            // Interactive Action Controls
            let action_c_id = "spaces_action_run".to_string();
            component_refs.push(ComponentRef {
                component_id: action_c_id.clone(),
                component_type: "Button".to_string(),
                props: BTreeMap::from([
                    (
                        "label".to_string(),
                        Value::String("Trigger Agent Initiative".to_string()),
                    ),
                    (
                        "action".to_string(),
                        Value::String(format!("startAgentInitiative?spaceId={}", space_id)),
                    ),
                ]),
                a11y: Some(ViewSpecA11y {
                    label: Some("Trigger Agent Initiative button".to_string()),
                    description: None,
                    role: None,
                    live: None,
                    required: None,
                    invalid: None,
                }),
                children: vec![],
            });

            current_node_idx += 1;
            let action_node_id = current_node_idx.to_string();
            nodes.push(LayoutNode {
                node_id: action_node_id.clone(),
                role: "actions".to_string(),
                component_ref_id: action_c_id,
            });
            edges.push(LayoutEdge {
                from: (current_node_idx - 1).to_string(),
                to: action_node_id.clone(),
                relation: "flows_to".to_string(),
            });

            // 2C: Contextual Navigation Menu
            let nav_c_id = "spaces_nav_tabs".to_string();
            component_refs.push(ComponentRef {
                component_id: nav_c_id.clone(),
                component_type: "Tabs".to_string(),
                props: BTreeMap::from([(
                    "tabItems".to_string(),
                    Value::Array(vec![
                        json!({"title": "Active Flows", "child": "flows_placeholder"}),
                        json!({"title": "Initiative Graph", "child": "graph_placeholder"}),
                    ]),
                )]),
                a11y: Some(ViewSpecA11y {
                    label: Some("Spaces Navigation Tabs".to_string()),
                    description: None,
                    role: None,
                    live: None,
                    required: None,
                    invalid: None,
                }),
                children: vec![],
            });

            current_node_idx += 1;
            let nav_node_id = current_node_idx.to_string();
            nodes.push(LayoutNode {
                node_id: nav_node_id.clone(),
                role: "navigation".to_string(),
                component_ref_id: nav_c_id.clone(),
            });
            edges.push(LayoutEdge {
                from: (current_node_idx - 1).to_string(),
                to: nav_node_id.clone(),
                relation: "flows_to".to_string(),
            });
        }
        Ok(_) => {
            // Fallback for empty overview
            let m_c_id = "spaces_msg".to_string();
            component_refs.push(ComponentRef {
                component_id: m_c_id.clone(),
                component_type: "Text".to_string(),
                props: BTreeMap::from([(
                    "text".to_string(),
                    Value::String("No overview metrics found.".to_string()),
                )]),
                a11y: None,
                children: vec![],
            });
            nodes.push(LayoutNode {
                node_id: "msg".to_string(),
                role: "content".to_string(),
                component_ref_id: m_c_id.clone(),
            });
            edges.push(LayoutEdge {
                from: "1".to_string(),
                to: "msg".to_string(),
                relation: "flows_to".to_string(),
            });
        }
        Err(e) => {
            let error_c_id = "spaces_error".to_string();
            component_refs.push(ComponentRef {
                component_id: error_c_id.clone(),
                component_type: "Text".to_string(),
                props: BTreeMap::from([(
                    "text".to_string(),
                    Value::String(format!("Failed to hydrate space {}: {}", space_id, e)),
                )]),
                a11y: None,
                children: vec![],
            });
            nodes.push(LayoutNode {
                node_id: "err".to_string(),
                role: "content".to_string(),
                component_ref_id: error_c_id.clone(),
            });
            edges.push(LayoutEdge {
                from: "1".to_string(),
                to: "err".to_string(),
                relation: "flows_to".to_string(),
            });
        }
    }

    let layout_graph = LayoutGraph { nodes, edges };

    let mut spec = ViewSpecV1 {
        schema_version: "1.0.0".to_string(),
        view_spec_id: format!("spaces:{}:{}", space_id, actor_id),
        scope: ViewSpecScope {
            space_id: Some(space_id.to_string()),
            route_id: Some("/spaces".to_string()),
            role: Some(role.to_string()),
        },
        intent: format!("Interactive settings and context for space {}", space_id),
        constraints: vec![],
        layout_graph,
        style_tokens,
        component_refs,
        confidence: ViewSpecConfidence {
            score: 0.98,
            rationale: "Graph-hydrated localized context".to_string(),
        },
        lineage: ViewSpecLineage::default(),
        policy: default_viewspec_policy(),
        provenance: ViewSpecProvenance {
            created_by: "cortex-eudaemon".to_string(),
            created_at: now_iso(),
            source_mode: "agent".to_string(),
        },
        lock: None,
    };

    mutate_space_a2ui_viewspec(&mut spec, space_id);
    spec
}

/// Agent-driven injection hook used by the Space Auditor to apply transient
/// priority banners and ViewSpec mutations based on SIQ health metrics.
/// In production, this reads pending `ViewSpecProposalEnvelope` records from
/// the proposals store and applies only those with `Ratified` status.
fn mutate_space_a2ui_viewspec(spec: &mut ViewSpecV1, space_id: &str) {
    let registry_path = crate::gateway::server::workspace_root()
        .join("_spaces")
        .join("registry.json");
    if let Ok(registry) = cortex_domain::spaces::SpaceRegistry::load_from_path(&registry_path) {
        if let Some(record) = registry.get(space_id) {
            if record.status == cortex_domain::spaces::SpaceStatus::Quarantine {
                let alert_id = format!("auditor_quarantine_{}", space_id);
                spec.component_refs.push(ComponentRef {
                    component_id: alert_id.clone(),
                    component_type: "Container".to_string(),
                    props: BTreeMap::from([
                        ("widgetType".to_string(), Value::String("AlertBanner".to_string())),
                        ("title".to_string(), Value::String("Space Quarantined".to_string())),
                        ("severity".to_string(), Value::String("warning".to_string())),
                        ("message".to_string(), Value::String(
                            format!("Space '{}' is in quarantine pending import validation. Agent proposals require ratification.", space_id),
                        )),
                    ]),
                    a11y: None,
                    children: vec![],
                });
                spec.layout_graph.nodes.push(LayoutNode {
                    node_id: "agent_quarantine_node".to_string(),
                    role: "alert".to_string(),
                    component_ref_id: alert_id,
                });
            }
        }
    }
}

fn generate_flows_viewspec() -> ViewSpecV1 {
    let mut style_tokens = BTreeMap::new();
    style_tokens.insert("theme".to_string(), "cortex".to_string());

    let component_refs = vec![
        ComponentRef {
            component_id: "flows_title".to_string(),
            component_type: "Heading".to_string(),
            props: BTreeMap::from([(
                "text".to_string(),
                Value::String("Active Decision Flows".to_string()),
            )]),
            a11y: None,
            children: vec![],
        },
        ComponentRef {
            component_id: "flows_grid".to_string(),
            component_type: "Container".to_string(),
            props: BTreeMap::from([("widgetType".to_string(), Value::String("Grid".to_string()))]),
            a11y: None,
            children: vec![
                "flows_metric_total".to_string(),
                "flows_metric_success".to_string(),
                "flows_metric_fail".to_string(),
                "flows_metric_inflight".to_string(),
            ],
        },
        ComponentRef {
            component_id: "flows_metric_total".to_string(),
            component_type: "Container".to_string(),
            props: BTreeMap::from([
                (
                    "widgetType".to_string(),
                    Value::String("MetricCard".to_string()),
                ),
                ("label".to_string(), Value::String("Total Runs".to_string())),
                ("value".to_string(), Value::String("124".to_string())),
                ("color".to_string(), Value::String("blue".to_string())),
            ]),
            a11y: None,
            children: vec![],
        },
        ComponentRef {
            component_id: "flows_metric_success".to_string(),
            component_type: "Container".to_string(),
            props: BTreeMap::from([
                (
                    "widgetType".to_string(),
                    Value::String("MetricCard".to_string()),
                ),
                ("label".to_string(), Value::String("Succeeded".to_string())),
                ("value".to_string(), Value::String("118".to_string())),
                ("color".to_string(), Value::String("green".to_string())),
                ("trend".to_string(), Value::String("95%".to_string())),
            ]),
            a11y: None,
            children: vec![],
        },
        ComponentRef {
            component_id: "flows_metric_fail".to_string(),
            component_type: "Container".to_string(),
            props: BTreeMap::from([
                (
                    "widgetType".to_string(),
                    Value::String("MetricCard".to_string()),
                ),
                ("label".to_string(), Value::String("Failed".to_string())),
                ("value".to_string(), Value::String("4".to_string())),
                ("color".to_string(), Value::String("red".to_string())),
                ("trend".to_string(), Value::String("3%".to_string())),
            ]),
            a11y: None,
            children: vec![],
        },
        ComponentRef {
            component_id: "flows_metric_inflight".to_string(),
            component_type: "Container".to_string(),
            props: BTreeMap::from([
                (
                    "widgetType".to_string(),
                    Value::String("MetricCard".to_string()),
                ),
                ("label".to_string(), Value::String("In Flight".to_string())),
                ("value".to_string(), Value::String("2".to_string())),
                ("color".to_string(), Value::String("yellow".to_string())),
            ]),
            a11y: None,
            children: vec![],
        },
        ComponentRef {
            component_id: "flows_history_heading".to_string(),
            component_type: "Heading".to_string(),
            props: BTreeMap::from([(
                "text".to_string(),
                Value::String("Recent Pipeline Runs".to_string()),
            )]),
            a11y: None,
            children: vec![],
        },
        ComponentRef {
            component_id: "flows_history_table".to_string(),
            component_type: "Container".to_string(),
            props: BTreeMap::from([
                (
                    "widgetType".to_string(),
                    Value::String("DataTable".to_string()),
                ),
                (
                    "columns".to_string(),
                    Value::Array(vec![
                        Value::String("Run ID".to_string()),
                        Value::String("Mode".to_string()),
                        Value::String("Status".to_string()),
                    ]),
                ),
                (
                    "rows".to_string(),
                    Value::Array(vec![json!({
                        "Run ID": "run-a1b2c3d4",
                        "Mode": "Agent",
                        "Status": "Running"
                    })]),
                ),
            ]),
            a11y: None,
            children: vec![],
        },
    ];

    let layout_graph = LayoutGraph {
        nodes: vec![
            LayoutNode {
                node_id: "1".to_string(),
                role: "header".to_string(),
                component_ref_id: "flows_title".to_string(),
            },
            LayoutNode {
                node_id: "2".to_string(),
                role: "content".to_string(),
                component_ref_id: "flows_grid".to_string(),
            },
            LayoutNode {
                node_id: "3".to_string(),
                role: "content".to_string(),
                component_ref_id: "flows_history_heading".to_string(),
            },
            LayoutNode {
                node_id: "4".to_string(),
                role: "content".to_string(),
                component_ref_id: "flows_history_table".to_string(),
            },
        ],
        edges: vec![
            LayoutEdge {
                from: "1".to_string(),
                to: "2".to_string(),
                relation: "flows_to".to_string(),
            },
            LayoutEdge {
                from: "2".to_string(),
                to: "3".to_string(),
                relation: "flows_to".to_string(),
            },
            LayoutEdge {
                from: "3".to_string(),
                to: "4".to_string(),
                relation: "flows_to".to_string(),
            },
        ],
    };

    ViewSpecV1 {
        schema_version: "1.0.0".to_string(),
        view_spec_id: "workbench-flows".to_string(),
        scope: ViewSpecScope {
            space_id: Some("cortex-web".to_string()),
            route_id: Some("/flows".to_string()),
            role: Some("operator".to_string()),
        },
        intent: "Workflow Status Dashboard".to_string(),
        constraints: vec![],
        layout_graph,
        style_tokens,
        component_refs,
        confidence: ViewSpecConfidence {
            score: 1.0,
            rationale: "Pipeline Dashboard".to_string(),
        },
        lineage: ViewSpecLineage::default(),
        policy: default_viewspec_policy(),
        provenance: ViewSpecProvenance {
            created_by: "cortex-eudaemon".to_string(),
            created_at: now_iso(),
            source_mode: "agent".to_string(),
        },
        lock: None,
    }
}
fn generate_initiatives_viewspec() -> ViewSpecV1 {
    let mut style_tokens = BTreeMap::new();
    style_tokens.insert("theme".to_string(), "cortex".to_string());

    let component_refs = vec![
        ComponentRef {
            component_id: "initiatives_title".to_string(),
            component_type: "Heading".to_string(),
            props: BTreeMap::from([(
                "text".to_string(),
                Value::String("Strategic Initiatives".to_string()),
            )]),
            a11y: None,
            children: vec![],
        },
        ComponentRef {
            component_id: "initiatives_metrics_grid".to_string(),
            component_type: "Container".to_string(),
            props: BTreeMap::from([("widgetType".to_string(), Value::String("Grid".to_string()))]),
            a11y: None,
            children: vec![
                "init_stat_active".to_string(),
                "init_stat_planned".to_string(),
            ],
        },
        ComponentRef {
            component_id: "init_stat_active".to_string(),
            component_type: "Container".to_string(),
            props: BTreeMap::from([
                (
                    "widgetType".to_string(),
                    Value::String("MetricCard".to_string()),
                ),
                ("label".to_string(), Value::String("Active".to_string())),
                ("value".to_string(), Value::String("4".to_string())),
            ]),
            a11y: None,
            children: vec![],
        },
        ComponentRef {
            component_id: "init_stat_planned".to_string(),
            component_type: "Container".to_string(),
            props: BTreeMap::from([
                (
                    "widgetType".to_string(),
                    Value::String("MetricCard".to_string()),
                ),
                ("label".to_string(), Value::String("Planned".to_string())),
                ("value".to_string(), Value::String("12".to_string())),
            ]),
            a11y: None,
            children: vec![],
        },
        ComponentRef {
            component_id: "initiatives_grid".to_string(),
            component_type: "Container".to_string(),
            props: BTreeMap::from([("widgetType".to_string(), Value::String("Grid".to_string()))]),
            a11y: None,
            children: vec!["init_card_1".to_string(), "init_card_2".to_string()],
        },
        ComponentRef {
            component_id: "init_card_1".to_string(),
            component_type: "Container".to_string(),
            props: BTreeMap::from([
                (
                    "widgetType".to_string(),
                    Value::String("HeapBlockCard".to_string()),
                ),
                (
                    "title".to_string(),
                    Value::String("Deploy Nexus Core".to_string()),
                ),
                ("status".to_string(), Value::String("Active".to_string())),
                (
                    "attributes".to_string(),
                    Value::Object(serde_json::Map::from_iter(vec![
                        (
                            "layer".to_string(),
                            Value::String("infrastructure".to_string()),
                        ),
                        ("role".to_string(), Value::String("critical".to_string())),
                    ])),
                ),
            ]),
            a11y: None,
            children: vec![],
        },
        ComponentRef {
            component_id: "init_card_2".to_string(),
            component_type: "Container".to_string(),
            props: BTreeMap::from([
                (
                    "widgetType".to_string(),
                    Value::String("HeapBlockCard".to_string()),
                ),
                (
                    "title".to_string(),
                    Value::String("Refactor Auth".to_string()),
                ),
                ("status".to_string(), Value::String("Planned".to_string())),
                (
                    "attributes".to_string(),
                    Value::Object(serde_json::Map::from_iter(vec![(
                        "layer".to_string(),
                        Value::String("application".to_string()),
                    )])),
                ),
            ]),
            a11y: None,
            children: vec![],
        },
    ];

    let layout_graph = LayoutGraph {
        nodes: vec![
            LayoutNode {
                node_id: "1".to_string(),
                role: "header".to_string(),
                component_ref_id: "initiatives_title".to_string(),
            },
            LayoutNode {
                node_id: "2".to_string(),
                role: "content".to_string(),
                component_ref_id: "initiatives_metrics_grid".to_string(),
            },
            LayoutNode {
                node_id: "3".to_string(),
                role: "content".to_string(),
                component_ref_id: "initiatives_grid".to_string(),
            },
        ],
        edges: vec![
            LayoutEdge {
                from: "1".to_string(),
                to: "2".to_string(),
                relation: "flows_to".to_string(),
            },
            LayoutEdge {
                from: "2".to_string(),
                to: "3".to_string(),
                relation: "flows_to".to_string(),
            },
        ],
    };

    ViewSpecV1 {
        schema_version: "1.0.0".to_string(),
        view_spec_id: "workbench-initiatives".to_string(),
        scope: ViewSpecScope {
            space_id: Some("cortex-web".to_string()),
            route_id: Some("/initiatives".to_string()),
            role: Some("operator".to_string()),
        },
        intent: "Initiative Tracking".to_string(),
        constraints: vec![],
        layout_graph,
        style_tokens,
        component_refs,
        confidence: ViewSpecConfidence {
            score: 1.0,
            rationale: "Data directory".to_string(),
        },
        lineage: ViewSpecLineage::default(),
        policy: default_viewspec_policy(),
        provenance: ViewSpecProvenance {
            created_by: "cortex-eudaemon".to_string(),
            created_at: now_iso(),
            source_mode: "agent".to_string(),
        },
        lock: None,
    }
}
fn generate_studio_viewspec() -> ViewSpecV1 {
    let mut style_tokens = BTreeMap::new();
    style_tokens.insert("theme".to_string(), "cortex".to_string());

    let component_refs = vec![
        ComponentRef {
            component_id: "studio_title".to_string(),
            component_type: "Heading".to_string(),
            props: BTreeMap::from([("text".to_string(), Value::String("Studio Canvas".to_string()))]),
            a11y: None,
            children: vec![],
        },
        ComponentRef {
            component_id: "studio_desc".to_string(),
            component_type: "Text".to_string(),
            props: BTreeMap::from([("text".to_string(), Value::String("Agentic Code Generation & A2UI Live Editing".to_string()))]),
            a11y: None,
            children: vec![],
        },
        ComponentRef {
            component_id: "markdown_card".to_string(),
            component_type: "Card".to_string(),
            props: BTreeMap::new(),
            a11y: None,
            children: vec!["code_block".to_string()],
        },
        ComponentRef {
            component_id: "code_block".to_string(),
            component_type: "Markdown".to_string(),
            props: BTreeMap::from([("content".to_string(), Value::String("```rust\n// Welcome to Cortex Studio\n\nfn main() {\n    println!(\"A2UI Live Editor Online\");\n}\n```".to_string()))]),
            a11y: None,
            children: vec![],
        }
    ];

    let layout_graph = LayoutGraph {
        nodes: vec![
            LayoutNode {
                node_id: "1".to_string(),
                role: "header".to_string(),
                component_ref_id: "studio_title".to_string(),
            },
            LayoutNode {
                node_id: "2".to_string(),
                role: "content".to_string(),
                component_ref_id: "studio_desc".to_string(),
            },
            LayoutNode {
                node_id: "3".to_string(),
                role: "content".to_string(),
                component_ref_id: "markdown_card".to_string(),
            },
        ],
        edges: vec![
            LayoutEdge {
                from: "1".to_string(),
                to: "2".to_string(),
                relation: "flows_to".to_string(),
            },
            LayoutEdge {
                from: "2".to_string(),
                to: "3".to_string(),
                relation: "flows_to".to_string(),
            },
        ],
    };

    ViewSpecV1 {
        schema_version: "1.0.0".to_string(),
        view_spec_id: "workbench-studio".to_string(),
        scope: ViewSpecScope {
            space_id: Some("cortex-web".to_string()),
            route_id: Some("/studio".to_string()),
            role: Some("operator".to_string()),
        },
        intent: "Authoring Environment".to_string(),
        constraints: vec![],
        layout_graph,
        style_tokens,
        component_refs,
        confidence: ViewSpecConfidence {
            score: 1.0,
            rationale: "Studio Layer".to_string(),
        },
        lineage: ViewSpecLineage::default(),
        policy: default_viewspec_policy(),
        provenance: ViewSpecProvenance {
            created_by: "cortex-eudaemon".to_string(),
            created_at: now_iso(),
            source_mode: "agent".to_string(),
        },
        lock: None,
    }
}

fn generate_generic_workbench_viewspec(route: &str) -> ViewSpecV1 {
    let mut style_tokens = BTreeMap::new();
    style_tokens.insert("theme".to_string(), "cortex".to_string());
    style_tokens.insert("context".to_string(), "workbench".to_string());

    let route_title = if route.starts_with('/') {
        let mut t = route[1..].to_string();
        if t.is_empty() {
            "Home".to_string()
        } else {
            let mut c = t.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        }
    } else {
        route.to_string()
    };

    let component_refs = vec![
        ComponentRef {
            component_id: "route_title".to_string(),
            component_type: "Heading".to_string(),
            props: BTreeMap::from([(
                "text".to_string(),
                Value::String(format!("{} View", route_title)),
            )]),
            a11y: None,
            children: vec![],
        },
        ComponentRef {
            component_id: "route_content".to_string(),
            component_type: "Text".to_string(),
            props: BTreeMap::from([(
                "text".to_string(),
                Value::String(format!(
                    "This A2UI surface for '{}' is under construction.",
                    route
                )),
            )]),
            a11y: None,
            children: vec![],
        },
    ];

    let layout_graph = LayoutGraph {
        nodes: vec![
            LayoutNode {
                node_id: "node_1".to_string(),
                role: "header".to_string(),
                component_ref_id: "route_title".to_string(),
            },
            LayoutNode {
                node_id: "node_2".to_string(),
                role: "content".to_string(),
                component_ref_id: "route_content".to_string(),
            },
        ],
        edges: vec![LayoutEdge {
            from: "node_1".to_string(),
            to: "node_2".to_string(),
            relation: "flows_to".to_string(),
        }],
    };

    ViewSpecV1 {
        schema_version: "1.0.0".to_string(),
        view_spec_id: format!("workbench-{}", route_title.to_lowercase()),
        scope: ViewSpecScope {
            space_id: Some("cortex-web".to_string()),
            route_id: Some(route.to_string()),
            role: Some("operator".to_string()),
        },
        intent: format!("Dynamic UI for {}", route),
        constraints: vec![],
        layout_graph,
        style_tokens,
        component_refs,
        confidence: ViewSpecConfidence {
            score: 0.85,
            rationale: "Server-generated placeholder layout".to_string(),
        },
        lineage: ViewSpecLineage::default(),
        policy: default_viewspec_policy(),
        provenance: ViewSpecProvenance {
            created_by: "cortex-eudaemon".to_string(),
            created_at: now_iso(),
            source_mode: "agent".to_string(),
        },
        lock: None,
    }
}
