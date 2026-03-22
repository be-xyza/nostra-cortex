use dioxus::prelude::*;

pub mod components;
pub mod elements;
pub mod protocol;
pub mod renderer;
pub mod theme;

use protocol::{ComponentNode, Message};
use theme::ThemeManager;

#[derive(Clone, Debug)]
pub struct CortexRenderer {
    // Stores the current state of the UI surface
    pub surface_id: Option<String>,
    pub root_component_id: Option<String>,
    pub components: std::collections::HashMap<String, ComponentNode>,
    // Manages the active visual theme
    pub theme_manager: std::sync::Arc<std::sync::Mutex<ThemeManager>>,
}

impl PartialEq for CortexRenderer {
    fn eq(&self, other: &Self) -> bool {
        self.surface_id == other.surface_id
            && self.root_component_id == other.root_component_id
            && self.components == other.components
    }
}

impl Default for CortexRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl CortexRenderer {
    pub fn new() -> Self {
        Self {
            surface_id: None,
            root_component_id: None,
            components: std::collections::HashMap::new(),
            theme_manager: std::sync::Arc::new(std::sync::Mutex::new(ThemeManager::new())),
        }
    }

    /// Ingests an A2UI message and updates the state
    pub fn handle_message(&mut self, msg_str: &str) -> Result<(), anyhow::Error> {
        let msg: Message = serde_json::from_str(msg_str)?;

        match msg {
            Message::BeginRendering(cmd) => {
                self.surface_id = Some(cmd.surface_id);
                self.root_component_id = Some(cmd.root);
                // Reset components on new render
                self.components.clear();

                // Handle Theme Injection
                if let Some(styles) = cmd.styles {
                    if let Ok(mut tm) = self.theme_manager.lock() {
                        if let Err(e) = tm.load_from_value(styles) {
                            tracing::error!("Failed to load theme: {}", e);
                        }
                    }
                }
            }
            Message::SurfaceUpdate(cmd) => {
                if let Some(current_surface) = &self.surface_id {
                    if *current_surface != cmd.surface_id {
                        tracing::warn!(
                            "Received update for surface {} while focused on {}",
                            cmd.surface_id,
                            current_surface
                        );
                    }
                } else {
                    self.surface_id = Some(cmd.surface_id);
                }

                for comp in cmd.components {
                    self.components.insert(comp.id.clone(), comp);
                }
            }
            Message::DataModelUpdate(_cmd) => {
                // TODO: Implement state/signals for data model
            }
            Message::DeleteSurface(cmd) => {
                if self.surface_id.as_deref() == Some(&cmd.surface_id) {
                    self.surface_id = None;
                    self.root_component_id = None;
                    self.components.clear();
                }
            }
        }
        Ok(())
    }

    /// Renders the current state to a Dioxus Element
    pub fn render(&self) -> Element {
        let css_vars = if let Ok(tm) = self.theme_manager.lock() {
            tm.active_theme.to_css_variables()
        } else {
            String::new()
        };

        let base_styles = r#"
            @keyframes fade-in { from { opacity: 0; } to { opacity: 1; } }
            @keyframes fade-out { from { opacity: 1; } to { opacity: 0; } }
            @keyframes slide-in-top { from { transform: translateY(-10px); opacity: 0; } to { transform: translateY(0); opacity: 1; } }
            
            .cortex-root { display: contents; }
            .cortex-enter { 
                animation: fade-in var(--ctx-motion-duration-normal) var(--ctx-motion-easing-ease-out) forwards; 
            }
        "#;

        if let Some(root_id) = &self.root_component_id {
            // Use the recursive renderer to generate the components
            let content = renderer::render_node(self, root_id);
            rsx! {
                style { "{base_styles}" }
                div {
                    class: "cortex-root cortex-enter",
                    style: "{css_vars}",
                    {content}
                }
            }
        } else {
            rsx! {
                style { "{base_styles}" }
                div {
                    class: "cortex-root cortex-idle",
                    style: "{css_vars}",
                    "Cortex Renderer: Idle"
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ingest_begin_rendering() {
        let mut renderer = CortexRenderer::new();
        let msg = r#"{
            "beginRendering": {
                "surfaceId": "surface-1",
                "root": "root-1"
            }
        }"#;

        renderer.handle_message(msg).unwrap();
        assert_eq!(renderer.surface_id, Some("surface-1".to_string()));
        assert_eq!(renderer.root_component_id, Some("root-1".to_string()));
    }

    #[test]
    fn test_ingest_surface_update_text() {
        let mut renderer = CortexRenderer::new();
        renderer.surface_id = Some("surface-1".to_string());

        let msg = r#"{
            "surfaceUpdate": {
                "surfaceId": "surface-1",
                "components": [
                    {
                        "id": "root-1",
                        "component": {
                            "Text": {
                                "text": { "literalString": "Hello World" },
                                "usageHint": "h1"
                            }
                        }
                    }
                ]
            }
        }"#;

        renderer.handle_message(msg).unwrap();
        assert!(renderer.components.contains_key("root-1"));

        // Verify component type
        let node = renderer.components.get("root-1").unwrap();
        if let components::ComponentWrapper::Text(props) = &node.component {
            if let components::PropertyValue::Literal(l) = &props.text {
                assert_eq!(l.value, "Hello World");
            } else {
                panic!("Expected Literal property value");
            }
        } else {
            panic!("Expected Text component");
        }
    }

    #[test]
    fn test_theme_injection() {
        let mut renderer = CortexRenderer::new();
        let msg = r##"{
            "beginRendering": {
                "surfaceId": "surface-theme",
                "root": "root-1",
                "styles": {
                    "id": "custom-theme",
                    "name": "Custom Dark",
                    "version": "1.0.0",
                    "tokens": {
                         "colors": {
                             "brand": { "primary": "#123456", "secondary": "#654321", "tertiary": "#000000" },
                             "semantic": { "success": "g", "warning": "y", "error": "r", "info": "b" },
                             "fills": { "quiet": "t", "normal": "g", "loud": "w" },
                             "on": { "quiet": "b", "normal": "b", "loud": "b" },
                             "borders": { "quiet": "g", "normal": "g", "loud": "g" },
                             "background": "#000",
                             "surface": "#111"
                         },
                         "typography": {},
                         "spacing": {},
                         "radii": {},
                         "shadows": {},
                         "motion": {
                             "durations": { "instant": "0", "fast": "1", "normal": "2", "slow": "3" },
                             "easings": { "linear": "l", "easeIn": "i", "easeOut": "o", "easeInOut": "io" },
                             "transitions": { "enter": "e", "exit": "x" }
                         }
                    }
                }
            }
        }"##;

        renderer.handle_message(msg).unwrap();
        let tm = renderer.theme_manager.lock().unwrap();
        assert_eq!(tm.active_theme.name, "Custom Dark");
        assert_eq!(tm.active_theme.tokens.colors.brand.primary, "#123456");

        let css = tm.active_theme.to_css_variables();
        assert!(css.contains("--ctx-color-primary: #123456;"));
        assert!(css.contains("--ctx-motion-duration-normal: 2;"));
    }

    #[test]
    fn test_render_button_ssr() {
        let mut renderer = CortexRenderer::new();
        renderer.surface_id = Some("surface-1".to_string());
        renderer.root_component_id = Some("root-1".to_string());

        let msg = r#"{
            "surfaceUpdate": {
                "surfaceId": "surface-1",
                "components": [
                    {
                        "id": "root-1",
                        "component": {
                            "Button": {
                                "child": "text-1",
                                "action": { "name": "do-something" },
                                "primary": true
                            }
                        }
                    },
                    {
                        "id": "text-1",
                        "component": {
                            "Text": { "text": { "literalString": "Click Me" } }
                        }
                    }
                ]
            }
        }"#;

        renderer.handle_message(msg).unwrap();

        // Use VirtualDom to provide a runtime for EventHandler creation
        let mut vdom = VirtualDom::new_with_props(
            |renderer: CortexRenderer| rsx! { {renderer.render()} },
            renderer,
        );
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);

        // Check for Shoelace tag
        assert!(html.contains("<sl-button"));
        // Check for variant attribute
        assert!(html.contains("variant=\"primary\""));
        // Check for child text
        assert!(html.contains("Click Me"));
        // Check for motion variables in the root style
        assert!(html.contains("--ctx-motion-duration-normal"));
        // Check for enter animation class
        assert!(html.contains("cortex-enter"));
    }

    #[test]
    fn test_render_card_ssr() {
        let mut renderer = CortexRenderer::new();
        renderer.surface_id = Some("surface-1".to_string());
        renderer.root_component_id = Some("root-1".to_string());

        let msg = r#"{
            "surfaceUpdate": {
                "surfaceId": "surface-1",
                "components": [
                    {
                        "id": "root-1",
                        "component": {
                            "Card": {
                                "child": "text-1"
                            }
                        }
                    },
                    {
                        "id": "text-1",
                        "component": {
                            "Text": { "text": { "literalString": "Inside Card" } }
                        }
                    }
                ]
            }
        }"#;

        renderer.handle_message(msg).unwrap();
        let html = dioxus_ssr::render_element(renderer.render());

        assert!(html.contains("<sl-card"));
        assert!(html.contains("Inside Card"));
        // Check for governed card transition
        assert!(html.contains("transition: transform var(--ctx-motion-duration-normal) var(--ctx-motion-easing-ease-out)"));
    }
}
