use crate::api::{create_agent, execute_kip_mutation, execute_kip_query};
use crate::types::Entity;
use dioxus::prelude::*;

#[derive(Clone, Debug, PartialEq)]
struct SchemaPropertyState {
    name: String,
    data_type: String, // Text, Nat, Bool, Principal
    required: bool,
}

#[component]
pub fn SchemasPage() -> Element {
    let mut schemas = use_signal(|| Vec::<Entity>::new());
    let mut proposals = use_signal(|| Vec::<Entity>::new());
    let mut error_msg = use_signal(|| None::<String>);
    let mut success_msg = use_signal(|| None::<String>);
    let mut loading = use_signal(|| true);

    // View State
    let mut active_tab = use_signal(|| "schemas"); // "schemas" or "proposals"

    // Editor State
    let mut show_editor = use_signal(|| false);
    let mut editor_name = use_signal(|| String::new());
    let mut editor_desc = use_signal(|| String::new());
    let mut editor_props = use_signal(|| {
        vec![SchemaPropertyState {
            name: "".to_string(),
            data_type: "Text".to_string(),
            required: false,
        }]
    });
    let mut submit_as_proposal = use_signal(|| false); // Governance Mode Toggle
    let mut is_submitting = use_signal(|| false);

    let fetch_data = move || async move {
        loading.set(true);
        let agent = create_agent().await;

        // Fetch Schemas
        match execute_kip_query(&agent, "FIND { @type: \"$ConceptType\" }".to_string()).await {
            Ok(json_str) => match serde_json::from_str::<Vec<Entity>>(&json_str) {
                Ok(list) => schemas.set(list),
                Err(e) => error_msg.set(Some(format!("Failed to parse schemas: {}", e))),
            },
            Err(e) => error_msg.set(Some(format!("Failed to fetch schemas: {}", e))),
        }

        // Fetch Proposals
        match execute_kip_query(
            &agent,
            "FIND { @type: \"Proposal\", status: \"Pending\" }".to_string(),
        )
        .await
        {
            Ok(json_str) => match serde_json::from_str::<Vec<Entity>>(&json_str) {
                Ok(list) => proposals.set(list),
                Err(_) => {} // Ignore if none or parse fail (schema might not exist yet)
            },
            Err(_) => {} // Ignore errors for proposals for now
        }

        loading.set(false);
    };

    use_future(move || fetch_data());

    let mut handle_save = move || {
        is_submitting.set(true);
        spawn(async move {
            let name = editor_name.read().clone();
            let desc = editor_desc.read().clone();
            let is_proposal = *submit_as_proposal.read();

            if name.is_empty() {
                error_msg.set(Some("Name is required".to_string()));
                is_submitting.set(false);
                return;
            }

            // Construct Props String
            let props_str = editor_props
                .read()
                .iter()
                .filter(|p| !p.name.is_empty())
                .map(|p| {
                    let req_flag = if p.required { "required" } else { "optional" };
                    format!("\"prop:{}\": \"{},{}\"", p.name, p.data_type, req_flag)
                })
                .collect::<Vec<String>>()
                .join(", ");

            // Construct Inner Command (The Schema Definition)
            let mut schema_command = format!(
                "UPSERT {{ @type: \"$ConceptType\", name: \"{}\", description: \"{}\"",
                name, desc
            );
            if !props_str.is_empty() {
                schema_command.push_str(", ");
                schema_command.push_str(&props_str);
            }
            schema_command.push_str(" }");

            // Construct Final Mutation Command
            let final_command = if is_proposal {
                // Wrap in Proposal Entity
                // We escape inner quotes for the command string
                let escaped_command = schema_command.replace("\"", "\\\"");
                format!(
                    "UPSERT {{ @type: \"Proposal\", target: \"{}\", command: \"{}\", status: \"Pending\" }}",
                    name, escaped_command
                )
            } else {
                schema_command
            };

            let agent = create_agent().await;
            match execute_kip_mutation(&agent, final_command).await {
                Ok(_) => {
                    let type_msg = if is_proposal {
                        "Proposal submitted"
                    } else {
                        "Schema saved"
                    };
                    success_msg.set(Some(format!("{} successfully", type_msg)));
                    error_msg.set(None);
                    show_editor.set(false);
                    fetch_data().await;
                }
                Err(e) => {
                    error_msg.set(Some(format!("Failed to save: {}", e)));
                }
            }
            is_submitting.set(false);
        });
    };

    let handle_approve = move |proposal: Entity| {
        spawn(async move {
            let agent = create_agent().await;
            // Extract command from attributes.
            // Attributes: [("command", "UPSERT ..."), ("status", "Pending")...]
            let command_opt = proposal
                .attributes
                .iter()
                .find(|(k, _)| k == "command")
                .map(|(_, v)| v.clone());

            if let Some(cmd) = command_opt {
                // 1. Execute the Payload
                match execute_kip_mutation(&agent, cmd).await {
                    Ok(_) => {
                        // 2. Mark Proposal as Executed
                        let update_prop =
                            format!("UPSERT {{ id: \"{}\", status: \"Executed\" }}", proposal.id);
                        let _ = execute_kip_mutation(&agent, update_prop).await;

                        success_msg.set(Some("Proposal approved and executed!".to_string()));
                        fetch_data().await;
                    }
                    Err(e) => error_msg.set(Some(format!("Failed to execute proposal: {}", e))),
                }
            } else {
                error_msg.set(Some("Proposal has no command attribute".to_string()));
            }
        });
    };

    let open_create = move |_| {
        editor_name.set("".to_string());
        editor_desc.set("".to_string());
        editor_props.set(vec![SchemaPropertyState {
            name: "".to_string(),
            data_type: "Text".to_string(),
            required: false,
        }]);
        submit_as_proposal.set(false);
        error_msg.set(None);
        success_msg.set(None);
        show_editor.set(true);
    };

    rsx! {
        div { class: "p-6 space-y-6 animate-in fade-in duration-500",
            // Header
            div { class: "flex items-center justify-between",
                div {
                    h1 { class: "text-3xl font-bold tracking-tight", "Schema Explorer" }
                    if !show_editor() {
                        div { class: "flex gap-4 mt-2",
                            button {
                                class: format_args!("text-sm font-medium transition-colors hover:text-primary {}", if *active_tab.read() == "schemas" { "text-primary border-b-2 border-primary" } else { "text-muted-foreground" }),
                                onclick: move |_| active_tab.set("schemas"),
                                "Definitions ({schemas.len()})"
                            }
                            button {
                                class: format_args!("text-sm font-medium transition-colors hover:text-primary {}", if *active_tab.read() == "proposals" { "text-primary border-b-2 border-primary" } else { "text-muted-foreground" }),
                                onclick: move |_| active_tab.set("proposals"),
                                "Governance Proposals ({proposals.len()})"
                            }
                        }
                    }
                }
                if !show_editor() {
                    button {
                        class: "inline-flex items-center justify-center rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 bg-primary text-primary-foreground hover:bg-primary/90 h-10 px-4 py-2",
                        onclick: open_create,
                        "Create New Type"
                    }
                }
            }

            // Messages
            if let Some(err) = error_msg.read().as_ref() {
                div { class: "p-4 border border-destructive/50 bg-destructive/10 text-destructive rounded-md", "Error: {err}" }
            }
            if let Some(msg) = success_msg.read().as_ref() {
                div { class: "p-4 border border-green-500/50 bg-green-500/10 text-green-700 rounded-md", "{msg}" }
            }

            if show_editor() {
                // EDITOR FORM
                div { class: "bg-card text-card-foreground p-6 rounded-lg border shadow-sm space-y-6 max-w-2xl mx-auto",
                    div { class: "flex justify-between items-center border-b pb-4",
                        h2 { class: "text-xl font-semibold", "Define Concept Type" }
                        button { class: "text-muted-foreground hover:text-foreground", onclick: move |_| show_editor.set(false), "Cancel" }
                    }
                    div { class: "space-y-4",
                        div { class: "space-y-2",
                            label { class: "text-sm font-medium", "Concept Name (Unique ID)" }
                            input {
                                class: "flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm",
                                placeholder: "e.g. Project, Idea",
                                value: "{editor_name}",
                                oninput: move |e| editor_name.set(e.value())
                            }
                        }
                        div { class: "space-y-2",
                            label { class: "text-sm font-medium", "Description" }
                            textarea {
                                class: "flex min-h-[80px] w-full rounded-md border border-input bg-background px-3 py-2 text-sm",
                                placeholder: "Describe what this concept represents...",
                                value: "{editor_desc}",
                                oninput: move |e| editor_desc.set(e.value())
                            }
                        }
                        div { class: "space-y-2 pt-4 border-t",
                            div { class: "flex justify-between items-center",
                                label { class: "text-sm font-medium", "Properties" }
                                button {
                                    class: "text-xs bg-secondary text-secondary-foreground px-2 py-1 rounded hover:bg-secondary/80",
                                    onclick: move |_| { editor_props.write().push(SchemaPropertyState { name: "".to_string(), data_type: "Text".to_string(), required: false }); },
                                    "+ Add Property"
                                }
                            }
                            for (idx, prop) in editor_props.read().iter().enumerate() {
                                div { key: "{idx}", class: "flex gap-2 items-start p-2 bg-muted/50 rounded-md",
                                    div { class: "flex-1", input { class: "flex h-9 w-full rounded-md border border-input bg-background px-3 py-1 text-sm", value: "{prop.name}", oninput: move |e| editor_props.write()[idx].name = e.value() } }
                                    div { class: "w-[120px]", select { class: "flex h-9 w-full rounded-md border border-input bg-background px-3 py-1 text-sm", onchange: move |e| editor_props.write()[idx].data_type = e.value(),
                                        option { value: "Text", selected: prop.data_type == "Text", "Text" }
                                        option { value: "Nat", selected: prop.data_type == "Nat", "Number" }
                                        option { value: "Bool", selected: prop.data_type == "Bool", "Boolean" }
                                        option { value: "Principal", selected: prop.data_type == "Principal", "Principal" }
                                    } }
                                    div { class: "flex items-center h-9 px-2", input { type: "checkbox", checked: prop.required, onchange: move |e| editor_props.write()[idx].required = e.value() == "true" } }
                                    button { class: "h-9 w-9 flex items-center justify-center text-muted-foreground hover:text-destructive", onclick: move |_| { editor_props.write().remove(idx); }, "×" }
                                }
                            }
                        }
                        // Governance Toggle
                        div { class: "flex items-center space-x-2 pt-4 border-t",
                            input {
                                type: "checkbox",
                                id: "gov-mode",
                                checked: *submit_as_proposal.read(),
                                onchange: move |e| submit_as_proposal.set(e.value() == "true")
                            }
                            label {
                                for: "gov-mode",
                                class: "text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70",
                                "Submit as Proposal (Governance Mode)"
                            }
                        }
                    }
                    div { class: "flex justify-end gap-2 pt-4 border-t",
                        button { class: "px-4 py-2 text-sm font-medium border rounded-md hover:bg-accent", onclick: move |_| show_editor.set(false), "Cancel" }
                        button {
                            class: "px-4 py-2 text-sm font-medium bg-primary text-primary-foreground rounded-md hover:bg-primary/90 disabled:opacity-50",
                            onclick: move |_| handle_save(),
                            disabled: is_submitting(),
                            if is_submitting() { "Processing..." } else if *submit_as_proposal.read() { "Submit Proposal" } else { "Save Type Definition" }
                        }
                    }
                }

            } else {
                // LIST VIEWS
                if *loading.read() {
                    div { class: "flex items-center justify-center p-12",
                        div { class: "animate-spin rounded-full h-8 w-8 border-b-2 border-primary" }
                    }
                } else {
                    if *active_tab.read() == "schemas" {
                        div { class: "grid gap-6 md:grid-cols-2 lg:grid-cols-3",
                            for schema in schemas.read().iter() { SchemaCard { entity: schema.clone() } }
                        }
                    } else {
                        // PROPOSALS LIST
                         div { class: "space-y-4",
                            if proposals.read().is_empty() {
                                div { class: "text-center text-muted-foreground py-12", "No pending proposals" }
                            }
                            // FIXED: Clone list to avoid borrowing issues
                            for proposal in proposals.read().clone().into_iter() {
                                ProposalCard {
                                    entity: proposal.clone(),
                                    on_approve: move |_| handle_approve(proposal.clone())
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn SchemaCard(entity: Entity) -> Element {
    let properties: Vec<(String, String, bool)> = entity
        .attributes
        .iter()
        .filter(|(k, _)| k.starts_with("prop:"))
        .map(|(k, v)| {
            let name = k.trim_start_matches("prop:").to_string();
            let parts: Vec<&str> = v.split(',').collect();
            let type_ = parts.get(0).unwrap_or(&"Text").to_string();
            let required = parts
                .get(1)
                .map(|s| *s == "true" || *s == "required")
                .unwrap_or(false);
            (name, type_, required)
        })
        .collect();

    rsx! {
        div { class: "group relative rounded-lg border bg-card text-card-foreground shadow-sm hover:shadow-md transition-all duration-200",
            div { class: "p-6 space-y-4",
                div { class: "space-y-2",
                    div { class: "flex items-center justify-between",
                        h3 { class: "font-semibold leading-none tracking-tight text-lg", "{entity.name}" }
                        div { class: "px-2.5 py-0.5 rounded-full text-xs font-medium bg-primary/10 text-primary", "Model" }
                    }
                    p { class: "text-sm text-muted-foreground line-clamp-2 min-h-[2.5rem]", "{entity.description}" }
                }
                div { class: "pt-4 border-t",
                    h4 { class: "text-xs font-semibold uppercase tracking-wider text-muted-foreground mb-3", "Properties" }
                    div { class: "space-y-2",
                        if properties.is_empty() { div { class: "text-sm text-muted-foreground italic", "No properties defined" } }
                        for (name, type_, req) in properties {
                            div { class: "flex items-center justify-between text-sm",
                                span { class: "font-medium font-mono text-xs", "{name}" }
                                div { class: "flex items-center gap-2", span { class: "text-muted-foreground text-xs", "{type_}" } if req { span { class: "text-[10px] uppercase font-bold text-destructive", "Req" } } }
                            }
                        }
                    }
                }
            }
            div { class: "px-6 py-3 bg-muted/30 border-t rounded-b-lg flex justify-between items-center",
                code { class: "text-[10px] text-muted-foreground font-mono", "{entity.id}" }
            }
        }
    }
}

#[component]
fn ProposalCard(entity: Entity, on_approve: EventHandler<()>) -> Element {
    let target = entity
        .attributes
        .iter()
        .find(|(k, _)| k == "target")
        .map(|(_, v)| v)
        .unwrap_or(&"Unknown".to_string())
        .clone();
    let command = entity
        .attributes
        .iter()
        .find(|(k, _)| k == "command")
        .map(|(_, v)| v)
        .unwrap_or(&"Unknown".to_string())
        .clone();
    let proposer = entity
        .attributes
        .iter()
        .find(|(k, _)| k == "proposer")
        .map(|(_, v)| v)
        .unwrap_or(&"Anonymous".to_string())
        .clone();

    rsx! {
        div { class: "rounded-lg border bg-card text-card-foreground shadow-sm p-6 space-y-4",
            div { class: "flex justify-between items-start",
                div {
                    h3 { class: "font-semibold text-lg", "Proposal: Update {target}" }
                    p { class: "text-sm text-muted-foreground", "Proposed by {proposer}" }
                }
                div { class: "px-2.5 py-0.5 rounded-full text-xs font-medium bg-yellow-100 text-yellow-800", "Pending" }
            }
            div { class: "bg-muted p-4 rounded-md font-mono text-xs overflow-x-auto whitespace-pre-wrap",
                "{command}"
            }
            div { class: "flex justify-end gap-2 pt-2",
                 button {
                    class: "px-3 py-1.5 text-sm font-medium border border-destructive text-destructive rounded-md hover:bg-destructive/10",
                    "Reject"
                }
                button {
                    class: "px-3 py-1.5 text-sm font-medium bg-green-600 text-white rounded-md hover:bg-green-700",
                    onclick: move |_| on_approve.call(()),
                    "Approve & Execute"
                }
            }
        }
    }
}
