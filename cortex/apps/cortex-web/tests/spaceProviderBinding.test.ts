import assert from "node:assert/strict";
import test from "node:test";

import {
  applyAuthBindingToSpaceRouting,
  parseAuthMetadataJson,
} from "../src/components/spaces/spaceProviderBinding.ts";

test("parseAuthMetadataJson keeps metadata compact and stringified", () => {
  assert.deepEqual(
    parseAuthMetadataJson('{ "source": "github", "scopes": ["repo"], "active": true }'),
    {
      source: "github",
      scopes: '["repo"]',
      active: "true",
    },
  );
});

test("applyAuthBindingToSpaceRouting preserves routing state while adding a binding", () => {
  assert.deepEqual(
    applyAuthBindingToSpaceRouting(
      {
        adapterSetRef: "adapter.default",
        providerId: "openrouter_primary",
        defaultModel: "gpt-5.4",
        authBindingId: null,
        agentRoutingPolicy: "space_default_with_agent_overrides",
        agentOverrides: {
          "agent:alpha": {
            agentId: "agent:alpha",
            providerId: "openrouter_primary",
            defaultModel: "llama3.1:8b",
            authBindingId: "auth-old",
            authMode: "auth_binding",
          },
        },
        updatedAt: "2026-03-23T00:00:00Z",
        updatedBy: "cortex-web",
      },
      "auth-new",
      "openrouter_primary",
      "gpt-5.4",
      "adapter.primary",
    ),
    {
      adapterSetRef: "adapter.primary",
      providerId: "openrouter_primary",
      defaultModel: "gpt-5.4",
      authBindingId: "auth-new",
      agentRoutingPolicy: "space_default_with_agent_overrides",
      agentOverrides: {
        "agent:alpha": {
          agentId: "agent:alpha",
          providerId: "openrouter_primary",
          defaultModel: "llama3.1:8b",
          authBindingId: "auth-old",
          authMode: "auth_binding",
        },
      },
      updatedAt: "2026-03-23T00:00:00Z",
      updatedBy: "cortex-web",
    },
  );
});
