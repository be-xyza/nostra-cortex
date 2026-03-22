import type { ShellLayoutSpec, WhoAmIResponse } from "../../contracts.ts";

const FALLBACK_LAYOUT_SPEC: ShellLayoutSpec = {
  layoutId: "default",
  navigationGraph: {
    entries: [
      {
        routeId: "/explore",
        label: "Explore",
        icon: "compass",
        category: "execution",
        requiredRole: "operator",
        navSlot: "primary_platform",
      },
      {
        routeId: "/workflows",
        label: "Flows",
        icon: "git-branch",
        category: "bridge",
        requiredRole: "operator",
        navSlot: "primary_execute",
      },
      {
        routeId: "/contributions",
        label: "Contributions",
        icon: "git-merge",
        category: "bridge",
        requiredRole: "operator",
        navSlot: "secondary_ops",
      },
      {
        routeId: "/artifacts",
        label: "Artifacts",
        icon: "file-code",
        category: "bridge",
        requiredRole: "operator",
        navSlot: "secondary_ops",
      },
      {
        routeId: "/logs",
        label: "System Logs",
        icon: "terminal",
        category: "bridge",
        requiredRole: "operator",
        navSlot: "secondary_ops",
      },
      {
        routeId: "/studio",
        label: "Flow Studio",
        icon: "code",
        category: "execution",
        requiredRole: "operator",
        navSlot: "primary_execute",
      },
      {
        routeId: "/labs",
        label: "Labs",
        icon: "flask-conical",
        category: "workbench",
        requiredRole: "operator",
        navSlot: "labs",
      },
      {
        routeId: "/system/providers",
        label: "API Providers",
        icon: "shield-alert",
        category: "system",
        requiredRole: "operator",
        navSlot: "secondary_admin",
      },
    ],
  },
};

const FALLBACK_WHOAMI: WhoAmIResponse = {
  schemaVersion: "1.0.0",
  generatedAt: "1970-01-01T00:00:00.000Z",
  principal: "local-user",
  requestedRole: "operator",
  effectiveRole: "operator",
  claims: ["*"],
  identityVerified: true,
  identitySource: "local",
  authzDevMode: true,
  allowUnverifiedRoleHeader: true,
  authzDecisionVersion: "1.0",
};

function normalizeRole(role?: string): string {
  const normalized = role?.trim().toLowerCase();
  return normalized && normalized.length > 0
    ? normalized
    : FALLBACK_WHOAMI.effectiveRole;
}

function normalizeActorId(actorId?: string): string {
  const normalized = actorId?.trim();
  return normalized && normalized.length > 0
    ? normalized
    : FALLBACK_WHOAMI.principal || "local-user";
}

export function buildFallbackShellLayoutSpec(): ShellLayoutSpec {
  return {
    layoutId: FALLBACK_LAYOUT_SPEC.layoutId,
    navigationGraph: {
      entries: FALLBACK_LAYOUT_SPEC.navigationGraph.entries.map((entry) => ({
        ...entry,
      })),
    },
  };
}

export function buildFallbackWhoami(
  actorId?: string,
  actorRole?: string,
  generatedAt = new Date().toISOString(),
): WhoAmIResponse {
  const normalizedRole = normalizeRole(actorRole);
  return {
    ...FALLBACK_WHOAMI,
    generatedAt,
    principal: normalizeActorId(actorId),
    requestedRole: normalizedRole,
    effectiveRole: normalizedRole,
  };
}

export function formatShellBootstrapWarning(
  context: "layout" | "identity",
  error: string,
  gatewayTarget?: string,
): string {
  const targetSuffix = gatewayTarget?.trim()
    ? ` Target: ${gatewayTarget.trim()}.`
    : "";
  const prefix =
    context === "layout"
      ? "Gateway unavailable. Using local preview shell."
      : "Identity endpoint unavailable. Using local preview role.";
  return `${prefix}${targetSuffix} ${error}`.trim();
}
