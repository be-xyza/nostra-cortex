import assert from "node:assert/strict";
import test from "node:test";

import { parseAgentActivityEvent } from "../src/components/heap/agentActivity.ts";

test("parseAgentActivityEvent extracts execution record details", () => {
  const event = parseAgentActivityEvent({
    type: "agent_execution_record",
    timestamp: "2026-03-18T00:00:00Z",
    payload: {
      agent_id: "agent:cortex-worker-01",
      phase: "analysis",
      status: "completed",
      provider_kind: "codex_subscription",
      auth_mode: "subscription_profile",
      benchmark: {
        overall_grade: "PASS",
      },
    },
  });

  assert.ok(event);
  assert.equal(event?.agent, "agent:cortex-worker-01");
  assert.equal(event?.status, "completed");
  assert.match(event?.details ?? "", /analysis/i);
  assert.match(event?.details ?? "", /PASS/i);
  assert.match(event?.details ?? "", /provider:codex_subscription/i);
  assert.match(event?.details ?? "", /auth:subscription_profile/i);
});
