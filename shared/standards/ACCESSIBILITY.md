# Accessibility & Authority Containment

**Type**: Constitutional Standard
**Status**: DRAFT
**Context**: Defines Accessibility not as a feature, but as a condition of legitimate authority. If a user cannot perceive or operate the system, the system's authority over them is illegitimate.

---

## 1. The Core Doctrine

> **"Inaccessible Authority is Illegitimate."**

In Nostra, purely technical limitations are acceptable (e.g., bandwidth, compute). However, limitations imposed by *design* that exclude users based on sensory, cognitive, or motor variance are classified as **Coercion**.

If a user cannot:
1.  **Perceive** the state of the system,
2.  **Operate** the controls of the system, or
3.  **Understand** the consequences of their actions,

Then they are not participating in governance; they are being *ruled* by it.

## 2. The Constitutional Boundary

We explicitly categorize accessibility concerns into invariants (Constitutional) and implementations (Adapter):

| Plane | Definition | Properties | Examples |
| :--- | :--- | :--- | :--- |
| **Constitutional (Invariant)** | The "Right" to participate. Defines *what* must be accessible. | Human-Centric, Verified, Non-Negotiable. | Operability, Perceivability, No Coercive Patterns. |
| **Implementational (Adapter)** | The "Method" of access. Defines *how* the invariant is met. | Device-Specific, Replaceable. | Screen Readers, Haptic Vests, Reduced Motion Modes, AI Summarizers. |

### 2.1 The Adapter Rule
**Rule**: The System must never assume a specific sensorium (Sight, Sound, Mouse, Touch). It must emit semantic signals that can be adapted to *any* sensorium.

*   **Bad**: "Click the red button." (Assumes sight + mouse).
*   **Good**: "Submit the 'Confirm' action." (Semantic; can be clicked, spoken, or thought).

---

## 3. The 4 Invariants

### 3.1 Perceivability (The Right to Know)
*   **Invariant**: Information and UI components must be presentable to users in ways they can perceive.
*   **Requirement**: Information must never reside *solely* in one sensory channel (e.g., Color alone, Sound alone).
*   **Corruption**: A dashboard that uses only Red/Green for status is physically opaque to 8% of men. This is a legitimacy failure.

### 3.2 Operability (The Right to Act)
*   **Invariant**: UI components and navigation must be operable.
*   **Requirement**: Interaction must not rely on specific physical parameters unless essential to the activity (e.g., drawing). Time limits must be extendable.
*   **Corruption**: A governance vote that requires a precise mouse hover is a poll tax on those with motor tremors.

### 3.3 Comprehensibility (The Right to Understand)
*   **Invariant**: Information and the operation of the user interface must be understandable.
*   **Requirement**: The system must explain itself. It must avoid "Dark Patterns" that rely on cognitive overwhelm or confusion to force consent.
*   **Corruption**: "Click here to accept consequences you didn't read" is a violation of the Non-Coercion principle.

### 3.4 Adaptability (The Right to Fork the View)
*   **Invariant**: Content must be robust enough that it can be interpreted reliably by a wide variety of user agents, including assistive technologies.
*   **Requirement**: Data and Logic must be separate from Presentation (A2UI).
*   **Corruption**: A hardcoded React component that traps focus violates the user's right to bring their own accessible renderer.

---

## 4. Implementation Guidelines

### 4.1 A2UI Schema Enforcement
*   **Semantics**: Every component in the schema (`Button`, `Card`, `Row`) carries mandatory semantic metadata. `<div>` soup is forbidden.
*   **Roles**: Custom components must declare ARIA-equivalent roles (`role="alert"`, `role="navigation"`).

### 4.2 Motion Governance
*   **Context**: Motion triggers vestibular disorders and seizures in susceptible populations.
*   **Rule**: Motion is a *privilege*, not a default.
*   **Token System**: All animation must use a Governed Motion Token that respects the `prefers-reduced-motion` signal.

### 4.3 Cognitive Load & Progressive Disclosure
*   **Context**: Neurodivergent users (ADHD, Autism) can be paralyzed by information density.
*   **Rule**: Advanced features must be "Pull", not "Push".
*   **Implementation**: Default views must be minimal. Complexity is unlocked by explicit user intent (Progressive Disclosure).

---

## 5. The "Turing" Test for Accessibility
To validate any interface, ask:

> "If I replaced the screen with a phone call, could I still accurately navigate this governance flow?"

If **Yes** -> The architecture is sound (Semantic).
If **No** -> The architecture is coupled to the screen (Implementation leak).
