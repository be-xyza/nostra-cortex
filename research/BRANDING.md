# Nostra & Cortex Design Language

This document codifies the core visual identity and design philosophy for the Nostra and Cortex ecosystems. The branding is not merely aesthetic; it is a direct visual translation of the underlying constitutional architecture defined in `AGENTS.md` and the platform specifications.

## 1. The Core Concept: Heart and Mind

The ecosystem is defined by a deep structural duality:

*   **Nostra (The Warm Heart):** Represents the platform authority, sovereignty, governance, culture, and canonical memory (`what exists`).
*   **Cortex (The Cool Mind):** Represents the precise execution runtime, agents, workflows, and deterministic intelligence (`how work runs`).

### The Master Symbol: Two Concentric Circles

The visual identity is anchored by a shared mark consisting of two geometric elements: an outer ring and an inner dot.

1.  **The Outer Ring (Nostra):** A warm, open circle.
    *   **Meaning:** Protective containment, community bounds, the open commons, and sovereignty.
    *   **Form:** A stylized, broken 'O' with an intentional gap. This gap signifies openness, an entry point (invitation), forkability, and the breathing, evolving nature of human knowledge. Context: `N⭕stra`.
2.  **The Inner Dot (Cortex):** A cool, solid circle.
    *   **Meaning:** Deterministic execution, focused intelligence, the Minimal Viable Kernel (MVK), precision.
    *   **Form:** A perfectly closed, mathematically centered, dense dot. Context: `C•rtex`.

**Architectural Alignment:**
The symbol states visually what is true technically: Cortex cannot exist without Nostra's authority layer. Execution (Cortex) is contained within governance (Nostra). Mind operates inside heart-defined values. 

---

## 2. The Programmable Identity (The Hybrid Approach)

The Nostra platform/Cortex runtime identity is not static. It is a **context-aware, programmable state machine**. The core geometric constraint (Outer broken ring + Inner solid dot) is the *immutable kernel* of the brand, while the aesthetic expression (colors, stroke width, gap depth, motion) is the *mutable memory*.

This allows the brand to seamlessly shift between "modes of being" based on its environment, governance rules, or experimental "Labs" freedom.

### Brand Dimensions (Contextual Props)

The brand component (`<BrandLogo />`) accepts parameters that define its expression:

*   **Mode (Expressive vs. Precise):**
    *   *Expressive (Organic):* Employs gradients (Sunset Orange to Crimson, Electric Cyan to Azure), an organic, softer gap in the ring, and smooth, "breathing" CSS transitions. Used for consumer-facing Spaces, cultural curation, media, and knowledge exploration.
    *   *Precise (Instrumental):* Employs flat, brutalist, high-contrast colors (`#E63946` and `#1D3557`), a sharp, mathematically precise 45-degree gap, and rigid, step-based motion. Used for Execution Spaces, developer tools, agent logs, and dense data environments.
    *   *Custom:* Labs-only request mode that applies bounded overrides to the baseline geometry of the space's active Mode.
*   **Authority State (Official vs. Labs):**
    *   *Official/Canonical:* Strictly adheres to the defined Primary Palettes.
    *   *Labs/Experimental:* Allows users to override colors, gap angles, and scale within predefined limits, symbolizing forkability and sovereign freedom before a formal proposal is adopted.
*   **Temporal/Governance Overrides (Seasonal/Event):**
    *   The branding engine supports programmatic overrides defined by on-chain governance or dates (e.g., A "Christmas Variant" activating globally between Dec 15 - Dec 26, introducing custom gradient logic or particle effects to the ring, while maintaining the core geometry).

### Constraint Kernel (v1.0)

The following constraints are normative for host implementations:

*   **Immutable Geometry Identity:** The mark always remains an outer broken ring plus inner closed dot.
*   **Technical Canonical Gap:** Technical mode uses a 45-degree ring gap.
*   **Labs Override Bounds:** Gap angle overrides are constrained to `12°..160°`; ring stroke overrides are constrained to `4px..20px`.
*   **Official Authority Lock:** `official` authority forbids all custom geometry/color overrides. A `custom` mode request under `official` falls back to technical baseline.
*   **Temporal Precedence:** Temporal/governance variants override style-layer values and take precedence over labs overrides for their active window.

### Governance Configurability (Stage Guidance)

At this stage, governance configurability is recommended for style-layer parameters only, while kernel geometry remains fixed unless a steward-gated structural decision is approved.

*   **Recommended configurable now (style layer):**
    *   Official palette tokens (`outerBase`, `outerGradientTo`, `innerBase`, `innerGradientTo`)
    *   Temporal variant palettes and stroke-cap behavior
    *   Motion timing profiles (transition/animation durations, pulse amplitude)
    *   Labs enable/disable switch and labs override bounds
*   **Not recommended for routine governance at this stage (kernel layer):**
    *   Core mark composition (broken outer ring + closed inner dot)
    *   Canonical technical gap (`45°`) and base ring/dot geometry
*   **Escalation rule:** Any kernel-level change is a steward-gated structural mutation and must be ratified with lineage recorded in `DECISIONS.md` before host rollout.

---

## 3. Design System Tokens

### Palette: The Heart (Nostra)
*   **Primary Base:** `Crimson Ember` (`#E63946`)
*   **Gradient/Accent:** `Sunset Orange` (`#F4A261`)

### Palette: The Mind (Cortex)
*   **Primary Base:** `Deep Azure` (`#1D3557`)
*   **Gradient/Accent:** `Electric Cyan` (`#00B4D8`)

---

## 4. Product Tiering & Application

The separation of these shapes allows for dynamic product tiering in the UI:
*   **Nostra-Only Surfaces:** Utilize the warm palette and the open ring motif (e.g., Space management, access control).
*   **Cortex Desktop/Runtime:** Utilize the cool palette and the precise dot motif (e.g., agent task runners, log viewers).
*   **Combined Surfaces (A2UI / Chat):** Utilize the dual-mode "Red Ring + Blue Dot" master symbol.

### Animation Primitives
When animating the brand mark (e.g., canonical loading states):
1.  **Birth:** The open red ring appears first (governance established).
2.  **Ignition:** The blue dot fades into the precise center (runtime instantiated).
3.  **Operation:** The dot pulses with compute activity, while the ring gently rotates or "breathes" to indicate an active, living ecosystem.
