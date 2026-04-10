use crate::types::GeoLocation;
use dioxus::prelude::*;

#[derive(Props, PartialEq, Clone)]
pub struct MapViewProps {
    pub location: Option<GeoLocation>,
    pub markers: Vec<GeoLocation>,
}

#[component]
pub fn MapView(props: MapViewProps) -> Element {
    // Phase 1: Placeholder or basic Leaflet if JS interop is ready.
    // Assuming simple placeholder for now as per plan MVP.
    // If JS interop is needed, we'd use `use_eval` or similar.

    rsx! {
        div {
            class: "map-view-container p-4 border rounded bg-gray-100 dark:bg-gray-800",
            h3 { "Map View (Location Context)" }
            div {
                "Current Location: "
                if let Some(loc) = &props.location {
                    "Lat: {loc.latitude:.4}, Long: {loc.longitude:.4}"
                } else {
                    "Unknown"
                }
            }
            div {
                "Markers: "
                "{props.markers.len()}"
            }
            // Future: Embed Leaflet/Mapbox via JS
            // Simple SVG Radar / Map Visual
            div {
                class: "relative w-full aspect-[2/1] bg-[#1a1b26] rounded-lg overflow-hidden border border-[#30363d] shadow-inner",
                // Grid Lines
                svg {
                    width: "100%",
                    height: "100%",
                    view_box: "0 0 360 180",
                    preserve_aspect_ratio: "none",

                    // Equator
                    line { x1: "0", y1: "90", x2: "360", y2: "90", stroke: "#2f3542", stroke_width: "1" }
                    // Prime Meridian
                    line { x1: "180", y1: "0", x2: "180", y2: "180", stroke: "#2f3542", stroke_width: "1" }

                    // Continents (Rough simplified paths strictly for visual context)
                    // Configured as a background pattern for "tech" feel
                    path {
                         d: "M60,40 Q90,20 120,40 T180,40 M240,40 Q270,20 300,40",
                         stroke: "#2f3542",
                         fill: "none",
                         opacity: "0.3"
                    }

                    // Location Dot
                    if let Some(loc) = &props.location {
                        {
                            let cx = loc.longitude + 180.0;
                            let cy = 90.0 - loc.latitude;
                            rsx! {
                                circle {
                                    cx: "{cx}",
                                    cy: "{cy}",
                                    r: "4",
                                    fill: "#3fb950",
                                    class: "animate-pulse",
                                    stroke: "#ffffff",
                                    stroke_width: "1"
                                }
                                // Crosshair lines
                                line { x1: "{cx}", y1: "0", x2: "{cx}", y2: "180", stroke: "#3fb950", stroke_width: "0.5", opacity: "0.5" }
                                line { x1: "0", y1: "{cy}", x2: "360", y2: "{cy}", stroke: "#3fb950", stroke_width: "0.5", opacity: "0.5" }
                            }
                        }
                    }
                }

                // Overlay Text
                div { class: "absolute bottom-2 right-2 text-[10px] font-mono text-[#3fb950]",
                    "SAT-LINK: ACTIVE"
                }
            }
        }
    }
}
