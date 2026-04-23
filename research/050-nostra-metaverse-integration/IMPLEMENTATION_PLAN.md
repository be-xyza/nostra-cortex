# Implementation Plan: Godot Bridge Lab

## Goal
Implement `labs:godot-bridge` to validate the "Godot Web Export + Internet Identity" authentication flow.

## User Review Required
None. This is a non-breaking addition to the Labs system.

## Proposed Changes

### Frontend (Rust/Dioxus)

#### [NEW] `src/labs/godot_bridge_lab.rs`
- Defines `GodotBridgeLab` component.
- Renders `iframe src="/labs/godot_mock/index.html"`.
- Uses `eval()` to attach a `window.addEventListener('message', ...)` that listens for:
    - `{ type: 'LOGIN_REQUEST' }` -> Triggers Host mock login (or real II if integrated).
    - Sends back `{ type: 'LOGIN_SUCCESS', delegation: '...' }` to the iframe.

#### [MODIFY] `src/labs/mod.rs`
- Add `pub mod godot_bridge_lab;`.

#### [MODIFY] `src/labs/registry.rs`
- Add `labs:godot-bridge` entry to `get_lab_registry()`.
    - **ID**: `labs:godot-bridge`
    - **Route**: `/labs/godot-bridge`
    - **Contexts**: `CORTEX_LABS`

#### [MODIFY] `src/labs/components_lab.rs` (or `lab_view.rs`)
- Ensure the routing handles the new lab ID (likely handled dynamically by `LabView`, but need to check if map needs update. `registry.rs` drives the menu, but `LabView` needs to match the ID to the Component).
    - *Correction*: I need to check `LabView` switch statement.

### Frontend (Public Assets)

#### [NEW] `public/labs/godot_mock/index.html`
- A simple HTML page simulating a Godot game.
- Buttons: "Connect to Nostra", "Sign Message".
- text area to show logs.

## Verification Plan

### Manual Verification
1.  Navigate to Labs > Godot Bridge.
2.  Click "Launch".
3.  In the iframe (Mock Game), click "Connect".
4.  Verify Dioxus Host receives the message (logs to console).
5.  Verify Host sends back a dummy delegation.
6.  Verify Iframe displays "Delegation Received".
