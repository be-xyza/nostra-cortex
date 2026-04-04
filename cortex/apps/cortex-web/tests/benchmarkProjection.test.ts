import assert from "node:assert/strict";
import test from "node:test";

import {
  buildBenchmarkProjection,
} from "../src/components/heap/benchmarkProjection.ts";

test("buildBenchmarkProjection prefers canonical benchmark fields", () => {
  const projection = buildBenchmarkProjection({
    pass_rate: 0.92,
    latency_ms: 1842,
    total_tokens: 1532,
    assertions_passed: 23,
    assertions_total: 25,
  });

  assert.ok(projection);
  assert.equal(projection?.grade, "WARN");
  assert.equal(projection?.passRate, 0.92);
  assert.equal(projection?.latencyMs, 1842);
  assert.equal(projection?.totalTokens, 1532);
  assert.equal(projection?.assertionsPassed, 23);
  assert.equal(projection?.assertionsTotal, 25);
  assert.match(projection?.summary ?? "", /92%/);
  assert.match(projection?.summary ?? "", /23 of 25/i);
  assert.match(projection?.summary ?? "", /1532 tokens/i);
});

test("buildBenchmarkProjection maps a high quality run to PASS", () => {
  const projection = buildBenchmarkProjection({
    pass_rate: 0.99,
    latency_ms: 550,
    total_tokens: 402,
    assertions_passed: 12,
    assertions_total: 12,
  });

  assert.equal(projection?.grade, "PASS");
  assert.match(projection?.summary ?? "", /550ms/i);
});
