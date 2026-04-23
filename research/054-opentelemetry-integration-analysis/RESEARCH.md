---
id: '054'
name: opentelemetry-integration-analysis
title: 'Research Findings: OpenTelemetry Architecture'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-02-01'
---

# Research Findings: OpenTelemetry Architecture

## 1. Executive Summary
OpenTelemetry Collector provides the industry standard for observability pipelines.
-   **Pipeline**: Uses a `Receiver -> Processor -> Exporter` flow, orchestrated by a `Service`.
-   **Data Model**: Uses `pdata` (immutable, efficient wrappers around OTLP Protobufs).
**Recommendation**: Nostra should adopt the **Pipeline Architecture** for its Log Registry, but implement it in **Rust** (Canisters) rather than Go. The `pdata` model should be mirrored in our Log Event Schema.

## Layered Architecture Role

> [!IMPORTANT]
> **Consolidation Decision**: This initiative owns the **Pipeline Layer** of Nostra's unified Observability stack.

| Layer | Owner | Responsibility |
|-------|-------|----------------|
| **Data Model** | 019-Log-Registry | `LogEntry`, `Span`, `Metric` schemas |
| **Pipeline** | **054-OpenTelemetry** (this) | Receiver→Processor→Exporter traits |
| **Consumption** | 033-Cortex-Monitor | Dashboard, Triage, Alerts |

**Implementation Target**: Rust traits mirroring OTel patterns, NOT Go runtime adoption.

---

## 2. Pipeline Architecture Analysis

### 2.1 The "Signal" Pattern (`pipeline.go`)
OTel strictly separates pipelines by "Signal" (Traces vs Metrics vs Logs).
-   Nostra should strictly separate **Logs** (System Events) from **Traces** (Workflow Steps).
-   We should define a `PipelineID` struct similar to OTel's `ID{Signal, Name}` to manage multiple parallel logging pipes (e.g., "Standard Debug", "Audit Trail").

### 2.2 Component Abstractions
The core interface is the `Factory` pattern:
```go
CreateLogs(ctx, set, cfg, next consumer.Logs) (Logs, error)
```
-   **Receiver**: Pushes data to `next`.
-   **Processor**: Intercepts data, modifies it, pushes to `next`.
-   **Exporter**: The final `next` (no downstream consumer).

## 3. Component Model Analysis

### 3.1 Receivers (`receiver/`)
OTel receivers typically listen on ports (Push).
-   **Nostra Port**: Our Receivers will be Canister Public Methods (`update` calls) accepting `LogBatch` payloads from Agents.

### 3.2 Processors (`processor/`)
Processors handle batching and filtering.
-   **Nostra Port**: We need a `BatchProcessor` (to group writes to stable memory) and a `FilterProcessor` (to drop debug logs in prod).

### 3.3 Exporters (`exporter/`)
Exporters write to backends.
-   **Nostra Port**: Our Exporter implements an `IExporterAdapter` trait.
    *   *Default Adapter*: `StableBTreeMap` (IC Storage).
    *   *External Adapter*: `ClickHouseExporter` or `ElasticSearchExporter` (via HTTPS Outcalls).

## 4. Adaptation Recommendations

| Feature | OpenTelemetry Implementation | Nostra Port |
| :--- | :--- | :--- |
| **Data Model** | `pdata` (Go/Protobuf) | `NostraLog` (Rust/Candid) mirroring OTLP fields. |
| **Pipeline** | `Receiver -> Processor -> Exporter` | Same architecture, implemented as Rust Traits. |
| **Transport** | gRPC/HTTP | IC Cross-Canister Calls. |
| **Buffer** | In-memory Channel | Canister Message Queue (built-in). |
