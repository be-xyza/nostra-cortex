# Implementation Plan: Cortex Simulation Adapter (Godot GDExtension)

**Objective**: Develop a proof-of-concept (POC) integration demonstrating native, zero-overhead execution between the Cortex/Nostra Rust architecture and the Godot Engine (C++) using `godot-rust` (GDExtension). Aligning with Initiative 049, this positions Godot not just as a game engine, but as the high-frequency off-chain "Intelligence Layer" compute environment for simulations, spatial data, and physics calculations.

## User Review Required
> [!IMPORTANT]
> - By adhering to `AGENTS.md`, the simulation adapter is placed in `cortex/libraries`. To avoid polluting production orchestrations in `cortex/apps`, the lab project is placed in a new `cortex/labs/` equivalent directory.
> - We assume Godot 4.3 and the corresponding `godot-rust` branch (`gdext`).

## Proposed Changes

### 1. Workspace Configuration & Library Setup

#### [MODIFY] `Cargo.toml` (Nostra Root/Workspace)
- Add `cortex/libraries/cortex-simulation-adapter` to the `[workspace.members]` array.

#### [NEW] `cortex/libraries/cortex-simulation-adapter/Cargo.toml`
- Define the new crate as a `cdylib` (C dynamic library), required by Godot to load the extension.
- Add dependencies:
  - `godot = { git = "https://github.com/godot-rust/gdext", branch = "master" }` (or specific stable tag for 4.3).

---
### 2. GDExtension Rust Implementation

#### [NEW] `cortex/libraries/cortex-simulation-adapter/src/lib.rs`
- Define the `ExtensionLibrary` entry point `struct CortexSimulationExtension;`.
- Implement #[gdextension] macro to register classes.
- Define a basic native Rust class (e.g., `#[class] struct AgentSimulationNode { base: Base<Node> }`).
  - Expose a `#[func]` method like `inject_simulation_data(data: String)` that Godot can call natively.
  - Implement the `INode` trait's `process` loop to demonstrate the Rust logic successfully ticking inside the Godot Engine thread.

---
### 3. Godot Prototype Project

#### [NEW] `cortex/labs/cortex-simulation-lab/project.godot`
- A minimal Godot 4.3 project initialization file configured for headless/simulation testing.

#### [NEW] `cortex/labs/cortex-simulation-lab/cortex_adapter.gdextension`
- The configuration file instructing Godot where to find the compiled `.dylib` (Mac), `.so` (Linux), or `.dll` (Windows) built by the `cortex-simulation-adapter` crate.
- Points paths to `../../../target/debug/libcortex_simulation_adapter.dylib`.

#### [NEW] `cortex/labs/cortex-simulation-lab/Main.tscn`
- A minimal Godot scene that instantiates the `AgentSimulationNode` (which is compiled dynamically from Rust).
- A basic GDScript attached to the scene root that calls `inject_simulation_data("Hello from GDScript")` to prove bidirectional FFI works.

---
### 4. Headless Execution Proof (The "Intelligence" Layer)

#### [NEW] `cortex/labs/cortex-simulation-lab/run_headless.sh`
- A script to compile the Rust `cdylib` and then launch the Godot executable against the `project.godot` in headless mode `--headless`.
- Proves that the Godot engine can be used purely as a math/simulation background process driven natively by our Rust logic.

## Verification Plan

### Automated/Compilation Tests
- `cargo build -p cortex-simulation-adapter` successfully compiles the dynamic library without linking errors.

### Manual Verification
1. Run `cargo build -p cortex-simulation-adapter` on the extension.
2. Open the `cortex/labs/cortex-simulation-lab` project in the Godot 4.3 Editor.
3. Observe the `AgentSimulationNode` (defined in Rust) is available in the Node creation menu.
4. Run the Godot scene (or run headless).
5. Verify Rust `println!` macros fire in the Godot console, proving the engine successfully executed Rust-native GDExtension FFI logic.
