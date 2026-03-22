use crate::CortexRenderer;
use crate::components::*;
use crate::elements::*;
use dioxus::prelude::*;

pub fn render_node(renderer: &CortexRenderer, node_id: &str) -> Element {
    let node = match renderer.components.get(node_id) {
        Some(n) => n,
        None => return rsx! { "Missing Node: {node_id}" },
    };

    match &node.component {
        ComponentWrapper::Text(props) => {
            let content = resolve_string(&props.text);
            let tag = props.usage_hint.as_deref().unwrap_or("div");
            match tag {
                "h1" => rsx! { h1 { "{content}" } },
                "h2" => rsx! { h2 { "{content}" } },
                "h3" => rsx! { h3 { "{content}" } },
                "p" => rsx! { p { "{content}" } },
                "span" => rsx! { span { "{content}" } },
                _ => rsx! { div { "{content}" } },
            }
        }
        ComponentWrapper::Button(props) => {
            let child = render_node(renderer, &props.child);
            let action_name = props.action.name.clone();
            let variant = if props.primary.unwrap_or(false) {
                "primary"
            } else {
                "default"
            };

            rsx! {
                SlButton {
                    variant: variant,
                    onclick: move |_| {
                        tracing::info!("Action triggered: {}", action_name);
                    },
                    {child}
                }
            }
        }
        ComponentWrapper::Card(props) => {
            let child = render_node(renderer, &props.child);
            rsx! {
                SlCard {
                    {child}
                }
            }
        }
        ComponentWrapper::Image(props) => {
            let url = resolve_string(&props.url);
            let alt = props
                .alt_text
                .as_ref()
                .map(resolve_string)
                .unwrap_or_default();
            rsx! {
                img {
                    src: "{url}",
                    alt: "{alt}",
                    style: "max-width: 100%; height: auto;"
                }
            }
        }
        ComponentWrapper::Icon(props) => {
            let name = resolve_string(&props.name);
            rsx! {
                SlIcon {
                    name: name
                }
            }
        }
        ComponentWrapper::TextField(props) => {
            let label = resolve_string(&props.label);
            let value = props.text.as_ref().map(resolve_string).unwrap_or_default();
            rsx! {
                SlInput {
                    label: label,
                    value: value,
                    oninput: move |evt: FormEvent| {
                        tracing::info!("Input changed: {:?}", evt.value());
                    }
                }
            }
        }
        ComponentWrapper::Row(props) => match &props.children {
            ChildrenDefinition::ExplicitList(ids) => {
                rsx! {
                    div {
                        style: "display: flex; flex-direction: row; gap: 1rem; align-items: center;",
                        for id in ids {
                            {render_node(renderer, id)}
                        }
                    }
                }
            }
            ChildrenDefinition::Template(_) => rsx! { "Templates not supported yet" },
        },
        ComponentWrapper::Column(props) => match &props.children {
            ChildrenDefinition::ExplicitList(ids) => {
                rsx! {
                    div {
                        style: "display: flex; flex-direction: column; gap: 1rem;",
                        for id in ids {
                            {render_node(renderer, id)}
                        }
                    }
                }
            }
            ChildrenDefinition::Template(_) => rsx! { "Templates not supported yet" },
        },
        ComponentWrapper::Divider(props) => {
            let axis = props.axis.as_deref().unwrap_or("horizontal");
            if axis == "vertical" {
                rsx! { sl-divider { "vertical": "true" } }
            } else {
                rsx! { sl-divider {} }
            }
        }
        ComponentWrapper::Slider(props) => {
            let value = match props.value {
                PropertyValue::Literal(ref l) => l.value,
                PropertyValue::Path(_) => 0.0,
            };
            rsx! {
                sl-range {
                    "label": "Slider",
                    "value": "{value}",
                    "min": "0",
                    "max": "100"
                }
            }
        }
        _ => rsx! {
            div { "Unsupported Component Type" }
        },
    }
}

fn resolve_string(prop: &PropertyValue<String>) -> String {
    match prop {
        PropertyValue::Literal(l) => l.value.clone(),
        PropertyValue::Path(p) => format!("${{{}}}", p.path),
    }
}
