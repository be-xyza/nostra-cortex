use crate::components::icons::{Icon, IconName};
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct MetricCardProps {
    title: String,
    value: String,
    subtitle: String,
    icon: IconName,
}

#[component]
pub fn MetricCard(props: MetricCardProps) -> Element {
    rsx! {
        div { class: "p-6 rounded-xl border bg-card text-card-foreground shadow-sm",
            div { class: "flex flex-row items-center justify-between space-y-0 pb-2",
                h3 { class: "tracking-tight text-sm font-medium", "{props.title}" }
                Icon { name: props.icon, class: "h-4 w-4 text-muted-foreground" }
            }
            div { class: "text-2xl font-bold", "{props.value}" }
            p { class: "text-xs text-muted-foreground", "{props.subtitle}" }
        }
    }
}
