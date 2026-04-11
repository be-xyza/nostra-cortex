import assert from "node:assert/strict";
import test from "node:test";

import type { ProviderBatchPolicy, ProviderRecord } from "../src/contracts.ts";

test("provider batch policy round-trips through JSON without losing batch strategy fields", () => {
  const policy: ProviderBatchPolicy = {
    providerFamilyId: "doubleword",
    providerProfileId: "batch.small",
    cadenceKind: "Interval",
    scopeKind: "Space",
    flushPolicy: "OnInterval",
    orderingKey: "space_id",
    dedupeKey: "request_hash",
    batchWindow: {
      intervalSeconds: 60,
      maxItems: 100,
      maxAgeSeconds: 600,
      timezone: "UTC",
    },
  };

  const encoded = JSON.stringify(policy);
  const decoded = JSON.parse(encoded) as ProviderBatchPolicy;

  assert.deepEqual(decoded, policy);
});

test("doubleword provider records can carry an explicit batch policy", () => {
  const provider: ProviderRecord = {
    id: "provider-doubleword",
    name: "DoubleWord Batch",
    providerType: "Batch",
    llmType: "DoubleWord",
    endpoint: "https://doubleword.example/api",
    isActive: true,
    priority: 10,
    configJson: "{\"mode\":\"batch\"}",
    batchPolicy: {
      providerFamilyId: "doubleword",
      providerProfileId: "batch.small",
      cadenceKind: "TimeWindow",
      scopeKind: "RequestGroup",
      flushPolicy: "OnWindowClose",
      orderingKey: "request_group_id",
      dedupeKey: "request_hash",
      batchWindow: {
        maxItems: 25,
        maxAgeSeconds: 300,
      },
    },
  };

  assert.equal(provider.batchPolicy?.providerFamilyId, "doubleword");
  assert.equal(provider.batchPolicy?.flushPolicy, "OnWindowClose");
});
