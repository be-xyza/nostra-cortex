export type BenchmarkGrade = "PASS" | "WARN" | "FAIL" | "UNKNOWN";

export interface BenchmarkProjection {
  grade: BenchmarkGrade;
  passRate: number | null;
  latencyMs: number | null;
  totalTokens: number | null;
  assertionsPassed: number | null;
  assertionsTotal: number | null;
  summary: string;
}

type BenchmarkLike = Record<string, unknown> | null | undefined;

function readNumber(value: unknown): number | null {
  if (typeof value === "number" && Number.isFinite(value)) return value;
  if (typeof value === "string") {
    const parsed = Number(value.trim());
    return Number.isFinite(parsed) ? parsed : null;
  }
  return null;
}

function readNumericField(record: Record<string, unknown>, ...keys: string[]): number | null {
  for (const key of keys) {
    const value = readNumber(record[key]);
    if (value !== null) return value;
  }
  return null;
}

function normalizePassRate(value: number | null): number | null {
  if (value === null) return null;
  if (value > 1 && value <= 100) {
    return value / 100;
  }
  return value;
}

function deriveGrade(passRate: number | null, assertionsPassed: number | null, assertionsTotal: number | null): BenchmarkGrade {
  const normalizedPassRate = normalizePassRate(passRate);
  const assertionRatio =
    assertionsPassed !== null && assertionsTotal !== null && assertionsTotal > 0
      ? assertionsPassed / assertionsTotal
      : null;
  const score = normalizedPassRate ?? assertionRatio;

  if (score === null) return "UNKNOWN";

  if (score >= 0.95 && (assertionRatio === null || assertionRatio >= 0.95)) {
    return "PASS";
  }

  if (score < 0.75 || (assertionRatio !== null && assertionRatio < 0.75)) {
    return "FAIL";
  }

  return "WARN";
}

function formatPassRate(passRate: number | null): string | null {
  if (passRate === null) return null;
  const normalized = normalizePassRate(passRate);
  if (normalized === null) return null;
  return `${Math.round(normalized * 100)}% pass rate`;
}

function formatAssertions(assertionsPassed: number | null, assertionsTotal: number | null): string | null {
  if (assertionsPassed === null || assertionsTotal === null) return null;
  return `${assertionsPassed} of ${assertionsTotal} assertions passed`;
}

function formatLatency(latencyMs: number | null): string | null {
  if (latencyMs === null) return null;
  return `${Math.round(latencyMs)}ms latency`;
}

function formatTotalTokens(totalTokens: number | null): string | null {
  if (totalTokens === null) return null;
  return `${Math.round(totalTokens)} tokens`;
}

export function buildBenchmarkProjection(benchmark: BenchmarkLike): BenchmarkProjection | null {
  if (!benchmark || typeof benchmark !== "object" || Array.isArray(benchmark)) {
    return null;
  }

  const record = benchmark as Record<string, unknown>;
  const passRate = readNumericField(record, "pass_rate", "passRate");
  const latencyMs = readNumericField(record, "latency_ms", "latencyMs");
  const totalTokens = readNumericField(record, "total_tokens", "totalTokens");
  const assertionsPassed = readNumericField(record, "assertions_passed", "assertionsPassed");
  const assertionsTotal = readNumericField(record, "assertions_total", "assertionsTotal");

  const summaryParts = [
    formatPassRate(passRate),
    formatAssertions(assertionsPassed, assertionsTotal),
    formatLatency(latencyMs),
    formatTotalTokens(totalTokens),
  ].filter((part): part is string => Boolean(part));

  return {
    grade: deriveGrade(passRate, assertionsPassed, assertionsTotal),
    passRate: normalizePassRate(passRate),
    latencyMs,
    totalTokens,
    assertionsPassed,
    assertionsTotal,
    summary: summaryParts.length > 0 ? summaryParts.join(" • ") : "No benchmark data available",
  };
}
