use crate::components::icons::{Icon, IconName};
use crate::types::*;
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct ChatBubbleProps {
    message_type: ChatMessageType,
    content: String,
    // Add callback for dangerous HTML rendering if needed, or pass rendered content
    // For now, we assume raw content and handle simple text
}

// Note: Complex markdown rendering logic from main.rs might need to be passed in or duplicated
// For this refactor, we focus on the container structure.

#[component]
pub fn ChatBubble(props: ChatBubbleProps) -> Element {
    let msg_type = props.message_type;
    let content = props.content;

    let is_user = matches!(msg_type, ChatMessageType::User);
    let align_class = if is_user {
        "justify-end"
    } else {
        "justify-start"
    };

    let bubble_class = format!(
        "p-4 rounded-2xl text-sm shadow-sm max-w-[80%] leading-relaxed {}",
        if is_user {
            "bg-primary text-primary-foreground rounded-br-none"
        } else {
            "bg-muted/50 border border-border/50 text-foreground rounded-bl-none"
        }
    );

    rsx! {
        div {
            class: format!("flex w-full gap-4 max-w-3xl mx-auto {}", align_class),

            // AI Avatar
            if !is_user {
                div { class: "w-8 h-8 rounded-lg bg-primary/10 flex items-center justify-center shrink-0 mt-1",
                    Icon { name: IconName::Bot, class: "w-5 h-5 text-primary" }
                }
            }

            div { class: "{bubble_class}",
                 "{content}"
            }

            // User Avatar
            if is_user {
                div { class: "w-8 h-8 rounded-lg bg-muted flex items-center justify-center shrink-0 mt-1",
                    Icon { name: IconName::User, class: "w-5 h-5 text-muted-foreground" }
                }
            }
        }
    }
}
