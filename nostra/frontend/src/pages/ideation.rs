use crate::api::{create_agent, execute_kip_mutation, execute_kip_query};
use crate::components::knowledge_results_panel::KnowledgeResultsPanel;
use crate::services::knowledge_search::{
    SurfaceAskInput, SurfaceSearchInput, ask_knowledge_for_surface, parse_csv_field,
    search_knowledge_for_surface,
};
use crate::types::Entity;
use crate::types::{KnowledgeAskResponse, KnowledgeSearchResult, SearchFilters};
use dioxus::prelude::*;

#[component]
pub fn IdeationPage() -> Element {
    let mut ideas = use_signal(|| Vec::<Entity>::new());
    let mut error_msg = use_signal(|| None::<String>);
    let mut success_msg = use_signal(|| None::<String>);
    let mut loading = use_signal(|| true);

    // Form State
    let mut title = use_signal(|| String::new());
    let mut summary = use_signal(|| String::new());
    let mut is_submitting = use_signal(|| false);
    let mut related_results = use_signal(|| Vec::<KnowledgeSearchResult>::new());
    let mut related_feedback = use_signal(|| None::<String>);
    let mut related_error = use_signal(|| None::<String>);
    let mut related_loading = use_signal(|| false);
    let mut related_ask = use_signal(|| None::<KnowledgeAskResponse>);
    let mut related_context = use_signal(|| None::<String>);
    let mut related_retrieval_mode = use_signal(|| "hybrid".to_string());
    let mut filter_perspective_scope = use_signal(|| String::new());
    let mut filter_produced_by_agent = use_signal(|| String::new());
    let mut filter_source_version_id = use_signal(|| String::new());
    let mut filter_modalities = use_signal(|| String::new());
    let mut diagnostics = use_signal(|| false);

    let fetch_ideas = move || async move {
        loading.set(true);
        let agent = create_agent().await;
        match execute_kip_query(&agent, "FIND { @type: \"Idea\" }".to_string()).await {
            Ok(json_str) => match serde_json::from_str::<Vec<Entity>>(&json_str) {
                Ok(list) => {
                    ideas.set(list);
                    error_msg.set(None);
                }
                Err(e) => {
                    error_msg.set(Some(format!("Failed to parse ideas: {}", e)));
                }
            },
            Err(e) => {
                error_msg.set(Some(format!("Failed to fetch ideas: {}", e)));
            }
        }
        loading.set(false);
    };

    use_future(move || fetch_ideas());

    let mut handle_submit = move || {
        if title.read().is_empty() {
            error_msg.set(Some("Title is required".to_string()));
            return;
        }

        is_submitting.set(true);
        spawn(async move {
            let t = title.read().clone();
            let s = summary.read().clone();
            let safe_title = t.replace("\"", "\\\"");
            let safe_summary = s.replace("\"", "\\\"");

            let command = format!(
                "UPSERT {{ @type: \"Idea\", prop:title: \"{}\", prop:summary: \"{}\", prop:status: \"Draft\" }}",
                safe_title, safe_summary
            );

            let agent = create_agent().await;
            match execute_kip_mutation(&agent, command).await {
                Ok(_) => {
                    success_msg.set(Some("Idea captured successfully!".to_string()));
                    error_msg.set(None);
                    title.set(String::new());
                    summary.set(String::new());
                    fetch_ideas().await;
                }
                Err(e) => error_msg.set(Some(format!("Failed to submit idea: {}", e))),
            }
            is_submitting.set(false);
        });
    };

    // PROMOTE HANDLER: Creates a Project from this Idea
    let handle_promote = move |idea: Entity| {
        spawn(async move {
            let agent = create_agent().await;

            // Extract title from idea
            let idea_title = idea
                .attributes
                .iter()
                .find(|(k, _)| k == "prop:title")
                .map(|(_, v)| v.clone())
                .unwrap_or_else(|| "Untitled".to_string());
            let idea_summary = idea
                .attributes
                .iter()
                .find(|(k, _)| k == "prop:summary")
                .map(|(_, v)| v.clone())
                .unwrap_or_default();

            let safe_title = idea_title.replace("\"", "\\\"");
            let safe_summary = idea_summary.replace("\"", "\\\"");

            // 1. Create Project Entity linked to Idea
            let create_project = format!(
                "UPSERT {{ @type: \"Project\", prop:name: \"{}\", prop:description: \"{}\", prop:status: \"Active\", prop:source_idea: \"{}\" }}",
                safe_title, safe_summary, idea.id
            );

            match execute_kip_mutation(&agent, create_project).await {
                Ok(_) => {
                    // 2. Update Idea status to "Promoted"
                    let update_idea = format!(
                        "UPSERT {{ id: \"{}\", prop:status: \"Promoted\" }}",
                        idea.id
                    );
                    let _ = execute_kip_mutation(&agent, update_idea).await;

                    success_msg.set(Some(format!("'{}' promoted to Project!", idea_title)));
                    fetch_ideas().await;
                }
                Err(e) => {
                    error_msg.set(Some(format!("Failed to promote: {}", e)));
                }
            }
        });
    };

    let mut handle_find_related = move |idea: Entity| {
        let title_value = idea
            .attributes
            .iter()
            .find(|(k, _)| k == "prop:title")
            .map(|(_, v)| v.clone())
            .unwrap_or_else(|| "Untitled".to_string());
        let summary_value = idea
            .attributes
            .iter()
            .find(|(k, _)| k == "prop:summary")
            .map(|(_, v)| v.clone())
            .unwrap_or_default();
        let tags = idea
            .attributes
            .iter()
            .find(|(k, _)| k == "prop:tags")
            .map(|(_, v)| parse_csv_field(v))
            .unwrap_or_else(|| vec!["idea".to_string()]);
        let space_id = idea
            .attributes
            .iter()
            .find(|(k, _)| k == "prop:space_id")
            .map(|(_, v)| v.clone());
        let query = format!("{} {}", title_value, summary_value).trim().to_string();

        if query.is_empty() {
            related_error.set(Some("Selected idea has no searchable content.".to_string()));
            return;
        }

        let mut filters = SearchFilters::default();
        filters.space_id = space_id;
        filters.tags = tags;
        let perspective_scope = filter_perspective_scope.read().trim().to_string();
        let produced_by_agent = filter_produced_by_agent.read().trim().to_string();
        let source_version_id = filter_source_version_id.read().trim().to_string();
        let modalities = parse_csv_field(filter_modalities.read().trim());
        if !perspective_scope.is_empty() {
            filters.perspective_scope = Some(perspective_scope);
        }
        if !produced_by_agent.is_empty() {
            filters.produced_by_agent = Some(produced_by_agent);
        }
        if !source_version_id.is_empty() {
            filters.source_version_id = Some(source_version_id);
        }
        if !modalities.is_empty() {
            filters.modalities = modalities;
        }

        related_loading.set(true);
        related_error.set(None);
        related_feedback.set(Some("Finding related knowledge...".to_string()));
        related_context.set(Some(title_value.clone()));
        related_ask.set(None);

        spawn(async move {
            let request = SurfaceSearchInput {
                query,
                retrieval_mode: related_retrieval_mode.read().clone(),
                diagnostics: *diagnostics.read(),
                filters: Some(filters),
                ..SurfaceSearchInput::default()
            };
            match search_knowledge_for_surface("ideation", request).await {
                Ok(results) => {
                    related_feedback.set(Some(format!("Found {} related results.", results.len())));
                    related_results.set(results);
                }
                Err(err) => {
                    related_results.set(Vec::new());
                    related_error.set(Some(format!("Knowledge lookup failed: {}", err)));
                }
            }
            related_loading.set(false);
        });
    };

    let handle_related_ask = move |_| {
        let Some(context) = related_context.read().clone() else {
            related_error.set(Some("Select an idea and run related search first.".to_string()));
            return;
        };
        related_loading.set(true);
        related_error.set(None);
        related_feedback.set(Some("Generating grounded answer...".to_string()));
        let request = SurfaceAskInput {
            question: format!("What knowledge best supports this idea: {}", context),
            retrieval_mode: related_retrieval_mode.read().clone(),
            diagnostics: *diagnostics.read(),
            filters: Some(SearchFilters {
                perspective_scope: if filter_perspective_scope.read().trim().is_empty() {
                    None
                } else {
                    Some(filter_perspective_scope.read().trim().to_string())
                },
                produced_by_agent: if filter_produced_by_agent.read().trim().is_empty() {
                    None
                } else {
                    Some(filter_produced_by_agent.read().trim().to_string())
                },
                source_version_id: if filter_source_version_id.read().trim().is_empty() {
                    None
                } else {
                    Some(filter_source_version_id.read().trim().to_string())
                },
                modalities: parse_csv_field(filter_modalities.read().trim()),
                ..SearchFilters::default()
            }),
            ..SurfaceAskInput::default()
        };

        spawn(async move {
            match ask_knowledge_for_surface("ideation", request).await {
                Ok(answer) => {
                    related_ask.set(Some(answer));
                    related_feedback.set(Some("Grounded answer ready.".to_string()));
                }
                Err(err) => {
                    related_ask.set(None);
                    related_error.set(Some(format!("Grounded ask failed: {}", err)));
                }
            }
            related_loading.set(false);
        });
    };

    let selected_modality = {
        let parsed = parse_csv_field(filter_modalities.read().trim());
        if parsed.len() == 1 {
            parsed[0].clone()
        } else {
            "all".to_string()
        }
    };

    rsx! {
        div { class: "p-6 space-y-8 animate-in fade-in duration-500 max-w-4xl mx-auto",
            div { class: "space-y-2",
                h1 { class: "text-3xl font-bold tracking-tight", "Ideation Lab" }
                p { class: "text-muted-foreground", "Capture, refine, and promote your ideas into reality." }
            }

            // FEEDBACK
            if let Some(err) = error_msg.read().as_ref() {
                div { class: "p-4 border border-destructive/50 bg-destructive/10 text-destructive rounded-md", "Error: {err}" }
            }
            if let Some(msg) = success_msg.read().as_ref() {
                div { class: "p-4 border border-green-500/50 bg-green-500/10 text-green-700 rounded-md", "{msg}" }
            }

            div { class: "grid gap-8 md:grid-cols-[1fr_300px]",
                // MAIN: Idea List
                div { class: "space-y-6",
                    div { class: "flex items-center justify-between",
                        h2 { class: "text-xl font-semibold", "My Ideas" }
                        button {
                            class: "text-sm text-primary hover:underline",
                            onclick: move |_| { spawn(fetch_ideas()); },
                            "Refresh"
                        }
                    }

                    div { class: "rounded-lg border bg-card p-4 space-y-3",
                        div { class: "flex flex-wrap items-center gap-2 justify-between",
                            p { class: "text-sm font-medium", "Find related knowledge" }
                            button {
                                class: "inline-flex items-center justify-center rounded-md border border-input px-3 py-1 text-xs hover:bg-accent",
                                onclick: handle_related_ask,
                                disabled: related_loading() || related_context().is_none(),
                                "Ask with provenance"
                            }
                        }
                        div { class: "grid gap-2 md:grid-cols-3",
                            input {
                                class: "h-8 rounded-md border border-input bg-background px-2 text-xs font-mono",
                                placeholder: "perspective_scope",
                                value: "{filter_perspective_scope}",
                                oninput: move |evt| filter_perspective_scope.set(evt.value())
                            }
                            input {
                                class: "h-8 rounded-md border border-input bg-background px-2 text-xs font-mono",
                                placeholder: "produced_by_agent",
                                value: "{filter_produced_by_agent}",
                                oninput: move |evt| filter_produced_by_agent.set(evt.value())
                            }
                            input {
                                class: "h-8 rounded-md border border-input bg-background px-2 text-xs font-mono",
                                placeholder: "source_version_id",
                                value: "{filter_source_version_id}",
                                oninput: move |evt| filter_source_version_id.set(evt.value())
                            }
                        }
                        div { class: "grid gap-2",
                            input {
                                class: "h-8 rounded-md border border-input bg-background px-2 text-xs font-mono",
                                placeholder: "modalities (csv): text,image,audio,video",
                                value: "{filter_modalities}",
                                oninput: move |evt| filter_modalities.set(evt.value())
                            }
                        }
                        div { class: "flex items-center gap-2 text-xs text-muted-foreground",
                            span { "Search mode" }
                            select {
                                class: "h-8 rounded-md border border-input bg-background px-2 text-xs",
                                "data-a2ui-id": "ideation-search-mode",
                                value: "{related_retrieval_mode}",
                                onchange: move |evt| related_retrieval_mode.set(evt.value()),
                                option { value: "hybrid", "Semantic" }
                                option { value: "lexical", "Keyword" }
                            }
                            span { "Modality" }
                            select {
                                class: "h-8 rounded-md border border-input bg-background px-2 text-xs",
                                "data-a2ui-id": "ideation-search-modality",
                                value: "{selected_modality}",
                                onchange: move |evt| {
                                    let value = evt.value();
                                    if value == "all" {
                                        filter_modalities.set(String::new());
                                    } else {
                                        filter_modalities.set(value);
                                    }
                                },
                                option { value: "all", "All" }
                                option { value: "text", "Text" }
                                option { value: "image", "Image" }
                                option { value: "audio", "Audio" }
                                option { value: "video", "Video" }
                            }
                        }
                        label { class: "text-xs text-muted-foreground flex items-center gap-2",
                            input {
                                r#type: "checkbox",
                                checked: *diagnostics.read(),
                                onchange: move |evt| diagnostics.set(evt.value() == "true" || evt.value() == "on")
                            }
                            "Include diagnostics"
                        }
                        KnowledgeResultsPanel {
                            title: "Ideation Knowledge Results".to_string(),
                            subtitle: related_context()
                                .map(|ctx| format!("Context: {}", ctx)),
                            feedback: related_feedback(),
                            error: related_error(),
                            is_loading: related_loading(),
                            show_diagnostics: *diagnostics.read(),
                            results: related_results(),
                            ask_response: related_ask()
                        }
                    }

                    if *loading.read() {
                        div { class: "flex justify-center p-8",
                            div { class: "animate-spin h-6 w-6 border-2 border-primary border-t-transparent rounded-full" }
                        }
                    } else if ideas.read().is_empty() {
                        div { class: "text-center p-12 border-2 border-dashed rounded-lg text-muted-foreground",
                            "No ideas captured yet. Start by creating one!"
                        }
                    } else {
                        div { class: "grid gap-4",
                            for idea in ideas.read().clone().into_iter() {
                                {
                                    let idea_for_promote = idea.clone();
                                    let idea_for_related = idea.clone();
                                    rsx! {
                                        IdeaCard {
                                            entity: idea.clone(),
                                            on_promote: move |_| handle_promote(idea_for_promote.clone()),
                                            on_related: move |_| handle_find_related(idea_for_related.clone())
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // SIDEBAR: Create Form
                div {
                    div { class: "sticky top-6 rounded-lg border bg-card text-card-foreground shadow-sm p-6 space-y-4",
                        h3 { class: "font-semibold", "Capture New Idea" }

                        div { class: "space-y-2",
                            label { class: "text-sm font-medium", "Title" }
                            input {
                                class: "flex h-9 w-full rounded-md border border-input bg-background px-3 py-1 text-sm shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring",
                                placeholder: "e.g. AI Swarm Orchestrator",
                                value: "{title}",
                                oninput: move |e| title.set(e.value())
                            }
                        }

                        div { class: "space-y-2",
                            label { class: "text-sm font-medium", "Summary" }
                            textarea {
                                class: "flex min-h-[100px] w-full rounded-md border border-input bg-background px-3 py-2 text-sm shadow-sm focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring",
                                placeholder: "What problem does this solve?",
                                value: "{summary}",
                                oninput: move |e| summary.set(e.value())
                            }
                        }

                        button {
                            class: "w-full inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring bg-primary text-primary-foreground hover:bg-primary/90 h-9 px-4 py-2 disabled:opacity-50",
                            onclick: move |_| handle_submit(),
                            disabled: is_submitting(),
                            if is_submitting() { "Saving..." } else { "Capture Idea" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn IdeaCard(entity: Entity, on_promote: EventHandler<()>, on_related: EventHandler<()>) -> Element {
    let get_prop = |key: &str| {
        entity
            .attributes
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.clone())
            .unwrap_or_default()
    };

    let title = get_prop("prop:title");
    let summary = get_prop("prop:summary");
    let status = get_prop("prop:status");

    let status_class = if status == "Draft" {
        "bg-secondary text-secondary-foreground"
    } else if status == "Promoted" {
        "bg-green-100 text-green-800"
    } else {
        "bg-muted text-muted-foreground"
    };

    rsx! {
        div { class: "rounded-lg border bg-card text-card-foreground shadow-sm p-6 hover:shadow-md transition-shadow",
            div { class: "flex justify-between items-start mb-2",
                div {
                    h3 { class: "font-semibold text-lg leading-none tracking-tight", "{title}" }
                    if title.is_empty() {
                        span { class: "font-mono text-xs text-muted-foreground", "{entity.id}" }
                    }
                }
                div { class: "inline-flex items-center rounded-md border px-2 py-0.5 text-xs font-semibold {status_class}",
                    "{status}"
                }
            }
            p { class: "text-sm text-muted-foreground line-clamp-3", "{summary}" }

            div { class: "flex gap-2 mt-4",
                button { class: "text-xs font-medium text-primary hover:underline", "View Details" }
                button {
                    class: "text-xs font-medium text-indigo-600 hover:text-indigo-700",
                    onclick: move |_| on_related.call(()),
                    "Find related knowledge"
                }
                if status == "Draft" {
                    button { class: "text-xs font-medium text-muted-foreground hover:text-foreground", "Edit" }
                    button {
                        class: "text-xs font-medium text-green-600 hover:text-green-700 ml-auto",
                        onclick: move |_| on_promote.call(()),
                        "🚀 Promote to Project"
                    }
                }
            }
        }
    }
}
