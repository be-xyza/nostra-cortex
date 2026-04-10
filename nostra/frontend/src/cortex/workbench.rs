use crate::api;
use crate::types::{FlowGraph, FlowHandlePosition, FlowLayoutInput, FlowNodePosition};
use candid::Principal;
use dioxus::document::eval;
use dioxus::prelude::*;
use js_sys::Date;
use serde::Deserialize;
use serde_json::json;
use std::collections::HashSet;
use wasm_bindgen::JsValue;

#[derive(Clone, Copy, PartialEq)]
enum WorkbenchTab {
    FlowGraph,
    Traces,
    Logs,
}

#[derive(Deserialize)]
struct LayoutPoint {
    id: String,
    x: i64,
    y: i64,
}

#[derive(Deserialize)]
struct HandlePoint {
    handle_id: String,
    source: String,
    target: String,
}

#[derive(Deserialize)]
struct LayoutBundle {
    nodes: Vec<LayoutPoint>,
    handles: Vec<HandlePoint>,
    collapsed: Vec<String>,
}

#[derive(Clone, Debug)]
struct LayoutPreview {
    updated_at: u64,
    timestamp: String,
    moved: i64,
    max: i64,
    handle_diff: usize,
    group_diff: usize,
}

fn format_saved_at(ts: u64) -> String {
    let seconds = ts / 1_000_000_000;
    let millis = seconds.saturating_mul(1000);
    let date = Date::new(&JsValue::from_f64(millis as f64));
    date.to_iso_string()
        .as_string()
        .unwrap_or_else(|| seconds.to_string())
}

fn truncate_principal(p: Principal) -> String {
    let s = p.to_text();
    if s.len() > 15 {
        format!("{}...{}", &s[0..6], &s[s.len() - 4..])
    } else {
        s
    }
}

#[component]
pub fn CortexWorkbench() -> Element {
    let mut tab = use_signal(|| WorkbenchTab::FlowGraph);
    let flow_graph = use_signal(|| None as Option<FlowGraph>);
    let flow_error = use_signal(|| None as Option<String>);
    let save_status = use_signal(|| None as Option<String>);
    let saving = use_signal(|| false);
    let layout_status = use_signal(|| None as Option<String>);
    let layout_dirty = use_signal(|| false);
    let layout_refreshing = use_signal(|| false);
    let layout_resetting = use_signal(|| false);
    let layout_locked = use_signal(|| false);
    let layout_autolayout = use_signal(|| false);
    let layout_inspector_open = use_signal(|| false);
    let layout_delta_count = use_signal(|| 0usize);
    let layout_delta_list = use_signal(|| Vec::<(String, i64, i64, f64)>::new());
    let layout_compare_mode = use_signal(|| false);
    let overlay_cleared = use_signal(|| false);
    let layout_heat_mode = use_signal(|| false);
    let layout_saved_by = use_signal(|| None as Option<String>);
    let layout_saved_at = use_signal(|| None as Option<String>);
    let layout_history = use_signal(|| Vec::new());
    let layout_history_loading = use_signal(|| false);
    let layout_history_error = use_signal(|| None as Option<String>);
    let layout_commit_message = use_signal(|| "".to_string());
    let layout_commit_tag = use_signal(|| "".to_string());
    let layout_diff_preview = use_signal(|| None as Option<LayoutPreview>);
    let layout_handle_positions = use_signal(|| Vec::<FlowHandlePosition>::new());
    let layout_collapsed_groups = use_signal(|| Vec::<String>::new());
    let new_handle_id = use_signal(|| "".to_string());
    let new_handle_source = use_signal(|| "".to_string());
    let new_handle_target = use_signal(|| "".to_string());
    let new_collapsed_group = use_signal(|| "".to_string());

    let workflow_id = "workflow:mvp".to_string();
    let workflow_id_label = workflow_id.clone();
    let workflow_id_fetch = workflow_id.clone();
    let workflow_id_layout = workflow_id.clone();
    let graph_container_id = "cortex-flow-graph".to_string();

    let flow_graph_signal = flow_graph.clone();
    let flow_error_signal = flow_error.clone();

    use_future(move || {
        let workflow_id = workflow_id_fetch.clone();
        let mut flow_graph = flow_graph_signal;
        let mut flow_error = flow_error_signal;
        async move {
            let agent = api::create_agent().await;
            match api::get_flow_graph(&agent, workflow_id, None).await {
                Ok(graph) => {
                    flow_graph.set(Some(graph));
                    flow_error.set(None);
                }
                Err(err) => {
                    flow_error.set(Some(err));
                }
            }
        }
    });

    // Render D3 graph when flow graph loads
    {
        let graph_container_id = graph_container_id.clone();
        let flow_graph = flow_graph.clone();
        let mut layout_status = layout_status.clone();
        let mut layout_dirty = layout_dirty.clone();
        let mut layout_saved_by = layout_saved_by.clone();
        let mut layout_saved_at = layout_saved_at.clone();
        let mut layout_history = layout_history.clone();
        let mut layout_history_loading = layout_history_loading.clone();
        let mut layout_history_error = layout_history_error.clone();
        let mut layout_commit_message = layout_commit_message.clone();
        let mut layout_commit_tag = layout_commit_tag.clone();
        let mut layout_diff_preview = layout_diff_preview.clone();
        let mut layout_handle_positions = layout_handle_positions.clone();
        let mut layout_collapsed_groups = layout_collapsed_groups.clone();
        let layout_heat_mode = layout_heat_mode.clone();
        let workflow_id = workflow_id_label.clone();
        use_effect(move || {
            if let Some(graph) = flow_graph() {
                let nodes = graph
                    .nodes
                    .iter()
                    .map(|node| {
                        let color = match node.node_type.as_str() {
                            "user_task" => "#f97316",
                            "system_op" => "#0ea5e9",
                            "async_external_op" => "#22c55e",
                            "noop" => "#64748b",
                            _ => "#f43f5e",
                        };
                        json!({
                            "id": node.id,
                            "label": node.name,
                            "color": color,
                            "val": 18
                        })
                    })
                    .collect::<Vec<_>>();
                let links = graph
                    .edges
                    .iter()
                    .map(|edge| {
                        json!({
                            "source": edge.source,
                            "target": edge.target,
                            "label": edge.variant
                        })
                    })
                    .collect::<Vec<_>>();
                let payload = json!({ "nodes": nodes, "links": links });

                let container_id = graph_container_id.clone();
                let graph_version = graph.version.clone();
                let workflow_id = workflow_id.clone();
                spawn(async move {
                    let js = format!(
                        r#"(function() {{
                            if (!window.graphLab) return;
                            if (!window.graphLab.instance || window.graphLab.instance.containerId !== "{container_id}") {{
                                window.graphLab.initLab("{container_id}");
                            }}
                            window.graphLab.data = {data};
                            window.graphLab.renderLabGraph();
                            window.graphLab.updateMetricCounts();
                            if (window.graphLab.overlayPaused === undefined) {{
                                window.graphLab.overlayPaused = false;
                            }}
                            if (!window.graphLab.renderLayoutOverlay) {{
                                window.graphLab.renderLayoutOverlay = function() {{
                                    if (!window.graphLab || !window.graphLab.instance || !window.graphLab.data) return;
                                    if (window.graphLab.overlayPaused) return;
                                    const svg = window.graphLab.instance.svg;
                                    if (!svg) return;
                                    let overlay = svg.select('g.layout-overlay');
                                    if (overlay.empty()) {{
                                        overlay = svg.append('g').attr('class', 'layout-overlay');
                                    }}
                                    overlay.selectAll('*').remove();
                                    if (!window.graphLab.savedPositions) return;
                                    const map = new Map(window.graphLab.savedPositions.map(p => [p.id, p]));
                                    const nodes = window.graphLab.data.nodes || [];
                                    const links = nodes.map(n => {{
                                        const p = map.get(n.id);
                                        if (!p) return null;
                                        return {{ id: n.id, x1: n.x || 0, y1: n.y || 0, x2: p.x, y2: p.y }};
                                    }}).filter(Boolean);
                                    overlay.selectAll('line')
                                        .data(links, d => d.id)
                                        .enter()
                                        .append('line')
                                        .attr('x1', d => d.x1)
                                        .attr('y1', d => d.y1)
                                        .attr('x2', d => d.x2)
                                        .attr('y2', d => d.y2)
                                        .attr('stroke', 'rgba(248, 250, 252, 0.25)')
                                        .attr('stroke-dasharray', '2,4')
                                        .attr('stroke-width', 1);
                                    overlay.selectAll('circle')
                                        .data(window.graphLab.savedPositions, d => d.id)
                                        .enter()
                                        .append('circle')
                                        .attr('cx', d => d.x)
                                        .attr('cy', d => d.y)
                                        .attr('r', 5)
                                        .attr('fill', 'rgba(148, 163, 184, 0.2)')
                                        .attr('stroke', 'rgba(148, 163, 184, 0.8)')
                                        .attr('stroke-width', 1);
                                }}
                            }}
                            if (!window.graphLab.applyHeatMode) {{
                                window.graphLab.applyHeatMode = function(enabled) {{
                                    if (!window.graphLab || !window.graphLab.data || !window.graphLab.instance) return;
                                    const nodes = window.graphLab.data.nodes || [];
                                    const map = new Map((window.graphLab.savedPositions || []).map(p => [p.id, p]));
                                    const maxDist = Math.max(1, ...nodes.map(n => {{
                                        const p = map.get(n.id);
                                        if (!p) return 0;
                                        const dx = (n.x || 0) - p.x;
                                        const dy = (n.y || 0) - p.y;
                                        return Math.sqrt(dx * dx + dy * dy);
                                    }}));
                                    const lerp = (a, b, t) => Math.round(a + (b - a) * t);
                                    const mix = (c1, c2, t) => 'rgb(' + lerp(c1[0], c2[0], t) + ', ' + lerp(c1[1], c2[1], t) + ', ' + lerp(c1[2], c2[2], t) + ')';
                                    const heat = (t) => {{
                                        if (t < 0.5) {{
                                            return mix([34, 197, 94], [249, 115, 22], t * 2);
                                        }}
                                        return mix([249, 115, 22], [239, 68, 68], (t - 0.5) * 2);
                                    }};
                                    window.graphLab.instance.nodesGroup.selectAll('g.lab-node').each(function(d) {{
                                        const node = d3.select(this);
                                        if (!d._baseColor) d._baseColor = d.color || '#6366f1';
                                        if (!enabled || map.size === 0) {{
                                            const base = d._baseColor;
                                            node.select('.node-core').attr('fill', base);
                                            node.select('.node-glow').attr('fill', base).style('opacity', 0.35);
                                            return;
                                        }}
                                        const p = map.get(d.id);
                                        if (!p) return;
                                        const dx = (d.x || 0) - p.x;
                                        const dy = (d.y || 0) - p.y;
                                        const dist = Math.sqrt(dx * dx + dy * dy);
                                        const color = heat(Math.min(1, dist / maxDist));
                                        node.select('.node-core').attr('fill', color);
                                        node.select('.node-glow').attr('fill', color).style('opacity', 0.5);
                                    }});
                                }}
                            }}
                            window.graphLab.renderLayoutOverlay();
                        }})()"#,
                        container_id = container_id,
                        data = payload.to_string()
                    );
                    let _ = eval(&js).await;

                    let agent = api::create_agent().await;
                    match api::get_flow_layout_backend(&agent, workflow_id.clone(), Some(graph_version.clone())).await {
                        Ok(Some(layout)) => {
                            let positions = layout
                                .node_positions
                                .iter()
                                .map(|pos| {
                                    let x = pos.x.0.to_string().parse::<i64>().unwrap_or(0);
                                    let y = pos.y.0.to_string().parse::<i64>().unwrap_or(0);
                                    json!({
                                        "id": pos.node_id,
                                        "x": x,
                                        "y": y,
                                    })
                                })
                                .collect::<Vec<_>>();
                            let handles = layout
                                .handle_positions
                                .iter()
                                .map(|handle| {
                                    json!({
                                        "handle_id": handle.handle_id,
                                        "source": handle.source,
                                        "target": handle.target,
                                    })
                                })
                                .collect::<Vec<_>>();
                            let collapsed = layout.collapsed_groups.clone();
                            let heat_enabled = layout_heat_mode();
                            let apply_js = format!(
                                r#"(function() {{
                                    if (!window.graphLab || !window.graphLab.data) return;
                                    const positions = {positions};
                                    const handles = {handles};
                                    const collapsed = {collapsed};
                                    window.graphLab.savedPositions = positions;
                                    window.graphLab.handlePositions = handles;
                                    window.graphLab.collapsedGroups = collapsed;
                                    const map = new Map(positions.map(p => [p.id, p]));
                                    window.graphLab.data.nodes.forEach(n => {{
                                        const p = map.get(n.id);
                                        if (p) {{
                                            n.fx = p.x;
                                            n.fy = p.y;
                                            n.x = p.x;
                                            n.y = p.y;
                                        }}
                                    }});
                                    if (window.graphLab.instance && window.graphLab.instance.simulation) {{
                                        window.graphLab.instance.simulation.alpha(0.6).restart();
                                    }}
                                    if (window.graphLab.renderLayoutOverlay) {{
                                        window.graphLab.renderLayoutOverlay();
                                    }}
                                    if (window.graphLab.applyHeatMode) {{
                                        window.graphLab.applyHeatMode({heat});
                                    }}
                                }})()"#,
                                positions = serde_json::to_string(&positions).unwrap_or("[]".to_string()),
                                handles = serde_json::to_string(&handles).unwrap_or("[]".to_string()),
                                collapsed = serde_json::to_string(&collapsed).unwrap_or("[]".to_string()),
                                heat = heat_enabled
                            );
                            let _ = eval(&apply_js).await;
                            layout_status.set(Some("Layout loaded".to_string()));
                            layout_dirty.set(false);
                            layout_saved_by.set(Some(truncate_principal(layout.updated_by)));
                            layout_saved_at.set(Some(format_saved_at(layout.updated_at)));
                            layout_commit_message.set(layout.commit_message.clone().unwrap_or_default());
                            layout_commit_tag.set(layout.commit_tag.clone().unwrap_or_default());
                            layout_diff_preview.set(None);
                            layout_handle_positions.set(layout.handle_positions.clone());
                            layout_collapsed_groups.set(layout.collapsed_groups.clone());
                            layout_history_loading.set(true);
                            layout_history_error.set(None);
                            match api::get_flow_layout_history_backend(
                                &agent,
                                workflow_id.clone(),
                                Some(graph_version.clone()),
                                Some(10),
                            )
                            .await
                            {
                                Ok(entries) => {
                                    layout_history.set(entries);
                                }
                                Err(err) => {
                                    layout_history_error.set(Some(err));
                                }
                            }
                            layout_history_loading.set(false);
                        }
                        Ok(None) => {
                            layout_status.set(Some("No saved layout".to_string()));
                            layout_saved_by.set(None);
                            layout_saved_at.set(None);
                            layout_commit_message.set("".to_string());
                            layout_commit_tag.set("".to_string());
                            layout_diff_preview.set(None);
                            layout_history.set(Vec::new());
                            layout_handle_positions.set(Vec::new());
                            layout_collapsed_groups.set(Vec::new());
                            layout_history_loading.set(false);
                            layout_history_error.set(None);
                        }
                        Err(err) => {
                            layout_status.set(Some(format!("Layout load failed: {}", err)));
                        }
                    }
                });
            }
        });
    }

    // Poll for layout drift vs saved positions
    {
        let mut layout_dirty = layout_dirty.clone();
        let mut layout_delta_count = layout_delta_count.clone();
        let mut layout_delta_list = layout_delta_list.clone();
        let overlay_cleared = overlay_cleared.clone();
        let layout_heat_mode = layout_heat_mode.clone();
        use_future(move || async move {
            loop {
                gloo_timers::future::TimeoutFuture::new(2000).await;
                let js = r#"(function() {
                    if (!window.graphLab || !window.graphLab.data || !window.graphLab.savedPositions) return false;
                    const map = new Map(window.graphLab.savedPositions.map(p => [p.id, p]));
                    const tol = 2;
                    for (const n of window.graphLab.data.nodes) {
                        const p = map.get(n.id);
                        if (!p) return true;
                        const dx = Math.abs((n.x || 0) - p.x);
                        const dy = Math.abs((n.y || 0) - p.y);
                        if (dx > tol || dy > tol) return true;
                    }
                    return false;
                })()"#;
                if let Ok(val) = eval(js).await {
                    if let Some(is_dirty) = val.as_bool() {
                        layout_dirty.set(is_dirty);
                    }
                }
                let delta_payload = eval(
                    r#"(function() {
                        if (!window.graphLab || !window.graphLab.data || !window.graphLab.savedPositions) {
                            return JSON.stringify({ count: 0, deltas: [] });
                        }
                        const map = new Map(window.graphLab.savedPositions.map(p => [p.id, p]));
                        const tol = 2;
                        const deltas = [];
                        let count = 0;
                        for (const n of window.graphLab.data.nodes) {
                            const p = map.get(n.id);
                            if (!p) { count++; continue; }
                            const dx = Math.round((n.x || 0) - p.x);
                            const dy = Math.round((n.y || 0) - p.y);
                            const dist = Math.sqrt(dx * dx + dy * dy);
                            if (Math.abs(dx) > tol || Math.abs(dy) > tol) {
                                count++;
                                deltas.push({ id: n.id, dx, dy, dist });
                            }
                        }
                        deltas.sort((a, b) => b.dist - a.dist);
                        return JSON.stringify({ count, deltas: deltas.slice(0, 6) });
                    })()"#,
                )
                .await;
                if let Ok(val) = delta_payload {
                    if let Some(raw) = val.as_str() {
                        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(raw) {
                            if let Some(count) = parsed.get("count").and_then(|v| v.as_u64()) {
                                layout_delta_count.set(count as usize);
                            }
                            if let Some(deltas) = parsed.get("deltas").and_then(|v| v.as_array()) {
                                let list = deltas
                                    .iter()
                                    .filter_map(|d| {
                                        Some((
                                            d.get("id")?.as_str()?.to_string(),
                                            d.get("dx")?.as_i64()?,
                                            d.get("dy")?.as_i64()?,
                                            d.get("dist")?.as_f64()?,
                                        ))
                                    })
                                    .collect::<Vec<_>>();
                                layout_delta_list.set(list);
                            }
                        }
                    }
                }
                if !overlay_cleared() {
                    let _ = eval("window.graphLab && window.graphLab.renderLayoutOverlay && window.graphLab.renderLayoutOverlay();").await;
                }
                if layout_heat_mode() {
                    let _ = eval("window.graphLab && window.graphLab.applyHeatMode && window.graphLab.applyHeatMode(true);").await;
                }
            }
        });
    }

    let on_save_layout = {
        let workflow_id = workflow_id_layout.clone();
        let flow_graph = flow_graph.clone();
        let mut save_status = save_status.clone();
        let mut saving = saving.clone();
        let mut layout_dirty = layout_dirty.clone();
        let mut layout_status = layout_status.clone();
        let mut layout_saved_by = layout_saved_by.clone();
        let mut layout_saved_at = layout_saved_at.clone();
        let mut layout_history = layout_history.clone();
        let mut layout_history_loading = layout_history_loading.clone();
        let mut layout_history_error = layout_history_error.clone();
        let layout_commit_message = layout_commit_message.clone();
        let layout_commit_tag = layout_commit_tag.clone();
        let mut layout_diff_preview = layout_diff_preview.clone();
        let mut layout_handle_positions = layout_handle_positions.clone();
        let mut layout_collapsed_groups = layout_collapsed_groups.clone();
        let layout_heat_mode = layout_heat_mode.clone();
        move |_| {
            let workflow_id = workflow_id.clone();
            let flow_graph = flow_graph();
            if flow_graph.is_none() {
                save_status.set(Some("No graph loaded".to_string()));
                return;
            }
            let graph_version = flow_graph.unwrap().version;
            saving.set(true);
            save_status.set(None);
            layout_status.set(Some("Saving layout".to_string()));

            spawn(async move {
                let json_layout = eval(
                    r#"(function() {
                        if (!window.graphLab || !window.graphLab.data) return "";
                        const nodes = window.graphLab.data.nodes || [];
                        const handles = (window.graphLab.handlePositions || []).map(h => ({
                            handle_id: h.handle_id || h.handleId || h.id || "",
                            source: h.source || "",
                            target: h.target || ""
                        })).filter(h => h.handle_id && h.source && h.target);
                        const collapsed = (window.graphLab.collapsedGroups || []).filter(Boolean);
                        return JSON.stringify({
                            nodes: nodes.map(n => ({
                                id: n.id,
                                x: Math.round(n.x || 0),
                                y: Math.round(n.y || 0)
                            })),
                            handles,
                            collapsed
                        });
                    })()"#,
                )
                .await;

                let bundle = match json_layout.ok().and_then(|v| v.as_str().map(|s| s.to_string())) {
                    Some(raw) if !raw.is_empty() => serde_json::from_str::<LayoutBundle>(&raw)
                        .unwrap_or(LayoutBundle {
                            nodes: Vec::new(),
                            handles: Vec::new(),
                            collapsed: Vec::new(),
                        }),
                    _ => LayoutBundle {
                        nodes: Vec::new(),
                        handles: Vec::new(),
                        collapsed: Vec::new(),
                    },
                };

                let node_positions = bundle
                    .nodes
                    .into_iter()
                    .map(|pt| FlowNodePosition {
                        node_id: pt.id,
                        x: pt.x.into(),
                        y: pt.y.into(),
                    })
                    .collect::<Vec<_>>();

                let handle_positions = bundle
                    .handles
                    .into_iter()
                    .map(|handle| FlowHandlePosition {
                        handle_id: handle.handle_id,
                        source: handle.source,
                        target: handle.target,
                    })
                    .collect::<Vec<_>>();
                let commit_message_raw = layout_commit_message();
                let commit_message = {
                    let trimmed = commit_message_raw.trim();
                    if trimmed.is_empty() {
                        None
                    } else {
                        Some(trimmed.to_string())
                    }
                };
                let commit_tag_raw = layout_commit_tag();
                let commit_tag = {
                    let trimmed = commit_tag_raw.trim();
                    if trimmed.is_empty() {
                        None
                    } else {
                        Some(trimmed.to_string())
                    }
                };

                let input = FlowLayoutInput {
                    workflow_id,
                    graph_version,
                    node_positions,
                    handle_positions,
                    collapsed_groups: bundle.collapsed,
                    commit_message,
                    commit_tag,
                };

                let agent = api::create_agent().await;
                let result = api::set_flow_layout_backend(&agent, input.clone()).await;
                saving.set(false);
                match result {
                    Ok(layout) => {
                        let _ = api::set_flow_layout(&agent, input.clone()).await;
                        layout_dirty.set(false);
                        layout_status.set(Some("Layout saved".to_string()));
                        save_status.set(Some("Layout saved".to_string()));
                        layout_saved_by.set(Some(truncate_principal(layout.updated_by)));
                        layout_saved_at.set(Some(format_saved_at(layout.updated_at)));
                        layout_diff_preview.set(None);
                        layout_handle_positions.set(input.handle_positions.clone());
                        layout_collapsed_groups.set(input.collapsed_groups.clone());
                        let history_workflow_id = input.workflow_id.clone();
                        let history_graph_version = input.graph_version.clone();
                        let positions = input
                            .node_positions
                            .iter()
                            .map(|pos| {
                                let x = pos.x.0.to_string().parse::<i64>().unwrap_or(0);
                                let y = pos.y.0.to_string().parse::<i64>().unwrap_or(0);
                                serde_json::json!({
                                    "id": pos.node_id,
                                    "x": x,
                                    "y": y,
                                })
                            })
                            .collect::<Vec<_>>();
                        let handles = input
                            .handle_positions
                            .iter()
                            .map(|handle| {
                                serde_json::json!({
                                    "handle_id": handle.handle_id,
                                    "source": handle.source,
                                    "target": handle.target,
                                })
                            })
                            .collect::<Vec<_>>();
                        let collapsed = input.collapsed_groups.clone();
                        let heat_enabled = layout_heat_mode();
                        let apply_js = format!(
                            r#"(function() {{
                                if (!window.graphLab || !window.graphLab.data) return;
                                const positions = {positions};
                                const handles = {handles};
                                const collapsed = {collapsed};
                                window.graphLab.savedPositions = positions;
                                window.graphLab.handlePositions = handles;
                                window.graphLab.collapsedGroups = collapsed;
                                const map = new Map(positions.map(p => [p.id, p]));
                                window.graphLab.data.nodes.forEach(n => {{
                                    const p = map.get(n.id);
                                    if (p) {{
                                        n.fx = p.x;
                                        n.fy = p.y;
                                        n.x = p.x;
                                        n.y = p.y;
                                    }}
                                }});
                                if (window.graphLab.renderLayoutOverlay) {{
                                    window.graphLab.renderLayoutOverlay();
                                }}
                                if (window.graphLab.applyHeatMode) {{
                                    window.graphLab.applyHeatMode({heat});
                                }}
                            }})()"#,
                            positions = serde_json::to_string(&positions).unwrap_or("[]".to_string()),
                            handles = serde_json::to_string(&handles).unwrap_or("[]".to_string()),
                            collapsed = serde_json::to_string(&collapsed).unwrap_or("[]".to_string()),
                            heat = heat_enabled
                        );
                        let _ = eval(&apply_js).await;
                        layout_history_loading.set(true);
                        layout_history_error.set(None);
                        match api::get_flow_layout_history_backend(
                            &agent,
                            history_workflow_id,
                            Some(history_graph_version),
                            Some(10),
                        )
                        .await
                        {
                            Ok(entries) => {
                                layout_history.set(entries);
                            }
                            Err(err) => {
                                layout_history_error.set(Some(err));
                            }
                        }
                        layout_history_loading.set(false);
                    }
                    Err(err) => save_status.set(Some(format!("Save failed: {}", err))),
                }
            });
        }
    };

    let on_refresh_layout = {
        let mut layout_refreshing = layout_refreshing.clone();
        let mut layout_status = layout_status.clone();
        let mut layout_dirty = layout_dirty.clone();
        let mut layout_saved_by = layout_saved_by.clone();
        let mut layout_saved_at = layout_saved_at.clone();
        let mut layout_history = layout_history.clone();
        let mut layout_history_loading = layout_history_loading.clone();
        let mut layout_history_error = layout_history_error.clone();
        let mut layout_commit_message = layout_commit_message.clone();
        let mut layout_commit_tag = layout_commit_tag.clone();
        let mut layout_diff_preview = layout_diff_preview.clone();
        let mut layout_handle_positions = layout_handle_positions.clone();
        let mut layout_collapsed_groups = layout_collapsed_groups.clone();
        let flow_graph = flow_graph.clone();
        let workflow_id = workflow_id_label.clone();
        let layout_heat_mode = layout_heat_mode.clone();
        move |_| {
            if layout_refreshing() {
                return;
            }
            let Some(graph) = flow_graph() else {
                layout_status.set(Some("No graph loaded".to_string()));
                return;
            };
            layout_refreshing.set(true);
            let graph_version = graph.version;
            let workflow_id = workflow_id.clone();
            spawn(async move {
                let agent = api::create_agent().await;
                match api::get_flow_layout_backend(&agent, workflow_id.clone(), Some(graph_version.clone())).await {
                    Ok(Some(layout)) => {
                        let positions = layout
                            .node_positions
                            .iter()
                            .map(|pos| {
                                let x = pos.x.0.to_string().parse::<i64>().unwrap_or(0);
                                let y = pos.y.0.to_string().parse::<i64>().unwrap_or(0);
                                json!({
                                    "id": pos.node_id,
                                    "x": x,
                                    "y": y,
                                })
                            })
                            .collect::<Vec<_>>();
                        let handles = layout
                            .handle_positions
                            .iter()
                            .map(|handle| {
                                json!({
                                    "handle_id": handle.handle_id,
                                    "source": handle.source,
                                    "target": handle.target,
                                })
                            })
                            .collect::<Vec<_>>();
                        let collapsed = layout.collapsed_groups.clone();
                        let heat_enabled = layout_heat_mode();
                        let apply_js = format!(
                            r#"(function() {{
                                if (!window.graphLab || !window.graphLab.data) return;
                                const positions = {positions};
                                const handles = {handles};
                                const collapsed = {collapsed};
                                window.graphLab.savedPositions = positions;
                                window.graphLab.handlePositions = handles;
                                window.graphLab.collapsedGroups = collapsed;
                                const map = new Map(positions.map(p => [p.id, p]));
                                window.graphLab.data.nodes.forEach(n => {{
                                    const p = map.get(n.id);
                                    if (p) {{
                                        n.fx = p.x;
                                        n.fy = p.y;
                                        n.x = p.x;
                                        n.y = p.y;
                                    }}
                                }});
                                if (window.graphLab.instance && window.graphLab.instance.simulation) {{
                                    window.graphLab.instance.simulation.alpha(0.6).restart();
                                }}
                                if (window.graphLab.renderLayoutOverlay) {{
                                    window.graphLab.renderLayoutOverlay();
                                }}
                                if (window.graphLab.applyHeatMode) {{
                                    window.graphLab.applyHeatMode({heat});
                                }}
                            }})()"#,
                            positions = serde_json::to_string(&positions).unwrap_or("[]".to_string()),
                            handles = serde_json::to_string(&handles).unwrap_or("[]".to_string()),
                            collapsed = serde_json::to_string(&collapsed).unwrap_or("[]".to_string()),
                            heat = heat_enabled
                        );
                        let _ = eval(&apply_js).await;
                        layout_status.set(Some("Layout refreshed".to_string()));
                        layout_dirty.set(false);
                        layout_saved_by.set(Some(truncate_principal(layout.updated_by)));
                        layout_saved_at.set(Some(format_saved_at(layout.updated_at)));
                        layout_commit_message.set(layout.commit_message.clone().unwrap_or_default());
                        layout_commit_tag.set(layout.commit_tag.clone().unwrap_or_default());
                        layout_diff_preview.set(None);
                        layout_handle_positions.set(layout.handle_positions.clone());
                        layout_collapsed_groups.set(layout.collapsed_groups.clone());
                        layout_history_loading.set(true);
                        layout_history_error.set(None);
                        match api::get_flow_layout_history_backend(
                            &agent,
                            workflow_id.clone(),
                            Some(graph_version),
                            Some(10),
                        )
                        .await
                        {
                            Ok(entries) => {
                                layout_history.set(entries);
                            }
                            Err(err) => {
                                layout_history_error.set(Some(err));
                            }
                        }
                        layout_history_loading.set(false);
                    }
                    Ok(None) => {
                        layout_status.set(Some("No saved layout".to_string()));
                        layout_saved_by.set(None);
                        layout_saved_at.set(None);
                        layout_commit_message.set("".to_string());
                        layout_commit_tag.set("".to_string());
                        layout_diff_preview.set(None);
                        layout_history.set(Vec::new());
                        layout_handle_positions.set(Vec::new());
                        layout_collapsed_groups.set(Vec::new());
                        layout_history_loading.set(false);
                        layout_history_error.set(None);
                    }
                    Err(err) => {
                        layout_status.set(Some(format!("Refresh failed: {}", err)));
                    }
                }
                layout_refreshing.set(false);
            });
        }
    };

    let on_reset_layout = {
        let mut layout_resetting = layout_resetting.clone();
        let mut layout_status = layout_status.clone();
        let mut layout_dirty = layout_dirty.clone();
        let layout_heat_mode = layout_heat_mode.clone();
        move |_| {
            if layout_resetting() {
                return;
            }
            layout_resetting.set(true);
            spawn(async move {
                let js = r#"(function() {
                    if (!window.graphLab || !window.graphLab.data || !window.graphLab.savedPositions) return false;
                    const map = new Map(window.graphLab.savedPositions.map(p => [p.id, p]));
                    window.graphLab.data.nodes.forEach(n => {
                        const p = map.get(n.id);
                        if (p) {
                            n.fx = p.x;
                            n.fy = p.y;
                            n.x = p.x;
                            n.y = p.y;
                        }
                    });
                    if (window.graphLab.instance && window.graphLab.instance.simulation) {
                        window.graphLab.instance.simulation.alpha(0.6).restart();
                    }
                    if (window.graphLab.renderLayoutOverlay) {
                        window.graphLab.renderLayoutOverlay();
                    }
                    return true;
                })()"#;
                let mut ok = false;
                if let Ok(val) = eval(js).await {
                    ok = val.as_bool().unwrap_or(false);
                }
                if ok {
                    layout_status.set(Some("Layout reset to saved".to_string()));
                    layout_dirty.set(false);
                    if layout_heat_mode() {
                        let _ = eval("window.graphLab && window.graphLab.applyHeatMode && window.graphLab.applyHeatMode(true);").await;
                    }
                } else {
                    layout_status.set(Some("No saved layout to reset".to_string()));
                }
                layout_resetting.set(false);
            });
        }
    };

    let on_toggle_lock = {
        let mut layout_locked = layout_locked.clone();
        move |_| {
            let next = !layout_locked();
            layout_locked.set(next);
            spawn(async move {
                let js = if next {
                    r#"(function() {
                        if (!window.graphLab || !window.graphLab.data) return;
                        window.graphLab.layoutLocked = true;
                        window.graphLab.data.nodes.forEach(n => {
                            n.fx = n.x || 0;
                            n.fy = n.y || 0;
                        });
                        if (window.graphLab.instance && window.graphLab.instance.simulation) {
                            window.graphLab.instance.simulation.alphaTarget(0).restart();
                        }
                    })()"#
                } else {
                    r#"(function() {
                        if (!window.graphLab || !window.graphLab.data) return;
                        window.graphLab.layoutLocked = false;
                        window.graphLab.data.nodes.forEach(n => {
                            n.fx = null;
                            n.fy = null;
                        });
                        if (window.graphLab.instance && window.graphLab.instance.simulation) {
                            window.graphLab.instance.simulation.alpha(0.6).restart();
                        }
                    })()"#
                };
                let _ = eval(js).await;
            });
        }
    };



    let on_autolayout = {
        let mut layout_autolayout = layout_autolayout.clone();
        let mut layout_status = layout_status.clone();
        move |_| {
            if layout_autolayout() {
                return;
            }
            layout_autolayout.set(true);
            layout_status.set(Some("Auto-layout running".to_string()));
            spawn(async move {
                let js = r#"(function() {
                    if (!window.graphLab || !window.graphLab.data) return false;
                    window.graphLab.data.nodes.forEach(n => {
                        n.fx = null;
                        n.fy = null;
                    });
                    if (window.graphLab.instance && window.graphLab.instance.simulation) {
                        window.graphLab.instance.simulation.alpha(1.0).restart();
                    }
                    return true;
                })()"#;
                let mut ok = false;
                if let Ok(val) = eval(js).await {
                    ok = val.as_bool().unwrap_or(false);
                }
                if ok {
                    layout_status.set(Some("Auto-layout complete".to_string()));
                } else {
                    layout_status.set(Some("Auto-layout failed".to_string()));
                }
                layout_autolayout.set(false);
            });
        }
    };

    let on_toggle_inspector = {
        let mut layout_inspector_open = layout_inspector_open.clone();
        let mut overlay_cleared = overlay_cleared.clone();
        let flow_graph = flow_graph.clone();
        let workflow_id = workflow_id_label.clone();
        let mut layout_history = layout_history.clone();
        let mut layout_history_loading = layout_history_loading.clone();
        let mut layout_history_error = layout_history_error.clone();
        move |_| {
            let next = !layout_inspector_open();
            layout_inspector_open.set(next);
            overlay_cleared.set(false);
            if next {
                if let Some(graph) = flow_graph() {
                    let workflow_id = workflow_id.clone();
                    let graph_version = graph.version;
                    layout_history_loading.set(true);
                    layout_history_error.set(None);
                    spawn(async move {
                        let agent = api::create_agent().await;
                        match api::get_flow_layout_history_backend(
                            &agent,
                            workflow_id,
                            Some(graph_version),
                            Some(10),
                        )
                        .await
                        {
                            Ok(entries) => {
                                layout_history.set(entries);
                            }
                            Err(err) => {
                                layout_history_error.set(Some(err));
                            }
                        }
                        layout_history_loading.set(false);
                    });
                }
                spawn(async move {
                    let _ = eval(
                        r#"(function() {
                            if (!window.graphLab) return;
                            window.graphLab.overlayPaused = false;
                            if (window.graphLab.renderLayoutOverlay) {
                                window.graphLab.renderLayoutOverlay();
                            }
                        })()"#,
                    )
                    .await;
                });
            }
        }
    };

    let on_toggle_compare = {
        let mut layout_compare_mode = layout_compare_mode.clone();
        let mut overlay_cleared = overlay_cleared.clone();
        move |_| {
            let next = !layout_compare_mode();
            layout_compare_mode.set(next);
            overlay_cleared.set(false);
            spawn(async move {
                let js = if next {
                    r#"(function() {
                        if (!window.graphLab || !window.graphLab.data || !window.graphLab.savedPositions) return;
                        window.graphLab.overlayPaused = false;
                        const map = new Map(window.graphLab.savedPositions.map(p => [p.id, p]));
                        window.graphLab.data.nodes.forEach(n => {
                            const p = map.get(n.id);
                            if (p) {
                                n._compare = { x: p.x, y: p.y };
                            }
                        });
                        if (!window.graphLab.renderCompareOverlay) {
                            window.graphLab.renderCompareOverlay = function() {
                                if (!window.graphLab || !window.graphLab.instance || !window.graphLab.data) return;
                                if (window.graphLab.overlayPaused) return;
                                const svg = window.graphLab.instance.svg;
                                if (!svg) return;
                                let overlay = svg.select('g.compare-overlay');
                                if (overlay.empty()) {
                                    overlay = svg.append('g').attr('class', 'compare-overlay');
                                }
                                overlay.selectAll('*').remove();
                                const nodes = window.graphLab.data.nodes || [];
                                overlay.selectAll('circle')
                                    .data(nodes.filter(n => n._compare), d => d.id)
                                    .enter()
                                    .append('circle')
                                    .attr('cx', d => d._compare.x)
                                    .attr('cy', d => d._compare.y)
                                    .attr('r', 6)
                                    .attr('fill', 'rgba(56, 189, 248, 0.15)')
                                    .attr('stroke', 'rgba(56, 189, 248, 0.7)')
                                    .attr('stroke-width', 1.2);
                            }
                        }
                        window.graphLab.renderCompareOverlay();
                    })()"#
                } else {
                    r#"(function() {
                        if (!window.graphLab || !window.graphLab.instance) return;
                        const svg = window.graphLab.instance.svg;
                        if (!svg) return;
                        svg.select('g.compare-overlay').selectAll('*').remove();
                    })()"#
                };
                let _ = eval(js).await;
            });
        }
    };

    let on_toggle_heat = {
        let mut layout_heat_mode = layout_heat_mode.clone();
        move |_| {
            let next = !layout_heat_mode();
            layout_heat_mode.set(next);
            spawn(async move {
                let js = format!(
                    "window.graphLab && window.graphLab.applyHeatMode && window.graphLab.applyHeatMode({});",
                    if next { "true" } else { "false" }
                );
                let _ = eval(&js).await;
            });
        }
    };


    let on_clear_overlays = {
        let mut overlay_cleared = overlay_cleared.clone();
        let mut layout_status = layout_status.clone();
        move |_| {
            overlay_cleared.set(true);
            layout_status.set(Some("Overlays cleared".to_string()));
            spawn(async move {
                let js = r#"(function() {
                    if (!window.graphLab || !window.graphLab.instance) return;
                    window.graphLab.overlayPaused = true;
                    const svg = window.graphLab.instance.svg;
                    if (!svg) return;
                    svg.select('g.layout-overlay').selectAll('*').remove();
                    svg.select('g.compare-overlay').selectAll('*').remove();
                })()"#;
                let _ = eval(js).await;
            });
        }
    };

    let on_snap_moved = {
        let mut layout_status = layout_status.clone();
        let mut layout_dirty = layout_dirty.clone();
        move |_| {
            spawn(async move {
                let js = r#"(function() {
                    if (!window.graphLab || !window.graphLab.data || !window.graphLab.savedPositions) return false;
                    const map = new Map(window.graphLab.savedPositions.map(p => [p.id, p]));
                    const tol = 2;
                    window.graphLab.data.nodes.forEach(n => {
                        const p = map.get(n.id);
                        if (!p) return;
                        const dx = Math.abs((n.x || 0) - p.x);
                        const dy = Math.abs((n.y || 0) - p.y);
                        if (dx > tol || dy > tol) {
                            n.fx = p.x;
                            n.fy = p.y;
                            n.x = p.x;
                            n.y = p.y;
                        }
                    });
                    if (window.graphLab.instance && window.graphLab.instance.simulation) {
                        window.graphLab.instance.simulation.alpha(0.4).restart();
                    }
                    if (window.graphLab.renderLayoutOverlay) {
                        window.graphLab.renderLayoutOverlay();
                    }
                    return true;
                })()"#;
                let mut ok = false;
                if let Ok(val) = eval(js).await {
                    ok = val.as_bool().unwrap_or(false);
                }
                if ok {
                    layout_status.set(Some("Snapped moved nodes".to_string()));
                    layout_dirty.set(false);
                } else {
                    layout_status.set(Some("Snap failed".to_string()));
                }
            });
        }
    };

    let on_add_handle = {
        let mut layout_handle_positions = layout_handle_positions.clone();
        let mut new_handle_id = new_handle_id.clone();
        let mut new_handle_source = new_handle_source.clone();
        let mut new_handle_target = new_handle_target.clone();
        let mut layout_status = layout_status.clone();
        move |_| {
            let source = new_handle_source().trim().to_string();
            let target = new_handle_target().trim().to_string();
            if source.is_empty() || target.is_empty() {
                layout_status.set(Some("Handle source + target required".to_string()));
                return;
            }
            let id_input = new_handle_id().trim().to_string();
            let handle_id = if id_input.is_empty() {
                format!("handle:{}:{}", source, target)
            } else {
                id_input
            };
            let mut next = layout_handle_positions();
            if next.iter().any(|h| h.handle_id == handle_id) {
                layout_status.set(Some("Handle id already exists".to_string()));
                return;
            }
            next.push(FlowHandlePosition {
                handle_id: handle_id.clone(),
                source: source.clone(),
                target: target.clone(),
            });
            layout_handle_positions.set(next.clone());
            new_handle_id.set("".to_string());
            new_handle_source.set("".to_string());
            new_handle_target.set("".to_string());
            let payload = serde_json::to_string(&next).unwrap_or("[]".to_string());
            spawn(async move {
                let js = format!(
                    "window.graphLab && (window.graphLab.handlePositions = {});",
                    payload
                );
                let _ = eval(&js).await;
            });
        }
    };

    let on_add_collapsed_group = {
        let mut layout_collapsed_groups = layout_collapsed_groups.clone();
        let mut new_collapsed_group = new_collapsed_group.clone();
        let mut layout_status = layout_status.clone();
        move |_| {
            let group = new_collapsed_group().trim().to_string();
            if group.is_empty() {
                layout_status.set(Some("Collapsed group id required".to_string()));
                return;
            }
            let mut next = layout_collapsed_groups();
            if next.iter().any(|g| g == &group) {
                layout_status.set(Some("Group already listed".to_string()));
                return;
            }
            next.push(group.clone());
            layout_collapsed_groups.set(next.clone());
            new_collapsed_group.set("".to_string());
            let payload = serde_json::to_string(&next).unwrap_or("[]".to_string());
            spawn(async move {
                let js = format!(
                    "window.graphLab && (window.graphLab.collapsedGroups = {});",
                    payload
                );
                let _ = eval(&js).await;
            });
        }
    };

    let on_export_delta = {
        let flow_graph = flow_graph.clone();
        let workflow_id = workflow_id_label.clone();
        let mut layout_status = layout_status.clone();
        move |_| {
            let graph_version = flow_graph()
                .map(|graph| graph.version)
                .unwrap_or_else(|| "unknown".to_string());
            let workflow_id = workflow_id.clone();
            spawn(async move {
                let workflow_json =
                    serde_json::to_string(&workflow_id).unwrap_or("\"workflow\"".to_string());
                let graph_json =
                    serde_json::to_string(&graph_version).unwrap_or("\"graph\"".to_string());
                let js = format!(
                    r#"(function() {{
                        if (!window.graphLab || !window.graphLab.data || !window.graphLab.savedPositions) return false;
                        const workflowId = {workflow_id};
                        const graphVersion = {graph_version};
                        const map = new Map(window.graphLab.savedPositions.map(p => [p.id, p]));
                        const tol = 2;
                        const deltas = [];
                        const nodes = window.graphLab.data.nodes || [];
                        for (const n of nodes) {{
                            const p = map.get(n.id);
                            if (!p) continue;
                            const dx = Math.round((n.x || 0) - p.x);
                            const dy = Math.round((n.y || 0) - p.y);
                            const dist = Math.sqrt(dx * dx + dy * dy);
                            if (Math.abs(dx) > tol || Math.abs(dy) > tol) {{
                                deltas.push({{ id: n.id, dx, dy, dist }});
                            }}
                        }}
                        const report = {{
                            workflow_id: workflowId,
                            graph_version: graphVersion,
                            generated_at: new Date().toISOString(),
                            node_count: nodes.length,
                            delta_count: deltas.length,
                            deltas,
                            handle_positions_count: (window.graphLab.handlePositions || []).length,
                            collapsed_groups_count: (window.graphLab.collapsedGroups || []).length
                        }};
                        const blob = new Blob([JSON.stringify(report, null, 2)], {{ type: 'application/json' }});
                        const url = URL.createObjectURL(blob);
                        const safeWorkflow = String(workflowId || 'workflow').replace(/[^a-z0-9-_]+/gi, '_');
                        const safeVersion = String(graphVersion || 'graph').replace(/[^a-z0-9-_]+/gi, '_');
                        const link = document.createElement('a');
                        link.href = url;
                        link.download = 'layout_delta_' + safeWorkflow + '_' + safeVersion + '.json';
                        document.body.appendChild(link);
                        link.click();
                        link.remove();
                        setTimeout(() => URL.revokeObjectURL(url), 1000);
                        return true;
                    }})()"#,
                    workflow_id = workflow_json,
                    graph_version = graph_json
                );
                let mut ok = false;
                if let Ok(val) = eval(&js).await {
                    ok = val.as_bool().unwrap_or(false);
                }
                if ok {
                    layout_status.set(Some("Delta report exported".to_string()));
                } else {
                    layout_status.set(Some("Delta export failed".to_string()));
                }
            });
        }
    };

    let (node_count, edge_count, graph_version, node_ids) = match flow_graph() {
        Some(ref graph) => (
            graph.nodes.len(),
            graph.edges.len(),
            graph.version.clone(),
            graph.nodes.iter().map(|node| node.id.clone()).collect::<Vec<_>>(),
        ),
        None => (0, 0, "—".to_string(), Vec::new()),
    };
    let history_stub = vec![
        ("Baseline layout", "Imported workflow map"),
        ("Auto-layout pass", "System stabilization"),
        ("Session adjustments", "Local edits"),
    ];
    let history_entries = layout_history();
    let history_recent = history_entries
        .iter()
        .rev()
        .take(6)
        .cloned()
        .collect::<Vec<_>>();
    let mut layout_commit_message_input = layout_commit_message.clone();
    let mut layout_commit_tag_input = layout_commit_tag.clone();
    let mut new_handle_id_input = new_handle_id.clone();
    let mut new_handle_source_input = new_handle_source.clone();
    let mut new_handle_target_input = new_handle_target.clone();
    let mut new_collapsed_group_input = new_collapsed_group.clone();

    rsx! {
        div {
            class: "flex flex-col h-full w-full gap-6 p-6",
            style: "font-family: 'IBM Plex Sans', sans-serif;",
            div { class: "flex items-center justify-between",
                div { class: "flex flex-col",
                    span { class: "uppercase tracking-[0.32em] text-[10px] text-[#94A3B8]", "Cortex Workbench" }
                    h2 {
                        class: "text-3xl text-[#E2E8F0] tracking-[0.08em]",
                        style: "font-family: 'Bebas Neue', sans-serif;",
                        "Execution Observatory"
                    }
                }
                div { class: "flex items-center gap-4 text-xs text-[#94A3B8]",
                    div { class: "px-3 py-1 rounded-full border border-[#1E293B] bg-black/20", "Nodes: {node_count}" }
                    div { class: "px-3 py-1 rounded-full border border-[#1E293B] bg-black/20", "Edges: {edge_count}" }
                    div { class: "px-3 py-1 rounded-full border border-[#1E293B] bg-black/20", "Graph: {graph_version}" }
                }
            }

            div { class: "flex items-center gap-2",
                TabButton {
                    label: "Flow Graph",
                    active: tab() == WorkbenchTab::FlowGraph,
                    onclick: move |_| tab.set(WorkbenchTab::FlowGraph),
                }
                TabButton {
                    label: "Traces",
                    active: tab() == WorkbenchTab::Traces,
                    onclick: move |_| tab.set(WorkbenchTab::Traces),
                }
                TabButton {
                    label: "Logs",
                    active: tab() == WorkbenchTab::Logs,
                    onclick: move |_| tab.set(WorkbenchTab::Logs),
                }
            }

            match tab() {
                WorkbenchTab::FlowGraph => rsx! {
                    div { class: "flex-1 grid grid-cols-12 gap-6",
                        div { class: "col-span-8 rounded-2xl border border-[#1E293B] bg-[#0B1120]/80 p-6 shadow-[0_0_40px_rgba(15,23,42,0.6)] relative overflow-hidden",
                            div { class: "absolute inset-0 bg-[radial-gradient(circle_at_top,#1F2937,transparent_60%)] opacity-70" }
                            div { class: "absolute inset-0 bg-[linear-gradient(135deg,rgba(14,165,233,0.08),transparent_40%,rgba(34,197,94,0.08))]" }
                            div { class: "relative z-10 flex h-full flex-col gap-4",
                                datalist { id: "workflow-node-ids",
                                    for id in node_ids.iter() {
                                        option { value: "{id}" }
                                    }
                                }
                                div { class: "flex items-center justify-between",
                                    div { class: "text-sm text-[#94A3B8]", "Workflow: {workflow_id_label}" }
                                    div { class: "text-xs text-[#64748B]", "Auto-layout enabled" }
                                }
                                div {
                                    id: "{graph_container_id}",
                                    class: "flex-1 rounded-xl border border-dashed border-[#1E293B] bg-[radial-gradient(#1E293B_1px,transparent_1px)] [background-size:24px_24px] relative overflow-hidden",
                                    if let Some(err) = flow_error() {
                                        div { class: "absolute inset-0 flex items-center justify-center text-sm text-red-400", "Flow graph error: {err}" }
                                    } else if flow_graph().is_none() {
                                        div { class: "absolute inset-0 flex items-center justify-center text-sm text-[#94A3B8]", "Loading flow graph…" }
                                    } else {
                                        div { class: "absolute top-4 right-4 text-[10px] uppercase tracking-[0.25em] text-[#64748B]", "D3 Live" }
                                    }
                                }
                        }
                        div { class: "col-span-4 flex flex-col gap-4",
                            div { class: "rounded-2xl border border-[#1E293B] bg-black/30 p-5",
                                div { class: "text-xs uppercase tracking-[0.2em] text-[#64748B]", "Lineage" }
                                div { class: "mt-3 text-sm text-[#CBD5F5]", "Lineage edges will appear here as contributions link workflows to ideas and decisions." }
                            }
                            div { class: "rounded-2xl border border-[#1E293B] bg-black/30 p-5",
                                div { class: "text-xs uppercase tracking-[0.2em] text-[#64748B]", "Layout" }
                                div { class: "mt-3 text-sm text-[#CBD5F5]", "Layouts persist as Nostra contributions and are cached in Cortex." }
                                div { class: "flex items-center gap-2",
                                    button {
                                        class: "flex-1 rounded-xl border border-[#38BDF8]/30 bg-[#0EA5E9]/10 px-3 py-2 text-[10px] font-semibold uppercase tracking-[0.2em] text-[#BAE6FD] hover:bg-[#0EA5E9]/20 transition disabled:opacity-50",
                                        disabled: layout_refreshing(),
                                        onclick: on_refresh_layout,
                                        if layout_refreshing() { "Refreshing..." } else { "Refresh" }
                                    }
                                    button {
                                        class: "flex-1 rounded-xl border border-[#F97316]/30 bg-[#F97316]/10 px-3 py-2 text-[10px] font-semibold uppercase tracking-[0.2em] text-[#FDBA74] hover:bg-[#F97316]/20 transition disabled:opacity-50",
                                        disabled: layout_resetting(),
                                        onclick: on_reset_layout,
                                        if layout_resetting() { "Resetting..." } else { "Reset" }
                                    }
                                }
                                button {
                                    class: "mt-2 w-full rounded-xl border border-[#06B6D4]/30 bg-[#06B6D4]/10 px-3 py-2 text-[10px] font-semibold uppercase tracking-[0.2em] text-[#A5F3FC] hover:bg-[#06B6D4]/20 transition disabled:opacity-50",
                                    disabled: layout_autolayout(),
                                    onclick: on_autolayout,
                                    if layout_autolayout() { "Auto-layout..." } else { "Auto-layout" }
                                }
                                button {
                                    class: "mt-2 w-full rounded-xl border border-[#A855F7]/30 bg-[#A855F7]/10 px-3 py-2 text-[10px] font-semibold uppercase tracking-[0.2em] text-[#E9D5FF] hover:bg-[#A855F7]/20 transition",
                                    onclick: on_toggle_lock,
                                    if layout_locked() { "Unlock Layout" } else { "Lock Layout" }
                                }
                                button {
                                    class: "mt-2 w-full rounded-xl border border-[#334155] bg-[#0F172A]/60 px-3 py-2 text-[10px] font-semibold uppercase tracking-[0.2em] text-[#CBD5F5] hover:bg-[#1E293B]/70 transition",
                                    onclick: on_toggle_inspector,
                                    if layout_inspector_open() { "Hide Deltas" } else { "Show Deltas" }
                                }
                                button {
                                    class: "mt-2 w-full rounded-xl border border-[#38BDF8]/30 bg-[#0EA5E9]/10 px-3 py-2 text-[10px] font-semibold uppercase tracking-[0.2em] text-[#BAE6FD] hover:bg-[#0EA5E9]/20 transition",
                                    onclick: on_toggle_compare,
                                    if layout_compare_mode() { "Hide Compare" } else { "Compare Saved" }
                                }
                                button {
                                    class: "mt-2 w-full rounded-xl border border-[#F59E0B]/30 bg-[#F59E0B]/10 px-3 py-2 text-[10px] font-semibold uppercase tracking-[0.2em] text-[#FDE68A] hover:bg-[#F59E0B]/20 transition",
                                    onclick: on_toggle_heat,
                                    if layout_heat_mode() { "Heat Drift On" } else { "Heat Drift" }
                                }
                                div { class: "mt-2 grid grid-cols-2 gap-2",
                                    button {
                                        class: "rounded-xl border border-[#475569] bg-[#0F172A]/70 px-3 py-2 text-[10px] font-semibold uppercase tracking-[0.2em] text-[#CBD5F5] hover:bg-[#1E293B]/70 transition",
                                        onclick: on_clear_overlays,
                                        if overlay_cleared() { "Overlays Cleared" } else { "Clear Overlays" }
                                    }
                                    button {
                                        class: "rounded-xl border border-[#22C55E]/30 bg-[#22C55E]/10 px-3 py-2 text-[10px] font-semibold uppercase tracking-[0.2em] text-[#86EFAC] hover:bg-[#22C55E]/20 transition",
                                        onclick: on_snap_moved,
                                        "Snap Moved"
                                    }
                                }
                                div { class: "mt-3 rounded-xl border border-[#1E293B] bg-[#0B1120]/70 p-3",
                                    div { class: "text-[10px] uppercase tracking-[0.2em] text-[#64748B]", "Commit Note" }
                                    input {
                                        class: "mt-2 w-full rounded-lg border border-[#1E293B] bg-[#0F172A]/80 px-3 py-2 text-xs text-[#CBD5F5] focus:outline-none focus:border-[#38BDF8]/60",
                                        placeholder: "Describe this layout change",
                                        value: "{layout_commit_message()}",
                                        oninput: move |evt| layout_commit_message_input.set(evt.value()),
                                    }
                                    input {
                                        class: "mt-2 w-full rounded-lg border border-[#1E293B] bg-[#0F172A]/80 px-3 py-2 text-[10px] uppercase tracking-[0.2em] text-[#94A3B8] focus:outline-none focus:border-[#38BDF8]/60",
                                        placeholder: "Tag (optional)",
                                        value: "{layout_commit_tag()}",
                                        oninput: move |evt| layout_commit_tag_input.set(evt.value()),
                                    }
                                }
                                button {
                                    class: "mt-2 w-full rounded-xl border border-[#38BDF8]/30 bg-[#0EA5E9]/10 px-3 py-2 text-[10px] font-semibold uppercase tracking-[0.2em] text-[#BAE6FD] hover:bg-[#0EA5E9]/20 transition",
                                    onclick: on_export_delta,
                                    "Export Deltas"
                                }
                                button {
                                    class: "mt-4 w-full rounded-xl border border-[#22C55E]/30 bg-[#22C55E]/10 px-4 py-2 text-xs font-semibold uppercase tracking-[0.2em] text-[#86EFAC] hover:bg-[#22C55E]/20 transition disabled:opacity-50",
                                    disabled: saving(),
                                    onclick: on_save_layout,
                                    if saving() { "Saving..." } else { "Save Layout" }
                                }
                                if layout_dirty() {
                                    div { class: "mt-3 inline-flex items-center gap-2 rounded-full border border-[#F97316]/30 bg-[#F97316]/10 px-3 py-1 text-[10px] uppercase tracking-[0.2em] text-[#FDBA74]",
                                        span { class: "h-1.5 w-1.5 rounded-full bg-[#F97316] animate-pulse" }
                                        "Unsaved changes ({layout_delta_count})"
                                    }
                                } else {
                                    div { class: "mt-3 inline-flex items-center gap-2 rounded-full border border-[#22C55E]/30 bg-[#22C55E]/10 px-3 py-1 text-[10px] uppercase tracking-[0.2em] text-[#86EFAC]",
                                        span { class: "h-1.5 w-1.5 rounded-full bg-[#22C55E]" }
                                        "Synced"
                                    }
                                }
                                if let Some(msg) = layout_status() {
                                    div { class: "mt-2 text-[10px] uppercase tracking-[0.2em] text-[#64748B]", "{msg}" }
                                }
                                if let Some(msg) = save_status() {
                                    div { class: "mt-3 text-xs text-[#94A3B8]", "{msg}" }
                                }
                            }
                            if layout_inspector_open() {
                                div { class: "rounded-2xl border border-[#1E293B] bg-black/30 p-5",
                                    div { class: "text-xs uppercase tracking-[0.2em] text-[#64748B]", "Layout Deltas" }
                                    div { class: "mt-3 text-sm text-[#CBD5F5]", "Nodes moved: {layout_delta_count}" }
                                    div { class: "mt-2 text-xs text-[#94A3B8]", "Use Reset to snap back or Save to persist." }
                                    div { class: "mt-4 space-y-2",
                                        for (id, dx, dy, dist) in layout_delta_list() {
                                            div { class: "flex items-center justify-between rounded-lg border border-[#1E293B] bg-[#0F172A]/70 px-3 py-2 text-xs text-[#CBD5F5]",
                                                span { class: "truncate", "{id}" }
                                                span { class: "text-[#94A3B8]", "Δ {dx},{dy} · {dist}" }
                                            }
                                        }
                                    }
                                    div { class: "mt-4 text-xs uppercase tracking-[0.2em] text-[#64748B]", "Handle Links" }
                                    div { class: "mt-2 text-xs text-[#94A3B8]", "Stored: {layout_handle_positions().len()}" }
                                    div { class: "mt-3 grid grid-cols-3 gap-2",
                                        input {
                                            class: "rounded-lg border border-[#1E293B] bg-[#0B1120]/70 px-2 py-2 text-xs text-[#CBD5F5] focus:outline-none focus:border-[#38BDF8]/60",
                                            placeholder: "Handle id",
                                            value: "{new_handle_id()}",
                                            oninput: move |evt| new_handle_id_input.set(evt.value()),
                                        }
                                        div { class: "flex flex-col gap-2",
                                            input {
                                                class: "rounded-lg border border-[#1E293B] bg-[#0B1120]/70 px-2 py-2 text-xs text-[#CBD5F5] focus:outline-none focus:border-[#38BDF8]/60",
                                                placeholder: "Source node",
                                                list: "workflow-node-ids",
                                                value: "{new_handle_source()}",
                                                oninput: move |evt| new_handle_source_input.set(evt.value()),
                                            }
                                            select {
                                                class: "rounded-lg border border-[#1E293B] bg-[#0B1120]/70 px-2 py-2 text-[10px] uppercase tracking-[0.2em] text-[#94A3B8] focus:outline-none focus:border-[#38BDF8]/60",
                                                onchange: move |evt| new_handle_source_input.set(evt.value()),
                                                option { value: "", "Pick Source" }
                                                for id in node_ids.iter() {
                                                    option { value: "{id}", "{id}" }
                                                }
                                            }
                                        }
                                        div { class: "flex flex-col gap-2",
                                            input {
                                                class: "rounded-lg border border-[#1E293B] bg-[#0B1120]/70 px-2 py-2 text-xs text-[#CBD5F5] focus:outline-none focus:border-[#38BDF8]/60",
                                                placeholder: "Target node",
                                                list: "workflow-node-ids",
                                                value: "{new_handle_target()}",
                                                oninput: move |evt| new_handle_target_input.set(evt.value()),
                                            }
                                            select {
                                                class: "rounded-lg border border-[#1E293B] bg-[#0B1120]/70 px-2 py-2 text-[10px] uppercase tracking-[0.2em] text-[#94A3B8] focus:outline-none focus:border-[#38BDF8]/60",
                                                onchange: move |evt| new_handle_target_input.set(evt.value()),
                                                option { value: "", "Pick Target" }
                                                for id in node_ids.iter() {
                                                    option { value: "{id}", "{id}" }
                                                }
                                            }
                                        }
                                    }
                                    button {
                                        class: "mt-2 w-full rounded-lg border border-[#38BDF8]/30 bg-[#0EA5E9]/10 px-3 py-2 text-[10px] font-semibold uppercase tracking-[0.2em] text-[#BAE6FD] hover:bg-[#0EA5E9]/20 transition",
                                        onclick: on_add_handle,
                                        "Add Handle Link"
                                    }
                                    div { class: "mt-3 space-y-2",
                                        for handle in layout_handle_positions() {
                                            div { class: "flex items-center justify-between rounded-lg border border-[#1E293B] bg-[#0B1120]/70 px-3 py-2 text-xs text-[#CBD5F5]",
                                                div { class: "flex flex-col",
                                                    span { class: "truncate", "{handle.handle_id}" }
                                                    span { class: "text-[#94A3B8]", "{handle.source} → {handle.target}" }
                                                }
                                                button {
                                                    class: "rounded-md border border-[#F97316]/40 px-2 py-1 text-[10px] uppercase tracking-[0.2em] text-[#FDBA74] hover:bg-[#F97316]/20 transition",
                                                    onclick: {
                                                        let handle_id = handle.handle_id.clone();
                                                        let mut layout_handle_positions = layout_handle_positions.clone();
                                                        move |_| {
                                                            let mut next = layout_handle_positions();
                                                            next.retain(|h| h.handle_id != handle_id);
                                                            layout_handle_positions.set(next.clone());
                                                            let payload = serde_json::to_string(&next).unwrap_or("[]".to_string());
                                                            spawn(async move {
                                                                let js = format!("window.graphLab && (window.graphLab.handlePositions = {});", payload);
                                                                let _ = eval(&js).await;
                                                            });
                                                        }
                                                    },
                                                    "Remove"
                                                }
                                            }
                                        }
                                    }
                                    div { class: "mt-4 text-xs uppercase tracking-[0.2em] text-[#64748B]", "Collapsed Groups" }
                                    div { class: "mt-2 text-xs text-[#94A3B8]", "Collapsed: {layout_collapsed_groups().len()}" }
                                    div { class: "mt-3 flex items-center gap-2",
                                        input {
                                            class: "flex-1 rounded-lg border border-[#1E293B] bg-[#0B1120]/70 px-2 py-2 text-xs text-[#CBD5F5] focus:outline-none focus:border-[#38BDF8]/60",
                                            placeholder: "Group id",
                                            value: "{new_collapsed_group()}",
                                            oninput: move |evt| new_collapsed_group_input.set(evt.value()),
                                        }
                                        button {
                                            class: "rounded-lg border border-[#22C55E]/30 bg-[#22C55E]/10 px-3 py-2 text-[10px] font-semibold uppercase tracking-[0.2em] text-[#86EFAC] hover:bg-[#22C55E]/20 transition",
                                            onclick: on_add_collapsed_group,
                                            "Add"
                                        }
                                    }
                                    div { class: "mt-3 space-y-2",
                                        for group in layout_collapsed_groups() {
                                            div { class: "flex items-center justify-between rounded-lg border border-[#1E293B] bg-[#0B1120]/70 px-3 py-2 text-xs text-[#CBD5F5]",
                                                span { class: "truncate", "{group}" }
                                                button {
                                                    class: "rounded-md border border-[#F97316]/40 px-2 py-1 text-[10px] uppercase tracking-[0.2em] text-[#FDBA74] hover:bg-[#F97316]/20 transition",
                                                    onclick: {
                                                        let group_id = group.clone();
                                                        let mut layout_collapsed_groups = layout_collapsed_groups.clone();
                                                        move |_| {
                                                            let mut next = layout_collapsed_groups();
                                                            next.retain(|g| g != &group_id);
                                                            layout_collapsed_groups.set(next.clone());
                                                            let payload = serde_json::to_string(&next).unwrap_or("[]".to_string());
                                                            spawn(async move {
                                                                let js = format!("window.graphLab && (window.graphLab.collapsedGroups = {});", payload);
                                                                let _ = eval(&js).await;
                                                            });
                                                        }
                                                    },
                                                    "Remove"
                                                }
                                            }
                                        }
                                    }
                                    if let Some(at) = layout_saved_at() {
                                        div { class: "mt-4 text-xs uppercase tracking-[0.2em] text-[#64748B]", "Last Saved" }
                                        div { class: "mt-2 text-xs text-[#CBD5F5]", "{at}" }
                                    }
                                    if let Some(by) = layout_saved_by() {
                                        div { class: "mt-1 text-xs text-[#94A3B8]", "By {by}" }
                                    }
                                    div { class: "mt-4 flex items-center justify-between",
                                        div { class: "text-xs uppercase tracking-[0.2em] text-[#64748B]", "Commit History" }
                                        if layout_diff_preview().is_some() {
                                            button {
                                                class: "rounded-md border border-[#334155] px-2 py-1 text-[10px] uppercase tracking-[0.2em] text-[#94A3B8] hover:bg-[#1E293B]/60 transition",
                                                onclick: {
                                                    let mut layout_diff_preview = layout_diff_preview.clone();
                                                    move |_| layout_diff_preview.set(None)
                                                },
                                                "Clear Preview"
                                            }
                                        }
                                    }
                                    if let Some(preview) = layout_diff_preview() {
                                        div {
                                            class: "mt-2 rounded-lg border border-[#1E293B] bg-[#0F172A]/70 px-3 py-2 text-xs text-[#BAE6FD]",
                                            "Preview {preview.timestamp}: moved {preview.moved} · max drift {preview.max} · handles Δ{preview.handle_diff} · groups Δ{preview.group_diff}"
                                        }
                                    }
                                    if layout_history_loading() {
                                        div { class: "mt-2 text-xs text-[#94A3B8]", "Loading history..." }
                                    }
                                    if let Some(err) = layout_history_error() {
                                        div { class: "mt-2 text-xs text-red-300", "History error: {err}" }
                                    }
                                    if history_entries.is_empty() && !layout_history_loading() {
                                        div { class: "mt-2 text-xs text-[#94A3B8]", "No layout history yet." }
                                        div { class: "mt-3 space-y-2",
                                            for (title, detail) in history_stub.iter() {
                                                div { class: "flex items-center justify-between rounded-lg border border-[#1E293B] bg-[#0B1120]/70 px-3 py-2 text-xs text-[#CBD5F5]",
                                                    span { class: "truncate", "{title}" }
                                                    span { class: "text-[#94A3B8]", "{detail}" }
                                                }
                                            }
                                        }
                                    } else {
                                        div { class: "mt-3 space-y-2",
                                            for entry in history_recent.into_iter() {
                                                div { class: "flex items-center justify-between rounded-lg border border-[#1E293B] bg-[#0B1120]/70 px-3 py-2 text-xs text-[#CBD5F5]",
                                                    div { class: "flex flex-col",
                                                        span { class: "truncate", "{format_saved_at(entry.updated_at)}" }
                                                        span { class: "text-[#94A3B8]", "{entry.node_positions.len()} nodes · {truncate_principal(entry.updated_by)}" }
                                                        if let Some(message) = &entry.commit_message {
                                                            if !message.is_empty() {
                                                                span { class: "text-[#94A3B8]", "“{message}”" }
                                                            }
                                                        }
                                                        if let Some(tag) = &entry.commit_tag {
                                                            if !tag.is_empty() {
                                                                span { class: "text-[10px] uppercase tracking-[0.2em] text-[#64748B]", "#{tag}" }
                                                            }
                                                        }
                                                        if let Some(preview) = layout_diff_preview() {
                                                            if preview.updated_at == entry.updated_at {
                                                                span {
                                                                    class: "text-[10px] text-[#BAE6FD]",
                                                                    "Moved {preview.moved} · drift {preview.max} · handles Δ{preview.handle_diff} · groups Δ{preview.group_diff}"
                                                                }
                                                            }
                                                        }
                                                    }
                                                    div { class: "flex flex-col gap-2",
                                                        button {
                                                            class: "rounded-md border border-[#F59E0B]/50 px-2 py-1 text-[10px] uppercase tracking-[0.2em] text-[#FDE68A] hover:bg-[#F59E0B]/20 transition",
                                                            onclick: {
                                                                let node_positions = entry.node_positions.clone();
                                                                let handle_positions = entry.handle_positions.clone();
                                                                let collapsed_groups = entry.collapsed_groups.clone();
                                                                let layout_handle_positions = layout_handle_positions.clone();
                                                                let layout_collapsed_groups = layout_collapsed_groups.clone();
                                                                let mut layout_diff_preview = layout_diff_preview.clone();
                                                                let timestamp = format_saved_at(entry.updated_at);
                                                                move |_| {
                                                                    let current_handles = layout_handle_positions();
                                                                    let current_groups = layout_collapsed_groups();
                                                                    let mut entry_handle_set = HashSet::new();
                                                                    for handle in handle_positions.iter() {
                                                                        entry_handle_set.insert(format!("{}:{}:{}", handle.handle_id, handle.source, handle.target));
                                                                    }
                                                                    let mut current_handle_set = HashSet::new();
                                                                    for handle in current_handles.iter() {
                                                                        current_handle_set.insert(format!("{}:{}:{}", handle.handle_id, handle.source, handle.target));
                                                                    }
                                                                    let handle_diff = entry_handle_set
                                                                        .symmetric_difference(&current_handle_set)
                                                                        .count();
                                                                    let mut entry_group_set = HashSet::new();
                                                                    for group in collapsed_groups.iter() {
                                                                        entry_group_set.insert(group.clone());
                                                                    }
                                                                    let mut current_group_set = HashSet::new();
                                                                    for group in current_groups.iter() {
                                                                        current_group_set.insert(group.clone());
                                                                    }
                                                                    let group_diff = entry_group_set
                                                                        .symmetric_difference(&current_group_set)
                                                                        .count();
                                                                    let positions = node_positions
                                                                        .iter()
                                                                        .map(|pos| {
                                                                            let x = pos.x.0.to_string().parse::<i64>().unwrap_or(0);
                                                                            let y = pos.y.0.to_string().parse::<i64>().unwrap_or(0);
                                                                            serde_json::json!({
                                                                                "id": pos.node_id,
                                                                                "x": x,
                                                                                "y": y,
                                                                            })
                                                                        })
                                                                        .collect::<Vec<_>>();
                                                                    let positions_json = serde_json::to_string(&positions).unwrap_or("[]".to_string());
                                                                    let timestamp = timestamp.clone();
                                                                    spawn(async move {
                                                                        let js = format!(
                                                                            r#"(function() {{
                                                                                if (!window.graphLab || !window.graphLab.data) return "";
                                                                                const positions = {positions};
                                                                                const map = new Map(positions.map(p => [p.id, p]));
                                                                                let moved = 0;
                                                                                let max = 0;
                                                                                const tol = 2;
                                                                                (window.graphLab.data.nodes || []).forEach(n => {{
                                                                                    const p = map.get(n.id);
                                                                                    if (!p) return;
                                                                                    const dx = (n.x || 0) - p.x;
                                                                                    const dy = (n.y || 0) - p.y;
                                                                                    const dist = Math.sqrt(dx * dx + dy * dy);
                                                                                    if (dist > tol) moved += 1;
                                                                                    if (dist > max) max = dist;
                                                                                }});
                                                                                return JSON.stringify({{ moved, max: Math.round(max) }});
                                                                            }})()"#,
                                                                            positions = positions_json
                                                                        );
                                                                        let mut moved = 0;
                                                                        let mut max = 0;
                                                                        if let Ok(val) = eval(&js).await {
                                                                            if let Some(raw) = val.as_str() {
                                                                                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(raw) {
                                                                                    moved = parsed.get("moved").and_then(|v| v.as_i64()).unwrap_or(0);
                                                                                    max = parsed.get("max").and_then(|v| v.as_i64()).unwrap_or(0);
                                                                                }
                                                                            }
                                                                        }
                                                                        let preview = LayoutPreview {
                                                                            updated_at: entry.updated_at,
                                                                            timestamp,
                                                                            moved,
                                                                            max,
                                                                            handle_diff,
                                                                            group_diff,
                                                                        };
                                                                        layout_diff_preview.set(Some(preview));
                                                                    });
                                                                }
                                                            },
                                                            "Preview"
                                                        }
                                                        button {
                                                            class: "rounded-md border border-[#38BDF8]/40 px-2 py-1 text-[10px] uppercase tracking-[0.2em] text-[#BAE6FD] hover:bg-[#0EA5E9]/20 transition",
                                                            onclick: {
                                                                let node_positions = entry.node_positions.clone();
                                                                let handle_positions = entry.handle_positions.clone();
                                                                let collapsed_groups = entry.collapsed_groups.clone();
                                                                let mut layout_status = layout_status.clone();
                                                                let mut layout_dirty = layout_dirty.clone();
                                                                let mut layout_handle_positions = layout_handle_positions.clone();
                                                                let mut layout_collapsed_groups = layout_collapsed_groups.clone();
                                                                let layout_heat_mode = layout_heat_mode.clone();
                                                                let mut layout_commit_message = layout_commit_message.clone();
                                                                let mut layout_commit_tag = layout_commit_tag.clone();
                                                                move |_| {
                                                                    let positions = node_positions
                                                                        .iter()
                                                                        .map(|pos| {
                                                                            let x = pos.x.0.to_string().parse::<i64>().unwrap_or(0);
                                                                            let y = pos.y.0.to_string().parse::<i64>().unwrap_or(0);
                                                                            serde_json::json!({
                                                                                "id": pos.node_id,
                                                                                "x": x,
                                                                                "y": y,
                                                                            })
                                                                        })
                                                                        .collect::<Vec<_>>();
                                                                    let handles = handle_positions
                                                                        .iter()
                                                                        .map(|handle| {
                                                                            serde_json::json!({
                                                                                "handle_id": handle.handle_id,
                                                                                "source": handle.source,
                                                                                "target": handle.target,
                                                                            })
                                                                        })
                                                                        .collect::<Vec<_>>();
                                                                    let collapsed = collapsed_groups.clone();
                                                                    let heat_enabled = layout_heat_mode();
                                                                    let apply_js = format!(
                                                                        r#"(function() {{
                                                                            if (!window.graphLab || !window.graphLab.data) return;
                                                                            const positions = {positions};
                                                                            const handles = {handles};
                                                                            const collapsed = {collapsed};
                                                                            window.graphLab.savedPositions = positions;
                                                                            window.graphLab.handlePositions = handles;
                                                                            window.graphLab.collapsedGroups = collapsed;
                                                                            const map = new Map(positions.map(p => [p.id, p]));
                                                                            window.graphLab.data.nodes.forEach(n => {{
                                                                                const p = map.get(n.id);
                                                                                if (p) {{
                                                                                    n.fx = p.x;
                                                                                    n.fy = p.y;
                                                                                    n.x = p.x;
                                                                                    n.y = p.y;
                                                                                }}
                                                                            }});
                                                                            if (window.graphLab.instance && window.graphLab.instance.simulation) {{
                                                                                window.graphLab.instance.simulation.alpha(0.6).restart();
                                                                            }}
                                                                            if (window.graphLab.renderLayoutOverlay) {{
                                                                                window.graphLab.renderLayoutOverlay();
                                                                            }}
                                                                            if (window.graphLab.applyHeatMode) {{
                                                                                window.graphLab.applyHeatMode({heat});
                                                                            }}
                                                                        }})()"#,
                                                                        positions = serde_json::to_string(&positions).unwrap_or("[]".to_string()),
                                                                        handles = serde_json::to_string(&handles).unwrap_or("[]".to_string()),
                                                                        collapsed = serde_json::to_string(&collapsed).unwrap_or("[]".to_string()),
                                                                        heat = heat_enabled
                                                                    );
                                                                    layout_handle_positions.set(handle_positions.clone());
                                                                    layout_collapsed_groups.set(collapsed_groups.clone());
                                                                    layout_commit_message.set(entry.commit_message.clone().unwrap_or_default());
                                                                    layout_commit_tag.set(entry.commit_tag.clone().unwrap_or_default());
                                                                    layout_dirty.set(true);
                                                                    layout_status.set(Some("History applied (unsaved)".to_string()));
                                                                    spawn(async move {
                                                                        let _ = eval(&apply_js).await;
                                                                    });
                                                                }
                                                            },
                                                            "Apply"
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            }
                        }
                    }
                },
                WorkbenchTab::Traces => rsx! {
                    div { class: "flex-1 rounded-2xl border border-[#1E293B] bg-black/30 p-6",
                        div { class: "text-xs uppercase tracking-[0.2em] text-[#64748B]", "Trace Timeline" }
                        div { class: "mt-4 text-sm text-[#94A3B8]", "No traces yet. Traces will appear after workflow execution." }
                    }
                },
                WorkbenchTab::Logs => rsx! {
                    div { class: "flex-1 rounded-2xl border border-[#1E293B] bg-black/30 p-6",
                        div { class: "text-xs uppercase tracking-[0.2em] text-[#64748B]", "Logs" }
                        div { class: "mt-4 text-sm text-[#94A3B8]", "Log stream pending Log Registry integration." }
                    }
                },
            }
        }
    }
}

#[component]
fn TabButton(label: &'static str, active: bool, onclick: EventHandler<MouseEvent>) -> Element {
    let base = "px-4 py-2 rounded-full text-xs uppercase tracking-[0.2em] transition border";
    let class = if active {
        format!("{} border-[#38BDF8]/60 bg-[#0EA5E9]/10 text-[#E0F2FE]", base)
    } else {
        format!("{} border-[#1E293B] text-[#94A3B8] hover:text-[#E2E8F0] hover:border-[#334155]", base)
    };

    rsx! {
        button { class: "{class}", onclick, "{label}" }
    }
}
