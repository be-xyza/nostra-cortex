use crate::components::d3_graph::{D3GraphComponent, GraphConfig};
use crate::components::popup::Popup;
use crate::components::popup_config::{ActionConfig, ActionStyle, PopupConfig, PopupSize};
use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
enum ComponentCategory {
    Primitives,
    Typography,
    Inputs,
    Display,
    Feedback,
    GraphVisualizer,
    Popups,
}

#[component]
pub fn ComponentsLab(on_back: EventHandler<()>) -> Element {
    let mut selected_category = use_signal(|| ComponentCategory::GraphVisualizer);

    rsx! {
        div { class: "flex h-full w-full bg-background text-foreground",
            // Sidebar
            div { class: "w-64 border-r bg-muted/20 flex flex-col",
                div { class: "p-4 border-b flex items-center justify-between",
                    h2 { class: "font-semibold", "Components Lab" }
                    button {
                        class: "p-1 hover:bg-muted rounded-md transition-colors",
                        onclick: move |_| on_back.call(()),
                        svg { class: "w-4 h-4", fill: "none", stroke: "currentColor", view_box: "0 0 24 24",
                            path { stroke_linecap: "round", stroke_linejoin: "round", stroke_width: "2", d: "M6 18L18 6M6 6l12 12" }
                        }
                    }
                }
                div { class: "flex-1 overflow-y-auto p-2 space-y-1",
                    NavButton {
                        active: selected_category() == ComponentCategory::Primitives,
                        label: "Primitives",
                        onclick: move |_| selected_category.set(ComponentCategory::Primitives)
                    }
                    NavButton {
                        active: selected_category() == ComponentCategory::Typography,
                        label: "Typography",
                        onclick: move |_| selected_category.set(ComponentCategory::Typography)
                    }
                    NavButton {
                        active: selected_category() == ComponentCategory::Inputs,
                        label: "Inputs",
                        onclick: move |_| selected_category.set(ComponentCategory::Inputs)
                    }
                    NavButton {
                        active: selected_category() == ComponentCategory::Display,
                        label: "Display",
                        onclick: move |_| selected_category.set(ComponentCategory::Display)
                    }
                    NavButton {
                        active: selected_category() == ComponentCategory::Feedback,
                        label: "Feedback",
                        onclick: move |_| selected_category.set(ComponentCategory::Feedback)
                    }
                    NavButton {
                        active: selected_category() == ComponentCategory::GraphVisualizer,
                        label: "Graph Visualization",
                        onclick: move |_| selected_category.set(ComponentCategory::GraphVisualizer)
                    }
                    NavButton {
                        active: selected_category() == ComponentCategory::Popups,
                        label: "Popups",
                        onclick: move |_| selected_category.set(ComponentCategory::Popups)
                    }
                }
            }

            // Main Content
            div { class: "flex-1 overflow-y-auto bg-card/50",
                div { class: "container mx-auto max-w-4xl p-8 space-y-12",
                    match selected_category() {
                        ComponentCategory::Primitives => rsx! { PrimitivesView {} },
                        ComponentCategory::Typography => rsx! { TypographyView {} },
                        ComponentCategory::Inputs => rsx! { InputsView {} },
                        ComponentCategory::Display => rsx! { DisplayView {} },
                        ComponentCategory::Feedback => rsx! { FeedbackView {} },
                        ComponentCategory::GraphVisualizer => rsx! { GraphVisualizerView {} },
                        ComponentCategory::Popups => rsx! { PopupsView {} },
                    }
                }
            }
        }
    }
}

#[component]
fn NavButton(active: bool, label: String, onclick: EventHandler<MouseEvent>) -> Element {
    let base_classes =
        "w-full text-left px-3 py-2 rounded-md text-sm font-medium transition-colors";
    let active_classes = if active {
        "bg-primary/10 text-primary"
    } else {
        "text-muted-foreground hover:bg-muted hover:text-foreground"
    };

    rsx! {
        button {
            class: "{base_classes} {active_classes}",
            onclick: move |evt| onclick.call(evt),
            "{label}"
        }
    }
}

#[component]
fn Section(title: String, children: Element) -> Element {
    rsx! {
        div { class: "space-y-4",
            h3 { class: "text-lg font-semibold border-b pb-2", "{title}" }
            div { class: "p-6 border rounded-lg bg-background/50 space-y-8",
                {children}
            }
        }
    }
}

#[component]
fn PrimitivesView() -> Element {
    rsx! {
        div { class: "space-y-10",
            div {
                h1 { class: "text-3xl font-bold mb-4", "Primitives" }
                p { class: "text-muted-foreground", "Core design tokens including colors, spacing, and radius." }
            }

            Section { title: "Colors: Background & Foreground".to_string(),
                div { class: "grid grid-cols-2 md:grid-cols-4 gap-4",
                    ColorSwatch { name: "Background", class: "bg-background border", text_class: "text-foreground" }
                    ColorSwatch { name: "Foreground", class: "bg-foreground", text_class: "text-background" }
                    ColorSwatch { name: "Card", class: "bg-card border", text_class: "text-card-foreground" }
                    ColorSwatch { name: "Muted", class: "bg-muted", text_class: "text-muted-foreground" }
                    ColorSwatch { name: "Accent", class: "bg-accent", text_class: "text-accent-foreground" }
                }
            }

            Section { title: "Colors: Primary & Secondary".to_string(),
                div { class: "grid grid-cols-2 md:grid-cols-4 gap-4",
                    ColorSwatch { name: "Primary", class: "bg-primary", text_class: "text-primary-foreground" }
                    ColorSwatch { name: "Secondary", class: "bg-secondary", text_class: "text-secondary-foreground" }
                    ColorSwatch { name: "Destructive", class: "bg-destructive", text_class: "text-destructive-foreground" }
                    ColorSwatch { name: "Border", class: "bg-border", text_class: "text-foreground" }
                }
            }
        }
    }
}

#[component]
fn ColorSwatch(name: &'static str, class: &'static str, text_class: &'static str) -> Element {
    rsx! {
        div { class: "space-y-2",
            div { class: "h-20 rounded-md shadow-sm flex items-center justify-center {class}",
                span { class: "text-xs font-bold {text_class}", "Aa" }
            }
            div {
                p { class: "text-sm font-medium", "{name}" }
                code { class: "text-xs text-muted-foreground", "{class}" }
            }
        }
    }
}

#[component]
fn TypographyView() -> Element {
    rsx! {
        div { class: "space-y-10",
            div {
                h1 { class: "text-3xl font-bold mb-4", "Typography" }
                p { class: "text-muted-foreground", "Font styles, weights, and hierarchy." }
            }

            Section { title: "Headings".to_string(),
                div { class: "space-y-4",
                    div { class: "space-y-1",
                        h1 { class: "text-4xl font-extrabold tracking-tight lg:text-5xl", "Heading 1" }
                        p { class: "text-xs text-muted-foreground font-mono", "text-4xl font-extrabold tracking-tight lg:text-5xl" }
                    }
                    div { class: "space-y-1",
                        h2 { class: "text-3xl font-semibold tracking-tight", "Heading 2" }
                        p { class: "text-xs text-muted-foreground font-mono", "text-3xl font-semibold tracking-tight" }
                    }
                    div { class: "space-y-1",
                        h3 { class: "text-2xl font-semibold tracking-tight", "Heading 3" }
                        p { class: "text-xs text-muted-foreground font-mono", "text-2xl font-semibold tracking-tight" }
                    }
                    div { class: "space-y-1",
                        h4 { class: "text-xl font-semibold tracking-tight", "Heading 4" }
                        p { class: "text-xs text-muted-foreground font-mono", "text-xl font-semibold tracking-tight" }
                    }
                }
            }

            Section { title: "Body Text".to_string(),
                div { class: "space-y-6 max-w-2xl",
                    div {
                        p { class: "leading-7 [&:not(:first-child)]:mt-6",
                            "The king, seeing how much simpler it was to implement components, declared: "
                            em { "\"Clean code is not just a practice, it is a virtue.\"" }
                        }
                        p { class: "leading-7 [&:not(:first-child)]:mt-6",
                            "This is a standard paragraph with leading-7. It provides comfortable reading density for long-form content."
                        }
                    }
                    div {
                        p { class: "text-sm text-muted-foreground", "This is muted small text, often used for captions or help text." }
                    }
                }
            }
        }
    }
}

#[component]
fn InputsView() -> Element {
    rsx! {
        div { class: "space-y-10",
            div {
                h1 { class: "text-3xl font-bold mb-4", "Inputs" }
                p { class: "text-muted-foreground", "Interactive elements for user input." }
            }

            Section { title: "Buttons".to_string(),
                div { class: "flex flex-wrap gap-4 items-center",
                    button { class: "inline-flex items-center justify-center rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 bg-primary text-primary-foreground hover:bg-primary/90 h-10 px-4 py-2",
                        "Primary Button"
                    }
                    button { class: "inline-flex items-center justify-center rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 bg-secondary text-secondary-foreground hover:bg-secondary/80 h-10 px-4 py-2",
                        "Secondary Button"
                    }
                    button { class: "inline-flex items-center justify-center rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 border border-input bg-background hover:bg-accent hover:text-accent-foreground h-10 px-4 py-2",
                        "Outline Button"
                    }
                    button { class: "inline-flex items-center justify-center rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 hover:bg-accent hover:text-accent-foreground h-10 px-4 py-2",
                        "Ghost Button"
                    }
                }
            }

            Section { title: "Sizes".to_string(),
                div { class: "flex flex-wrap gap-4 items-center",
                    button { class: "inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors bg-primary text-primary-foreground hover:bg-primary/90 h-8 px-3 text-xs",
                        "Small (h-8)"
                    }
                    button { class: "inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors bg-primary text-primary-foreground hover:bg-primary/90 h-10 px-4 py-2",
                        "Default (h-10)"
                    }
                    button { class: "inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors bg-primary text-primary-foreground hover:bg-primary/90 h-11 px-8 rounded-md",
                        "Large (h-11)"
                    }
                }
            }
        }
    }
}

#[component]
fn DisplayView() -> Element {
    rsx! {
        div { class: "space-y-10",
            div {
                h1 { class: "text-3xl font-bold mb-4", "Display" }
                p { class: "text-muted-foreground", "Components for displaying data and content." }
            }

            Section { title: "Cards".to_string(),
                div { class: "grid gap-6 md:grid-cols-2",
                    // Standard Card
                    div { class: "rounded-xl border bg-card text-card-foreground shadow",
                        div { class: "flex flex-col space-y-1.5 p-6",
                            h3 { class: "font-semibold leading-none tracking-tight", "Card Title" }
                            p { class: "text-sm text-muted-foreground", "Card Description" }
                        }
                        div { class: "p-6 pt-0",
                            "Card Content Area. This is where the details go."
                        }
                        div { class: "flex items-center p-6 pt-0",
                            button { class: "inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors border border-input bg-background hover:bg-accent hover:text-accent-foreground h-9 px-4 py-2",
                                "Action"
                            }
                        }
                    }
                    // Interactive Card
                    div { class: "rounded-xl border bg-card text-card-foreground shadow hover:shadow-md transition-shadow cursor-pointer",
                        div { class: "flex flex-col space-y-1.5 p-6",
                            h3 { class: "font-semibold leading-none tracking-tight", "Interactive Card" }
                            p { class: "text-sm text-muted-foreground", "Hover me to see the effect." }
                        }
                        div { class: "p-6 pt-0",
                            "Uses hover:shadow-md and cursor-pointer."
                        }
                    }
                }
            }

            Section { title: "Badges".to_string(),
                div { class: "flex flex-wrap gap-2",
                    span { class: "inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 border-transparent bg-primary text-primary-foreground shadow hover:bg-primary/80",
                        "Primary"
                    }
                    span { class: "inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 border-transparent bg-secondary text-secondary-foreground hover:bg-secondary/80",
                        "Secondary"
                    }
                    span { class: "inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 border-transparent bg-destructive text-destructive-foreground shadow hover:bg-destructive/80",
                        "Destructive"
                    }
                    span { class: "inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 text-foreground",
                        "Outline"
                    }
                }
            }
        }
    }
}

#[component]
fn FeedbackView() -> Element {
    rsx! {
        div { class: "space-y-10",
            div {
                h1 { class: "text-3xl font-bold mb-4", "Feedback" }
                p { class: "text-muted-foreground", "Alerts, Spinners, and Loading states." }
            }

            Section { title: "Loading States".to_string(),
                div { class: "flex items-center gap-4",
                     div { class: "flex items-center justify-center p-4 border rounded-md",
                        div { class: "animate-spin rounded-full h-8 w-8 border-b-2 border-primary" }
                     }
                     div { class: "space-y-2 w-full max-w-xs",
                         div { class: "h-4 bg-muted animate-pulse rounded w-3/4" }
                         div { class: "h-4 bg-muted animate-pulse rounded" }
                         div { class: "h-4 bg-muted animate-pulse rounded w-5/6" }
                     }
                }
            }
        }
    }
}

#[component]
fn GraphVisualizerView() -> Element {
    let mut current_preset = use_signal(|| "Cosmic".to_string());
    let mut current_config = use_signal(GraphConfig::preset_cosmic);

    let mut set_preset = move |name: &str, config: GraphConfig| {
        current_preset.set(name.to_string());
        current_config.set(config);
    };

    rsx! {
        div { class: "h-full flex flex-col space-y-4",
            div { class: "flex items-center justify-between",
                div {
                    h1 { class: "text-3xl font-bold mb-1", "Graph Visualization" }
                    p { class: "text-muted-foreground", "Configure and test D3 force graph physics and aesthetics." }
                }
                div { class: "flex items-center gap-2 bg-muted/50 p-1 rounded-lg border",
                    button {
                        class: format!("px-3 py-1.5 text-sm font-medium rounded-md transition-colors {}", if current_preset() == "Default" { "bg-background text-foreground shadow-sm" } else { "text-muted-foreground hover:text-foreground" }),
                        onclick: move |_| set_preset("Default", GraphConfig::preset_default()),
                        "Default"
                    }
                    button {
                        class: format!("px-3 py-1.5 text-sm font-medium rounded-md transition-colors {}", if current_preset() == "Cosmic" { "bg-background text-foreground shadow-sm" } else { "text-muted-foreground hover:text-foreground" }),
                        onclick: move |_| set_preset("Cosmic", GraphConfig::preset_cosmic()),
                        "Cosmic"
                    }
                    button {
                        class: format!("px-3 py-1.5 text-sm font-medium rounded-md transition-colors {}", if current_preset() == "Dense" { "bg-background text-foreground shadow-sm" } else { "text-muted-foreground hover:text-foreground" }),
                        onclick: move |_| set_preset("Dense", GraphConfig::preset_dense()),
                        "Dense"
                    }
                }
            }

            div { class: "grid grid-cols-1 lg:grid-cols-3 gap-6 h-[600px]",
                // Configuration Panel
                div { class: "space-y-6 overflow-y-auto p-4 border rounded-lg bg-card/50",
                    div { class: "space-y-4",
                        h3 { class: "font-semibold border-b pb-2", "Current Configuration" }

                        div { class: "grid grid-cols-2 gap-4 text-sm",
                            div { class: "space-y-1",
                                label { class: "text-muted-foreground", "Preset" }
                                div { class: "font-mono font-medium", "{current_preset}" }
                            }
                            div { class: "space-y-1",
                                label { class: "text-muted-foreground", "Version" }
                                div { class: "font-mono", "{current_config.read().version}" }
                            }
                        }

                        div { class: "space-y-2",
                            h4 { class: "text-xs font-semibold uppercase tracking-wider text-muted-foreground", "Forces" }
                            div { class: "grid grid-cols-2 gap-2 text-sm",
                                div { "Charge Strength" }
                                div { class: "font-mono text-right", "{current_config.read().charge.strength}" }
                                div { "Link Distance" }
                                div { class: "font-mono text-right", "{current_config.read().link.distance}" }
                                div { "Link Strength" }
                                div { class: "font-mono text-right", "{current_config.read().link.strength}" }
                                div { "Collision Radius" }
                                div { class: "font-mono text-right", "{current_config.read().collision.radius}" }
                            }
                        }

                         div { class: "space-y-2",
                            h4 { class: "text-xs font-semibold uppercase tracking-wider text-muted-foreground", "Visuals" }
                            div { class: "grid grid-cols-2 gap-2 text-sm",
                                div { "Glow Effect" }
                                div { class: "font-mono text-right", "{current_config.read().visual.glow.enabled}" }
                                div { "Label Mode" }
                                div { class: "font-mono text-right", "{current_config.read().visual.labels.mode}" }
                            }
                        }
                    }
                }

                // Preview Area
                div { class: "lg:col-span-2 border rounded-lg overflow-hidden bg-black relative",
                    // Use the D3GraphComponent here
                    // Note: We need a way to mock data for the lab view.
                    // Since the component just sets config, and the graph logic pulls from window.graphData,
                    // we might need to inject some sample data if none exists.
                    // For now, it will likely be empty unless we are connected to the backend.
                    D3GraphComponent {
                        container_id: "component-lab-graph".to_string(),
                        config: current_config.read().clone(),
                        class: "w-full h-full"
                    }

                    // Overlay instruction
                    div { class: "absolute bottom-4 right-4 text-xs text-white/50 bg-black/50 px-2 py-1 rounded pointer-events-none",
                        "Preview uses live graph data if available"
                    }
                }
            }
        }
    }
}

#[component]
fn PopupsView() -> Element {
    let mut current_preset = use_signal(|| "Default".to_string());
    let mut current_config = use_signal(PopupConfig::default);
    let mut show_popup = use_signal(|| false);
    let mut last_action = use_signal(|| "None".to_string());

    let mut set_preset = move |name: &str, config: PopupConfig| {
        current_preset.set(name.to_string());
        current_config.set(config);
        show_popup.set(true);
        last_action.set("None".to_string());
    };

    rsx! {
        div { class: "h-full flex flex-col space-y-4",
            div { class: "flex items-center justify-between",
                div {
                    h1 { class: "text-3xl font-bold mb-1", "Popups" }
                    p { class: "text-muted-foreground", "Standardized popup dialogs with configuration presets." }
                }
                 div { class: "flex items-center gap-2 bg-muted/50 p-1 rounded-lg border",
                    button {
                        class: format!("px-3 py-1.5 text-sm font-medium rounded-md transition-colors {}", if current_preset() == "Default" { "bg-background text-foreground shadow-sm" } else { "text-muted-foreground hover:text-foreground" }),
                        onclick: move |_| set_preset("Default", PopupConfig::default()),
                        "Default"
                    }
                     button {
                        class: format!("px-3 py-1.5 text-sm font-medium rounded-md transition-colors {}", if current_preset() == "Welcome" { "bg-background text-foreground shadow-sm" } else { "text-muted-foreground hover:text-foreground" }),
                        onclick: move |_| set_preset("Welcome", PopupConfig {
                            version: "1.0.0".to_string(),
                            id: "welcome-popup".to_string(),
                            title: "Welcome to Nostra".to_string(),
                            body_markdown: "This is a **bold** welcome message.".to_string(),
                             primary_action: Some(ActionConfig {
                                label: "Get Started".to_string(),
                                action_id: "start".to_string(),
                                style: ActionStyle::Primary,
                            }),
                            secondary_action: None,
                            dismissible: true,
                            size: PopupSize::Medium,
                        }),
                        "Welcome"
                    }
                }
            }

            div { class: "grid grid-cols-1 lg:grid-cols-3 gap-6 h-[600px]",
                 // Configuration Panel
                 div { class: "space-y-6 overflow-y-auto p-4 border rounded-lg bg-card/50",
                    div { class: "space-y-4",
                         h3 { class: "font-semibold border-b pb-2", "Current State" }
                          div { class: "grid grid-cols-2 gap-4 text-sm",
                            div { class: "space-y-1",
                                label { class: "text-muted-foreground", "Active?" }
                                div { class: "font-mono font-medium", "{show_popup}" }
                            }
                             div { class: "space-y-1",
                                label { class: "text-muted-foreground", "Last Action" }
                                div { class: "font-mono font-medium", "{last_action}" }
                            }
                        }
                    }
                     div { class: "space-y-2",
                        button {
                             class: "w-full px-4 py-2 text-sm font-medium bg-primary text-primary-foreground hover:bg-primary/90 rounded-md",
                             onclick: move |_| show_popup.set(true),
                             "Open Popup"
                        }
                    }
                }
                 // Preview Area
                div { class: "lg:col-span-2 border rounded-lg bg-muted/20 relative flex items-center justify-center p-8",
                     if show_popup() {
                        Popup {
                            config: current_config.read().clone(),
                            on_close: move |_| show_popup.set(false),
                            on_action: move |action_id: String| {
                                last_action.set(action_id);
                                show_popup.set(false);
                            }
                        }
                    } else {
                        div { class: "text-muted-foreground", "Popup is closed. Click 'Open Popup' to view." }
                    }
                }
            }
        }
    }
}
