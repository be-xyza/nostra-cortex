use dioxus::document::eval;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

// ... (struct definitions will be included below)

#[derive(Props, Clone, PartialEq)]
pub struct D3GraphProps {
    #[props(default)]
    pub config: Option<GraphConfig>,
    #[props(default = "graph-container".to_string())]
    pub container_id: String,
    #[props(default)]
    pub class: String,
}

#[component]
pub fn D3GraphComponent(props: D3GraphProps) -> Element {
    let config = props.config.clone().unwrap_or_else(GraphConfig::default);
    let container_id = props.container_id.clone();

    // Effect to apply configuration to global window object for D3 to use
    use_effect(move || {
        let conf = config.clone();
        spawn(async move {
            if let Ok(json_str) = serde_json::to_string(&conf) {
                // We use a small timeout to ensure the D3 script has loaded if this is first render
                let _ = eval(&format!(
                    r#"
                    (function() {{
                        // Safety check for window object
                        if (typeof window !== 'undefined') {{
                            window.productionGraphConfig = JSON.parse('{}');
                            // If graph exists, restart simulation to apply new physics
                            if (window.graphInstance && window.graphInstance.simulation) {{
                                const cfg = window.productionGraphConfig;
                                const sim = window.graphInstance.simulation;
                                
                                // Apply physics updates
                                sim.force("link")
                                   .distance(cfg.link.distance)
                                   .strength(cfg.link.strength);
                                   
                                sim.force("charge")
                                   .strength(cfg.charge.strength);
                                   
                                if (cfg.collision.enabled) {{
                                    // Re-initialize collision force if needed
                                }}
                                
                                sim.alpha(0.3).restart();
                                console.log("Applied new graph config:", cfg.version);
                            }}
                        }}
                    }})()
                    "#,
                    json_str.replace("'", "\\'")
                ))
                .await;
            }
        }); // Added missing closing parenthesis and semicolon for spawn
    });

    rsx! {
        div {
            id: "{container_id}",
            class: "w-full h-full {props.class}",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GraphConfig {
    pub version: String,
    pub link: LinkConfig,
    pub charge: ChargeConfig,
    pub collision: CollisionConfig,
    pub positioning: PositioningConfig,
    pub simulation: SimulationConfig,
    pub visual: VisualConfig,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LinkConfig {
    pub distance: f64,
    pub strength: f64,
    pub iterations: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChargeConfig {
    pub strength: f64,
    pub distance_min: f64,
    pub distance_max: f64,
    pub theta: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CollisionConfig {
    pub enabled: bool,
    pub radius: f64,
    pub strength: f64,
    pub iterations: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PositioningConfig {
    pub x_strength: f64,
    pub y_strength: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SimulationConfig {
    pub alpha_decay: f64,
    pub velocity_decay: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VisualConfig {
    pub glow: GlowConfig,
    pub nodes: NodeVisualConfig,
    pub links: LinkVisualConfig,
    pub labels: LabelVisualConfig,
    pub link_labels: LinkLabelVisualConfig,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GlowConfig {
    pub enabled: bool,
    pub blur: f64,
    pub opacity: f64,
    pub radius_multiplier: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NodeVisualConfig {
    pub stroke_color: String,
    pub stroke_width: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LinkVisualConfig {
    pub color: String,
    pub width: f64,
    pub opacity: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LabelVisualConfig {
    pub show: bool,
    pub font_size: f64,
    pub mode: String, // "inside", "outside", "off"
    pub use_bbox: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LinkLabelVisualConfig {
    pub show: bool,
    pub use_bbox: bool,
}

impl Default for GraphConfig {
    fn default() -> Self {
        Self::preset_default()
    }
}

impl GraphConfig {
    pub fn preset_default() -> Self {
        Self {
            version: "1.0.0".to_string(),
            link: LinkConfig {
                distance: 180.0,
                strength: 0.3,
                iterations: 1,
            },
            charge: ChargeConfig {
                strength: -800.0,
                distance_min: 50.0,
                distance_max: 600.0,
                theta: 0.9,
            },
            collision: CollisionConfig {
                enabled: true,
                radius: 30.0,
                strength: 0.9,
                iterations: 1,
            },
            positioning: PositioningConfig {
                x_strength: 0.03,
                y_strength: 0.03,
            },
            simulation: SimulationConfig {
                alpha_decay: 0.02,
                velocity_decay: 0.3,
            },
            visual: VisualConfig {
                glow: GlowConfig {
                    enabled: true,
                    blur: 4.0,
                    opacity: 0.3,
                    radius_multiplier: 1.8,
                },
                nodes: NodeVisualConfig {
                    stroke_color: "rgba(255, 255, 255, 0.4)".to_string(),
                    stroke_width: 1.5,
                },
                links: LinkVisualConfig {
                    color: "#94a3b8".to_string(),
                    width: 1.5,
                    opacity: 0.6,
                },
                labels: LabelVisualConfig {
                    show: true,
                    font_size: 10.0,
                    mode: "outside".to_string(),
                    use_bbox: false,
                },
                link_labels: LinkLabelVisualConfig {
                    show: true,
                    use_bbox: false,
                },
            },
        }
    }

    pub fn preset_cosmic() -> Self {
        let mut config = Self::preset_default();
        config.version = "1.0.0-cosmic".to_string();
        // Stronger repulsion for clarity
        config.charge.strength = -1500.0;
        // Longer links for breathing room
        config.link.distance = 250.0;
        // Softer links for natural drift
        config.link.strength = 0.2;
        // Increased collision radius
        config.collision.radius = 40.0;
        // Inside labels by default
        config.visual.labels.mode = "inside".to_string();
        config
    }

    pub fn preset_dense() -> Self {
        let mut config = Self::preset_default();
        config.version = "1.0.0-dense".to_string();
        config.charge.strength = -400.0;
        config.link.distance = 100.0;
        config.link.strength = 0.8;
        config.collision.radius = 20.0;
        config
    }
}
