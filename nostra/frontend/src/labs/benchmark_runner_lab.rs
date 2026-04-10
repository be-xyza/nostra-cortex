use dioxus::prelude::*;

#[component]
pub fn BenchmarkRunnerLab() -> Element {
    let benchmarks = use_signal(|| {
        vec![
            "gaia_val_001.json".to_string(),
            "gaia_val_002.json".to_string(),
        ]
    });

    rsx! {
        div {
            class: "flex flex-col h-full p-4 space-y-4",
            h1 { class: "text-2xl font-bold", "Benchmark Runner" }
            div {
                class: "flex space-x-4",
                div {
                    class: "w-1/3 bg-gray-100 p-4 rounded",
                    h2 { class: "font-semibold mb-2", "Available Cases" }
                    ul {
                        for bench in benchmarks() {
                            li {
                                class: "cursor-pointer hover:text-blue-600",
                                "{bench}"
                            }
                        }
                    }
                }
                div {
                    class: "w-2/3 space-y-4",
                    // Quality Dashboard Section
                    div {
                        class: "bg-white border p-4 rounded",
                        h2 { class: "font-semibold mb-2", "Quality Dashboard (The Traffic Light)" }
                        div {
                            class: "grid grid-cols-3 gap-4",
                            div { class: "p-2 bg-green-50 border border-green-200 rounded text-center",
                                span { class: "block text-sm text-green-600", "L1/L2 Code" },
                                b { "PASS" }
                            }
                            div { class: "p-2 bg-blue-50 border border-blue-200 rounded text-center",
                                span { class: "block text-sm text-blue-600", "L3 Arena" },
                                b { "95% Score" }
                            }
                            div { class: "p-2 bg-yellow-50 border border-yellow-200 rounded text-center",
                                span { class: "block text-sm text-yellow-600", "L4 Policy" },
                                b { "Verified" }
                            }
                        }
                    }
                    // Replay Theater Section
                    div {
                        class: "bg-black text-white p-4 rounded h-64 flex flex-col items-center justify-center",
                        h2 { class: "font-semibold mb-2 text-gray-400", "Agent Replay Theater" }
                        div { class: "text-blue-400 text-4xl mb-4", "▶" }
                        p { class: "text-gray-500", "Rendering A2UI Stream: Replaying Session #004..." }
                    }
                    // Drift Visualizer Section
                    div {
                        class: "bg-white border p-4 rounded",
                        h2 { class: "font-semibold mb-2", "Drift Visualizer (Expected vs Actual)" }
                        div {
                            id: "drift-visualizer-container",
                            class: "h-64 bg-gray-50 border-2 border-dashed border-gray-200 flex items-center justify-center rounded",
                            p { class: "text-gray-400 italic", "D3.js Graph Rendering: Showing 15 node deviations..." }
                        }
                    }
                }
            }
        }
    }
}
