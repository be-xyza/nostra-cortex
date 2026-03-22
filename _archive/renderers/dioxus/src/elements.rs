use dioxus::prelude::*;

// Shoelace Element Wrappers with Governed Motion
// These wrappers use the CSS variables injected by the CortexRenderer root.

// --- Button ---

#[derive(Props, Clone, PartialEq)]
pub struct SlButtonProps {
    #[props(default)]
    pub variant: String,
    #[props(default)]
    pub size: String,
    #[props(default)]
    pub outline: bool,
    #[props(default)]
    pub disabled: bool,
    #[props(default)]
    pub loading: bool,
    pub children: Element,
    #[props(optional)]
    pub onclick: Option<EventHandler<MouseEvent>>,
}

#[allow(non_snake_case)]
pub fn SlButton(props: SlButtonProps) -> Element {
    let variant = props.variant.clone();
    let size = props.size.clone();
    let outline = props.outline.to_string();
    let disabled = props.disabled.to_string();
    let loading = props.loading.to_string();

    rsx! {
        sl-button {
            "variant": "{variant}",
            "size": "{size}",
            "outline": "{outline}",
            "disabled": "{disabled}",
            "loading": "{loading}",
            style: "transition: all var(--ctx-motion-duration-fast) var(--ctx-motion-easing-ease-in-out);",
            onclick: move |evt| if let Some(handler) = &props.onclick {
                handler.call(evt);
            },
            {props.children}
        }
    }
}

// --- Card ---

#[derive(Props, Clone, PartialEq)]
pub struct SlCardProps {
    pub children: Element,
    #[props(default)]
    pub class: String,
    #[props(optional)]
    pub image: Option<Element>,
    #[props(optional)]
    pub header: Option<Element>,
    #[props(optional)]
    pub footer: Option<Element>,
}

#[allow(non_snake_case)]
pub fn SlCard(props: SlCardProps) -> Element {
    let class = props.class.clone();
    rsx! {
        sl-card {
            "class": "{class}",
            style: "transition: transform var(--ctx-motion-duration-normal) var(--ctx-motion-easing-ease-out), opacity var(--ctx-motion-duration-normal) var(--ctx-motion-easing-ease-out);",
            if let Some(img) = props.image {
                div { "slot": "image", {img} }
            }
            if let Some(hdr) = props.header {
                div { "slot": "header", {hdr} }
            }
            {props.children}
            if let Some(ftr) = props.footer {
                div { "slot": "footer", {ftr} }
            }
        }
    }
}

// --- Input ---

#[derive(Props, Clone, PartialEq)]
pub struct SlInputProps {
    pub label: String,
    pub value: String,
    #[props(default)]
    pub placeholder: String,
    #[props(default)]
    pub clearable: bool,
    #[props(default)]
    pub disabled: bool,
    #[props(optional)]
    pub oninput: Option<EventHandler<FormEvent>>,
}

#[allow(non_snake_case)]
pub fn SlInput(props: SlInputProps) -> Element {
    let label = props.label.clone();
    let value = props.value.clone();
    let placeholder = props.placeholder.clone();
    let clearable = props.clearable.to_string();
    let disabled = props.disabled.to_string();

    rsx! {
        sl-input {
            style: "transition: border-color var(--ctx-motion-duration-fast) var(--ctx-motion-easing-linear);",
            "label": "{label}",
            "value": "{value}",
            "placeholder": "{placeholder}",
            "clearable": "{clearable}",
            "disabled": "{disabled}",
            oninput: move |evt| if let Some(handler) = &props.oninput {
                handler.call(evt);
            }
        }
    }
}

// --- Icon ---

#[derive(Props, Clone, PartialEq)]
pub struct SlIconProps {
    pub name: String,
    #[props(default)]
    pub library: String,
    #[props(optional)]
    pub label: String,
}

#[allow(non_snake_case)]
pub fn SlIcon(props: SlIconProps) -> Element {
    let name = props.name.clone();
    let library = props.library.clone();
    let label = props.label.clone();
    rsx! {
        sl-icon {
            "name": "{name}",
            "library": "{library}",
            "label": "{label}"
        }
    }
}
