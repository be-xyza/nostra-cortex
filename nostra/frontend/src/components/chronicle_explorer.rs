use crate::api::create_agent;
use crate::types::{ChronicleEvent, ChronicleEventType};
use dioxus::prelude::*;

pub fn ChronicleExplorer() -> Element {
    let mut events = use_signal(|| Vec::<ChronicleEvent>::new());
    let mut is_loading = use_signal(|| true);
    let mut error_msg = use_signal(|| Option::<String>::None);

    use_effect(move || {
        spawn(async move {
            is_loading.set(true);
            let agent = create_agent().await;

            // Fetch last 50 events
            // range=None implies everything (backend logic handles since/until optionality)
            // or we might pass a range if backend requires it.
            // In `getChronicleEvents` API wrapper we defined `None` as (None, None).

            match crate::api::get_chronicle_events(&agent, None, 50).await {
                Ok(evts) => {
                    events.set(evts);
                    error_msg.set(None);
                }
                Err(e) => {
                    error_msg.set(Some(format!("Failed to load chronicle events: {}", e)));
                }
            }
            is_loading.set(false);
        });
    });

    rsx! {
        div { class: "flex flex-col gap-4",
            // Header / Controls
            div { class: "flex items-center justify-between",
                h3 { class: "text-lg font-medium", "Living Library History" }
                button {
                    class: "px-3 py-1 text-sm bg-secondary text-secondary-foreground rounded hover:bg-secondary/80",
                    onclick: move |_| {
                        // Refresh logic - just re-trigger effect or separate function
                        // ideally refactor fetch into separate async fn
                    },
                    "Refresh"
                }
            }

            if let Some(err) = error_msg() {
                div { class: "p-4 text-sm text-red-500 bg-red-50 rounded border border-red-200",
                    "{err}"
                }
            }

            if is_loading() {
                div { class: "p-8 text-center text-muted-foreground animate-pulse",
                    "Loading history..."
                }
            } else if events().is_empty() {
                div { class: "p-8 text-center text-muted-foreground border border-dashed rounded-lg",
                    "No events recorded."
                }
            } else {
                div { class: "rounded-md border",
                    table { class: "w-full text-sm",
                        thead { class: "bg-muted/50 border-b",
                            tr {
                                th { class: "h-10 px-4 text-left font-medium align-middle", "Time" }
                                th { class: "h-10 px-4 text-left font-medium align-middle", "Actor" }
                                th { class: "h-10 px-4 text-left font-medium align-middle", "Event" }
                                th { class: "h-10 px-4 text-left font-medium align-middle", "Description" }
                                th { class: "h-10 px-4 text-left font-medium align-middle", "Details" }
                            }
                        }
                        tbody {
                            for event in events() {
                                tr { class: "border-b transition-colors hover:bg-muted/50",
                                    td { class: "p-4 align-middle",
                                        "{format_timestamp(event.timestamp.clone())}"
                                    }
                                    td { class: "p-4 align-middle font-mono text-xs",
                                        "{truncate_principal(event.actor_principal)}"
                                    }
                                    td { class: "p-4 align-middle",
                                        Badge { event_type: event.event_type.clone() }
                                    }
                                    td { class: "p-4 align-middle", "{event.description}" }
                                    td { class: "p-4 align-middle font-mono text-xs text-muted-foreground",
                                        if let Some(lib) = &event.library_id {
                                            div { "Lib: {lib}" }
                                        }
                                        div { "Entities: {event.affected_entities.len()}" }
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

// Helper Components

#[component]
fn Badge(event_type: ChronicleEventType) -> Element {
    let (bg, text, label) = match &event_type {
        ChronicleEventType::EntityCreated => (
            "bg-green-100 dark:bg-green-900",
            "text-green-700 dark:text-green-300",
            "Created",
        ),
        ChronicleEventType::EntityUpdated => (
            "bg-blue-100 dark:bg-blue-900",
            "text-blue-700 dark:text-blue-300",
            "Updated",
        ),
        ChronicleEventType::EntityDeleted => (
            "bg-red-100 dark:bg-red-900",
            "text-red-700 dark:text-red-300",
            "Deleted",
        ),
        ChronicleEventType::RelationshipFormed => (
            "bg-purple-100 dark:bg-purple-900",
            "text-purple-700 dark:text-purple-300",
            "Rel+",
        ),
        ChronicleEventType::RelationshipBroken => (
            "bg-orange-100 dark:bg-orange-900",
            "text-orange-700 dark:text-orange-300",
            "Rel-",
        ),
        ChronicleEventType::LibraryInstalled => (
            "bg-cyan-100 dark:bg-cyan-900",
            "text-cyan-700 dark:text-cyan-300",
            "Lib Install",
        ),
        ChronicleEventType::GovernanceProposal => (
            "bg-yellow-100 dark:bg-yellow-900",
            "text-yellow-700 dark:text-yellow-300",
            "Proposal",
        ),
        ChronicleEventType::GovernanceVote => (
            "bg-indigo-100 dark:bg-indigo-900",
            "text-indigo-700 dark:text-indigo-300",
            "Vote",
        ),
        ChronicleEventType::Custom(_c) => (
            "bg-gray-100 dark:bg-gray-800",
            "text-gray-700 dark:text-gray-300",
            "Custom",
        ),
    };

    // For custom, display the string in 'label' isn't quite right with the layout above
    // but the match arm returns "Custom". Let's format nicely.
    let display_label = match &event_type {
        ChronicleEventType::Custom(s) => s.as_str(),
        _ => label,
    };

    rsx! {
        span { class: "inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 {bg} {text} border-transparent",
            "{display_label}"
        }
    }
}

fn format_timestamp(ts: candid::Int) -> String {
    // Nanoseconds to readable string
    // ts.0 is BigInt. to_u64_digits returns (Sign, Vec<u64>)
    let parts = ts.0.to_u64_digits();
    let ns = parts.1.first().copied().unwrap_or(0);
    let seconds = ns / 1_000_000_000;

    // Using js_sys for consistent formatting
    let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64((seconds * 1000) as f64));

    // Use ISO string for robustness
    date.to_iso_string()
        .as_string()
        .unwrap_or_else(|| format!("{}", seconds))
}

fn truncate_principal(p: candid::Principal) -> String {
    let s = p.to_text();
    if s.len() > 15 {
        format!("{}...{}", &s[0..6], &s[s.len() - 4..])
    } else {
        s
    }
}
