use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NostraEntity {
    pub id: String,
    pub name: String,
    pub description: String,
    pub entity_type: Value,
    pub tags: Vec<String>,
    pub timestamp: u64,
    pub attributes: Vec<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NostraEdge {
    pub from: String,
    pub to: String,
    pub r#type: String,
    pub bidirectional: bool,
    pub timestamp: u64,
}

pub struct ParsingResult {
    pub entities: Vec<NostraEntity>,
    pub edges: Vec<NostraEdge>,
}

pub fn parse_codebase(root_dir: &Path) -> anyhow::Result<ParsingResult> {
    let mut entities = Vec::new();
    let mut edges = Vec::new();

    // Parse Research Initiatives
    let research_dir = root_dir.join("research");
    if research_dir.exists() {
        for entry in WalkDir::new(&research_dir).min_depth(1).max_depth(1) {
            let entry = entry?;
            if entry.file_type().is_dir() {
                let plan_md = entry.path().join("PLAN.md");
                if plan_md.exists() {
                    let dir_name = entry.file_name().to_string_lossy().to_string();
                    let name = format!("Research Initiative: {}", dir_name);
                    let contribution_id = format!(
                        "contribution_{}",
                        dir_name.replace("-", "_").replace(" ", "_")
                    );

                    entities.push(NostraEntity {
                        id: contribution_id.clone(),
                        name,
                        description: format!("Research plan for {}", dir_name),
                        entity_type: serde_json::json!({ "initiative": null }),
                        tags: vec!["research".to_string(), "initiative".to_string()],
                        timestamp: 1700000000,
                        attributes: vec![],
                    });

                    // Associate root space
                    edges.push(NostraEdge {
                        from: contribution_id.clone(),
                        to: "space_nostra_governance_v0".to_string(),
                        r#type: "belongs_to".to_string(),
                        bidirectional: false,
                        timestamp: 1700000000,
                    });

                    // Parse PLAN.md headers
                    if let Ok(content) = std::fs::read_to_string(&plan_md) {
                        for line in content.lines() {
                            let trimmed = line.trim();
                            if trimmed.starts_with("## ") {
                                let header_text = trimmed[3..].trim();
                                if header_text.is_empty() {
                                    continue;
                                }
                                let obj_id = format!(
                                    "{}_obj_{}",
                                    contribution_id,
                                    header_text.replace(" ", "_").to_lowercase()
                                );
                                entities.push(NostraEntity {
                                    id: obj_id.clone(),
                                    name: header_text.to_string(),
                                    description: format!("Objective from {}", dir_name),
                                    entity_type: serde_json::json!({ "objective": null }),
                                    tags: vec!["research".to_string(), "objective".to_string()],
                                    timestamp: 1700000000,
                                    attributes: vec![],
                                });
                                // Link objective to initiative
                                edges.push(NostraEdge {
                                    from: obj_id.clone(),
                                    to: contribution_id.clone(),
                                    r#type: "part_of".to_string(),
                                    bidirectional: false,
                                    timestamp: 1700000000,
                                });
                                // Link objective to space
                                edges.push(NostraEdge {
                                    from: obj_id,
                                    to: "space_nostra_governance_v0".to_string(),
                                    r#type: "belongs_to".to_string(),
                                    bidirectional: false,
                                    timestamp: 1700000000,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    // Parse Core Crates Delivery
    let crates_dirs = vec![root_dir.join("cortex"), root_dir.join("nostra")];
    for c_dir in crates_dirs {
        if !c_dir.exists() {
            continue;
        }
        for entry in WalkDir::new(&c_dir).min_depth(2).max_depth(3) {
            let entry = entry?;
            let cargo_toml = entry.path().join("Cargo.toml");
            if entry.file_type().is_dir() && cargo_toml.exists() {
                let crate_name = entry.file_name().to_string_lossy().to_string();
                let crate_id = format!("deliverable_{}", crate_name.replace("-", "_"));

                entities.push(NostraEntity {
                    id: crate_id.clone(),
                    name: format!("Crate: {}", crate_name),
                    description: format!("Rust crate: {}", crate_name),
                    entity_type: serde_json::json!({ "deliverable": null }),
                    tags: vec![
                        "codebase".to_string(),
                        "rust".to_string(),
                        "crate".to_string(),
                    ],
                    timestamp: 1700000000,
                    attributes: vec![],
                });

                edges.push(NostraEdge {
                    from: crate_id.clone(),
                    to: "space_nostra_governance_v0".to_string(),
                    r#type: "belongs_to".to_string(),
                    bidirectional: false,
                    timestamp: 1700000000,
                });

                // Parse Cargo dependencies
                if let Ok(content) = std::fs::read_to_string(&cargo_toml) {
                    let mut in_deps = false;
                    for line in content.lines() {
                        let trimmed = line.trim();
                        if trimmed.starts_with("[dependencies]")
                            || trimmed.starts_with("[dev-dependencies]")
                        {
                            in_deps = true;
                            continue;
                        } else if trimmed.starts_with("[") {
                            in_deps = false;
                        }

                        if in_deps && trimmed.contains("=") && !trimmed.starts_with("#") {
                            if let Some(dep_name) = trimmed.split('=').next() {
                                let dep_name = dep_name.trim();
                                // Check if it's an internal nostra/cortex crate
                                if dep_name.starts_with("nostra-")
                                    || dep_name.starts_with("cortex-")
                                    || dep_name == "cortex"
                                    || dep_name == "nostra"
                                {
                                    let dep_id =
                                        format!("deliverable_{}", dep_name.replace("-", "_"));
                                    edges.push(NostraEdge {
                                        from: crate_id.clone(),
                                        to: dep_id,
                                        r#type: "depends_on".to_string(),
                                        bidirectional: false,
                                        timestamp: 1700000000,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(ParsingResult { entities, edges })
}
