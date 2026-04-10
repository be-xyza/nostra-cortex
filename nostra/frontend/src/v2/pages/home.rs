use crate::v2::api::{fetch_siq_snapshot, sample_siq_snapshot};
use crate::v2::types::BuildStatus;
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct HomePageProps {
    pub health: BuildStatus,
    pub on_open_institutions: EventHandler<()>,
    pub on_open_questions: EventHandler<()>,
}

#[component]
pub fn HomePage(props: HomePageProps) -> Element {
    let snapshot = use_signal(sample_siq_snapshot);
    let mut loading = use_signal(|| false);
    let fetch_error = use_signal(|| None::<String>);

    let on_refresh_siq = move |_| {
        if loading() {
            return;
        }
        loading.set(true);
        let mut snapshot_signal = snapshot;
        let mut loading_signal = loading;
        let mut error_signal = fetch_error;
        spawn(async move {
            match fetch_siq_snapshot().await {
                Ok(next) => {
                    snapshot_signal.set(next);
                    error_signal.set(None);
                }
                Err(err) => error_signal.set(Some(err)),
            }
            loading_signal.set(false);
        });
    };

    let current = snapshot();
    let coverage_count = current.coverage.contributions.len();
    let closure_rows = current.dependency_closure.rows.len();
    let unresolved_rows = current
        .dependency_closure
        .rows
        .iter()
        .filter(|row| !row.missing_dependencies.is_empty())
        .count();
    let run_count = current.runs.len();
    let latest_rule_pass = current
        .latest_run
        .as_ref()
        .map(|run| {
            run.results
                .iter()
                .filter(|item| item.status == "pass")
                .count()
        })
        .unwrap_or(0);

    rsx! {
        section {
            class: "max-w-3xl space-y-4",
            div { class: "rounded-lg border border-zinc-800 bg-zinc-900/70 p-5 space-y-2",
                h2 { class: "text-lg font-medium", "System Snapshot" }
                p { class: "text-sm text-zinc-300",
                    "Mode: {props.health.mode} | Source: {props.health.source} | Verified: {props.health.last_verified}"
                }
            }
            div {
                class: "rounded-lg border border-zinc-800 bg-zinc-900/70 p-5 space-y-3",
                h3 { class: "text-base font-medium", "SIQ Gateway Intake" }
                p { class: "text-sm text-zinc-300",
                    "Verdict: {current.gates.overall_verdict} | Mode: {current.gates.mode} | Run: {current.gates.latest_run_id}"
                }
                p { class: "text-sm text-zinc-400",
                    "SIQ Status: {current.health.status} | Runs: {current.health.runs_count} | Pass/Fail: {current.gates.counts.pass}/{current.gates.counts.fail}"
                }
                p { class: "text-sm text-zinc-400",
                    "Coverage: {coverage_count} contributions | Closure rows: {closure_rows} | Unresolved: {unresolved_rows}"
                }
                p { class: "text-sm text-zinc-400",
                    "Runs: {run_count} | Latest run pass-rules: {latest_rule_pass}"
                }
                if let Some(err) = fetch_error() {
                    p { class: "text-sm text-rose-300", "Refresh failed: {err}" }
                }
                button {
                    class: "px-4 py-2 rounded-md border border-zinc-700 text-zinc-100 text-sm font-medium",
                    onclick: on_refresh_siq,
                    if loading() { "Refreshing SIQ..." } else { "Refresh SIQ from Gateway" }
                }
                div {
                    class: "mt-2 rounded border border-zinc-800 overflow-hidden",
                    table { class: "w-full text-sm",
                        thead { class: "bg-zinc-900",
                            tr {
                                th { class: "text-left px-3 py-2 text-zinc-400", "Contribution" }
                                th { class: "text-left px-3 py-2 text-zinc-400", "State" }
                                th { class: "text-left px-3 py-2 text-zinc-400", "Missing Deps" }
                            }
                        }
                        tbody {
                            for row in current.dependency_closure.rows.iter().take(6) {
                                tr {
                                    key: "{row.contribution_id}",
                                    class: "border-t border-zinc-800",
                                    td { class: "px-3 py-2 text-zinc-200", "{row.contribution_id}" }
                                    td { class: "px-3 py-2 text-zinc-300", "{row.closure_state}" }
                                    td { class: "px-3 py-2 text-zinc-400", "{row.missing_dependencies.len()}" }
                                }
                            }
                        }
                    }
                }
                if let Some(run) = current.latest_run.as_ref() {
                    div { class: "mt-3 rounded border border-zinc-800 bg-zinc-900/60 p-3",
                        h4 { class: "text-sm font-medium", "Latest Run Detail" }
                        p { class: "text-xs text-zinc-400", "Run: {run.run_id} | Verdict: {run.overall_verdict} | Commit: {run.git_commit}" }
                        p { class: "text-xs text-zinc-400", "Policy: {run.policy_path} (v{run.policy_version})" }
                    }
                }
                div {
                    class: "mt-2 rounded border border-zinc-800 overflow-hidden",
                    table { class: "w-full text-sm",
                        thead { class: "bg-zinc-900",
                            tr {
                                th { class: "text-left px-3 py-2 text-zinc-400", "Run ID" }
                                th { class: "text-left px-3 py-2 text-zinc-400", "Verdict" }
                                th { class: "text-left px-3 py-2 text-zinc-400", "Pass/Fail" }
                            }
                        }
                        tbody {
                            for run in current.runs.iter().take(6) {
                                tr {
                                    key: "{run.run_id}",
                                    class: "border-t border-zinc-800",
                                    td { class: "px-3 py-2 text-zinc-200", "{run.run_id}" }
                                    td { class: "px-3 py-2 text-zinc-300", "{run.overall_verdict}" }
                                    td { class: "px-3 py-2 text-zinc-400", "{run.counts.pass}/{run.counts.fail}" }
                                }
                            }
                        }
                    }
                }
            }
            div {
                class: "rounded-lg border border-zinc-800 bg-zinc-900/70 p-5 space-y-3",
                h3 { class: "text-base font-medium", "Restoration Workflow" }
                p { class: "text-sm text-zinc-400", "Frontend is being restored in bounded layers: types -> api -> pages." }
                div { class: "flex gap-3",
                    button {
                        class: "px-4 py-2 rounded-md bg-zinc-100 text-zinc-900 text-sm font-medium",
                        onclick: move |_| props.on_open_institutions.call(()),
                        "Open Institutions"
                    }
                    button {
                        class: "px-4 py-2 rounded-md border border-zinc-700 text-zinc-100 text-sm font-medium",
                        onclick: move |_| props.on_open_questions.call(()),
                        "Open Questions"
                    }
                }
            }
        }
    }
}
