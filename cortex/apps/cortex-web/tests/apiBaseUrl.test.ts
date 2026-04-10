import assert from "node:assert/strict";
import test from "node:test";

import { gatewayBaseUrl, gatewayWsBase, resolveGatewayBaseUrl } from "../src/api.ts";

test("resolveGatewayBaseUrl prefers the local proxy on localhost", () => {
  const originalWindow = globalThis.window;

  globalThis.window = {
    location: {
      hostname: "localhost",
      origin: "http://localhost:4173",
      protocol: "http:",
      host: "localhost:4173",
    },
  } as typeof window;

  try {
    assert.equal(resolveGatewayBaseUrl(), "http://localhost:4173");
    assert.equal(gatewayBaseUrl(), "http://localhost:4173");
  } finally {
    globalThis.window = originalWindow;
  }
});

test("gatewayWsBase prefers the local websocket origin on localhost", () => {
  const originalWindow = globalThis.window;

  globalThis.window = {
    location: {
      hostname: "localhost",
      origin: "http://localhost:4173",
      protocol: "http:",
      host: "localhost:4173",
    },
  } as typeof window;

  try {
    assert.equal(gatewayWsBase(), "ws://localhost:4173");
  } finally {
    globalThis.window = originalWindow;
  }
});
