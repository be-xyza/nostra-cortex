import test from "node:test";
import assert from "node:assert/strict";

import {
  classifyProviderRegistryError,
  providerRegistryStatusCopy,
} from "../src/store/providersRegistry.ts";

test("provider registry classifies 403 responses as access denied", () => {
  assert.equal(classifyProviderRegistryError("403 Forbidden"), "access_denied");
});

test("provider registry status copy explains operator-only access", () => {
  assert.deepEqual(providerRegistryStatusCopy("access_denied", "403 Forbidden"), {
    title: "Provider registry requires operator access.",
    body: "Switch to an operator session to inspect provider, runtime host, and auth topology.",
  });
});
