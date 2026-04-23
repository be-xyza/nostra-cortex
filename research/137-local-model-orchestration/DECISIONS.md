---
initiative: "137"
name: "local-model-orchestration"
date: "2026-03-23"
---

# Architectural Decisions: Local Model Orchestration

## DEC-137-001: Connectivity via Reverse SSH Tunnels

**Context**:
Eudaemon Alpha (VPS) needs to use models running on a local Mac (Ollama/HRM). Direct ingress to local workstations is insecure and often blocked by NAT.

**Decision**:
Use SSH reverse tunnels to expose local inference ports to the VPS loopback interface.
- Ollama: `11434:localhost:11434`
- HRM: `8001:localhost:8001` (if applicable)

**Rationale**:
- Zero-cost infrastructure (standard SSH).
- High security (encrypted, authenticated tunnel).
- No firewall modification required on the local side.

## DEC-137-002: Local Worker Classification

**Context**:
Local models have high latency when accessed over a tunnel vs local IPC, and their availability is intermittent.

**Decision**:
Classify local models as `recommendation_only` (L1) workers.
- No direct commit authority to the Nostra core graph.
- Results must be processed as "Proposals" or "Drafts" (Initiative 132/134 pattern).

**Rationale**:
Aligned with the Constitutional Framework regarding local resource usage, uncertainty, and limited authority.

## DEC-137-003: UI Visibility (Local Badge)

**Context**:
Users need to know when their local hardware is participating in inference to understand privacy and resource usage.

**Decision**:
Introduce `Local`, `Tunneled`, and `Cloud` badges in `ProviderDashboard` and related execution surfaces for any provider bound to a local, tunneled, or remote endpoint.

**Rationale**:
Enhances system legibility and keeps the user-facing vocabulary simple.

## DEC-137-004: Additive Provider Topology

**Context**:
The current provider registry is flat, but the system now needs to represent local machines, tunneled mirrors, and multi-model providers such as OpenRouter without breaking existing consumers.

**Decision**:
Keep the current provider registry response shape backward-compatible and add an optional structured topology block to each provider record.
- `familyId` identifies the provider family or vendor service.
- `profileId` identifies the selected model/config variant.
- `instanceId` identifies the concrete runtime endpoint.
- `deviceId` identifies the physical or virtual machine.
- `environmentId` identifies the deployment context.
- `localityKind` is one of `Local`, `Tunneled`, or `Cloud`.

**Rationale**:
This preserves compatibility, supports shared hardware and local worker discovery, and gives the UI a stable semantic layer for provider and locality badges.
