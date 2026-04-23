---
id: '075'
name: d3-graph-lab
title: D3 Graph Lab - Requirements
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# D3 Graph Lab - Requirements

## Functional Requirements

### FR-1: Force Configuration Panel

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-1.1 | Display sliders for all D3 force parameters | Must |
| FR-1.2 | Update graph simulation in real-time as parameters change | Must |
| FR-1.3 | Show current parameter values with numeric precision | Must |
| FR-1.4 | Provide reset-to-default button per parameter | Should |
| FR-1.5 | Group parameters by force type (link, charge, center, etc.) | Should |
| FR-1.6 | Support keyboard input for precise values | Could |

### FR-2: Graph Generator

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.1 | Generate random graphs with specified node count | Must |
| FR-2.2 | Generate random graphs with specified edge density | Must |
| FR-2.3 | Support clustered topology generation | Should |
| FR-2.4 | Support hierarchical topology generation | Should |
| FR-2.5 | Support scale-free (Barabási–Albert) topology | Could |
| FR-2.6 | Allow specifying node type distribution | Should |
| FR-2.7 | Provide size presets (Small/Medium/Large/Massive) | Must |

### FR-3: Performance Dashboard

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-3.1 | Display real-time FPS counter | Must |
| FR-3.2 | Display simulation tick duration (ms) | Must |
| FR-3.3 | Display total DOM element count | Should |
| FR-3.4 | Display JS heap size (if available) | Should |
| FR-3.5 | Show historical performance chart (last 30 seconds) | Could |
| FR-3.6 | Alert when FPS drops below threshold | Could |

### FR-4: Preset System

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-4.1 | Save current configuration as named preset | Must |
| FR-4.2 | Load preset and apply to current graph | Must |
| FR-4.3 | Include built-in optimized presets | Must |
| FR-4.4 | Export preset as JSON file | Should |
| FR-4.5 | Import preset from JSON file | Should |
| FR-4.6 | Share presets between users (backend storage) | Could |
| FR-4.7 | Rate/comment on community presets | Could |

### FR-5: Rendering Modes

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-5.1 | Default SVG rendering mode | Must |
| FR-5.2 | Canvas rendering mode toggle | Should |
| FR-5.3 | Maintain node click interactions in Canvas mode | Should |
| FR-5.4 | Maintain zoom/pan in Canvas mode | Should |
| FR-5.5 | WebGL rendering mode for very large graphs | Could |

### FR-6: Comparison Mode

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-6.1 | Display two graphs side-by-side | Should |
| FR-6.2 | Sync camera position between comparison graphs | Should |
| FR-6.3 | Apply different presets to each graph | Should |
| FR-6.4 | Show performance metrics for both simultaneously | Should |

### FR-7: Integration

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-7.1 | Load real knowledge graph data from backend | Must |
| FR-7.2 | Apply lab configurations to production Graph tab | Should |
| FR-7.3 | Persist user preferences to backend | Should |
| FR-7.4 | Auto-select preset based on graph size | Could |

---

## Non-Functional Requirements

### NFR-1: Performance

| ID | Requirement | Target |
|----|-------------|--------|
| NFR-1.1 | Lab UI initialization time | <500ms |
| NFR-1.2 | Parameter change response time | <50ms |
| NFR-1.3 | Preset load time | <100ms |
| NFR-1.4 | Graph generation (1000 nodes) | <2s |
| NFR-1.5 | No memory leaks after repeated resets | 0 leaks |

### NFR-2: Usability

| ID | Requirement | Target |
|----|-------------|--------|
| NFR-2.1 | All controls accessible via keyboard | 100% |
| NFR-2.2 | Mobile-responsive layout | Yes |
| NFR-2.3 | Tooltips on all parameter sliders | 100% |
| NFR-2.4 | Undo/redo for parameter changes | Should |

### NFR-3: Reliability

| ID | Requirement | Target |
|----|-------------|--------|
| NFR-3.1 | Graceful degradation on Canvas failure | Must |
| NFR-3.2 | Invalid preset handling with fallback | Must |
| NFR-3.3 | Cross-browser support (Chrome, Firefox, Safari) | Must |

---

## UI Wireframe

```
┌────────────────────────────────────────────────────────────────────────────┐
│  D3 Graph Lab ⚗️                                               [← Back]    │
├────────────────────────────────────────────────────────────────────────────┤
│ ┌─────────────────────────────────────────────────────────┐ ┌────────────┐ │
│ │                                                         │ │ METRICS    │ │
│ │                                                         │ │            │ │
│ │                    GRAPH CANVAS                         │ │ FPS: 58    │ │
│ │                                                         │ │ Tick: 4ms  │ │
│ │                    (Interactive D3)                     │ │ Nodes: 200 │ │
│ │                                                         │ │ Edges: 450 │ │
│ │                                                         │ │ DOM: 1350  │ │
│ │                                                         │ │            │ │
│ └─────────────────────────────────────────────────────────┘ │ [Chart]    │ │
│                                                              └────────────┘ │
├────────────────────────────────────────────────────────────────────────────┤
│ GRAPH GENERATOR                                                            │
│ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐   Topology: [Random ▼]    │
│ │ 50      │ │ 200     │ │ 500     │ │ 2000    │   Density:  [━━━━○━━━] 0.3│
│ │ Small   │ │ Medium  │ │ Large   │ │ Massive │                            │
│ └─────────┘ └─────────┘ └─────────┘ └─────────┘   [🔄 Generate Graph]      │
├────────────────────────────────────────────────────────────────────────────┤
│ FORCE CONFIGURATION                      │ PRESETS                         │
│ ┌──────────────────────────────────────┐ │ ┌─────────────────────────────┐ │
│ │ Link Force                           │ │ │ ○ Default                   │ │
│ │   Distance: [━━━━━○━━━━━━━━━] 180    │ │ │ ● Optimized (Small)         │ │
│ │   Strength: [━━━○━━━━━━━━━━━] 0.3    │ │ │ ○ Optimized (Large)         │ │
│ │                                      │ │ │ ○ Canvas Mode               │ │
│ │ Charge Force                         │ │ │ ○ Minimal                   │ │
│ │   Strength: [━━━━━━━━━━━━○━] -800    │ │ │ ────────────────────────    │ │
│ │   Theta:    [━━━━━━━━○━━━━━] 0.9     │ │ │ + Custom Preset 1           │ │
│ │   Max Dist: [━━━━━━○━━━━━━━] 300     │ │ └─────────────────────────────┘ │
│ │                                      │ │                                 │
│ │ Simulation                           │ │ [💾 Save] [📤 Export] [📥 Import]│
│ │   Alpha:    [━━━━━━━━○━━━━━] 0.8     │ │                                 │
│ │   Decay:    [━━━━○━━━━━━━━━] 0.05    │ │ Render Mode: [SVG ▼]           │
│ └──────────────────────────────────────┘ │ □ Apply to Production Graph     │
└──────────────────────────────────────────┴─────────────────────────────────┘
```

---

## Data Structures

### ForceConfiguration

```typescript
interface ForceConfiguration {
  link: {
    distance: number;      // 30-500, default: 180
    strength: number;      // 0-1, default: 0.3
    iterations: number;    // 1-10, default: 1
  };
  charge: {
    strength: number;      // -2000 to 0, default: -800
    distanceMin: number;   // 1-100, default: 1
    distanceMax: number;   // 100-1000, default: Infinity
    theta: number;         // 0-1, default: 0.9
  };
  center: {
    strength: number;      // 0-1, default: 1
  };
  collision: {
    enabled: boolean;      // default: true
    radius: number;        // 0-100, default: 30
    strength: number;      // 0-1, default: 0.9
    iterations: number;    // 1-10, default: 1
  };
  positioning: {
    xStrength: number;     // 0-0.5, default: 0.03
    yStrength: number;     // 0-0.5, default: 0.03
  };
  simulation: {
    alphaDecay: number;    // 0-0.1, default: 0.02
    alphaMin: number;      // 0-0.01, default: 0.001
    velocityDecay: number; // 0-1, default: 0.3
  };
}
```

### GraphPreset

```typescript
interface GraphPreset {
  id: string;
  name: string;
  description: string;
  author?: string;
  version: string;
  created: number;

  targetSize: {
    min: number;
    max: number;
  };

  forceConfig: ForceConfiguration;

  renderMode: 'svg' | 'canvas' | 'webgl';

  visual: {
    showLabels: boolean;
    labelMode: 'inside' | 'outside' | 'off';
    showLinkLabels: boolean;
    glowEffect: boolean;
    animationSpeed: number;
  };
}
```

### PerformanceMetrics

```typescript
interface PerformanceMetrics {
  fps: number;
  tickDuration: number;
  nodeCount: number;
  edgeCount: number;
  domElements: number;
  heapSize?: number;
  timestamp: number;
}

interface PerformanceHistory {
  samples: PerformanceMetrics[];
  maxSamples: number;  // e.g., 300 = 5 minutes at 1/sec
}
```

---

## API Surface

### JavaScript API (window.graphLab)

```javascript
// Graph generation
window.graphLab.generateGraph(nodeCount, options);
window.graphLab.loadProductionData();
window.graphLab.clear();

// Force configuration
window.graphLab.setForceConfig(config);
window.graphLab.getForceConfig();
window.graphLab.resetForces();

// Presets
window.graphLab.loadPreset(presetId);
window.graphLab.savePreset(name, description);
window.graphLab.exportPreset();
window.graphLab.importPreset(json);

// Rendering
window.graphLab.setRenderMode(mode);
window.graphLab.getRenderMode();

// Performance
window.graphLab.getMetrics();
window.graphLab.startBenchmark();
window.graphLab.stopBenchmark();

// Integration
window.graphLab.applyToProduction();
```

### Rust Bridge (Dioxus)

```rust
pub struct GraphLabBridge;

impl GraphLabBridge {
    pub fn generate_graph(&self, count: u32, options: &str);
    pub fn set_force_config(&self, config: &str);
    pub fn load_preset(&self, preset_id: &str);
    pub fn save_preset(&self, name: &str, description: &str);
    pub fn get_metrics(&self) -> Option<String>;
    pub fn apply_to_production(&self);
}
```

---

## Acceptance Criteria

### AC-1: Force Panel
- [ ] All 15+ force parameters adjustable via sliders
- [ ] Graph updates within 50ms of slider change
- [ ] Values display with appropriate precision
- [ ] Reset button restores defaults

### AC-2: Graph Generator
- [ ] Generate graph with 50-2000+ nodes
- [ ] Generation completes within 2 seconds for 1000 nodes
- [ ] Generated graphs have requested node count (±5%)
- [ ] Edge density matches specification

### AC-3: Performance Dashboard
- [ ] FPS updates at least 10 times per second
- [ ] Tick duration accurate within 1ms
- [ ] DOM element count updates on graph changes
- [ ] No performance impact from monitoring (<2%)

### AC-4: Presets
- [ ] Built-in presets load without error
- [ ] Custom presets save to localStorage
- [ ] Presets export as valid JSON
- [ ] Invalid presets handled gracefully

### AC-5: Integration
- [ ] Production graph applies lab settings correctly
- [ ] User preferences persist across sessions
- [ ] Lab accessible from Space navigation
