import assert from "node:assert/strict";
import test from "node:test";

import {
  internetIdentityProviderUrl,
  isInternetIdentityOperatorLoginEnabled,
} from "../src/components/commons/internetIdentityOperatorSession.ts";

function setWindowLocation(hostname: string, origin = `https://${hostname}`): void {
  globalThis.window = {
    location: {
      hostname,
      origin,
    },
  } as unknown as Window & typeof globalThis;
}

test("Internet Identity operator login is disabled by default even on public and local hosts", () => {
  setWindowLocation("nostra-cortex-eu1.vercel.app");
  assert.equal(isInternetIdentityOperatorLoginEnabled(), false);

  setWindowLocation("localhost", "http://localhost:5173");
  assert.equal(isInternetIdentityOperatorLoginEnabled(), false);
});

test("Internet Identity provider defaults to current mainnet provider", () => {
  assert.equal(internetIdentityProviderUrl(), "https://id.ai/authorize");
});
