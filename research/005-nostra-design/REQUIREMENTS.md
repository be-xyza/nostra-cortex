---
id: '005'
name: nostra-design
title: 'Requirements: Nostra Design System'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# Requirements: Nostra Design System

## 1. Functional Requirements (FR)

### FR-1: Bimodal Interface
| ID | Requirement | Priority |
|----|-------------|----------|
| FR-1.1 | App MUST support two primary modes: **Explorer (Nostra)** and **Executor (Cortex)**. | Must |
| FR-1.2 | Cortex Mode MUST be accessible via global hotkey and UI toggle. | Must |
| FR-1.3 | Cortex Mode SHOULD appear as an overlay/panel, preserving the context of the Explorer view. | Should |
| FR-1.4 | On Mobile, Cortex Mode MUST act as a full-screen view (Stack navigation). | Must |

### FR-2: Visual Design System
| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.1 | UI MUST use the "Cinematic Dark" theme (Base `#0F0F12`). | Must |
| FR-2.2 | Panels app-wide MUST implement "Glassmorphism" (Translucent + Blur). | Must |
| FR-2.3 | Typography MUST distinguish Hierarchy (Outfit), Body (Inter), and Data (Mono). | Must |
| FR-2.4 | Accent colors MUST correspond to Contribution Types (Idea=Gold, Project=Blue, Issue=Red). | Should |

### FR-3: Navigation
| ID | Requirement | Priority |
|----|-------------|----------|
| FR-3.1 | Primary Navigation (Spaces) MUST be distinct from Context Navigation (Within a Space). | Must |
| FR-3.2 | "breadcrumbs" or path indicator MUST be visible at all times. | Must |

## 2. Technical Requirements (TR)

### TR-1: Tech Stack
*   **Framework**: Dioxus (Rust)
*   **Styling**: Tailwind CSS (v3.x)
*   **Icons**: Heroicons or Lucide (via Dioxus crate)
*   **Fonts**: Google Fonts (served locally or via optimized CDN)

### TR-2: Performance
*   **Blur Cost**: Glassmorphism (`backdrop-filter`) is expensive. Usage must be limited to high-level containers, not every list item.
*   **Z-Index Management**: Layering must be strictly defined to prevent "modal hell".

## 3. Component Specifications

### 3.1. Cards (`ContributionCard`)
*   **Content**: Title, Type Icon, Status Badge, Summary (Truncated).
*   **Actions**: "Quick Actions" (e.g., Vote, Like) exposed on hover.
*   **Variant**: `Compact` (List) vs `Expanded` (Grid).

### 3.2. Cortex Panel (`CortexHUD`)
*   **Behavior**: Slide-over from right (Desktop) or bottom sheet (Mobile).
*   **Width**: ~400px fixed or 30% viewport (Desktop).
*   **State**: Persists state across route changes (e.g., if I'm writing a chat message, it stays if I navigate the background).

## 4. Assets
*   **Logo**: SVG adaptation for Dark Mode.
*   **Empty States**: Illustration style to match "Cinematic" vibe (Linear/Abstract).
