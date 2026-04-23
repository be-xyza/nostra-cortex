---
id: "128"
name: "cortex-desktop-gpui-refactor"
title: "Decisions: Cortex Desktop GPUI Refactor"
type: "decision"
project: "nostra"
status: active
authors: ["Antigravity", "User"]
tags: ["cortex", "desktop", "gpui", "native"]
created: "2026-03-01"
updated: "2026-03-01"
---

# Decisions: Cortex Desktop GPUI Refactor

## DEC-128-001: Formally Deprecate GPUI Experimental Shell
**Date**: 2026-03-01
**Status**: ✅ Decided

**Decision**: Formally deprecate the greenfield GPUI native experimental shell logic.

**Rationale**: GPUI offered theoretical improvements around avoiding webview overheads (eliminating the Dioxus JS-injection bridges). However, Nostra's adoption of the React ecosystem across the dual-host alignment (AG-UI transport and React/A2UI unified rendering via `cortex-web`) provides massive iteration velocity and an integrated tooling ecosystem that outweighs the performance novelties of building a custom GPU native application from scratch.

**Implications**: `cortex-desktop` is structurally recognized as primarily a headless Daemon environment, with visualization responsibility definitively delegated to the modularity of the `cortex-web` React shell.
