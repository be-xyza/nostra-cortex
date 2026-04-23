---
id: "074"
name: "cortex-ui-substrate"
title: "Decisions: Cortex UI Substrate & Governance"
type: "decision"
project: "nostra"
status: active
authors: ["Antigravity", "User"]
tags: ["cortex", "ui", "a2ui", "lit"]
created: "2026-03-01"
updated: "2026-03-01"
---

# Decisions: Cortex UI Substrate & Governance

## DEC-074-001: Formally Deprecate Lit A2UI Renderer
**Date**: 2026-03-01
**Status**: ✅ Decided

**Decision**: Formally deprecate the Lit + Shoelace A2UI renderer initially used as the boilerplate implementation. Standardize entirely on the React-based A2UI interpreter natively implemented within `cortex-web/src/components/a2ui/`.

**Rationale**: The initial A2UI rendering prototype used Lit web components for encapsulation. However, as the UI substrate advanced, the overhead of bridging React state to Lit web components and maintaining dual frontend patterns became a liability. By moving to a pure React A2UI renderer internally, we achieve unified component lifecycle management, easier integration with context (e.g., AG-UI), and a single source of truth for UI execution within the Cortex shell.

**Implications**: All future A2UI client-side widget definitions and block rendering logic must target the React component registry. The `lit` rendering wrappers are considered legacy and should not be expanded.
