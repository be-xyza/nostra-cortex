use serde::{Deserialize, Serialize};

// --- Types ---

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub enum TestResult {
    Pass,
    Fail(String), // Reason
    Warn(String),
    Pending,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct TestScenario {
    pub goal: String,
    pub constraints: Vec<String>,
    pub context_mock: String, // JSON string of mock profile/state
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct TestRun {
    pub id: String,
    pub timestamp: u64,
    pub scenario: TestScenario,
    pub ui_snapshot: String, // The generated A2UI JSON
    pub result: TestResult,
}

// --- Service ---

// In a real app, this might interact with a Backend Canister or LocalStorage.
// For this Lab, we'll use a static Recoil-like signal or simple in-memory simulation
// that persists via `web_sys::LocalStorage` if possible, or just session memory.

pub struct TestingIndexService {
    // We can't hold state easily in a struct without it being a Signal in Dioxus.
    // This helper just provides static methods to load/save from LocalStorage.
}

impl TestingIndexService {
    const STORAGE_KEY: &'static str = "nostra_a2ui_test_runs";

    pub fn load_runs() -> Vec<TestRun> {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                if let Ok(Some(json)) = storage.get_item(Self::STORAGE_KEY) {
                    if let Ok(runs) = serde_json::from_str::<Vec<TestRun>>(&json) {
                        return runs;
                    }
                }
            }
        }
        Vec::new()
    }

    pub fn save_run(run: TestRun) {
        let mut runs = Self::load_runs();
        // Remove existing if any (by id)
        runs.retain(|r| r.id != run.id);
        runs.push(run);

        // Sort by timestamp desc
        runs.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        Self::persist(runs);
    }

    fn persist(runs: Vec<TestRun>) {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                if let Ok(json) = serde_json::to_string(&runs) {
                    let _ = storage.set_item(Self::STORAGE_KEY, &json);
                }
            }
        }
    }

    // specific search/filter
    pub fn search_runs(query: &str) -> Vec<TestRun> {
        let runs = Self::load_runs();
        if query.is_empty() {
            return runs;
        }
        let q = query.to_lowercase();
        runs.into_iter()
            .filter(|r| {
                r.scenario.goal.to_lowercase().contains(&q)
                    || r.scenario
                        .constraints
                        .iter()
                        .any(|c| c.to_lowercase().contains(&q))
            })
            .collect()
    }

    pub fn delete_run(id: &str) {
        let mut runs = Self::load_runs();
        runs.retain(|r| r.id != id);
        Self::persist(runs);
    }
}
