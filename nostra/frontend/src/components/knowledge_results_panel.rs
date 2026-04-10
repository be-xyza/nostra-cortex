use crate::types::{KnowledgeAskResponse, KnowledgeSearchResult};
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct KnowledgeResultsPanelProps {
    pub title: String,
    #[props(default)]
    pub subtitle: Option<String>,
    #[props(default)]
    pub feedback: Option<String>,
    #[props(default)]
    pub error: Option<String>,
    #[props(default)]
    pub is_loading: bool,
    #[props(default)]
    pub show_diagnostics: bool,
    #[props(default)]
    pub results: Vec<KnowledgeSearchResult>,
    #[props(default)]
    pub ask_response: Option<KnowledgeAskResponse>,
}

#[component]
pub fn KnowledgeResultsPanel(props: KnowledgeResultsPanelProps) -> Element {
    rsx! {
        div { class: "rounded-lg border border-border bg-card p-4 space-y-3",
            div {
                h3 { class: "text-sm font-semibold tracking-wide", "{props.title}" }
                if let Some(subtitle) = &props.subtitle {
                    p { class: "text-xs text-muted-foreground", "{subtitle}" }
                }
            }

            if let Some(error) = &props.error {
                div { class: "rounded border border-destructive/60 bg-destructive/10 p-2 text-xs text-destructive",
                    "{error}"
                }
            } else if let Some(feedback) = &props.feedback {
                div { class: "text-xs text-muted-foreground", "{feedback}" }
            }

            if props.is_loading {
                div { class: "text-xs text-muted-foreground", "Loading..." }
            }

            if !props.results.is_empty() {
                div { class: "space-y-2 max-h-72 overflow-y-auto pr-1",
                    for item in props.results.iter() {
                        div { class: "rounded border border-border/60 bg-muted/20 p-3 space-y-1",
                            div { class: "flex items-center justify-between gap-2",
                                code { class: "text-[11px] text-primary truncate", "{item.id}" }
                                span { class: "text-[11px] text-muted-foreground", "score={item.score:.4}" }
                            }
                            if let Some(source) = &item.source_ref {
                                div { class: "text-[11px] text-muted-foreground", "source={source}" }
                            }
                            if let Some(space) = &item.space_id {
                                div { class: "text-[11px] text-muted-foreground", "space={space}" }
                            }
                            if let Some(modality) = &item.modality {
                                div { class: "text-[11px] text-muted-foreground", "modality={modality}" }
                            }
                            if let Some(content) = &item.content {
                                p { class: "text-xs text-foreground/90", "{content}" }
                            }
                            if let Some(prov) = &item.provenance {
                                div { class: "text-[11px] text-muted-foreground font-mono",
                                    "by={prov.author} at={prov.timestamp}"
                                }
                            }
                            if props.show_diagnostics {
                                if let Some(diag) = &item.diagnostic {
                                    div { class: "text-[11px] text-amber-700 dark:text-amber-300 font-mono",
                                        "v={diag.vector_score:.4} l={diag.lexical_score:.4} f={diag.fused_score:.4} ({diag.rank_reason}) [{diag.backend}|{diag.embedding_model}]"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if let Some(answer) = &props.ask_response {
                div { class: "rounded border border-border/70 bg-muted/30 p-3 space-y-2",
                    div { class: "text-xs text-muted-foreground font-mono",
                        "trace={answer.trace_id} model={answer.model}"
                    }
                    if let Some(surface) = &answer.ui_surface {
                        div { class: "text-xs text-muted-foreground font-mono", "surface={surface}" }
                    }
                    p { class: "text-sm whitespace-pre-wrap", "{answer.answer}" }
                    div { class: "space-y-1",
                        div { class: "text-xs font-semibold", "Citations" }
                        for citation in answer.citations.iter() {
                            div { class: "rounded border border-border/50 bg-background p-2",
                                div { class: "text-[11px] text-primary font-mono",
                                    "{citation.id} (score={citation.score:.4})"
                                }
                                if let Some(source) = &citation.source_ref {
                                    div { class: "text-[11px] text-muted-foreground", "{source}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
