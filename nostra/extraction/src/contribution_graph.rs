use anyhow::{Context, Result, anyhow, bail};
use cortex_domain::graph::diff::structural_graph;
use cortex_domain::graph::traversal::{dependency_walk, detect_cycles, topological_sort};
use cortex_domain::graph::{Edge, EdgeKind, Graph, Node};
use cortex_domain::integrity::predicate::{
    Constraint, Direction, EdgeSelector, IntegrityPredicate, NodeSelector,
};
use cortex_domain::integrity::rule::{IntegrityRule, IntegrityScope, Severity};
use cortex_domain::integrity::{IntegrityViolation, evaluate_all};
use cortex_domain::simulation::{
    SimulationSession, parse_scenario_yaml, run_deterministic_session,
};
use nostra_resource_ref::{PredicateRef, ResourceRef};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::iter::Peekable;
use std::path::{Path, PathBuf};
use std::str::Lines;
use std::time::{SystemTime, UNIX_EPOCH};

const ALLOWED_STATUS: [&str; 8] = [
    "draft",
    "active",
    "paused",
    "deferred",
    "completed",
    "superseded",
    "archived",
    "placeholder",
];

const ALLOWED_PORTFOLIO_ROLE: [&str; 4] = ["anchor", "satellite", "reference", "placeholder"];

const SCHEMA_CONTRIBUTION_NODE_V1: &str = "nostra.contribution_node.v1";
const SCHEMA_CONTRIBUTION_EDGE_V1: &str = "nostra.contribution_edge.v1";
const SCHEMA_CONTRIBUTION_GRAPH_V1: &str = "nostra.contribution_graph.v1";
const SCHEMA_PATH_ASSESSMENT_V1: &str = "nostra.path_assessment.v1";
const SCHEMA_PATH_ASSESSMENT_BUNDLE_V1: &str = "nostra.path_assessment_bundle.v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Stewardship {
    pub layer: String,
    pub primary_steward: String,
    pub domain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContributionKind {
    Initiative,
    Pr,
    Bounty,
    Decision,
    Question,
    Task,
}

impl Default for ContributionKind {
    fn default() -> Self {
        Self::Initiative
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ContributionNodeV1 {
    pub schema_id: String,
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_ref: Option<String>,
    pub title: String,
    #[serde(default)]
    pub kind: ContributionKind,
    pub status: String,
    pub layer: String,
    pub portfolio_role: String,
    pub structural_pivot_impact: String,
    pub tags: Vec<String>,
    pub stewardship: Stewardship,
    pub source_paths: Vec<String>,
    pub source_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ContributionEdgeV1 {
    pub schema_id: String,
    pub from: String,
    pub to: String,
    pub edge_kind: String,
    pub evidence_ref: String,
    #[serde(default)]
    pub evidence_lines: Vec<usize>,
    pub confidence: f32,
    #[serde(default)]
    pub is_explicit: bool,
    pub extracted_by: String,
    pub extracted_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct IntegrityCounts {
    pub critical: usize,
    pub violation: usize,
    pub warning: usize,
    pub info: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct BuildReport {
    pub strict_mode: bool,
    pub generated_at: String,
    pub contribution_count: usize,
    pub edge_count: usize,
    pub unresolved_references: Vec<String>,
    pub source_index_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct ExtractionReport {
    pub explicit_edge_count: usize,
    pub inferred_edge_count: usize,
    pub inferred_edge_ratio: f32,
    pub confidence_distribution: BTreeMap<String, usize>,
    pub unresolved_references: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct IntegrityReport {
    pub counts: IntegrityCounts,
    pub cycles: Vec<Vec<String>>,
    pub violations: Vec<IntegrityViolation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct ScoreBreakdown {
    pub critical: usize,
    pub violation: usize,
    pub warning: usize,
    pub superseded_edges: usize,
    pub cross_layer_jumps: usize,
    pub unresolved_ref_penalty: usize,
    pub total: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct PathEdgeEvidence {
    pub from: String,
    pub to: String,
    pub evidence_ref: String,
    pub evidence_lines: Vec<usize>,
    pub confidence: f32,
    pub is_explicit: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PathCandidate {
    pub name: String,
    pub node_ids: Vec<String>,
    pub risk_score: usize,
    pub blocking_nodes: Vec<String>,
    pub rationale: String,
    #[serde(default)]
    pub score_breakdown: ScoreBreakdown,
    #[serde(default)]
    pub rule_violations: Vec<String>,
    #[serde(default)]
    pub evidence: Vec<PathEdgeEvidence>,
    #[serde(default)]
    pub used_inferred_fallback: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PathAssessmentV1 {
    pub schema_id: String,
    pub goal: String,
    pub candidate_paths: Vec<PathCandidate>,
    pub recommended_path: PathCandidate,
    #[serde(default)]
    pub risk_score: usize,
    #[serde(default)]
    pub blocking_nodes: Vec<String>,
    #[serde(default)]
    pub score_breakdown: ScoreBreakdown,
    #[serde(default)]
    pub rule_violations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ContributionGraphV1 {
    pub schema_id: String,
    pub generated_at: String,
    pub nodes: Vec<ContributionNodeV1>,
    pub edges: Vec<ContributionEdgeV1>,
    pub graph_root_hash: String,
    pub integrity_report: IntegrityReport,
    pub build_report: BuildReport,
    #[serde(default)]
    pub extraction_report: ExtractionReport,
    pub path_assessment: PathAssessmentV1,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PathAssessmentBundleV1 {
    pub schema_id: String,
    pub generated_at: String,
    pub graph_root_hash: String,
    pub assessments: Vec<PathAssessmentV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct QueryResult {
    pub node_id: String,
    pub edge_kind: String,
    pub related_nodes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PublishResult {
    pub version: String,
    pub edition_manifest_path: String,
    pub snapshot_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DoctorRuleBacklog {
    pub rule_id: String,
    pub severity: String,
    pub affected_contributions: Vec<String>,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DoctorReport {
    pub schema_id: String,
    pub generated_at: String,
    pub graph_root_hash: String,
    pub integrity_counts: IntegrityCounts,
    pub unresolved_references: usize,
    pub thresholds: BTreeMap<String, usize>,
    pub pass: bool,
    pub backlog: Vec<DoctorRuleBacklog>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ExplainPathReport {
    pub schema_id: String,
    pub goal: String,
    pub graph_root_hash: String,
    pub recommended_path: PathCandidate,
    pub candidate_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct EditionDiffReport {
    pub schema_id: String,
    pub from_edition: String,
    pub to_edition: String,
    pub from_hash: String,
    pub to_hash: String,
    pub structural_diff: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct PlanFrontmatter {
    status: String,
    portfolio_role: String,
    title: String,
    tags: Vec<String>,
    depends_on: Vec<String>,
    blocks: Vec<String>,
    invalidated_by: Vec<String>,
    supersedes: Vec<String>,
    structural_pivot_impact: String,
    stewardship_layer: String,
    stewardship_primary_steward: String,
    stewardship_domain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DpubManifestV1 {
    pub schema_id: String,
    pub title: String,
    pub corpus_root: String,
    pub current_edition: String,
    pub graph_artifact: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
struct EditionManifestV1 {
    schema_id: String,
    edition_version: String,
    generated_at: String,
    graph_root_hash: String,
    snapshot_path: String,
    previous_edition: Option<String>,
    structural_diff_from_previous: Option<serde_json::Value>,
}

pub fn validate_research_portfolio(root: &Path) -> Result<()> {
    let _ = build_graph(root)?;
    Ok(())
}

pub fn ingest_and_write(root: &Path) -> Result<ContributionGraphV1> {
    let graph = build_graph(root)?;
    write_graph_bundle(root, &graph)?;
    Ok(graph)
}

pub fn query_graph(root: &Path, kind: &str, node_id: &str) -> Result<QueryResult> {
    let graph = load_graph_artifact(root)?;
    let mut related = BTreeSet::new();
    for edge in &graph.edges {
        if edge.edge_kind == kind && edge.from == node_id {
            related.insert(edge.to.clone());
        }
    }
    Ok(QueryResult {
        node_id: node_id.to_string(),
        edge_kind: kind.to_string(),
        related_nodes: related.into_iter().collect(),
    })
}

pub fn assess_path(root: &Path, goal: &str) -> Result<PathAssessmentV1> {
    let graph = load_graph_artifact(root)?;
    let (domain_graph, node_lookup, unresolved) = to_domain_graph(&graph.nodes, &graph.edges)?;
    Ok(compute_path_assessment(
        goal,
        &domain_graph,
        &node_lookup,
        &graph.edges,
        &graph.integrity_report,
        &unresolved,
    ))
}

pub fn doctor(root: &Path) -> Result<DoctorReport> {
    let graph = load_graph_artifact(root)?;
    let report = build_doctor_report(&graph);
    let workspace = graph_workspace_dir(root);
    write_json(&workspace.join("doctor_report.json"), &report)?;
    Ok(report)
}

pub fn explain_path(root: &Path, goal: &str) -> Result<ExplainPathReport> {
    let graph = load_graph_artifact(root)?;
    let (domain_graph, node_lookup, unresolved) = to_domain_graph(&graph.nodes, &graph.edges)?;
    let assessment = compute_path_assessment(
        goal,
        &domain_graph,
        &node_lookup,
        &graph.edges,
        &graph.integrity_report,
        &unresolved,
    );
    Ok(ExplainPathReport {
        schema_id: "nostra.path_explain.v1".to_string(),
        goal: goal.to_string(),
        graph_root_hash: graph.graph_root_hash,
        recommended_path: assessment.recommended_path,
        candidate_count: assessment.candidate_paths.len(),
    })
}

pub fn simulate(root: &Path, scenario_path: &Path) -> Result<SimulationSession> {
    let graph = load_graph_artifact(root)?;
    let (domain_graph, _, _) = to_domain_graph(&graph.nodes, &graph.edges)?;
    let raw = fs::read_to_string(scenario_path)
        .with_context(|| format!("failed to read scenario {}", scenario_path.display()))?;
    let scenario = parse_scenario_yaml(&raw).map_err(|err| anyhow!(err))?;
    let rules = default_integrity_rules();
    let session = run_deterministic_session(&domain_graph, &rules, &scenario);

    let sim_dir = graph_workspace_dir(root).join("simulations");
    fs::create_dir_all(&sim_dir)?;
    let safe_id = sanitize_for_filename(&scenario.scenario.id);
    let path = sim_dir.join(format!("{}.json", safe_id));
    write_json(&path, &session)?;
    Ok(session)
}

pub fn publish_edition(root: &Path, version: &str) -> Result<PublishResult> {
    let graph_workspace = graph_workspace_dir(root);
    let graph_artifact_path = graph_workspace.join("contribution_graph.json");
    if !graph_artifact_path.exists() {
        let _ = ingest_and_write(root)?;
    }
    let graph = load_graph_artifact(root)?;

    let editions_dir = graph_workspace.join("editions");
    let this_edition_dir = editions_dir.join(version);
    fs::create_dir_all(&this_edition_dir)?;

    let snapshot_path = this_edition_dir.join("snapshot.json");
    write_json(&snapshot_path, &graph)?;

    let previous = detect_previous_edition(&editions_dir, version)?;
    let diff = if let Some(prev_version) = &previous {
        let prev_snapshot = editions_dir.join(prev_version).join("snapshot.json");
        if prev_snapshot.exists() {
            let previous_graph = read_json::<ContributionGraphV1>(&prev_snapshot)?;
            let (before, _, _) = to_domain_graph(&previous_graph.nodes, &previous_graph.edges)?;
            let (after, _, _) = to_domain_graph(&graph.nodes, &graph.edges)?;
            Some(serde_json::to_value(structural_graph(&before, &after))?)
        } else {
            None
        }
    } else {
        None
    };

    let edition_manifest = EditionManifestV1 {
        schema_id: "nostra.contribution_edition_manifest.v1".to_string(),
        edition_version: version.to_string(),
        generated_at: now_epoch_millis_string(),
        graph_root_hash: graph.graph_root_hash.clone(),
        snapshot_path: to_relative_graph_path(root, &snapshot_path)?,
        previous_edition: previous.clone(),
        structural_diff_from_previous: diff,
    };
    let edition_manifest_path = this_edition_dir.join("edition_manifest.json");
    write_json(&edition_manifest_path, &edition_manifest)?;

    let dpub_path = graph_workspace.join("dpub.json");
    let mut dpub = if dpub_path.exists() {
        read_json::<DpubManifestV1>(&dpub_path)?
    } else {
        DpubManifestV1 {
            schema_id: "nostra.dpub.bootstrap.v1".to_string(),
            title: "Nostra Contribution Graph Corpus".to_string(),
            corpus_root: "research".to_string(),
            current_edition: version.to_string(),
            graph_artifact: "research/000-contribution-graph/contribution_graph.json".to_string(),
        }
    };
    dpub.current_edition = version.to_string();
    write_json(&dpub_path, &dpub)?;

    Ok(PublishResult {
        version: version.to_string(),
        edition_manifest_path: to_relative_graph_path(root, &edition_manifest_path)?,
        snapshot_path: to_relative_graph_path(root, &snapshot_path)?,
    })
}

pub fn diff_editions(root: &Path, from: &str, to: &str) -> Result<EditionDiffReport> {
    let editions_dir = graph_workspace_dir(root).join("editions");
    let from_snapshot = editions_dir.join(from).join("snapshot.json");
    let to_snapshot = editions_dir.join(to).join("snapshot.json");
    if !from_snapshot.exists() {
        bail!("missing edition snapshot {}", from_snapshot.display());
    }
    if !to_snapshot.exists() {
        bail!("missing edition snapshot {}", to_snapshot.display());
    }

    let from_graph = read_json::<ContributionGraphV1>(&from_snapshot)?;
    let to_graph = read_json::<ContributionGraphV1>(&to_snapshot)?;
    let (before, _, _) = to_domain_graph(&from_graph.nodes, &from_graph.edges)?;
    let (after, _, _) = to_domain_graph(&to_graph.nodes, &to_graph.edges)?;
    Ok(EditionDiffReport {
        schema_id: "nostra.edition_diff.v1".to_string(),
        from_edition: from.to_string(),
        to_edition: to.to_string(),
        from_hash: from_graph.graph_root_hash,
        to_hash: to_graph.graph_root_hash,
        structural_diff: serde_json::to_value(structural_graph(&before, &after))?,
    })
}

fn build_graph(root: &Path) -> Result<ContributionGraphV1> {
    let research_root = root.join("research");
    let status_index_path = research_root.join("RESEARCH_INITIATIVES_STATUS.md");
    let entries = parse_status_index(&status_index_path)?;
    if entries.is_empty() {
        bail!("status index has no initiative entries");
    }

    let mut nodes = Vec::new();
    let mut raw_edges: Vec<(String, String, String, String, Vec<usize>, f32, bool)> = Vec::new();
    let mut unresolved_refs = BTreeSet::new();
    let valid_ids: BTreeSet<String> = entries.iter().map(|(id, _, _)| id.clone()).collect();

    for (id, directory, status_from_index) in &entries {
        let dir_path = research_root.join(directory);
        if !dir_path.exists() {
            unresolved_refs.insert(format!("{} -> missing_directory:{}", id, directory));
            let source_paths: Vec<String> = Vec::new();
            let source_hash = compute_source_hash(root, &source_paths)?;
            let layer = default_layer_for_directory(directory).to_string();
            let stewardship = Stewardship {
                layer: layer.clone(),
                primary_steward: "unassigned".to_string(),
                domain: "unspecified".to_string(),
            };
            nodes.push(ContributionNodeV1 {
                schema_id: SCHEMA_CONTRIBUTION_NODE_V1.to_string(),
                id: id.clone(),
                resource_ref: Some(ResourceRef::contribution(id)?.to_string()),
                title: directory.clone(),
                kind: ContributionKind::Initiative,
                status: status_from_index.clone(),
                layer,
                portfolio_role: default_portfolio_role(status_from_index).to_string(),
                structural_pivot_impact: "none".to_string(),
                tags: Vec::new(),
                stewardship,
                source_paths,
                source_hash,
            });
            continue;
        }
        let plan_path = dir_path.join("PLAN.md");
        let (mut frontmatter, body, metadata_source_ref) = if plan_path.exists() {
            let plan_raw = fs::read_to_string(&plan_path)
                .with_context(|| format!("failed reading {}", plan_path.display()))?;
            let (frontmatter_raw, body) = split_frontmatter(&plan_raw)
                .ok_or_else(|| anyhow!("missing frontmatter in {}", plan_path.display()))?;
            (
                parse_frontmatter(frontmatter_raw.as_str()),
                body,
                to_relative_graph_path(root, &plan_path)?,
            )
        } else {
            // Legacy initiatives may not include PLAN.md yet; synthesize metadata
            // from index status so the corpus remains machine-addressable.
            unresolved_refs.insert(format!("{} -> missing_plan:{}", id, directory));
            (
                PlanFrontmatter {
                    status: status_from_index.clone(),
                    portfolio_role: default_portfolio_role(status_from_index).to_string(),
                    title: directory.clone(),
                    tags: Vec::new(),
                    depends_on: Vec::new(),
                    blocks: Vec::new(),
                    invalidated_by: Vec::new(),
                    supersedes: Vec::new(),
                    structural_pivot_impact: "none".to_string(),
                    stewardship_layer: default_layer_for_directory(directory).to_string(),
                    stewardship_primary_steward: "unassigned".to_string(),
                    stewardship_domain: "unspecified".to_string(),
                },
                String::new(),
                to_relative_graph_path(root, &plan_path)?,
            )
        };

        if frontmatter.status.is_empty() {
            frontmatter.status = status_from_index.clone();
        }
        if frontmatter.portfolio_role.is_empty() {
            frontmatter.portfolio_role = default_portfolio_role(status_from_index).to_string();
        }
        if frontmatter.stewardship_layer.is_empty() {
            frontmatter.stewardship_layer = default_layer_for_directory(directory).to_string();
        }
        if frontmatter.stewardship_primary_steward.is_empty() {
            frontmatter.stewardship_primary_steward = "unassigned".to_string();
        }
        if frontmatter.stewardship_domain.is_empty() {
            frontmatter.stewardship_domain = "unspecified".to_string();
        }

        if !ALLOWED_STATUS.contains(&frontmatter.status.as_str()) {
            bail!(
                "invalid status `{}` in {}",
                frontmatter.status,
                plan_path.display()
            );
        }
        if frontmatter.status != *status_from_index {
            bail!(
                "status mismatch for {}: plan={} index={}",
                directory,
                frontmatter.status,
                status_from_index
            );
        }
        if !ALLOWED_PORTFOLIO_ROLE.contains(&frontmatter.portfolio_role.as_str()) {
            unresolved_refs.insert(format!(
                "{} -> invalid_portfolio_role:{}",
                id, frontmatter.portfolio_role
            ));
            frontmatter.portfolio_role = default_portfolio_role(status_from_index).to_string();
        }

        let title = if !frontmatter.title.is_empty() {
            frontmatter.title.clone()
        } else {
            first_markdown_heading(body.as_str()).unwrap_or_else(|| directory.to_string())
        };
        let structural_pivot_impact = if !frontmatter.structural_pivot_impact.is_empty() {
            frontmatter.structural_pivot_impact.clone()
        } else if id == "118" {
            "major".to_string()
        } else if id == "119" {
            "minor".to_string()
        } else if frontmatter.status == "superseded" || frontmatter.status == "archived" {
            "superseded".to_string()
        } else {
            "none".to_string()
        };

        let source_paths = collect_source_paths(root, &dir_path)?;
        let source_hash = compute_source_hash(root, &source_paths)?;
        let stewardship = Stewardship {
            layer: frontmatter.stewardship_layer.clone(),
            primary_steward: frontmatter.stewardship_primary_steward.clone(),
            domain: frontmatter.stewardship_domain.clone(),
        };

        nodes.push(ContributionNodeV1 {
            schema_id: SCHEMA_CONTRIBUTION_NODE_V1.to_string(),
            id: id.clone(),
            resource_ref: Some(ResourceRef::contribution(id)?.to_string()),
            title,
            kind: ContributionKind::Initiative,
            status: frontmatter.status.clone(),
            layer: stewardship.layer.clone(),
            portfolio_role: frontmatter.portfolio_role.clone(),
            structural_pivot_impact,
            tags: frontmatter.tags.clone(),
            stewardship,
            source_paths: source_paths.clone(),
            source_hash,
        });

        for dep in frontmatter.depends_on {
            match normalize_reference_to_id(dep.as_str()) {
                Some(target) if valid_ids.contains(&target) => raw_edges.push((
                    id.clone(),
                    target,
                    "depends_on".to_string(),
                    metadata_source_ref.clone(),
                    vec![1],
                    0.95,
                    true,
                )),
                Some(target) => {
                    unresolved_refs.insert(format!("{} -> depends_on:{}", id, target));
                }
                None => {
                    unresolved_refs.insert(format!("{} -> depends_on:{}", id, dep));
                }
            }
        }

        for blk in frontmatter.blocks {
            match normalize_reference_to_id(blk.as_str()) {
                Some(target) if valid_ids.contains(&target) => raw_edges.push((
                    id.clone(),
                    target,
                    "invalidates".to_string(),
                    metadata_source_ref.clone(),
                    vec![1],
                    0.90,
                    true,
                )),
                Some(target) => {
                    unresolved_refs.insert(format!("{} -> blocks:{}", id, target));
                }
                None => {
                    unresolved_refs.insert(format!("{} -> blocks:{}", id, blk));
                }
            }
        }

        for inv in frontmatter.invalidated_by {
            match normalize_reference_to_id(inv.as_str()) {
                Some(source) if valid_ids.contains(&source) => raw_edges.push((
                    source,
                    id.clone(),
                    "invalidates".to_string(),
                    metadata_source_ref.clone(),
                    vec![1],
                    0.90,
                    true,
                )),
                Some(source) => {
                    unresolved_refs.insert(format!("{} -> invalidated_by:{}", id, source));
                }
                None => {
                    unresolved_refs.insert(format!("{} -> invalidated_by:{}", id, inv));
                }
            }
        }

        for sup in frontmatter.supersedes {
            match normalize_reference_to_id(sup.as_str()) {
                Some(target) if valid_ids.contains(&target) => raw_edges.push((
                    id.clone(),
                    target,
                    "supersedes".to_string(),
                    metadata_source_ref.clone(),
                    vec![1],
                    0.90,
                    true,
                )),
                Some(target) => {
                    unresolved_refs.insert(format!("{} -> supersedes:{}", id, target));
                }
                None => {
                    unresolved_refs.insert(format!("{} -> supersedes:{}", id, sup));
                }
            }
        }

        for source in &source_paths {
            let source_abs = root.join(source);
            let raw = fs::read_to_string(&source_abs).unwrap_or_default();
            for reference in parse_research_references(raw.as_str()) {
                let referenced_id = reference.target_id;
                if &referenced_id == id {
                    continue;
                }
                if valid_ids.contains(&referenced_id) {
                    let (edge_kind, confidence) =
                        infer_edge_kind(reference.context_line.as_str(), "references", 0.60);
                    raw_edges.push((
                        id.clone(),
                        referenced_id,
                        edge_kind,
                        source.clone(),
                        vec![reference.line],
                        confidence,
                        false,
                    ));
                }
            }
        }
    }

    nodes.sort_by(|a, b| a.id.cmp(&b.id));
    let strict_mode = true;

    let mut edge_map: BTreeMap<(String, String, String), ContributionEdgeV1> = BTreeMap::new();
    for (from, to, kind, evidence_ref, evidence_lines, confidence, is_explicit) in raw_edges {
        let key = (from.clone(), to.clone(), kind.clone());
        edge_map.entry(key).or_insert(ContributionEdgeV1 {
            schema_id: SCHEMA_CONTRIBUTION_EDGE_V1.to_string(),
            from,
            to,
            edge_kind: kind,
            evidence_ref,
            evidence_lines,
            confidence,
            is_explicit,
            extracted_by: "agent://nostra/extraction/contribution-graph".to_string(),
            extracted_at: now_epoch_millis_string(),
        });
    }
    let edges: Vec<ContributionEdgeV1> = edge_map.into_values().collect();

    if strict_mode {
        let unknown = edges
            .iter()
            .filter_map(|edge| {
                PredicateRef::governed(edge.edge_kind.as_str())
                    .err()
                    .map(|_| edge.edge_kind.clone())
            })
            .collect::<BTreeSet<_>>();
        if !unknown.is_empty() {
            bail!(
                "unknown governed edge_kind(s): {}",
                unknown.into_iter().collect::<Vec<_>>().join(", ")
            );
        }
    }

    let (domain_graph, node_lookup, unresolved_from_graph) = to_domain_graph(&nodes, &edges)?;
    unresolved_refs.extend(unresolved_from_graph);

    let base_violations = evaluate_all(&default_integrity_rules(), &domain_graph);
    let mut violations = base_violations;
    violations.extend(custom_integrity_violations(&domain_graph, &node_lookup));
    violations.sort_by(|a, b| {
        a.rule_id
            .cmp(&b.rule_id)
            .then(a.explanation.cmp(&b.explanation))
    });
    let cycles = detect_cycles(&domain_graph, EdgeKind::DependsOn);
    let counts = summarize_violation_counts(&violations);
    let integrity_report = IntegrityReport {
        counts,
        cycles,
        violations,
    };

    let unresolved_refs_vec = unresolved_refs.into_iter().collect::<Vec<_>>();
    let build_report = BuildReport {
        strict_mode,
        generated_at: now_epoch_millis_string(),
        contribution_count: nodes.len(),
        edge_count: edges.len(),
        unresolved_references: unresolved_refs_vec.clone(),
        source_index_path: "research/RESEARCH_INITIATIVES_STATUS.md".to_string(),
    };

    let path_assessment = compute_path_assessment(
        "stable-cortex-domain",
        &domain_graph,
        &node_lookup,
        &edges,
        &integrity_report,
        &unresolved_refs_vec,
    );

    let extraction_report = compute_extraction_report(&edges, &unresolved_refs_vec);

    Ok(ContributionGraphV1 {
        schema_id: SCHEMA_CONTRIBUTION_GRAPH_V1.to_string(),
        generated_at: now_epoch_millis_string(),
        graph_root_hash: domain_graph.root_hash_hex(),
        nodes,
        edges,
        integrity_report,
        build_report,
        extraction_report,
        path_assessment,
    })
}

fn write_graph_bundle(root: &Path, graph: &ContributionGraphV1) -> Result<()> {
    let workspace = graph_workspace_dir(root);
    fs::create_dir_all(&workspace)?;
    fs::create_dir_all(workspace.join("editions"))?;
    fs::create_dir_all(workspace.join("schemas"))?;
    fs::create_dir_all(workspace.join("scenarios"))?;

    write_json(&workspace.join("contribution_graph.json"), graph)?;
    let path_bundle = PathAssessmentBundleV1 {
        schema_id: SCHEMA_PATH_ASSESSMENT_BUNDLE_V1.to_string(),
        generated_at: now_epoch_millis_string(),
        graph_root_hash: graph.graph_root_hash.clone(),
        assessments: build_default_goal_assessments(graph)?,
    };
    write_json(&workspace.join("path_assessment.json"), &path_bundle)?;
    write_json(
        &workspace.join("dpub.json"),
        &DpubManifestV1 {
            schema_id: "nostra.dpub.bootstrap.v1".to_string(),
            title: "Nostra Contribution Graph Corpus".to_string(),
            corpus_root: "research".to_string(),
            current_edition: "v0.2.0".to_string(),
            graph_artifact: "research/000-contribution-graph/contribution_graph.json".to_string(),
        },
    )?;
    write_json(
        &workspace
            .join("schemas")
            .join("nostra.contribution_node.v1.json"),
        &contribution_node_schema_doc(),
    )?;
    write_json(
        &workspace
            .join("schemas")
            .join("nostra.contribution_edge.v1.json"),
        &contribution_edge_schema_doc(),
    )?;
    write_json(
        &workspace
            .join("schemas")
            .join("nostra.contribution_graph.v1.json"),
        &contribution_graph_schema_doc(),
    )?;
    write_json(
        &workspace
            .join("schemas")
            .join("nostra.path_assessment.v1.json"),
        &path_assessment_schema_doc(),
    )?;
    write_json(
        &workspace
            .join("schemas")
            .join("nostra.path_assessment_bundle.v1.json"),
        &path_assessment_bundle_schema_doc(),
    )?;

    write_default_scenarios(&workspace)?;
    Ok(())
}

fn write_default_scenarios(workspace: &Path) -> Result<()> {
    let scenarios = vec![
        (
            "accelerate_118.yaml",
            r#"scenario:
  id: "accelerate_118"
  name: "Accelerate 118"
  seed: 118
  commons_version: "nostra-core-v1"
  siqs_version: "1.0.0"
constraints:
  max_mutations: 32
  max_rounds: 8
  max_runtime_ms: 128
rounds:
  - round: 1
    actions:
      - actor: "steward"
        action: "add_node"
        node_id: "initiative-118-focus"
        node_type: "initiative"
        attributes:
          status: "active"
  - round: 2
    actions:
      - actor: "steward"
        action: "add_edge"
        source: "initiative-118-focus"
        target: "118"
        edge_kind: "depends_on"
"#,
        ),
        (
            "delay_013_bridge.yaml",
            r#"scenario:
  id: "delay_013_bridge"
  name: "Delay Workflow Bridge 013"
  seed: 13
  commons_version: "nostra-core-v1"
  siqs_version: "1.0.0"
constraints:
  max_mutations: 32
  max_rounds: 8
  max_runtime_ms: 128
rounds:
  - round: 1
    actions:
      - actor: "operator"
        action: "modify_attribute"
        node_id: "013"
        key: "status"
        value: "deferred"
"#,
        ),
        (
            "derisk_governance_first.yaml",
            r#"scenario:
  id: "derisk_governance_first"
  name: "De-risk Governance Dependencies First"
  seed: 42
  commons_version: "nostra-core-v1"
  siqs_version: "1.0.0"
constraints:
  max_mutations: 32
  max_rounds: 8
  max_runtime_ms: 128
rounds:
  - round: 1
    actions:
      - actor: "steward"
        action: "add_edge"
        source: "119"
        target: "118"
        edge_kind: "constitutional_basis"
"#,
        ),
    ];

    let scenario_dir = workspace.join("scenarios");
    fs::create_dir_all(&scenario_dir)?;
    for (name, body) in scenarios {
        let path = scenario_dir.join(name);
        if !path.exists() {
            fs::write(path, body)?;
        }
    }
    Ok(())
}

fn load_graph_artifact(root: &Path) -> Result<ContributionGraphV1> {
    let path = graph_workspace_dir(root).join("contribution_graph.json");
    if !path.exists() {
        bail!(
            "graph artifact missing at {} (run ingest first)",
            path.display()
        );
    }
    read_json(&path)
}

fn graph_workspace_dir(root: &Path) -> PathBuf {
    root.join("research").join("000-contribution-graph")
}

fn parse_status_index(path: &Path) -> Result<Vec<(String, String, String)>> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed reading status index {}", path.display()))?;
    let mut rows = Vec::new();
    for line in raw.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with('|') {
            continue;
        }
        let parts = trimmed
            .trim_matches('|')
            .split('|')
            .map(|p| p.trim().to_string())
            .collect::<Vec<_>>();
        if parts.len() != 3 {
            continue;
        }
        if parts[0] == "ID" || parts[0].starts_with("---") {
            continue;
        }
        if !ALLOWED_STATUS.contains(&parts[2].as_str()) {
            bail!("status index contains non-canonical status `{}`", parts[2]);
        }
        rows.push((parts[0].clone(), parts[1].clone(), parts[2].clone()));
    }
    Ok(rows)
}

fn split_frontmatter(raw: &str) -> Option<(String, String)> {
    let mut lines = raw.lines();
    let first = lines.next()?;
    if first.trim() != "---" {
        return None;
    }
    let mut fm = Vec::new();
    let mut rest = Vec::new();
    let mut in_fm = true;
    for line in lines {
        if in_fm && line.trim() == "---" {
            in_fm = false;
            continue;
        }
        if in_fm {
            fm.push(line.to_string());
        } else {
            rest.push(line.to_string());
        }
    }
    if in_fm {
        return None;
    }
    Some((fm.join("\n"), rest.join("\n")))
}

fn parse_frontmatter(raw: &str) -> PlanFrontmatter {
    let mut output = PlanFrontmatter::default();
    let mut lines = raw.lines().peekable();

    while let Some(line) = lines.next() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(value) = trimmed.strip_prefix("status:") {
            output.status = scalar(value);
            continue;
        }
        if let Some(value) = trimmed.strip_prefix("portfolio_role:") {
            output.portfolio_role = scalar(value);
            continue;
        }
        if let Some(value) = trimmed.strip_prefix("title:") {
            output.title = scalar(value);
            continue;
        }
        if let Some(value) = trimmed.strip_prefix("structural_pivot_impact:") {
            output.structural_pivot_impact = scalar(value);
            continue;
        }
        if let Some(value) = trimmed.strip_prefix("tags:") {
            output.tags = parse_list(value, &mut lines);
            continue;
        }
        if let Some(value) = trimmed.strip_prefix("depends_on:") {
            output.depends_on = parse_list(value, &mut lines);
            continue;
        }
        if let Some(value) = trimmed.strip_prefix("blocks:") {
            output.blocks = parse_list(value, &mut lines);
            continue;
        }
        if let Some(value) = trimmed.strip_prefix("invalidated_by:") {
            output.invalidated_by = parse_list(value, &mut lines);
            continue;
        }
        if let Some(value) = trimmed.strip_prefix("supersedes:") {
            output.supersedes = parse_list(value, &mut lines);
            continue;
        }
        if trimmed == "stewardship:" {
            parse_stewardship(&mut output, &mut lines);
            continue;
        }
    }

    output
}

fn default_portfolio_role(status: &str) -> &'static str {
    match status {
        "placeholder" => "placeholder",
        "active" | "completed" => "anchor",
        _ => "reference",
    }
}

fn default_layer_for_directory(directory: &str) -> &'static str {
    if directory.contains("cortex") {
        "runtime"
    } else if directory.contains("nostra") {
        "protocol"
    } else {
        "infrastructure"
    }
}

fn parse_stewardship(output: &mut PlanFrontmatter, lines: &mut Peekable<Lines<'_>>) {
    while let Some(peek) = lines.peek() {
        if !peek.starts_with(' ') && !peek.starts_with('\t') {
            break;
        }
        let line = match lines.next() {
            Some(v) => v.trim(),
            None => break,
        };
        if let Some(value) = line.strip_prefix("layer:") {
            output.stewardship_layer = scalar(value);
        } else if let Some(value) = line.strip_prefix("primary_steward:") {
            output.stewardship_primary_steward = scalar(value);
        } else if let Some(value) = line.strip_prefix("domain:") {
            output.stewardship_domain = scalar(value);
        }
    }
}

fn parse_list(inline_value: &str, lines: &mut Peekable<Lines<'_>>) -> Vec<String> {
    let inline_value = inline_value.trim();
    if inline_value.starts_with('[') && inline_value.ends_with(']') {
        return inline_value
            .trim_start_matches('[')
            .trim_end_matches(']')
            .split(',')
            .map(scalar)
            .filter(|v| !v.is_empty())
            .collect();
    }
    if !inline_value.is_empty() {
        return vec![scalar(inline_value)];
    }

    let mut out = Vec::new();
    while let Some(peek) = lines.peek() {
        let trimmed = peek.trim();
        if !peek.starts_with(' ') && !peek.starts_with('\t') {
            break;
        }
        if let Some(value) = trimmed.strip_prefix("- ") {
            out.push(scalar(value));
            let _ = lines.next();
            continue;
        }
        break;
    }
    out
}

fn scalar(raw: &str) -> String {
    raw.trim().trim_matches('"').trim_matches('\'').to_string()
}

fn first_markdown_heading(body: &str) -> Option<String> {
    for line in body.lines() {
        let trimmed = line.trim();
        if let Some(title) = trimmed.strip_prefix("# ") {
            return Some(title.trim().to_string());
        }
    }
    None
}

fn collect_source_paths(root: &Path, initiative_dir: &Path) -> Result<Vec<String>> {
    let mut out = Vec::new();
    let known = ["PLAN.md", "RESEARCH.md", "DECISIONS.md"];
    for file in &known {
        let path = initiative_dir.join(file);
        if path.exists() {
            out.push(to_relative_graph_path(root, &path)?);
        }
    }

    let mut spec_docs = Vec::new();
    for entry in fs::read_dir(initiative_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(v) => v,
            None => continue,
        };
        if name.starts_with("SPEC") && name.ends_with(".md") {
            spec_docs.push(path);
        }
    }
    spec_docs.sort();
    for path in spec_docs {
        out.push(to_relative_graph_path(root, &path)?);
    }

    out.sort();
    out.dedup();
    Ok(out)
}

fn compute_source_hash(root: &Path, source_paths: &[String]) -> Result<String> {
    let mut hasher = Sha256::new();
    for source in source_paths {
        hasher.update(source.as_bytes());
        let raw = fs::read(root.join(source))
            .with_context(|| format!("failed reading source for hash {}", source))?;
        hasher.update(raw);
    }
    Ok(hex::encode(hasher.finalize()))
}

#[derive(Debug, Clone)]
struct ReferenceHit {
    target_id: String,
    line: usize,
    context_line: String,
}

fn parse_research_references(raw: &str) -> Vec<ReferenceHit> {
    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    for (idx, line) in raw.lines().enumerate() {
        for id in parse_reference_ids_from_text(line) {
            let key = format!("{}:{}", id, idx + 1);
            if seen.insert(key) {
                out.push(ReferenceHit {
                    target_id: id,
                    line: idx + 1,
                    context_line: line.trim().to_string(),
                });
            }
        }
    }
    out
}

fn parse_reference_ids_from_text(raw: &str) -> Vec<String> {
    let mut out = BTreeSet::new();
    let normalized = raw.to_ascii_lowercase();
    let bytes = normalized.as_bytes();
    if bytes.len() < 3 {
        return Vec::new();
    }

    let path_needle = b"research/";
    let mut idx = 0usize;
    while idx + path_needle.len() + 4 <= bytes.len() {
        if &bytes[idx..idx + path_needle.len()] == path_needle {
            let start = idx + path_needle.len();
            if bytes[start].is_ascii_digit()
                && bytes[start + 1].is_ascii_digit()
                && bytes[start + 2].is_ascii_digit()
                && bytes[start + 3] == b'-'
            {
                out.insert(String::from_utf8_lossy(&bytes[start..start + 3]).to_string());
            }
        }
        idx += 1;
    }

    let word_needle = b"research ";
    idx = 0;
    while idx + word_needle.len() + 3 <= bytes.len() {
        if &bytes[idx..idx + word_needle.len()] == word_needle {
            let start = idx + word_needle.len();
            if bytes[start].is_ascii_digit()
                && bytes[start + 1].is_ascii_digit()
                && bytes[start + 2].is_ascii_digit()
            {
                out.insert(String::from_utf8_lossy(&bytes[start..start + 3]).to_string());
            }
        }
        idx += 1;
    }
    out.into_iter().collect()
}

fn infer_edge_kind(context: &str, default_kind: &str, default_confidence: f32) -> (String, f32) {
    let lower = context.to_ascii_lowercase();
    if lower.contains("supersedes") || lower.contains("replaces") {
        return ("supersedes".to_string(), 0.65);
    }
    if lower.contains("invalidates") || lower.contains("deprecated by") {
        return ("invalidates".to_string(), 0.65);
    }
    (default_kind.to_string(), default_confidence)
}

fn normalize_reference_to_id(raw: &str) -> Option<String> {
    let raw = scalar(raw);
    if raw.len() == 3 && raw.chars().all(|c| c.is_ascii_digit()) {
        return Some(raw);
    }

    if let Some(pos) = raw.find("research/") {
        let start = pos + "research/".len();
        if raw.len() >= start + 4 {
            let candidate = &raw[start..start + 3];
            let trailing = raw.as_bytes().get(start + 3).copied().unwrap_or_default();
            if candidate.chars().all(|c| c.is_ascii_digit()) && trailing == b'-' {
                return Some(candidate.to_string());
            }
        }
    }

    let bytes = raw.as_bytes();
    if bytes.len() < 3 {
        return None;
    }
    for i in 0..=bytes.len().saturating_sub(3) {
        if bytes[i].is_ascii_digit()
            && bytes
                .get(i + 1)
                .map(|b| b.is_ascii_digit())
                .unwrap_or(false)
            && bytes
                .get(i + 2)
                .map(|b| b.is_ascii_digit())
                .unwrap_or(false)
        {
            let next = bytes.get(i + 3).copied().unwrap_or_default();
            let prev_ok = if i == 0 {
                true
            } else {
                !bytes[i - 1].is_ascii_digit()
            };
            let next_ok = next == b'-' || next == b'/' || next == b' ' || next == b')' || next == 0;
            if prev_ok && next_ok {
                return Some(String::from_utf8_lossy(&bytes[i..i + 3]).to_string());
            }
        }
    }
    None
}

fn to_domain_graph(
    nodes: &[ContributionNodeV1],
    edges: &[ContributionEdgeV1],
) -> Result<(Graph, BTreeMap<String, ContributionNodeV1>, Vec<String>)> {
    let mut graph = Graph::default();
    let mut lookup = BTreeMap::new();
    let mut unresolved = Vec::new();

    for node in nodes {
        lookup.insert(node.id.clone(), node.clone());
        let mut attributes = BTreeMap::new();
        attributes.insert("status".to_string(), node.status.clone());
        attributes.insert("layer".to_string(), node.layer.clone());
        attributes.insert("portfolio_role".to_string(), node.portfolio_role.clone());
        attributes.insert(
            "structural_pivot_impact".to_string(),
            node.structural_pivot_impact.clone(),
        );
        attributes.insert("tags".to_string(), node.tags.join(","));
        attributes.insert("space_id".to_string(), "research".to_string());
        attributes.insert(
            "stewardship_domain".to_string(),
            node.stewardship.domain.clone(),
        );
        attributes.insert(
            "stewardship_layer".to_string(),
            node.stewardship.layer.clone(),
        );
        graph.add_node(Node {
            id: node.id.clone(),
            node_type: "initiative".to_string(),
            attributes,
        });
    }

    for edge in edges {
        if !graph.nodes.contains_key(&edge.from) || !graph.nodes.contains_key(&edge.to) {
            unresolved.push(format!("edge_out_of_domain:{}->{}", edge.from, edge.to));
            continue;
        }
        graph.add_edge(Edge {
            from: edge.from.clone(),
            to: edge.to.clone(),
            kind: parse_edge_kind(edge.edge_kind.as_str()),
        });
    }

    Ok((graph, lookup, unresolved))
}

fn parse_edge_kind(raw: &str) -> EdgeKind {
    match raw {
        "depends_on" => EdgeKind::DependsOn,
        "contradicts" => EdgeKind::Contradicts,
        "supersedes" => EdgeKind::Supersedes,
        "implements" => EdgeKind::Implements,
        "invalidates" => EdgeKind::Invalidates,
        "requires" => EdgeKind::Requires,
        "assumes" => EdgeKind::Assumes,
        "constitutional_basis" => EdgeKind::ConstitutionalBasis,
        "derives_from" => EdgeKind::DerivesFrom,
        "forked_into" => EdgeKind::ForkedInto,
        "governs" => EdgeKind::Governs,
        "produces" => EdgeKind::Produces,
        "references" => EdgeKind::References,
        other => EdgeKind::Custom(other.to_string()),
    }
}

fn default_integrity_rules() -> Vec<IntegrityRule> {
    vec![
        IntegrityRule {
            id: "initiative.no_cycles".to_string(),
            name: "No dependency cycles".to_string(),
            description: "Initiatives must not form dependency cycles".to_string(),
            scope: IntegrityScope::Global,
            predicate: IntegrityPredicate {
                target: NodeSelector {
                    entity_type: Some("initiative".to_string()),
                    tags: None,
                },
                relation: Some(EdgeSelector {
                    edge_kind: EdgeKind::DependsOn,
                    direction: Direction::Outgoing,
                }),
                constraint: Constraint::NoCycles,
            },
            severity: Severity::Critical,
            remediation_hint: Some("Remove circular depends_on edges".to_string()),
        },
        IntegrityRule {
            id: "initiative.min_dependency".to_string(),
            name: "Initiatives should declare dependencies".to_string(),
            description: "Every initiative should declare at least one depends_on edge".to_string(),
            scope: IntegrityScope::EntityType("initiative".to_string()),
            predicate: IntegrityPredicate {
                target: NodeSelector {
                    entity_type: Some("initiative".to_string()),
                    tags: None,
                },
                relation: Some(EdgeSelector {
                    edge_kind: EdgeKind::DependsOn,
                    direction: Direction::Outgoing,
                }),
                constraint: Constraint::MinCount(1),
            },
            severity: Severity::Warning,
            remediation_hint: Some("Link initiative dependencies explicitly".to_string()),
        },
    ]
}

fn custom_integrity_violations(
    graph: &Graph,
    node_lookup: &BTreeMap<String, ContributionNodeV1>,
) -> Vec<IntegrityViolation> {
    let mut out = Vec::new();
    for (id, node) in node_lookup {
        if node.status == "active" {
            let outgoing = graph.edges.iter().any(|edge| {
                (edge.kind == EdgeKind::DependsOn || edge.kind == EdgeKind::References)
                    && edge.from == *id
            });
            let incoming = graph.edges.iter().any(|edge| {
                (edge.kind == EdgeKind::DependsOn || edge.kind == EdgeKind::References)
                    && edge.to == *id
            });
            if !outgoing && !incoming {
                out.push(IntegrityViolation {
                    rule_id: "initiative.no_orphan_active".to_string(),
                    affected_nodes: vec![id.clone()],
                    severity: Severity::Violation,
                    explanation:
                        "Active initiative has no dependency/reference incoming or outgoing edge"
                            .to_string(),
                });
            }
        }

        let governance_sensitive = node
            .stewardship
            .domain
            .to_ascii_lowercase()
            .contains("governance")
            || node
                .tags
                .iter()
                .any(|tag| tag.to_ascii_lowercase().contains("governance"));
        if governance_sensitive {
            let has_basis = graph
                .edges
                .iter()
                .any(|edge| edge.kind == EdgeKind::ConstitutionalBasis && edge.from == *id);
            if !has_basis {
                out.push(IntegrityViolation {
                    rule_id: "initiative.requires_constitutional_basis".to_string(),
                    affected_nodes: vec![id.clone()],
                    severity: Severity::Warning,
                    explanation: "Governance-sensitive initiative has no constitutional_basis edge"
                        .to_string(),
                });
            }
        }
    }
    out
}

fn summarize_violation_counts(violations: &[IntegrityViolation]) -> IntegrityCounts {
    let mut counts = IntegrityCounts {
        critical: 0,
        violation: 0,
        warning: 0,
        info: 0,
    };
    for violation in violations {
        match violation.severity {
            Severity::Critical => counts.critical += 1,
            Severity::Violation => counts.violation += 1,
            Severity::Warning => counts.warning += 1,
            Severity::Info => counts.info += 1,
        }
    }
    counts
}

fn compute_path_assessment(
    goal: &str,
    graph: &Graph,
    node_lookup: &BTreeMap<String, ContributionNodeV1>,
    edges: &[ContributionEdgeV1],
    integrity_report: &IntegrityReport,
    unresolved_refs: &[String],
) -> PathAssessmentV1 {
    let explicit_path_graph = project_dependency_graph(graph, edges, false);
    let fallback_path_graph = project_dependency_graph(graph, edges, true);
    let explicit_has_dependencies = explicit_path_graph
        .edges
        .iter()
        .any(|edge| edge.kind == EdgeKind::DependsOn);
    let preferred_global_graph = if explicit_has_dependencies {
        (&explicit_path_graph, false)
    } else {
        (&fallback_path_graph, true)
    };
    let edge_lookup = build_edge_lookup(edges);

    let mut candidates = Vec::new();

    let topological = topological_sort(preferred_global_graph.0, EdgeKind::DependsOn)
        .unwrap_or_else(|_| {
            preferred_global_graph
                .0
                .nodes
                .keys()
                .cloned()
                .collect::<Vec<_>>()
        });
    candidates.push(build_candidate(
        "topological_global",
        topological,
        node_lookup,
        &edge_lookup,
        integrity_report,
        unresolved_refs,
        "Global dependency-aware order",
        preferred_global_graph.1,
    ));

    let target_id = goal_to_target(goal, node_lookup);
    let closure_candidate = if let Some(target) = target_id {
        let target_prefers_fallback =
            requires_inferred_fallback(&explicit_path_graph, &fallback_path_graph, target.as_str());
        let target_graph = if target_prefers_fallback {
            &fallback_path_graph
        } else {
            &explicit_path_graph
        };
        let deps = dependency_walk(target_graph, target.as_str(), EdgeKind::DependsOn)
            .into_iter()
            .collect::<Vec<_>>();
        let mut sub_ids = deps;
        sub_ids.push(target);
        sub_ids.sort();
        sub_ids.dedup();
        let sub_graph = induced_subgraph(target_graph, &sub_ids);
        let order =
            topological_sort(&sub_graph, EdgeKind::DependsOn).unwrap_or_else(|_| sub_ids.clone());
        build_candidate(
            "goal_dependency_closure",
            order,
            node_lookup,
            &edge_lookup,
            integrity_report,
            unresolved_refs,
            "Dependency closure for target goal",
            target_prefers_fallback,
        )
    } else {
        build_candidate(
            "goal_dependency_closure",
            Vec::new(),
            node_lookup,
            &edge_lookup,
            integrity_report,
            unresolved_refs,
            "Goal target not found in graph",
            false,
        )
    };
    candidates.push(closure_candidate);

    let active_ids = node_lookup
        .values()
        .filter(|node| node.status == "active")
        .map(|node| node.id.clone())
        .collect::<Vec<_>>();
    let explicit_active_graph = induced_subgraph(&explicit_path_graph, &active_ids);
    let fallback_active_graph = induced_subgraph(&fallback_path_graph, &active_ids);
    let use_fallback_active = explicit_active_graph
        .edges
        .iter()
        .all(|edge| edge.kind != EdgeKind::DependsOn)
        && fallback_active_graph
            .edges
            .iter()
            .any(|edge| edge.kind == EdgeKind::DependsOn);
    let active_graph = if use_fallback_active {
        fallback_active_graph
    } else {
        explicit_active_graph
    };
    let active_order =
        topological_sort(&active_graph, EdgeKind::DependsOn).unwrap_or_else(|_| active_ids.clone());
    candidates.push(build_candidate(
        "active_only_projection",
        active_order,
        node_lookup,
        &edge_lookup,
        integrity_report,
        unresolved_refs,
        "Active initiative projection",
        use_fallback_active,
    ));

    let mut sorted = candidates;
    sorted.sort_by(|a, b| a.risk_score.cmp(&b.risk_score).then(a.name.cmp(&b.name)));
    let recommended = sorted.first().cloned().unwrap_or(PathCandidate {
        name: "none".to_string(),
        node_ids: Vec::new(),
        risk_score: usize::MAX,
        blocking_nodes: Vec::new(),
        rationale: "No candidate paths available".to_string(),
        score_breakdown: ScoreBreakdown::default(),
        rule_violations: Vec::new(),
        evidence: Vec::new(),
        used_inferred_fallback: false,
    });

    PathAssessmentV1 {
        schema_id: SCHEMA_PATH_ASSESSMENT_V1.to_string(),
        goal: goal.to_string(),
        candidate_paths: sorted,
        risk_score: recommended.risk_score,
        blocking_nodes: recommended.blocking_nodes.clone(),
        score_breakdown: recommended.score_breakdown.clone(),
        rule_violations: recommended.rule_violations.clone(),
        recommended_path: recommended,
    }
}

fn goal_to_target(goal: &str, nodes: &BTreeMap<String, ContributionNodeV1>) -> Option<String> {
    if goal == "stable-cortex-domain" || goal == "accelerate-118" {
        return Some("118".to_string());
    }
    if goal.len() == 3 && goal.chars().all(|c| c.is_ascii_digit()) && nodes.contains_key(goal) {
        return Some(goal.to_string());
    }
    None
}

fn induced_subgraph(graph: &Graph, ids: &[String]) -> Graph {
    let set = ids.iter().cloned().collect::<BTreeSet<_>>();
    let mut sub = Graph::default();
    for id in &set {
        if let Some(node) = graph.nodes.get(id) {
            sub.add_node(node.clone());
        }
    }
    for edge in &graph.edges {
        if set.contains(&edge.from) && set.contains(&edge.to) {
            sub.add_edge(edge.clone());
        }
    }
    sub
}

fn build_edge_lookup(
    edges: &[ContributionEdgeV1],
) -> BTreeMap<(String, String, String), ContributionEdgeV1> {
    let mut out = BTreeMap::new();
    for edge in edges {
        let key = (edge.from.clone(), edge.to.clone(), edge.edge_kind.clone());
        out.entry(key).or_insert_with(|| edge.clone());
    }
    out
}

fn project_dependency_graph(
    graph: &Graph,
    edges: &[ContributionEdgeV1],
    include_low_confidence_inferred: bool,
) -> Graph {
    let edge_lookup = build_edge_lookup(edges);
    let mut projected = Graph::default();
    for node in graph.nodes.values() {
        projected.add_node(node.clone());
    }
    for edge in &graph.edges {
        if edge.kind != EdgeKind::DependsOn {
            projected.add_edge(edge.clone());
            continue;
        }
        let key = (edge.from.clone(), edge.to.clone(), "depends_on".to_string());
        let include = match edge_lookup.get(&key) {
            Some(meta) if meta.is_explicit => true,
            Some(meta) if meta.confidence >= 0.70 => true,
            Some(_) => include_low_confidence_inferred,
            None => true,
        };
        if include {
            projected.add_edge(edge.clone());
        }
    }
    projected
}

fn requires_inferred_fallback(explicit: &Graph, fallback: &Graph, target: &str) -> bool {
    let explicit_connected = explicit
        .edges
        .iter()
        .any(|edge| edge.kind == EdgeKind::DependsOn && (edge.from == target || edge.to == target));
    let fallback_connected = fallback
        .edges
        .iter()
        .any(|edge| edge.kind == EdgeKind::DependsOn && (edge.from == target || edge.to == target));
    !explicit_connected && fallback_connected
}

fn compute_extraction_report(
    edges: &[ContributionEdgeV1],
    unresolved_refs: &[String],
) -> ExtractionReport {
    let explicit_edge_count = edges.iter().filter(|edge| edge.is_explicit).count();
    let inferred_edge_count = edges.len().saturating_sub(explicit_edge_count);
    let total = edges.len();
    let inferred_edge_ratio = if total == 0 {
        0.0
    } else {
        inferred_edge_count as f32 / total as f32
    };
    let mut confidence_distribution = BTreeMap::new();
    for edge in edges {
        let bucket = if edge.confidence >= 0.90 {
            "high"
        } else if edge.confidence >= 0.70 {
            "medium"
        } else {
            "low"
        };
        *confidence_distribution
            .entry(bucket.to_string())
            .or_insert(0usize) += 1;
    }
    ExtractionReport {
        explicit_edge_count,
        inferred_edge_count,
        inferred_edge_ratio,
        confidence_distribution,
        unresolved_references: unresolved_refs.to_vec(),
    }
}

fn build_default_goal_assessments(graph: &ContributionGraphV1) -> Result<Vec<PathAssessmentV1>> {
    let (domain_graph, node_lookup, unresolved_from_graph) =
        to_domain_graph(&graph.nodes, &graph.edges)?;
    let mut unresolved = graph.build_report.unresolved_references.clone();
    unresolved.extend(unresolved_from_graph);
    unresolved.sort();
    unresolved.dedup();
    let goals = vec!["stable-cortex-domain", "accelerate-118"];
    let mut out = Vec::new();
    for goal in goals {
        out.push(compute_path_assessment(
            goal,
            &domain_graph,
            &node_lookup,
            &graph.edges,
            &graph.integrity_report,
            &unresolved,
        ));
    }
    Ok(out)
}

fn build_doctor_report(graph: &ContributionGraphV1) -> DoctorReport {
    let mut grouped: BTreeMap<(String, String), BTreeSet<String>> = BTreeMap::new();
    for violation in &graph.integrity_report.violations {
        let severity = match violation.severity {
            Severity::Critical => "critical",
            Severity::Violation => "violation",
            Severity::Warning => "warning",
            Severity::Info => "info",
        };
        let key = (violation.rule_id.clone(), severity.to_string());
        let entry = grouped.entry(key).or_default();
        for id in &violation.affected_nodes {
            entry.insert(id.clone());
        }
    }
    if !graph.build_report.unresolved_references.is_empty() {
        grouped.insert(
            (
                "build.unresolved_references".to_string(),
                "violation".to_string(),
            ),
            graph
                .build_report
                .unresolved_references
                .iter()
                .cloned()
                .collect(),
        );
    }
    let mut backlog = grouped
        .into_iter()
        .map(|((rule_id, severity), contributions)| DoctorRuleBacklog {
            rule_id,
            severity,
            count: contributions.len(),
            affected_contributions: contributions.into_iter().collect(),
        })
        .collect::<Vec<_>>();
    backlog.sort_by(|a, b| a.severity.cmp(&b.severity).then(a.rule_id.cmp(&b.rule_id)));

    let mut thresholds = BTreeMap::new();
    thresholds.insert("critical_max".to_string(), 0);
    thresholds.insert("violation_max".to_string(), 15);
    thresholds.insert("unresolved_refs_max".to_string(), 0);

    let critical_max = *thresholds.get("critical_max").unwrap_or(&0);
    let violation_max = *thresholds.get("violation_max").unwrap_or(&15);
    let unresolved_max = *thresholds.get("unresolved_refs_max").unwrap_or(&0);
    let pass = graph.integrity_report.counts.critical <= critical_max
        && graph.integrity_report.counts.violation <= violation_max
        && graph.build_report.unresolved_references.len() <= unresolved_max;
    DoctorReport {
        schema_id: "nostra.contribution_doctor.v1".to_string(),
        generated_at: now_epoch_millis_string(),
        graph_root_hash: graph.graph_root_hash.clone(),
        integrity_counts: graph.integrity_report.counts.clone(),
        unresolved_references: graph.build_report.unresolved_references.len(),
        thresholds,
        pass,
        backlog,
    }
}

fn build_candidate(
    name: &str,
    node_ids: Vec<String>,
    node_lookup: &BTreeMap<String, ContributionNodeV1>,
    edge_lookup: &BTreeMap<(String, String, String), ContributionEdgeV1>,
    integrity_report: &IntegrityReport,
    unresolved_refs: &[String],
    rationale: &str,
    used_inferred_fallback: bool,
) -> PathCandidate {
    let superseded_edges = node_ids
        .iter()
        .filter(|id| {
            node_lookup
                .get(*id)
                .map(|node| node.status == "superseded" || node.status == "archived")
                .unwrap_or(false)
        })
        .count();

    let mut cross_layer_jumps = 0usize;
    for pair in node_ids.windows(2) {
        if let (Some(a), Some(b)) = (node_lookup.get(&pair[0]), node_lookup.get(&pair[1])) {
            if a.layer != b.layer {
                cross_layer_jumps += 1;
            }
        }
    }

    let unresolved_ref_penalty = unresolved_refs.len();
    let counts = &integrity_report.counts;
    let risk_score = (5 * counts.critical)
        + (3 * counts.violation)
        + counts.warning
        + (2 * superseded_edges)
        + (2 * cross_layer_jumps)
        + unresolved_ref_penalty;
    let score_breakdown = ScoreBreakdown {
        critical: counts.critical,
        violation: counts.violation,
        warning: counts.warning,
        superseded_edges,
        cross_layer_jumps,
        unresolved_ref_penalty,
        total: risk_score,
    };

    let path_set = node_ids.iter().cloned().collect::<BTreeSet<_>>();
    let mut blocking = BTreeSet::new();
    let mut rule_violations = BTreeSet::new();
    for violation in &integrity_report.violations {
        if matches!(violation.severity, Severity::Critical | Severity::Violation) {
            for affected in &violation.affected_nodes {
                if path_set.contains(affected) {
                    blocking.insert(affected.clone());
                }
            }
        }
        if violation
            .affected_nodes
            .iter()
            .any(|affected| path_set.contains(affected))
        {
            rule_violations.insert(violation.rule_id.clone());
        }
    }
    for id in &node_ids {
        if node_lookup
            .get(id)
            .map(|node| node.status == "superseded" || node.status == "archived")
            .unwrap_or(false)
        {
            blocking.insert(id.clone());
        }
    }

    let mut evidence = Vec::new();
    for edge in edge_lookup.values() {
        if edge.edge_kind != "depends_on" {
            continue;
        }
        if path_set.contains(&edge.from) && path_set.contains(&edge.to) {
            evidence.push(PathEdgeEvidence {
                from: edge.from.clone(),
                to: edge.to.clone(),
                evidence_ref: edge.evidence_ref.clone(),
                evidence_lines: edge.evidence_lines.clone(),
                confidence: edge.confidence,
                is_explicit: edge.is_explicit,
            });
        }
    }
    evidence.sort_by(|a, b| {
        a.from
            .cmp(&b.from)
            .then(a.to.cmp(&b.to))
            .then(a.evidence_ref.cmp(&b.evidence_ref))
    });

    PathCandidate {
        name: name.to_string(),
        node_ids,
        risk_score,
        blocking_nodes: blocking.into_iter().collect(),
        rationale: rationale.to_string(),
        score_breakdown,
        rule_violations: rule_violations.into_iter().collect(),
        evidence,
        used_inferred_fallback,
    }
}

fn detect_previous_edition(editions_dir: &Path, current_version: &str) -> Result<Option<String>> {
    if !editions_dir.exists() {
        return Ok(None);
    }
    let mut versions = fs::read_dir(editions_dir)?
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            if entry.path().is_dir() {
                entry.file_name().to_str().map(|v| v.to_string())
            } else {
                None
            }
        })
        .filter(|v| v != current_version)
        .collect::<Vec<_>>();
    versions.sort();
    Ok(versions.pop())
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let encoded = serde_json::to_string_pretty(value)?;
    fs::write(path, encoded).with_context(|| format!("failed writing {}", path.display()))
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed reading json {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("failed decoding json {}", path.display()))
}

fn now_epoch_millis_string() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

fn sanitize_for_filename(raw: &str) -> String {
    raw.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn to_relative_graph_path(root: &Path, path: &Path) -> Result<String> {
    let rel = path.strip_prefix(root).with_context(|| {
        format!(
            "path {} is not under root {}",
            path.display(),
            root.display()
        )
    })?;
    Ok(rel.to_string_lossy().to_string())
}

fn contribution_node_schema_doc() -> serde_json::Value {
    serde_json::json!({
      "$id": SCHEMA_CONTRIBUTION_NODE_V1,
      "type": "object",
      "required": ["schema_id", "id", "title", "kind", "status", "layer", "portfolio_role", "structural_pivot_impact", "tags", "stewardship", "source_paths", "source_hash"],
      "properties": {
        "schema_id": {"const": SCHEMA_CONTRIBUTION_NODE_V1},
        "id": {"type": "string"},
        "resource_ref": {"type": "string"},
        "title": {"type": "string"},
        "kind": {"type": "string", "enum": ["initiative", "pr", "bounty", "decision", "question", "task"]},
        "status": {"type": "string", "enum": ALLOWED_STATUS},
        "layer": {"type": "string"},
        "portfolio_role": {"type": "string", "enum": ALLOWED_PORTFOLIO_ROLE},
        "structural_pivot_impact": {"type": "string", "enum": ["none", "minor", "major", "superseded"]},
        "tags": {"type": "array", "items": {"type": "string"}},
        "stewardship": {"type": "object"},
        "source_paths": {"type": "array", "items": {"type": "string"}},
        "source_hash": {"type": "string"}
      }
    })
}

fn contribution_edge_schema_doc() -> serde_json::Value {
    serde_json::json!({
      "$id": SCHEMA_CONTRIBUTION_EDGE_V1,
      "type": "object",
      "required": ["schema_id", "from", "to", "edge_kind", "evidence_ref", "evidence_lines", "confidence", "is_explicit", "extracted_by", "extracted_at"],
      "properties": {
        "schema_id": {"const": SCHEMA_CONTRIBUTION_EDGE_V1},
        "from": {"type": "string"},
        "to": {"type": "string"},
        "edge_kind": {"type": "string"},
        "evidence_ref": {"type": "string"},
        "evidence_lines": {"type": "array", "items": {"type": "integer"}},
        "confidence": {"type": "number"},
        "is_explicit": {"type": "boolean"},
        "extracted_by": {"type": "string"},
        "extracted_at": {"type": "string"}
      }
    })
}

fn contribution_graph_schema_doc() -> serde_json::Value {
    serde_json::json!({
      "$id": SCHEMA_CONTRIBUTION_GRAPH_V1,
      "type": "object",
      "required": ["schema_id", "nodes", "edges", "graph_root_hash", "integrity_report", "build_report", "extraction_report", "path_assessment"],
      "properties": {
        "schema_id": {"const": SCHEMA_CONTRIBUTION_GRAPH_V1},
        "generated_at": {"type": "string"},
        "nodes": {"type": "array"},
        "edges": {"type": "array"},
        "graph_root_hash": {"type": "string"},
        "integrity_report": {"type": "object"},
        "build_report": {"type": "object"},
        "extraction_report": {"type": "object"},
        "path_assessment": {"type": "object"}
      }
    })
}

fn path_assessment_schema_doc() -> serde_json::Value {
    serde_json::json!({
      "$id": SCHEMA_PATH_ASSESSMENT_V1,
      "type": "object",
      "required": ["schema_id", "goal", "candidate_paths", "recommended_path", "risk_score", "blocking_nodes", "score_breakdown", "rule_violations"],
      "properties": {
        "schema_id": {"const": SCHEMA_PATH_ASSESSMENT_V1},
        "goal": {"type": "string"},
        "candidate_paths": {"type": "array"},
        "recommended_path": {"type": "object"},
        "risk_score": {"type": "integer"},
        "blocking_nodes": {"type": "array", "items": {"type": "string"}},
        "score_breakdown": {"type": "object"},
        "rule_violations": {"type": "array", "items": {"type": "string"}}
      }
    })
}

fn path_assessment_bundle_schema_doc() -> serde_json::Value {
    serde_json::json!({
      "$id": SCHEMA_PATH_ASSESSMENT_BUNDLE_V1,
      "type": "object",
      "required": ["schema_id", "generated_at", "graph_root_hash", "assessments"],
      "properties": {
        "schema_id": {"const": SCHEMA_PATH_ASSESSMENT_BUNDLE_V1},
        "generated_at": {"type": "string"},
        "graph_root_hash": {"type": "string"},
        "assessments": {"type": "array", "items": {"type": "object"}}
      }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_workspace_root() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let path = std::env::temp_dir().join(format!(
            "nostra-extraction-tests-{}-{}",
            std::process::id(),
            nonce
        ));
        fs::create_dir_all(&path).expect("create temp workspace root");
        path
    }

    #[test]
    fn frontmatter_parser_supports_inline_and_block_lists() {
        let raw = r#"
status: active
portfolio_role: anchor
title: "Test"
tags: [a, b, c]
depends_on:
  - "013"
  - research/118-cortex-runtime-extraction/PLAN.md
stewardship:
  layer: "Systems"
  primary_steward: "Systems Steward"
  domain: "Agents & Execution"
"#;
        let parsed = parse_frontmatter(raw);
        assert_eq!(parsed.status, "active");
        assert_eq!(parsed.portfolio_role, "anchor");
        assert_eq!(parsed.tags.len(), 3);
        assert_eq!(parsed.depends_on.len(), 2);
        assert_eq!(parsed.stewardship_layer, "Systems");
    }

    #[test]
    fn normalize_reference_extracts_three_digit_id() {
        assert_eq!(
            normalize_reference_to_id("research/118-cortex-runtime-extraction/PLAN.md"),
            Some("118".to_string())
        );
        assert_eq!(normalize_reference_to_id("013"), Some("013".to_string()));
        assert_eq!(
            normalize_reference_to_id("resolved-by-074"),
            Some("074".to_string())
        );
    }

    #[test]
    fn research_reference_parser_finds_ids() {
        let raw = "See research/013-nostra-workflow-engine/PLAN.md and research/118-cortex-runtime-extraction/PLAN.md";
        let refs = parse_research_references(raw);
        assert!(refs.iter().any(|r| r.target_id == "013"));
        assert!(refs.iter().any(|r| r.target_id == "118"));
        assert!(refs.iter().all(|r| r.line == 1));
    }

    #[test]
    fn infer_edge_kind_detects_dependency_context() {
        let (kind, confidence) = infer_edge_kind(
            "This work depends on research/118-cortex-runtime-extraction/PLAN.md",
            "references",
            0.6,
        );
        assert_eq!(kind, "references");
        assert_eq!(confidence, 0.6);
    }

    #[test]
    fn contribution_kind_supports_question_nodes() {
        let encoded =
            serde_json::to_string(&ContributionKind::Question).expect("serialize question kind");
        assert_eq!(encoded, "\"question\"");
        let schema = contribution_node_schema_doc();
        let schema_json = serde_json::to_string(&schema).expect("serialize schema doc");
        assert!(
            schema_json.contains("\"question\""),
            "node schema should advertise question as an allowed kind"
        );
    }

    #[test]
    fn build_graph_sets_resource_ref_for_nodes() {
        let root = temp_workspace_root();
        let research = root.join("research");
        let initiative_dir = research.join("001-test-initiative");
        fs::create_dir_all(&initiative_dir).expect("create initiative dir");

        fs::write(
            research.join("RESEARCH_INITIATIVES_STATUS.md"),
            r#"# Research Initiatives Index
| ID | Directory | Status |
|---|---|---|
| 001 | 001-test-initiative | active |
"#,
        )
        .expect("write status index");

        fs::write(
            initiative_dir.join("PLAN.md"),
            r#"---
status: active
portfolio_role: anchor
title: "Test Initiative"
stewardship:
  layer: "Systems"
  primary_steward: "Test"
  domain: "Test"
---
# Test
"#,
        )
        .expect("write plan");

        let graph = build_graph(&root).expect("build graph");
        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.nodes[0].id, "001");
        assert_eq!(
            graph.nodes[0].resource_ref.as_deref(),
            Some("nostra://contribution?id=001")
        );
    }
}
