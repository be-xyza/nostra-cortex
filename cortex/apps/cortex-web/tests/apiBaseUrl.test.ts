import assert from "node:assert/strict";
import test from "node:test";

import {
  gatewayBaseUrl,
  gatewayWsBase,
  resolveGatewayBaseUrl,
  resolveRequestCredentials,
  resolveRequestCredentialsForBase,
} from "../src/api.ts";

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

test("cross-origin Vercel gateway requests omit browser credentials", () => {
  const originalWindow = globalThis.window;

  globalThis.window = {
    location: {
      hostname: "nostra-cortex-eu1.vercel.app",
      origin: "https://nostra-cortex-eu1.vercel.app",
      protocol: "https:",
      host: "nostra-cortex-eu1.vercel.app",
    },
  } as typeof window;

  try {
    assert.equal(
      resolveRequestCredentialsForBase("https://eudaemon-alpha-01.taild09100.ts.net"),
      "omit",
    );
  } finally {
    globalThis.window = originalWindow;
  }
});

test("same-origin local gateway requests keep credentials", () => {
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
    assert.equal(resolveRequestCredentials(), "include");
  } finally {
    globalThis.window = originalWindow;
  }
});
