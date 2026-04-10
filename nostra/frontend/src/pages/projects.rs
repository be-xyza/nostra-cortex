use crate::api::{create_agent, execute_kip_mutation, execute_kip_query};
use crate::components::knowledge_results_panel::KnowledgeResultsPanel;
use crate::services::knowledge_search::{
    SurfaceAskInput, SurfaceSearchInput, ask_knowledge_for_surface, parse_csv_field,
    search_knowledge_for_surface,
};
use crate::types::Entity;
use crate::types::{KnowledgeAskResponse, KnowledgeSearchResult, SearchFilters};
use dioxus::prelude::*;

/// Entity types that can be created within a Project
#[derive(Clone, PartialEq, Debug)]
pub enum ContributionType {
    Issue,
    Decision,
    Milestone,
    Deliverable,
}

impl ContributionType {
    fn label(&self) -> &'static str {
        match self {
            Self::Issue => "Issue",
            Self::Decision => "Decision",
            Self::Milestone => "Milestone",
            Self::Deliverable => "Deliverable",
        }
    }

    fn icon(&self) -> &'static str {
        match self {
            Self::Issue => "🐛",
            Self::Decision => "⚖️",
            Self::Milestone => "🏁",
            Self::Deliverable => "📦",
        }
    }

    fn kip_type(&self) -> &'static str {
        match self {
            Self::Issue => "Issue",
            Self::Decision => "Decision",
            Self::Milestone => "Milestone",
            Self::Deliverable => "Deliverable",
        }
    }
}

#[component]
pub fn ProjectsPage() -> Element {
    let mut projects = use_signal(|| Vec::<Entity>::new());
    let mut selected_project = use_signal(|| None::<Entity>);
    let mut contributions = use_signal(|| Vec::<Entity>::new());
    let mut error_msg = use_signal(|| None::<String>);
    let mut success_msg = use_signal(|| None::<String>);
    let mut loading = use_signal(|| true);

    // Create Contribution Form State
    let mut show_create_modal = use_signal(|| false);
    let mut create_type = use_signal(|| ContributionType::Issue);
    let mut create_title = use_signal(|| String::new());
    let mut create_description = use_signal(|| String::new());
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

    let fetch_projects = move || async move {
        loading.set(true);
        let agent = create_agent().await;
        match execute_kip_query(&agent, "FIND { @type: \"Project\" }".to_string()).await {
            Ok(json_str) => match serde_json::from_str::<Vec<Entity>>(&json_str) {
                Ok(list) => {
                    projects.set(list);
                    error_msg.set(None);
                }
                Err(e) => error_msg.set(Some(format!("Failed to parse projects: {}", e))),
            },
            Err(e) => error_msg.set(Some(format!("Failed to fetch projects: {}", e))),
        }
        loading.set(false);
    };

    let fetch_contributions = move |project_id: String| async move {
        let agent = create_agent().await;
        // Fetch all child entities linked to this project
        let query = format!("FIND {{ prop:project_id: \"{}\" }}", project_id);
        match execute_kip_query(&agent, query).await {
            Ok(json_str) => match serde_json::from_str::<Vec<Entity>>(&json_str) {
                Ok(list) => contributions.set(list),
                Err(_) => contributions.set(Vec::new()),
            },
            Err(_) => contributions.set(Vec::new()),
        }
    };

    use_future(move || fetch_projects());

    let mut handle_select_project = move |project: Entity| {
        let pid = project.id.clone();
        selected_project.set(Some(project));
        spawn(async move {
            fetch_contributions(pid).await;
        });
    };

    let mut handle_create_contribution = move || {
        if create_title.read().is_empty() {
            error_msg.set(Some("Title is required".to_string()));
            return;
        }

        let Some(project) = selected_project.read().clone() else {
            error_msg.set(Some("No project selected".to_string()));
            return;
        };

        is_submitting.set(true);
        let c_type = create_type.read().clone();

        spawn(async move {
            let t = create_title.read().clone();
            let d = create_description.read().clone();
            let safe_title = t.replace("\"", "\\\"");
            let safe_desc = d.replace("\"", "\\\"");

            let command = format!(
                "UPSERT {{ @type: \"{}\", prop:title: \"{}\", prop:description: \"{}\", prop:status: \"Open\", prop:project_id: \"{}\" }}",
                c_type.kip_type(),
                safe_title,
                safe_desc,
                project.id
            );

            let agent = create_agent().await;
            match execute_kip_mutation(&agent, command).await {
                Ok(_) => {
                    success_msg.set(Some(format!("{} created!", c_type.label())));
                    error_msg.set(None);
                    create_title.set(String::new());
                    create_description.set(String::new());
                    show_create_modal.set(false);
                    fetch_contributions(project.id.clone()).await;
                }
                Err(e) => error_msg.set(Some(format!("Failed to create: {}", e))),
            }
            is_submitting.set(false);
        });
    };

    // Transition Handler
    let handle_transition = move |entity: Entity, new_status: String| {
        // Create local copies of data needed inside the future
        let entity_id = entity.id.clone();
        let _entity_type_str = entity
            .attributes
            .iter()
            .find(|(k, _)| k == "@type")
            .map(|(_, v)| v.clone())
            .unwrap_or_default();
        let project_id = entity
            .attributes
            .iter()
            .find(|(k, _)| k == "prop:project_id")
            .map(|(_, v)| v.clone())
            .unwrap_or_default();

        spawn(async move {
            let update_cmd = format!(
                "UPSERT {{ id: \"{}\", prop:status: \"{}\" }}",
                entity_id, new_status
            );

            let agent = create_agent().await;
            match execute_kip_mutation(&agent, update_cmd).await {
                Ok(_) => {
                    // If we have a project ID, refresh the list
                    if !project_id.is_empty() {
                        fetch_contributions(project_id).await;
                    }
                }
                Err(e) => {
                    // Ideally we'd set an error message here, but we'd need to thread the signal through or ignore it
                    println!("Failed to transition status: {}", e);
                }
            }
        });
    };

    let handle_find_related = move |_| {
        let Some(project) = selected_project.read().clone() else {
            related_error.set(Some("Select a project before searching.".to_string()));
            return;
        };

        let project_name = project
            .attributes
            .iter()
            .find(|(k, _)| k == "prop:name")
            .map(|(_, v)| v.clone())
            .unwrap_or_else(|| "Untitled".to_string());
        let project_desc = project
            .attributes
            .iter()
            .find(|(k, _)| k == "prop:description")
            .map(|(_, v)| v.clone())
            .unwrap_or_default();
        let project_tags = project
            .attributes
            .iter()
            .find(|(k, _)| k == "prop:tags")
            .map(|(_, v)| parse_csv_field(v))
            .unwrap_or_else(|| vec!["project".to_string()]);

        let mut filters = SearchFilters::default();
        filters.space_id = project
            .attributes
            .iter()
            .find(|(k, _)| k == "prop:space_id")
            .map(|(_, v)| v.clone());
        filters.tags = project_tags;
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

        let query = format!("{} {}", project_name, project_desc).trim().to_string();
        if query.is_empty() {
            related_error.set(Some("Selected project has no searchable content.".to_string()));
            return;
        }

        related_loading.set(true);
        related_error.set(None);
        related_feedback.set(Some("Finding related knowledge...".to_string()));
        related_context.set(Some(project_name));
        related_ask.set(None);

        spawn(async move {
            let request = SurfaceSearchInput {
                query,
                retrieval_mode: related_retrieval_mode.read().clone(),
                diagnostics: *diagnostics.read(),
                filters: Some(filters),
                ..SurfaceSearchInput::default()
            };
            match search_knowledge_for_surface("projects", request).await {
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
            related_error.set(Some("Run related search first.".to_string()));
            return;
        };

        related_loading.set(true);
        related_error.set(None);
        related_feedback.set(Some("Generating grounded answer...".to_string()));
        let request = SurfaceAskInput {
            question: format!("What knowledge is most relevant to project: {}", context),
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
            match ask_knowledge_for_surface("projects", request).await {
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
        div { class: "p-6 space-y-6 animate-in fade-in duration-500",
            // Header
            div { class: "flex items-center justify-between",
                div {
                    h1 { class: "text-3xl font-bold tracking-tight", "Projects" }
                    p { class: "text-muted-foreground", "Track ideas promoted to active projects." }
                }
            }

            // Feedback
            if let Some(err) = error_msg.read().as_ref() {
                div { class: "p-4 border border-destructive/50 bg-destructive/10 text-destructive rounded-md", "{err}" }
            }
            if let Some(msg) = success_msg.read().as_ref() {
                div { class: "p-4 border border-green-500/50 bg-green-500/10 text-green-700 rounded-md", "{msg}" }
            }

            div { class: "grid gap-6 lg:grid-cols-[300px_1fr]",
                // LEFT: Project List
                div { class: "space-y-4",
                    h2 { class: "text-lg font-semibold", "Active Projects" }
                    if *loading.read() {
                        div { class: "flex justify-center p-8",
                            div { class: "animate-spin h-6 w-6 border-2 border-primary border-t-transparent rounded-full" }
                        }
                    } else if projects.read().is_empty() {
                        div { class: "text-center p-8 border-2 border-dashed rounded-lg text-muted-foreground",
                            "No projects yet. Promote an idea to start!"
                        }
                    } else {
                        div { class: "space-y-2",
                            for project in projects.read().clone().into_iter() {
                                {
                                    let is_selected = selected_project.read().as_ref()
                                        .map(|p| p.id == project.id)
                                        .unwrap_or(false);
                                    let proj_clone = project.clone();

                                    rsx! {
                                        button {
                                            class: format!(
                                                "w-full text-left p-4 rounded-lg border transition-colors {}",
                                                if is_selected { "border-primary bg-primary/5" } else { "hover:bg-muted/50" }
                                            ),
                                            onclick: move |_| handle_select_project(proj_clone.clone()),
                                            ProjectListItem { entity: project.clone() }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // RIGHT: Project Detail + Contributions
                div { class: "min-h-[400px]",
                    if let Some(project) = selected_project.read().clone() {
                        div { class: "space-y-6",
                            // Project Header
                            div { class: "flex items-start justify-between p-6 rounded-lg border bg-card",
                                div {
                                    h2 { class: "text-2xl font-bold",
                                        {project.attributes.iter().find(|(k, _)| k == "prop:name").map(|(_, v)| v.as_str()).unwrap_or("Untitled")}
                                    }
                                    p { class: "text-muted-foreground mt-1",
                                        {project.attributes.iter().find(|(k, _)| k == "prop:description").map(|(_, v)| v.as_str()).unwrap_or("")}
                                    }
                                }
                                div { class: "flex gap-2",
                                    button {
                                        class: "inline-flex items-center justify-center rounded-md border border-input bg-background px-3 text-sm hover:bg-accent h-9",
                                        onclick: handle_find_related,
                                        "Find related knowledge"
                                    }
                                    button {
                                        class: "inline-flex items-center justify-center rounded-md text-sm font-medium bg-primary text-primary-foreground hover:bg-primary/90 h-9 px-4",
                                        onclick: move |_| show_create_modal.set(true),
                                        "+ Add Contribution"
                                    }
                                }
                            }

                            div { class: "rounded-lg border bg-card p-4 space-y-3",
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
                                div { class: "flex items-center justify-between",
                                    div { class: "flex items-center gap-3",
                                        div { class: "flex items-center gap-2 text-xs text-muted-foreground",
                                            span { "Search mode" }
                                            select {
                                                class: "h-8 rounded-md border border-input bg-background px-2 text-xs",
                                                "data-a2ui-id": "projects-search-mode",
                                                value: "{related_retrieval_mode}",
                                                onchange: move |evt| related_retrieval_mode.set(evt.value()),
                                                option { value: "hybrid", "Semantic" }
                                                option { value: "lexical", "Keyword" }
                                            }
                                        }
                                        div { class: "flex items-center gap-2 text-xs text-muted-foreground",
                                            span { "Modality" }
                                            select {
                                                class: "h-8 rounded-md border border-input bg-background px-2 text-xs",
                                                "data-a2ui-id": "projects-search-modality",
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
                                    }
                                    button {
                                        class: "inline-flex items-center justify-center rounded-md border border-input bg-background px-3 py-1 text-xs hover:bg-accent",
                                        onclick: handle_related_ask,
                                        disabled: related_loading() || related_context().is_none(),
                                        "Ask with provenance"
                                    }
                                }
                                KnowledgeResultsPanel {
                                    title: "Project Knowledge Results".to_string(),
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

                            // Contribution Type Tabs
                            div { class: "flex gap-2 border-b pb-2",
                                for ctype in [ContributionType::Issue, ContributionType::Decision, ContributionType::Milestone, ContributionType::Deliverable] {
                                    button {
                                        class: "px-3 py-1.5 text-sm font-medium rounded-md hover:bg-muted transition-colors",
                                        "{ctype.icon()} {ctype.label()}s"
                                    }
                                }
                            }

                            // Contributions List
                            div { class: "space-y-3",
                                if contributions.read().is_empty() {
                                    div { class: "text-center p-12 border-2 border-dashed rounded-lg text-muted-foreground",
                                        "No contributions yet. Add an Issue, Decision, Milestone, or Deliverable."
                                    }
                                } else {
                                    for contrib in contributions.read().iter() {
                                        {
                                            let entity = contrib.clone();
                                            let h_trans = handle_transition; // Copy closure
                                            rsx! {
                                                ContributionCard {
                                                    entity: entity.clone(),
                                                    on_transition: move |new_status| h_trans(entity.clone(), new_status)
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        div { class: "flex items-center justify-center h-full border-2 border-dashed rounded-lg text-muted-foreground",
                            "Select a project to view details"
                        }
                    }
                }
            }

            // Create Modal (unchanged logic, omitted for brevity but included in output if file rewritten)
             if *show_create_modal.read() {
                div { class: "fixed inset-0 bg-black/50 flex items-center justify-center z-50",
                    div { class: "bg-background rounded-lg shadow-lg p-6 w-full max-w-md space-y-4",
                        h3 { class: "text-lg font-semibold", "Add Contribution" }

                        // Type Selector
                        div { class: "grid grid-cols-4 gap-2",
                            for ctype in [ContributionType::Issue, ContributionType::Decision, ContributionType::Milestone, ContributionType::Deliverable] {
                                {
                                    let is_selected = *create_type.read() == ctype;
                                    let ct = ctype.clone();
                                    rsx! {
                                        button {
                                            class: format!(
                                                "flex flex-col items-center p-3 rounded-lg border transition-colors {}",
                                                if is_selected { "border-primary bg-primary/5" } else { "hover:bg-muted/50" }
                                            ),
                                            onclick: move |_| create_type.set(ct.clone()),
                                            span { class: "text-2xl", "{ctype.icon()}" }
                                            span { class: "text-xs mt-1", "{ctype.label()}" }
                                        }
                                    }
                                }
                            }
                        }

                        // Title
                        div { class: "space-y-2",
                            label { class: "text-sm font-medium", "Title" }
                            input {
                                class: "flex h-9 w-full rounded-md border bg-background px-3 py-1 text-sm",
                                placeholder: "Enter title...",
                                value: "{create_title}",
                                oninput: move |e| create_title.set(e.value())
                            }
                        }

                        // Description
                        div { class: "space-y-2",
                            label { class: "text-sm font-medium", "Description" }
                            textarea {
                                class: "w-full min-h-[80px] rounded-md border bg-background px-3 py-2 text-sm",
                                placeholder: "Details...",
                                value: "{create_description}",
                                oninput: move |e| create_description.set(e.value())
                            }
                        }

                         // Actions
                        div { class: "flex justify-end gap-2 pt-4",
                            button {
                                class: "px-4 py-2 text-sm font-medium rounded-md border hover:bg-muted",
                                onclick: move |_| show_create_modal.set(false),
                                "Cancel"
                            }
                            button {
                                class: "px-4 py-2 text-sm font-medium rounded-md bg-primary text-primary-foreground hover:bg-primary/90 disabled:opacity-50",
                                disabled: is_submitting(),
                                onclick: move |_| handle_create_contribution(),
                                if is_submitting() { "Creating..." } else { "Create" }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ProjectListItem(entity: Entity) -> Element {
    let name = entity
        .attributes
        .iter()
        .find(|(k, _)| k == "prop:name")
        .map(|(_, v)| v.clone())
        .unwrap_or_else(|| "Untitled".to_string());
    let status = entity
        .attributes
        .iter()
        .find(|(k, _)| k == "prop:status")
        .map(|(_, v)| v.clone())
        .unwrap_or_else(|| "Active".to_string());

    rsx! {
        div { class: "flex items-center justify-between",
            span { class: "font-medium truncate", "{name}" }
            span { class: "text-xs px-2 py-0.5 rounded bg-green-100 text-green-800", "{status}" }
        }
    }
}

#[component]
fn ContributionCard(entity: Entity, on_transition: EventHandler<String>) -> Element {
    // Get entity type from attributes (since entity_type is an enum, use a fallback attribute)
    let entity_type_str = entity
        .attributes
        .iter()
        .find(|(k, _)| k == "@type")
        .map(|(_, v)| v.clone())
        .unwrap_or_else(|| format!("{:?}", entity.entity_type));
    let title = entity
        .attributes
        .iter()
        .find(|(k, _)| k == "prop:title")
        .map(|(_, v)| v.clone())
        .unwrap_or_default();
    let status = entity
        .attributes
        .iter()
        .find(|(k, _)| k == "prop:status")
        .map(|(_, v)| v.clone())
        .unwrap_or_else(|| "Open".to_string());

    let icon = match entity_type_str.as_str() {
        "Issue" => "🐛",
        "Decision" => "⚖️",
        "Milestone" => "🏁",
        "Deliverable" => "📦",
        _ => "📄",
    };

    let status_class = match status.as_str() {
        "Open" => "bg-blue-100 text-blue-800",
        "InProgress" => "bg-yellow-100 text-yellow-800",
        "Completed" | "Approved" => "bg-green-100 text-green-800",
        "Rejected" => "bg-red-100 text-red-800",
        _ => "bg-muted text-muted-foreground",
    };

    // Determine available actions based on status
    let actions = match status.as_str() {
        "Open" => vec!["Start", "Reject"],
        "InProgress" | "Start" => vec!["Complete", "Reject"], // Treat "Start" same as InProgress if backend misses update
        "Completed" => vec!["Approve", "Reject"],
        _ => vec![],
    };

    rsx! {
        div { class: "flex items-center gap-3 p-4 rounded-lg border hover:shadow-sm transition-shadow",
             span { class: "text-xl", "{icon}" }
            div { class: "flex-1 min-w-0",
                div { class: "font-medium truncate", "{title}" }
                div { class: "text-xs text-muted-foreground", "{entity_type_str}" }
            }
             div { class: "flex items-center gap-2",
                span { class: "text-xs px-2 py-0.5 rounded {status_class}", "{status}" }

                // Action Buttons
                for action in actions {
                     button {
                        class: "text-xs px-2 py-0.5 rounded border hover:bg-muted transition-colors",
                        onclick: move |_| on_transition.call(match action {
                            "Start" => "InProgress".to_string(),
                            "Complete" => "Completed".to_string(),
                            "Approve" => "Approved".to_string(),
                            "Reject" => "Rejected".to_string(),
                            _ => action.to_string(),
                        }),
                        "{action}"
                    }
                }
            }
        }
    }
}
