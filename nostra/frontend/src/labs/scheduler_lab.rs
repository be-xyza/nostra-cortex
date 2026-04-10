use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Props, Clone, PartialEq)]
pub struct SchedulerLabProps {
    pub on_back: EventHandler<()>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Constraint {
    row: usize,
    col: usize,
    val: usize,
}

#[component]
pub fn SchedulerLab(props: SchedulerLabProps) -> Element {
    // State for constraints input (JSON)
    let default_constraints = r#"[
    { "row": 0, "col": 0, "val": 1 },
    { "row": 1, "col": 1, "val": 2 }
]"#;
    let mut json_input = use_signal(|| default_constraints.to_string());

    // State for the solved grid (9x9 flattened or 2D)
    // 0 = empty
    let mut grid = use_signal(|| vec![0usize; 81]);

    // Status
    let mut is_loading = use_signal(|| false);
    let mut error_msg = use_signal(|| None::<String>);
    let mut verification_status = use_signal(|| None::<String>); // None, "verified", "invalid"

    // Mock Solver Function (In real app, this calls Backend -> Worker)
    let solve_puzzle = move |_| async move {
        is_loading.set(true);
        error_msg.set(None);
        verification_status.set(None);

        // checking parse
        if let Err(e) = serde_json::from_str::<Vec<Constraint>>(&json_input.read()) {
            error_msg.set(Some(format!("Invalid JSON: {}", e)));
            is_loading.set(false);
            return;
        }

        // Simulating Network Delay / Worker Call
        // async_std::task::sleep(std::time::Duration::from_secs(2)).await;

        // TODO: Call actual backend here.
        // For visual prototyping, let's just fill a mock result or partial result.

        let mut new_grid = vec![0usize; 81];
        // simple fill for demo (simulating a "solved" grid roughly)
        for i in 0..81 {
            // Just a dummy fill pattern
            new_grid[i] = (i % 9) + 1;
        }

        grid.set(new_grid.clone());

        // NOW: Verify On-Chain
        let agent = crate::api::create_agent().await;

        // Convert grid to Vec<Vec<candid::Int>> for Motoko
        let mut matrix: Vec<Vec<candid::Int>> = vec![];
        for r in 0..9 {
            let mut row_vec = vec![];
            for c in 0..9 {
                let val = new_grid[r * 9 + c];
                row_vec.push(candid::Int::from(val));
            }
            matrix.push(row_vec);
        }

        match crate::api::verify_schedule(&agent, matrix).await {
            Ok(res) => {
                verification_status.set(Some(res.status));
                if !res.is_valid {
                    error_msg.set(Some("Schedule Verification Failed On-Chain".to_string()));
                }
            }
            Err(e) => {
                error_msg.set(Some(format!("Verification Error: {}", e)));
            }
        }

        is_loading.set(false);
    };

    rsx! {
        div { class: "flex flex-col h-full bg-background text-foreground",
            // Header
            div { class: "border-b p-4 flex items-center justify-between bg-card",
                div { class: "flex items-center gap-4",
                    button {
                        class: "p-2 hover:bg-muted rounded-full transition-colors",
                        onclick: move |_| props.on_back.call(()),
                        "← Back"
                    }
                    div {
                        h1 { class: "font-semibold text-lg", "HRM Scheduler" }
                        p { class: "text-xs text-muted-foreground", "Sudoku-based Resource Optimization" }
                    }
                }
                button {
                    class: "px-4 py-2 bg-primary text-primary-foreground rounded hover:bg-primary/90 disabled:opacity-50",
                    disabled: is_loading(),
                    onclick: solve_puzzle,
                    if is_loading() { "Solving..." } else { "Solve Schedule" }
                }
            }

            // Body
            div { class: "flex-1 flex overflow-hidden",
                // Left: Input
                div { class: "w-1/3 border-r flex flex-col p-4 gap-2 bg-muted/5",
                    label { class: "text-sm font-medium", "Constraints (JSON)" }
                    textarea {
                        class: "flex-1 p-2 font-mono text-sm border rounded resize-none focus:outline-none focus:ring-1 focus:ring-primary",
                        value: "{json_input}",
                        oninput: move |e| json_input.set(e.value())
                    }
                    if let Some(err) = error_msg.read().as_ref() {
                        div { class: "text-destructive text-xs", "{err}" }
                    }
                }

                // Right: Visualization
                div { class: "flex-1 p-8 overflow-y-auto flex flex-col items-center justify-center bg-muted/20 gap-4",

                    // Verification Badge
                    if let Some(status) = verification_status.read().as_ref() {
                        div {
                            class: if status == "verified" {
                                "px-4 py-1 bg-green-500/10 text-green-500 border border-green-500/20 rounded-full text-sm font-medium flex items-center gap-2"
                            } else {
                                "px-4 py-1 bg-red-500/10 text-red-500 border border-red-500/20 rounded-full text-sm font-medium flex items-center gap-2"
                            },
                            if status == "verified" { "✓ Verified On-Chain" } else { "⚠ Validation Failed" }
                        }
                    }

                    div { class: "bg-background p-6 rounded-xl shadow-sm border",
                        div { class: "grid grid-cols-9 gap-1 border-2 border-slate-800 p-1 bg-slate-800",
                            for (idx, &cell) in grid.read().iter().enumerate() {
                                {
                                    // Formatting for 3x3 Block visuals
                                    let row = idx / 9;
                                    let col = idx % 9;
                                    let mb = if (row + 1) % 3 == 0 && row < 8 { "mb-1" } else { "" };
                                    let mr = if (col + 1) % 3 == 0 && col < 8 { "mr-1" } else { "" };

                                    rsx! {
                                        div {
                                            class: "w-10 h-10 flex items-center justify-center bg-white text-lg font-bold {mb} {mr}",
                                            key: "{idx}",
                                            if cell > 0 {
                                                span { class: "text-slate-900", "{cell}" }
                                            } else {
                                                span { class: "text-slate-200", "." }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
