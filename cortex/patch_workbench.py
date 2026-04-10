import re
import sys

def main():
    with open('/Users/xaoj/ICP/cortex/apps/cortex-eudaemon/src/services/workbench_ux.rs', 'r') as f:
        content = f.read()

    # We need to replace `generate_flows_viewspec`
    flows_start = content.find("fn generate_flows_viewspec() -> ViewSpecV1 {")
    flows_end = content.find("fn generate_initiatives_viewspec() -> ViewSpecV1 {")
    if flows_start == -1 or flows_end == -1:
        print("Could not find flows")
        sys.exit(1)

    new_flows = """fn generate_flows_viewspec() -> ViewSpecV1 {
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
            props: BTreeMap::from([(
                "widgetType".to_string(),
                Value::String("Grid".to_string()),
            )]),
            a11y: None,
            children: vec![
                "flows_metric_total".to_string(),
                "flows_metric_success".to_string(),
                "flows_metric_fail".to_string(),
                "flows_metric_inflight".to_string()
            ],
        },
        ComponentRef {
            component_id: "flows_metric_total".to_string(),
            component_type: "Container".to_string(),
            props: BTreeMap::from([
                ("widgetType".to_string(), Value::String("MetricCard".to_string())),
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
                ("widgetType".to_string(), Value::String("MetricCard".to_string())),
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
                ("widgetType".to_string(), Value::String("MetricCard".to_string())),
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
                ("widgetType".to_string(), Value::String("MetricCard".to_string())),
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
                ("widgetType".to_string(), Value::String("DataTable".to_string())),
                ("columns".to_string(), Value::Array(vec![
                    Value::String("Run ID".to_string()),
                    Value::String("Mode".to_string()),
                    Value::String("Status".to_string()),
                ])),
                ("data".to_string(), Value::Array(vec![
                    Value::Array(vec![Value::String("run-a1b2c3d4".to_string()), Value::String("Agent".to_string()), Value::String("Running".to_string())])
                ])),
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
            LayoutEdge { from: "1".to_string(), to: "2".to_string(), relation: "flows_to".to_string() },
            LayoutEdge { from: "2".to_string(), to: "3".to_string(), relation: "flows_to".to_string() },
            LayoutEdge { from: "3".to_string(), to: "4".to_string(), relation: "flows_to".to_string() }
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
"""

    content = content[:flows_start] + new_flows + content[flows_end:]

    init_start = content.find("fn generate_initiatives_viewspec() -> ViewSpecV1 {")
    init_end = content.find("fn generate_studio_viewspec() -> ViewSpecV1 {")

    if init_start == -1 or init_end == -1:
        print("Could not find initiatives")
        sys.exit(1)

    new_init = """fn generate_initiatives_viewspec() -> ViewSpecV1 {
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
            children: vec!["init_stat_active".to_string(), "init_stat_planned".to_string()],
        },
        ComponentRef {
            component_id: "init_stat_active".to_string(),
            component_type: "Container".to_string(),
            props: BTreeMap::from([
                ("widgetType".to_string(), Value::String("MetricCard".to_string())),
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
                ("widgetType".to_string(), Value::String("MetricCard".to_string())),
                ("label".to_string(), Value::String("Planned".to_string())),
                ("value".to_string(), Value::String("12".to_string())),
            ]),
            a11y: None,
            children: vec![],
        },
        ComponentRef {
            component_id: "initiatives_grid".to_string(),
            component_type: "Container".to_string(),
            props: BTreeMap::from([(
                "widgetType".to_string(),
                Value::String("Grid".to_string()),
            )]),
            a11y: None,
            children: vec![
                "init_card_1".to_string(),
                "init_card_2".to_string(),
            ],
        },
        ComponentRef {
            component_id: "init_card_1".to_string(),
            component_type: "Container".to_string(),
            props: BTreeMap::from([
                ("widgetType".to_string(), Value::String("HeapBlockCard".to_string())),
                ("title".to_string(), Value::String("Deploy Nexus Core".to_string())),
                ("status".to_string(), Value::String("Active".to_string())),
                ("attributes".to_string(), Value::Object(serde_json::Map::from_iter(vec![
                    ("layer".to_string(), Value::String("infrastructure".to_string())),
                    ("role".to_string(), Value::String("critical".to_string()))
                ]))),
            ]),
            a11y: None,
            children: vec![],
        },
        ComponentRef {
            component_id: "init_card_2".to_string(),
            component_type: "Container".to_string(),
            props: BTreeMap::from([
                ("widgetType".to_string(), Value::String("HeapBlockCard".to_string())),
                ("title".to_string(), Value::String("Refactor Auth".to_string())),
                ("status".to_string(), Value::String("Planned".to_string())),
                ("attributes".to_string(), Value::Object(serde_json::Map::from_iter(vec![
                    ("layer".to_string(), Value::String("application".to_string())),
                ]))),
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
            LayoutEdge { from: "1".to_string(), to: "2".to_string(), relation: "flows_to".to_string() },
            LayoutEdge { from: "2".to_string(), to: "3".to_string(), relation: "flows_to".to_string() },
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
"""

    content = content[:init_start] + new_init + content[init_end:]

    # Now for spaces: replace `SpaceCreationWizard` in 
    wizard_ref_str = """    component_refs.push(ComponentRef {
        component_id: "space_creation_wizard".to_string(),
        component_type: "Container".to_string(),
        props: BTreeMap::from([(
            "widgetType".to_string(),
            Value::String("SpaceCreationWizard".to_string()),
        )]),
        a11y: None,
        children: vec![],
    });"""

    new_wizard_ref = """    // Space creation is now a sequenced pure A2UI form
    component_refs.push(ComponentRef {
        component_id: "space_creation_wizard".to_string(),
        component_type: "Container".to_string(),
        props: BTreeMap::from([(
            "widgetType".to_string(),
            Value::String("Card".to_string()),
        )]),
        a11y: None,
        children: vec![
            "wizard_title".to_string(),
            "wizard_input_name".to_string(),
            "wizard_submit".to_string()
        ],
    });

    component_refs.push(ComponentRef {
        component_id: "wizard_title".to_string(),
        component_type: "Heading".to_string(),
        props: BTreeMap::from([("text".to_string(), Value::String("Create New Space".to_string()))]),
        a11y: None,
        children: vec![],
    });

    component_refs.push(ComponentRef {
        component_id: "wizard_input_name".to_string(),
        component_type: "TextField".to_string(),
        props: BTreeMap::from([("label".to_string(), Value::String("Space Identifier".to_string()))]),
        a11y: None,
        children: vec![],
    });

    component_refs.push(ComponentRef {
        component_id: "wizard_submit".to_string(),
        component_type: "Button".to_string(),
        props: BTreeMap::from([
            ("label".to_string(), Value::String("Provision Space".to_string())),
            ("action".to_string(), Value::String("provisionSpace".to_string()))
        ]),
        a11y: None,
        children: vec![],
    });"""

    content = content.replace(wizard_ref_str, new_wizard_ref)

    with open('/Users/xaoj/ICP/cortex/apps/cortex-eudaemon/src/services/workbench_ux.rs', 'w') as f:
        f.write(content)

if __name__ == "__main__":
    main()
