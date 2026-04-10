import assert from "node:assert/strict";
import test from "node:test";

import { appendShellUtilityEntries } from "../src/components/commons/shellUtilityEntries.ts";

test("appendShellUtilityEntries adds a Conversations nav entry once", () => {
  const entries = appendShellUtilityEntries([
    {
      routeId: "/explore",
      label: "Explore",
      icon: "compass",
      category: "execution",
      requiredRole: "operator",
    },
  ]);

  const conversations = entries.filter((entry) => entry.routeId === "/conversations");

  assert.equal(conversations.length, 1);
  assert.equal(conversations[0]?.label, "Conversations");
});

test("appendShellUtilityEntries keeps system providers discoverable", () => {
  const entries = appendShellUtilityEntries([
    {
      routeId: "/explore",
      label: "Explore",
      icon: "compass",
      category: "execution",
      requiredRole: "operator",
    },
  ]);

  const providers = entries.filter((entry) => entry.routeId === "/system/providers");

  assert.equal(providers.length, 1);
  assert.equal(providers[0]?.label, "Providers");
});
