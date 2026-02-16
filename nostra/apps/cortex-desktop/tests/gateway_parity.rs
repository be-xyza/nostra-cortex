use serde::Deserialize;
use serde_json::Value;
use std::collections::{BTreeSet, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

const FIXTURE_ROOT: &str = "tests/fixtures/gateway_baseline";
const INVENTORY_PATH: &str = "tests/fixtures/gateway_baseline/endpoint_inventory.tsv";
const INVENTORY_JSON_PATH: &str = "tests/fixtures/gateway_baseline/endpoint_inventory.json";
const EXEMPTIONS_PATH: &str = "tests/fixtures/gateway_baseline/approved_exemptions.json";
const POLICY_PATH: &str = "tests/fixtures/gateway_baseline/PARITY_POLICY.md";

#[derive(Debug, Deserialize)]
struct EndpointClassSet {
    classes: Vec<EndpointClass>,
}

#[derive(Debug, Deserialize)]
struct EndpointClass {
    name: String,
    path_prefix: String,
}

#[derive(Debug, Deserialize)]
struct EndpointInventoryJson {
    endpoints: Vec<InventoryEndpoint>,
}

#[derive(Debug, Deserialize, Clone)]
struct InventoryEndpoint {
    method: String,
    path: String,
}

#[derive(Debug, Deserialize)]
struct ApprovedExemptions {
    #[serde(default)]
    exemptions: Vec<Exemption>,
}

#[derive(Debug, Deserialize)]
struct Exemption {
    method: String,
    path: String,
    #[allow(dead_code)]
    reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ParityCase {
    case_id: String,
    class: String,
    request: ParityRequest,
    #[serde(default)]
    normalization: Normalization,
    legacy_response: Value,
    runtime_response: Value,
}

#[derive(Debug, Deserialize)]
struct ParityRequest {
    method: String,
    path: String,
}

#[derive(Debug, Deserialize)]
struct Normalization {
    #[serde(default = "default_mode")]
    mode: String,
    #[serde(default)]
    ignored_fields: Vec<String>,
    #[serde(default)]
    required_keys: Vec<String>,
}

impl Default for Normalization {
    fn default() -> Self {
        Self {
            mode: default_mode(),
            ignored_fields: Vec::new(),
            required_keys: Vec::new(),
        }
    }
}

fn default_mode() -> String {
    "strict".to_string()
}

#[test]
fn endpoint_inventory_json_mirrors_tsv() {
    let tsv = load_inventory_tsv(INVENTORY_PATH);
    let json = load_inventory_json(INVENTORY_JSON_PATH);
    assert_eq!(tsv, json, "endpoint_inventory.json must mirror endpoint_inventory.tsv");
}

#[test]
fn gateway_inventory_covers_required_endpoint_classes() {
    let classes: EndpointClassSet = read_json(&format!("{}/endpoint_classes.json", FIXTURE_ROOT));
    let inventory = load_inventory_tsv(INVENTORY_PATH);

    let paths: Vec<&str> = inventory.iter().map(|(_, path)| path.as_str()).collect();
    for class in classes.classes {
        let has_match = paths.iter().any(|path| path.starts_with(&class.path_prefix));
        assert!(
            has_match,
            "missing endpoint coverage for class '{}' ({})",
            class.name,
            class.path_prefix
        );
    }
}

#[test]
fn inventory_fixture_counts_are_locked() {
    let inventory = load_inventory_tsv(INVENTORY_PATH);
    let case_paths = list_case_paths(&format!("{}/parity_cases", FIXTURE_ROOT));
    let exemptions: ApprovedExemptions = read_json(EXEMPTIONS_PATH);

    let inventory_count = inventory.len();
    let fixture_count = case_paths.len();
    let exemptions_count = exemptions.exemptions.len();

    assert_eq!(
        inventory_count,
        fixture_count + exemptions_count,
        "inventory_count must equal fixture_count + approved_exemptions_count"
    );
    assert_eq!(
        exemptions_count, 0,
        "approved_exemptions_count must be 0 by default"
    );
}

#[test]
fn gateway_parity_replay_cases_match_between_legacy_and_runtime() {
    let classes: EndpointClassSet = read_json(&format!("{}/endpoint_classes.json", FIXTURE_ROOT));
    let class_names: BTreeSet<String> = classes.classes.into_iter().map(|c| c.name).collect();
    let inventory = load_inventory_tsv(INVENTORY_PATH);

    let exemptions: ApprovedExemptions = read_json(EXEMPTIONS_PATH);
    let exempted: HashSet<(String, String)> = exemptions
        .exemptions
        .into_iter()
        .map(|e| (e.method.to_uppercase(), e.path))
        .collect();

    let case_paths = list_case_paths(&format!("{}/parity_cases", FIXTURE_ROOT));
    assert!(!case_paths.is_empty(), "no parity cases found");

    let mut seen_case_ids = HashSet::new();
    let mut seen_endpoints = HashSet::new();

    for case_path in case_paths {
        let case: ParityCase = read_json(case_path.to_str().expect("invalid fixture path"));
        assert!(
            seen_case_ids.insert(case.case_id.clone()),
            "duplicate case_id found: {}",
            case.case_id
        );

        assert!(
            class_names.contains(&case.class),
            "case '{}' references unknown class '{}'",
            case.case_id,
            case.class
        );

        let endpoint = (case.request.method.to_uppercase(), case.request.path.clone());
        assert!(
            inventory.contains(&endpoint),
            "case '{}' route missing from inventory: {} {}",
            case.case_id,
            endpoint.0,
            endpoint.1
        );

        assert!(
            !exempted.contains(&endpoint),
            "case '{}' duplicated an exempted endpoint",
            case.case_id
        );

        assert!(
            seen_endpoints.insert(endpoint.clone()),
            "duplicate fixture endpoint: {} {}",
            endpoint.0,
            endpoint.1
        );

        assert_case_parity(&case);
    }

    let expected_non_exempt = inventory.len() - exempted.len();
    assert_eq!(
        seen_endpoints.len(),
        expected_non_exempt,
        "fixture endpoint count mismatch after exemptions"
    );
}

#[test]
fn gateway_parity_policy_enforces_progression_gate() {
    let policy = fs::read_to_string(POLICY_PATH).expect("failed to read parity policy");
    assert!(
        policy.contains("No extraction phase advances unless parity suite passes."),
        "parity policy missing mandatory progression gate statement"
    );
}

#[test]
fn replay_cases_file_paths_are_relative_to_fixture_root() {
    for path in list_case_paths(&format!("{}/parity_cases", FIXTURE_ROOT)) {
        assert!(
            Path::new(FIXTURE_ROOT).join("parity_cases").exists(),
            "fixture root missing"
        );
        assert!(path.starts_with(Path::new(FIXTURE_ROOT)));
    }
}

fn assert_case_parity(case: &ParityCase) {
    let legacy = normalize_value(&case.legacy_response, &case.normalization.ignored_fields);
    let runtime = normalize_value(&case.runtime_response, &case.normalization.ignored_fields);

    let mode = case.normalization.mode.as_str();
    match mode {
        "strict" => {
            assert_eq!(
                canonicalize(&legacy),
                canonicalize(&runtime),
                "strict parity mismatch in case '{}'",
                case.case_id
            );
            for key in &case.normalization.required_keys {
                assert_required_key(&legacy, key, &case.case_id, "legacy");
                assert_required_key(&runtime, key, &case.case_id, "runtime");
            }
        }
        "subset" => {
            assert!(
                !case.normalization.required_keys.is_empty(),
                "subset mode requires required_keys for case '{}'",
                case.case_id
            );
            for key in &case.normalization.required_keys {
                let left = lookup_path(&legacy, key).unwrap_or_else(|| {
                    panic!("missing required key '{}' in legacy response for '{}'", key, case.case_id)
                });
                let right = lookup_path(&runtime, key).unwrap_or_else(|| {
                    panic!("missing required key '{}' in runtime response for '{}'", key, case.case_id)
                });
                assert_eq!(
                    canonicalize(left),
                    canonicalize(right),
                    "subset parity mismatch for key '{}' in case '{}'",
                    key,
                    case.case_id
                );
            }
        }
        other => panic!("unsupported normalization mode '{}' in case '{}'", other, case.case_id),
    }
}

fn assert_required_key(value: &Value, key: &str, case_id: &str, label: &str) {
    assert!(
        lookup_path(value, key).is_some(),
        "missing required key '{}' in {} response for '{}'",
        key,
        label,
        case_id
    );
}

fn lookup_path<'a>(value: &'a Value, key_path: &str) -> Option<&'a Value> {
    let mut current = value;
    for segment in key_path.split('.') {
        current = current.get(segment)?;
    }
    Some(current)
}

fn normalize_value(value: &Value, ignored_fields: &[String]) -> Value {
    let ignored: HashSet<&str> = ignored_fields.iter().map(|s| s.as_str()).collect();
    strip_ignored(value, &ignored)
}

fn strip_ignored(value: &Value, ignored_fields: &HashSet<&str>) -> Value {
    match value {
        Value::Array(items) => Value::Array(
            items
                .iter()
                .map(|item| strip_ignored(item, ignored_fields))
                .collect(),
        ),
        Value::Object(map) => {
            let filtered = map
                .iter()
                .filter(|(k, _)| !ignored_fields.contains(k.as_str()))
                .map(|(k, v)| (k.clone(), strip_ignored(v, ignored_fields)))
                .collect();
            Value::Object(filtered)
        }
        _ => value.clone(),
    }
}

fn load_inventory_tsv(path: &str) -> HashSet<(String, String)> {
    fs::read_to_string(path)
        .expect("failed to read endpoint inventory tsv")
        .lines()
        .filter_map(|line| {
            let mut parts = line.split('\t');
            let method = parts.next()?.trim().to_uppercase();
            let route = parts.next()?.trim().to_string();
            Some((method, route))
        })
        .collect()
}

fn load_inventory_json(path: &str) -> HashSet<(String, String)> {
    let parsed: EndpointInventoryJson = read_json(path);
    parsed
        .endpoints
        .into_iter()
        .map(|entry| (entry.method.to_uppercase(), entry.path))
        .collect()
}

fn list_case_paths(root: &str) -> Vec<PathBuf> {
    let mut entries: Vec<_> = fs::read_dir(root)
        .expect("failed to read parity case directory")
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|ext| ext.to_str()) == Some("json"))
        .collect();
    entries.sort();
    entries
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &str) -> T {
    let raw = fs::read_to_string(path).unwrap_or_else(|_| panic!("failed to read fixture: {}", path));
    serde_json::from_str(&raw).unwrap_or_else(|_| panic!("invalid JSON fixture: {}", path))
}

fn canonicalize(value: &Value) -> Value {
    match value {
        Value::Array(items) => Value::Array(items.iter().map(canonicalize).collect()),
        Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            let mut normalized = serde_json::Map::new();
            for key in keys {
                let item = map.get(key).expect("key should exist");
                normalized.insert(key.clone(), canonicalize(item));
            }
            Value::Object(normalized)
        }
        _ => value.clone(),
    }
}
