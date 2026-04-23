---
id: '076'
name: openrouter-icp-feasibility
title: 'Requirements: OpenRouter-like Service on Nostra/Cortex'
type: general
project: nostra
status: draft
authors:
- User
tags: []
created: '2026-01-28'
updated: '2026-01-28'
---

# Requirements: OpenRouter-like Service on Nostra/Cortex

## Functional Requirements

### FR-1: Unified API Access
| ID | Requirement | Priority |
|----|-------------|----------|
| FR-1.1 | Single API endpoint for all LLM providers | P0 |
| FR-1.2 | OpenAI API compatibility for easy migration | P1 |
| FR-1.3 | Model catalog with capabilities and pricing | P0 |
| FR-1.4 | Request format normalization across providers | P0 |
| FR-1.5 | On-chain fallback via DeepSeek (1.5B/7B) | P1 |

### FR-2: Intelligent Routing
| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.1 | Cost-optimized model selection | P0 |
| FR-2.2 | Latency-based routing for time-sensitive tasks | P1 |
| FR-2.3 | Capability matching (context window, modalities) | P0 |
| FR-2.4 | User preference overrides | P1 |
| FR-2.5 | Multi-modal capability (Vision/Audio) - Feasibility Study | P1 |

### FR-3: Resilience & Reliability
| ID | Requirement | Priority |
|----|-------------|----------|
| FR-3.1 | Automatic fallback on provider failure | P0 |
| FR-3.2 | Circuit breaker for unhealthy providers | P0 |
| FR-3.3 | Request retry with exponential backoff | P1 |
| FR-3.4 | Load balancing across provider endpoints | P1 |

### FR-4: Security
| ID | Requirement | Priority |
|----|-------------|----------|
| FR-4.1 | TEE-secured API key storage | P0 |
| FR-4.2 | End-to-end encryption for sensitive prompts | P1 |
| FR-4.3 | Internet Identity authentication | P0 |
| FR-4.4 | Per-user rate limiting | P0 |
| FR-4.5 | Support for User-Provided API Keys (Ephemeral/Encrypted) | P0 |

### FR-5: Payments & Metering
| ID | Requirement | Priority |
|----|-------------|----------|
| FR-5.1 | Direct ICP Cycle metering per request | P0 |
| FR-5.2 | Dynamic Cycle pricing based on USD cost | P0 |
| FR-5.3 | Bypass billing for user-provided keys (except base fee) | P0 |
| FR-5.4 | Cost estimation before request execution | P2 |
| FR-5.5 | Dual-Token Support (ICP Cycles + Nostra Token) | P2 |

### FR-6: MCP Integration
| ID | Requirement | Priority |
|----|-------------|----------|
| FR-6.1 | Expose router as MCP server (ic-rmcp) | P0 |
| FR-6.2 | `tools/list` returns available models | P0 |
| FR-6.3 | `tools/call` routes completion requests | P0 |
| FR-6.4 | Agent discovery via anda_registry | P2 |

---

## Non-Functional Requirements

### NFR-1: Performance
| ID | Requirement | Target |
|----|-------------|--------|
| NFR-1.1 | Routing latency overhead | < 100ms |
| NFR-1.2 | Request throughput | > 100 req/s |
| NFR-1.3 | P99 response time (small requests) | < 2s |
| NFR-1.4 | Cold start time | < 500ms |

### NFR-2: Availability
| ID | Requirement | Target |
|----|-------------|--------|
| NFR-2.1 | Service uptime | 99.9% |
| NFR-2.2 | Failover time | < 5s |
| NFR-2.3 | Graceful degradation | Automatic |

### NFR-3: Scalability
| ID | Requirement | Target |
|----|-------------|--------|
| NFR-3.1 | Concurrent users | > 1000 |
| NFR-3.2 | Horizontal scaling | Via subnet |
| NFR-3.3 | Provider scaling | > 10 providers |

### NFR-4: Security
| ID | Requirement | Standard |
|----|-------------|----------|
| NFR-4.1 | Key protection | TEE attestation |
| NFR-4.2 | Data encryption | AES-256-GCM |
| NFR-4.3 | Audit logging | Immutable on-chain |

---

## Technical Dependencies

### Core Infrastructure
| Component | Version | Purpose |
|-----------|---------|---------|
| ICP SDK (dfx) | >= 0.24 | Canister development |
| Rust | >= 1.75 | Canister implementation |
| ic-rmcp | >= 0.3.0 | MCP server SDK |

### LDC Labs Stack
| Component | Version | Purpose |
|-----------|---------|---------|
| ic-tee | Latest | TEE integration |
| ic-cose | Latest | Encrypted config storage |
| anda-cloud | Latest | Registry & payments |
| anda-db | Latest | Performance metrics |

### External APIs
| Provider | API Version | Required Capabilities |
|----------|-------------|----------------------|
| OpenAI | v1 | Chat completions |
| Anthropic | 2023-06-01 | Messages |
| Google AI | v1 | Generative AI |
| Mistral | v1 | Chat |

---

## Data Model

### ModelDefinition
```candid
type ModelDefinition = record {
    id: text;                    // e.g., "openai/gpt-4-turbo"
    provider: text;              // e.g., "openai"
    name: text;                  // e.g., "GPT-4 Turbo"
    context_window: nat32;       // e.g., 128000
    max_output: nat32;           // e.g., 4096
    input_cost_per_token: float64;
    output_cost_per_token: float64;
    modalities: vec text;        // ["text", "image"]
    capabilities: vec text;      // ["function_calling", "vision"]
    status: ModelStatus;
};

type ModelStatus = variant {
    Active;
    Degraded;
    Unavailable;
};
```

### RoutingRequest
```candid
type RoutingRequest = record {
    model: opt text;             // Specific model or null for auto
    api_key: opt text;           // User provided key (optional)
    messages: vec Message;
    max_tokens: opt nat32;
    temperature: opt float64;
    routing_preference: opt RoutingPreference;
};

type RoutingPreference = variant {
    CostOptimized;
    LatencyOptimized;
    CapabilityFirst: text;       // e.g., "vision"
};
```

### RoutingResponse
```candid
type RoutingResponse = record {
    id: text;
    model_used: text;
    provider: text;
    content: text;
    usage: UsageStats;
    cost: float64;
    latency_ms: nat64;
};

type UsageStats = record {
    prompt_tokens: nat32;
    completion_tokens: nat32;
    total_tokens: nat32;
};
```

---

## API Surface (MCP Tools)

### list_models
Returns catalog of available models with pricing and status.

**Input**: `{ filter: opt ModelFilter }`
**Output**: `{ models: vec ModelDefinition }`

### route_completion
Routes a completion request to the optimal provider.

**Input**: `RoutingRequest`
**Output**: `RoutingResponse`

### get_usage
Returns usage statistics for authenticated user.

**Input**: `{ period: text }`  // "day", "week", "month"
**Output**: `{ requests: nat64, tokens: nat64, cost: float64 }`

### estimate_cost
Estimates cost before executing a request.

**Input**: `RoutingRequest`
**Output**: `{ estimated_cost: float64, recommended_model: text }`

---

## Integration Points

### Nostra Cortex
- Chat interface calls `route_completion`
- Model selector populated from `list_models`
- Usage dashboard from `get_usage`

### Workflow Engine (013)
- `LLMCompletion` async operation type
- Cost estimation in workflow preview
- Fallback handling in task recovery

### Agent Framework (017)
- Agents call router via MCP
- Per-agent quota enforcement
- Agent-specific model preferences

### Knowledge Graph
- Entity extraction routed to optimal model
- Extraction quality logged to anda-db
- RAG context enhancement
