use dioxus::document::eval;
use dioxus::prelude::*;

#[component]
pub fn D3Lab() -> Element {
    // Graph Lab state
    let mut lab_initialized = use_signal(|| false);
    let mut node_count = use_signal(|| 200u32);
    let mut density = use_signal(|| 30u32); // 0-100
    let mut topology = use_signal(|| "random".to_string());
    let mut active_preset = use_signal(|| "default".to_string());

    // Force config sliders (stored as integers for easier slider handling)
    let mut link_distance = use_signal(|| 180i32);
    let mut link_strength = use_signal(|| 30i32); // 0-100
    let mut charge_strength = use_signal(|| 800i32); // positive, will negate
    let mut charge_theta = use_signal(|| 90i32); // 0-100
    let mut charge_max_dist = use_signal(|| 600i32);
    let mut collision_enabled = use_signal(|| true);
    let _collision_radius = use_signal(|| 30i32);
    let mut alpha_decay = use_signal(|| 2i32); // 0-10, divide by 100
    let mut velocity_decay = use_signal(|| 30i32); // 0-100

    // Visual config sliders
    let mut glow_enabled = use_signal(|| true);
    let mut glow_blur = use_signal(|| 4i32); // 1-10
    let mut glow_opacity = use_signal(|| 40i32); // 0-100
    let mut glow_radius = use_signal(|| 180i32); // 100-300 (divide by 100)
    let mut link_labels_enabled = use_signal(|| false); // Expensive - off by default
    let mut link_labels_bbox = use_signal(|| true); // getBBox calls
    let mut label_mode = use_signal(|| "inside".to_string()); // outside, inside, off

    // Metrics polling
    let mut fps = use_signal(|| 0u32);
    let mut tick_ms = use_signal(|| "0.00".to_string());
    let mut dom_count = use_signal(|| 0u32);

    // Initialize lab when tab is active
    use_effect(move || {
        if !*lab_initialized.read() {
            let _ = eval("window.graphLab.initLab('lab-graph-container');");
            lab_initialized.set(true);
        }
    });

    // Metrics polling
    use_future(move || async move {
        loop {
            gloo_timers::future::TimeoutFuture::new(500).await;
            let result = eval(
                r#"
                (function() {
                    const m = window.graphLab.getMetrics();
                    return JSON.stringify(m);
                })()
            "#,
            );
            if let Ok(val) = result.await {
                if let Some(json_str) = val.as_str() {
                    if let Ok(metrics) = serde_json::from_str::<serde_json::Value>(json_str) {
                        fps.set(metrics["fps"].as_u64().unwrap_or(0) as u32);
                        tick_ms.set(metrics["tickMs"].as_str().unwrap_or("0.00").to_string());
                        dom_count.set(metrics["domElements"].as_u64().unwrap_or(0) as u32);
                    }
                }
            }
        }
    });

    rsx! {
        div { class: "flex flex-col h-full",
            // Header is handled by parent LabView, so we just focus on content

            // Main content
            div { class: "flex flex-1 overflow-hidden",
                // Graph canvas
                div { class: "flex-1 relative",
                    div {
                        id: "lab-graph-container",
                        class: "absolute inset-0 bg-[#0a0a0a]"
                    }
                }

                // Control panel
                div { class: "w-80 border-l overflow-y-auto p-4 space-y-6 bg-card",
                    // Graph Generator Section
                    div { class: "space-y-3",
                        h3 { class: "font-semibold text-sm uppercase tracking-wider text-muted-foreground", "Graph Generator" }

                        // Size presets
                        div { class: "grid grid-cols-4 gap-2",
                            for (label, count) in [("Small", 50), ("Med", 200), ("Large", 500), ("XL", 1000)] {
                                button {
                                    class: format!(
                                        "px-2 py-1.5 text-xs font-medium rounded border transition {}",
                                        if node_count() == count { "bg-primary text-primary-foreground border-primary" } else { "bg-background border-border hover:bg-muted" }
                                    ),
                                    onclick: move |_| node_count.set(count),
                                    "{label}"
                                }
                            }
                        }

                        // Custom count
                        div { class: "flex items-center gap-2",
                            input {
                                r#type: "number",
                                class: "w-20 border rounded px-2 py-1 text-sm bg-background",
                                value: "{node_count}",
                                onchange: move |e| {
                                    if let Ok(v) = e.value().parse::<u32>() {
                                        node_count.set(v.clamp(10, 5000));
                                    }
                                }
                            }
                            span { class: "text-sm text-muted-foreground", "nodes" }
                        }

                        // Topology
                        div { class: "flex items-center gap-2",
                            label { class: "text-sm", "Topology:" }
                            select {
                                class: "flex-1 border rounded px-2 py-1 text-sm bg-background",
                                value: "{topology}",
                                onchange: move |e| topology.set(e.value()),
                                option { value: "random", "Random" }
                                option { value: "clustered", "Clustered" }
                                option { value: "hierarchical", "Hierarchical" }
                            }
                        }

                        // Density slider
                        div {
                            div { class: "flex justify-between text-xs text-muted-foreground mb-1",
                                span { "Edge Density" }
                                span { "{density}%" }
                            }
                            input {
                                r#type: "range",
                                class: "w-full",
                                min: "5",
                                max: "80",
                                value: "{density}",
                                onchange: move |e| {
                                    if let Ok(v) = e.value().parse::<u32>() {
                                        density.set(v);
                                    }
                                }
                            }
                        }

                        // Generate button
                        button {
                            class: "w-full bg-primary text-primary-foreground py-2 rounded-lg font-medium text-sm hover:bg-primary/90 transition",
                            onclick: move |_| {
                                let count = node_count();
                                let dens = density() as f64 / 100.0;
                                let topo = topology();
                                let _ = eval(&format!(
                                    "window.graphLab.generateGraph({}, {}, '{}');",
                                    count, dens, topo
                                ));
                            },
                            "🔄 Generate Graph"
                        }
                    }

                    // Presets Section
                    div { class: "space-y-3 border-t pt-4",
                        h3 { class: "font-semibold text-sm uppercase tracking-wider text-muted-foreground", "Presets" }

                        div { class: "space-y-1",
                            for (id, name) in [("default", "Default"), ("optimized-small", "Optimized (Small)"), ("optimized-medium", "Optimized (Medium)"), ("optimized-large", "Optimized (Large)"), ("minimal", "Minimal")] {
                                button {
                                    class: format!(
                                        "w-full text-left px-3 py-2 text-sm rounded transition {}",
                                        if active_preset() == id { "bg-primary/10 text-primary font-medium" } else { "hover:bg-muted" }
                                    ),
                                    onclick: move |_| {
                                        active_preset.set(id.to_string());
                                        let _ = eval(&format!("window.graphLab.loadPreset('{}');", id));
                                    },
                                    if active_preset() == id { "● " } else { "○ " }
                                    "{name}"
                                }
                            }
                        }
                    }

                    // Force Configuration Section
                    div { class: "space-y-3 border-t pt-4",
                        h3 { class: "font-semibold text-sm uppercase tracking-wider text-muted-foreground", "Force Configuration" }

                        // Link Force
                        div { class: "space-y-2 p-3 bg-muted/30 rounded-lg",
                            h4 { class: "text-xs font-semibold text-muted-foreground", "Link Force" }

                            // Distance
                            div {
                                div { class: "flex justify-between text-xs mb-1",
                                    span { "Distance" }
                                    span { class: "font-mono", "{link_distance}" }
                                }
                                input {
                                    r#type: "range",
                                    class: "w-full",
                                    min: "50",
                                    max: "400",
                                    value: "{link_distance}",
                                    onchange: move |e| {
                                        if let Ok(v) = e.value().parse::<i32>() {
                                            link_distance.set(v);
                                            let _ = eval(&format!("window.graphLab.setForceParam('link', 'distance', {});", v));
                                        }
                                    }
                                }
                            }

                            // Strength
                            div {
                                div { class: "flex justify-between text-xs mb-1",
                                    span { "Strength" }
                                    span { class: "font-mono", "{link_strength() as f64 / 100.0:.2}" }
                                }
                                input {
                                    r#type: "range",
                                    class: "w-full",
                                    min: "5",
                                    max: "100",
                                    value: "{link_strength}",
                                    onchange: move |e| {
                                        if let Ok(v) = e.value().parse::<i32>() {
                                            link_strength.set(v);
                                            let _ = eval(&format!("window.graphLab.setForceParam('link', 'strength', {});", v as f64 / 100.0));
                                        }
                                    }
                                }
                            }
                        }

                        // Charge Force
                        div { class: "space-y-2 p-3 bg-muted/30 rounded-lg",
                            h4 { class: "text-xs font-semibold text-muted-foreground", "Charge Force" }

                            // Strength (negative)
                            div {
                                div { class: "flex justify-between text-xs mb-1",
                                    span { "Repulsion" }
                                    span { class: "font-mono", "-{charge_strength}" }
                                }
                                input {
                                    r#type: "range",
                                    class: "w-full",
                                    min: "50",
                                    max: "1500",
                                    value: "{charge_strength}",
                                    onchange: move |e| {
                                        if let Ok(v) = e.value().parse::<i32>() {
                                            charge_strength.set(v);
                                            let _ = eval(&format!("window.graphLab.setForceParam('charge', 'strength', {});", -v));
                                        }
                                    }
                                }
                            }

                            // Theta (Barnes-Hut)
                            div {
                                div { class: "flex justify-between text-xs mb-1",
                                    span { "Theta (B-H)" }
                                    span { class: "font-mono", "{charge_theta() as f64 / 100.0:.2}" }
                                }
                                input {
                                    r#type: "range",
                                    class: "w-full",
                                    min: "50",
                                    max: "99",
                                    value: "{charge_theta}",
                                    onchange: move |e| {
                                        if let Ok(v) = e.value().parse::<i32>() {
                                            charge_theta.set(v);
                                            let _ = eval(&format!("window.graphLab.setForceParam('charge', 'theta', {});", v as f64 / 100.0));
                                        }
                                    }
                                }
                            }

                            // Max Distance
                            div {
                                div { class: "flex justify-between text-xs mb-1",
                                    span { "Max Distance" }
                                    span { class: "font-mono", "{charge_max_dist}" }
                                }
                                input {
                                    r#type: "range",
                                    class: "w-full",
                                    min: "100",
                                    max: "1000",
                                    value: "{charge_max_dist}",
                                    onchange: move |e| {
                                        if let Ok(v) = e.value().parse::<i32>() {
                                            charge_max_dist.set(v);
                                            let _ = eval(&format!("window.graphLab.setForceParam('charge', 'distanceMax', {});", v));
                                        }
                                    }
                                }
                            }
                        }

                        // Simulation
                        div { class: "space-y-2 p-3 bg-muted/30 rounded-lg",
                            h4 { class: "text-xs font-semibold text-muted-foreground", "Simulation" }

                            // Alpha Decay
                            div {
                                div { class: "flex justify-between text-xs mb-1",
                                    span { "Alpha Decay" }
                                    span { class: "font-mono", "{alpha_decay() as f64 / 100.0:.2}" }
                                }
                                input {
                                    r#type: "range",
                                    class: "w-full",
                                    min: "1",
                                    max: "15",
                                    value: "{alpha_decay}",
                                    onchange: move |e| {
                                        if let Ok(v) = e.value().parse::<i32>() {
                                            alpha_decay.set(v);
                                            let _ = eval(&format!("window.graphLab.setForceParam('simulation', 'alphaDecay', {});", v as f64 / 100.0));
                                        }
                                    }
                                }
                            }

                            // Velocity Decay
                            div {
                                div { class: "flex justify-between text-xs mb-1",
                                    span { "Velocity Decay" }
                                    span { class: "font-mono", "{velocity_decay() as f64 / 100.0:.2}" }
                                }
                                input {
                                    r#type: "range",
                                    class: "w-full",
                                    min: "10",
                                    max: "80",
                                    value: "{velocity_decay}",
                                    onchange: move |e| {
                                        if let Ok(v) = e.value().parse::<i32>() {
                                            velocity_decay.set(v);
                                            let _ = eval(&format!("window.graphLab.setForceParam('simulation', 'velocityDecay', {});", v as f64 / 100.0));
                                        }
                                    }
                                }
                            }
                        }

                        // Collision toggle
                        div { class: "flex items-center justify-between p-3 bg-muted/30 rounded-lg",
                            span { class: "text-sm", "Collision Detection" }
                            button {
                                class: format!(
                                    "relative inline-flex h-5 w-9 shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors {}",
                                    if collision_enabled() { "bg-primary" } else { "bg-muted" }
                                ),
                                onclick: move |_| {
                                    let new_val = !collision_enabled();
                                    collision_enabled.set(new_val);
                                    let _ = eval(&format!("window.graphLab.setForceParam('collision', 'enabled', {});", new_val));
                                },
                                span {
                                    class: format!(
                                        "pointer-events-none inline-block h-4 w-4 transform rounded-full bg-white shadow ring-0 transition {}",
                                        if collision_enabled() { "translate-x-4" } else { "translate-x-0" }
                                    )
                                }
                            }
                        }
                    }

                    // Visual Settings Section
                    div { class: "space-y-3 border-t pt-4",
                        h3 { class: "font-semibold text-sm uppercase tracking-wider text-muted-foreground", "Visual Settings" }

                        // Glow settings
                        div { class: "space-y-2 p-3 bg-muted/30 rounded-lg",
                            h4 { class: "text-xs font-semibold text-muted-foreground", "Glow Effect" }

                            // Glow toggle
                            div { class: "flex items-center justify-between mb-2",
                                span { class: "text-xs", "Enabled" }
                                button {
                                    class: format!(
                                        "relative inline-flex h-5 w-9 shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors {}",
                                        if glow_enabled() { "bg-primary" } else { "bg-muted" }
                                    ),
                                    onclick: move |_| {
                                        let new_val = !glow_enabled();
                                        glow_enabled.set(new_val);
                                        let _ = eval(&format!("window.graphLab.setVisualParam('glow', 'enabled', {});", new_val));
                                    },
                                    span {
                                        class: format!(
                                            "pointer-events-none inline-block h-4 w-4 transform rounded-full bg-white shadow ring-0 transition {}",
                                            if glow_enabled() { "translate-x-4" } else { "translate-x-0" }
                                        )
                                    }
                                }
                            }

                            // Blur
                            div {
                                div { class: "flex justify-between text-xs mb-1",
                                    span { "Blur" }
                                    span { class: "font-mono", "{glow_blur}" }
                                }
                                input {
                                    r#type: "range",
                                    class: "w-full",
                                    min: "1",
                                    max: "12",
                                    value: "{glow_blur}",
                                    onchange: move |e| {
                                        if let Ok(v) = e.value().parse::<i32>() {
                                            glow_blur.set(v);
                                            let _ = eval(&format!("window.graphLab.setVisualParam('glow', 'blur', {});", v));
                                        }
                                    }
                                }
                            }

                            // Opacity
                            div {
                                div { class: "flex justify-between text-xs mb-1",
                                    span { "Opacity" }
                                    span { class: "font-mono", "{glow_opacity() as f64 / 100.0:.2}" }
                                }
                                input {
                                    r#type: "range",
                                    class: "w-full",
                                    min: "10",
                                    max: "100",
                                    value: "{glow_opacity}",
                                    onchange: move |e| {
                                        if let Ok(v) = e.value().parse::<i32>() {
                                            glow_opacity.set(v);
                                            let _ = eval(&format!("window.graphLab.setVisualParam('glow', 'opacity', {});", v as f64 / 100.0));
                                        }
                                    }
                                }
                            }

                            // Radius Multiplier
                            div {
                                div { class: "flex justify-between text-xs mb-1",
                                    span { "Radius" }
                                    span { class: "font-mono", "{glow_radius() as f64 / 100.0:.1}x" }
                                }
                                input {
                                    r#type: "range",
                                    class: "w-full",
                                    min: "100",
                                    max: "300",
                                    value: "{glow_radius}",
                                    onchange: move |e| {
                                        if let Ok(v) = e.value().parse::<i32>() {
                                            glow_radius.set(v);
                                            let _ = eval(&format!("window.graphLab.setVisualParam('glow', 'radiusMultiplier', {});", v as f64 / 100.0));
                                        }
                                    }
                                }
                            }
                        }

                        // Node Labels
                        div { class: "space-y-2 p-3 bg-muted/30 rounded-lg",
                            h4 { class: "text-xs font-semibold text-muted-foreground", "Node Labels" }

                            div { class: "flex items-center gap-2",
                                label { class: "text-xs", "Position:" }
                                select {
                                    class: "flex-1 border rounded px-2 py-1 text-xs bg-background",
                                    value: "{label_mode}",
                                    onchange: move |e| {
                                        let val = e.value();
                                        label_mode.set(val.clone());
                                        if val == "off" {
                                            let _ = eval("window.graphLab.setVisualParam('labels', 'show', false);");
                                        } else {
                                            let _ = eval("window.graphLab.setVisualParam('labels', 'show', true);");
                                            let _ = eval(&format!("window.graphLab.setVisualParam('labels', 'mode', '{}');", val));
                                        }
                                    },
                                    option { value: "outside", "Outside" }
                                    option { value: "inside", "Inside" }
                                    option { value: "off", "Off" }
                                }
                            }
                        }

                        // Link Labels section (production bottleneck)
                        div { class: "space-y-2 p-3 bg-destructive/10 rounded-lg border border-destructive/30",
                            h4 { class: "text-xs font-semibold text-destructive", "⚠️ Link Labels (Expensive)" }
                            p { class: "text-xs text-muted-foreground mb-2", "These cause production lag via getBBox() calls" }

                            // Link labels toggle
                            div { class: "flex items-center justify-between",
                                span { class: "text-xs", "Show Link Labels" }
                                button {
                                    class: format!(
                                        "relative inline-flex h-5 w-9 shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors {}",
                                        if link_labels_enabled() { "bg-destructive" } else { "bg-muted" }
                                    ),
                                    onclick: move |_| {
                                        let new_val = !link_labels_enabled();
                                        link_labels_enabled.set(new_val);
                                        let _ = eval(&format!("window.graphLab.setVisualParam('linkLabels', 'show', {}); window.graphLab.renderLabGraph();", new_val));
                                    },
                                    span {
                                        class: format!(
                                            "pointer-events-none inline-block h-4 w-4 transform rounded-full bg-white shadow ring-0 transition {}",
                                            if link_labels_enabled() { "translate-x-4" } else { "translate-x-0" }
                                        )
                                    }
                                }
                            }

                            // getBBox toggle
                            div { class: "flex items-center justify-between mt-2",
                                span { class: "text-xs", "Use getBBox (slow)" }
                                button {
                                    class: format!(
                                        "relative inline-flex h-5 w-9 shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors {}",
                                        if link_labels_bbox() { "bg-destructive" } else { "bg-muted" }
                                    ),
                                    onclick: move |_| {
                                        let new_val = !link_labels_bbox();
                                        link_labels_bbox.set(new_val);
                                        let _ = eval(&format!("window.graphLab.setVisualParam('linkLabels', 'useBBox', {});", new_val));
                                    },
                                    span {
                                        class: format!(
                                            "pointer-events-none inline-block h-4 w-4 transform rounded-full bg-white shadow ring-0 transition {}",
                                            if link_labels_bbox() { "translate-x-4" } else { "translate-x-0" }
                                        )
                                    }
                                }
                            }
                        }
                    }

                    // Clear button
                    div { class: "border-t pt-4",
                        button {
                            class: "w-full border border-destructive text-destructive py-2 rounded-lg font-medium text-sm hover:bg-destructive/10 transition",
                            onclick: move |_| {
                                let _ = eval("window.graphLab.clear();");
                            },
                            "🗑️ Clear Graph"
                        }
                    }
                }
            }

            // Performance bar
            div { class: "border-t bg-muted/30 px-4 py-2 flex items-center gap-8 text-sm",
                div { class: "flex items-center gap-2",
                    span { class: "text-muted-foreground", "FPS:" }
                    span {
                        class: format!("font-mono font-bold {}",
                            if fps() >= 50 { "text-green-500" }
                            else if fps() >= 30 { "text-yellow-500" }
                            else { "text-red-500" }
                        ),
                        "{fps}"
                    }
                }
                div { class: "flex items-center gap-2",
                    span { class: "text-muted-foreground", "Tick:" }
                    span { class: "font-mono", "{tick_ms}ms" }
                }
                div { class: "flex items-center gap-2",
                    span { class: "text-muted-foreground", "DOM:" }
                    span { class: "font-mono", "{dom_count}" }
                }
                div { class: "flex-1" }
                div { class: "text-xs text-muted-foreground",
                    "Adjust parameters to optimize graph performance"
                }
            }
        }
    }
}
