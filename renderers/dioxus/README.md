# Cortex Renderer (Dioxus)

Part of **Initiative 074: Cortex UI Substrate**.

## Purpose
This library implements the **Layer 2 (Runtime Schema)** and **Layer 3 (Visual Primitives)** of the Cortex UI Architecture.
It transforms streaming A2UI JSON payloads into a reactive Dioxus Virtual DOM tree.

## Architecture

```ascii
[A2UI Stream] -> [Parser] -> [State Machine] -> [Dioxus VDOM] -> [WGPU/DOM]
```

## Features
- **Strict Schema Validation**: Verifies incoming A2UI against `0.8` spec.
- **Theme Injection**: Applies CSS variables from `ActiveTheme`.
- **Shoelace Bindings**: Maps A2UI components to Custom Elements.

## Usage
This crate is designed to be consumed by:
1.  **Cortex Desktop** (Tauri/Rust)
2.  **Cortex Web** (WASM)
