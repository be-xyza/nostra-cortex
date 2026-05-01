import assert from "node:assert/strict";
import test from "node:test";

import { formatHeapCardTimestamp } from "../src/components/heap/heapCardTimestamp.ts";

test("heap card timestamp includes date and time in one label", () => {
  const formatted = formatHeapCardTimestamp("2026-04-30T18:45:00Z");

  assert.match(formatted, /Apr|4/);
  assert.match(formatted, /30/);
  assert.match(formatted, /45/);
});

test("heap card timestamp preserves invalid fallback values", () => {
  assert.equal(formatHeapCardTimestamp("pending"), "pending");
  assert.equal(formatHeapCardTimestamp(""), "n/a");
});
