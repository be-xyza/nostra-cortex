use crate::v2::types::InstitutionSummary;
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct InstitutionsPageProps {
    pub institutions: Vec<InstitutionSummary>,
    pub on_back: EventHandler<()>,
}

#[component]
pub fn InstitutionsPage(props: InstitutionsPageProps) -> Element {
    rsx! {
        section {
            class: "max-w-4xl space-y-4",
            div { class: "flex items-center justify-between",
                h2 { class: "text-lg font-medium", "Institutions" }
                button {
                    class: "px-3 py-2 rounded-md border border-zinc-700 text-sm text-zinc-200",
                    onclick: move |_| props.on_back.call(()),
                    "Back"
                }
            }
            div { class: "grid gap-3",
                for institution in props.institutions.iter() {
                    article {
                        key: "{institution.id}",
                        class: "rounded-lg border border-zinc-800 bg-zinc-900/70 p-4",
                        h3 { class: "font-medium", "{institution.name}" }
                        p { class: "text-sm text-zinc-400", "Domain: {institution.stewardship_domain}" }
                        p { class: "text-xs text-zinc-500 uppercase tracking-wide mt-1", "Status: {institution.status}" }
                    }
                }
            }
        }
    }
}
