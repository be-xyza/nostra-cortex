import type { AuthSession, ShellLayoutSpec, WhoAmIResponse } from "../../contracts.ts";
import { isLocalDevBootstrapEnabled } from "../../localDevBootstrap.ts";

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
        label: "Providers",
        icon: "shield-alert",
        category: "system",
        requiredRole: "operator",
        navSlot: "secondary_admin",
      },
    ],
  },
};

const FALLBACK_SESSION: AuthSession = {
  schemaVersion: "1.0.0",
  generatedAt: "1970-01-01T00:00:00.000Z",
  principal: "local-user",
  sessionId: "fallback-session",
  identityVerified: false,
  identitySource: "read_fallback_viewer",
  authMode: "read_fallback",
  grantedRoles: ["viewer"],
  activeRole: "viewer",
  globalClaims: [],
  spaceGrants: [],
  allowRoleSwitch: false,
  allowUnverifiedRoleHeader: false,
};

const FALLBACK_WHOAMI: WhoAmIResponse = {
  schemaVersion: "1.0.0",
  generatedAt: "1970-01-01T00:00:00.000Z",
  principal: FALLBACK_SESSION.principal,
  requestedRole: "viewer",
  effectiveRole: FALLBACK_SESSION.activeRole,
  claims: [],
  identityVerified: FALLBACK_SESSION.identityVerified,
  identitySource: FALLBACK_SESSION.identitySource,
  authzDevMode: false,
  allowUnverifiedRoleHeader: false,
  authzDecisionVersion: "1.0",
};

function useLocalDevBootstrap(): boolean {
  return isLocalDevBootstrapEnabled();
}

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

function resolveDevBootstrapRole(role?: string): string {
  const normalized = normalizeRole(role);
  return normalized === "steward" ? "steward" : "operator";
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
  const session = buildFallbackAuthSession(actorId, actorRole, generatedAt);
  const devBootstrap = useLocalDevBootstrap();
  return {
    ...FALLBACK_WHOAMI,
    generatedAt,
    principal: session.principal,
    requestedRole: devBootstrap
      ? resolveDevBootstrapRole(actorRole)
      : normalizeRole(actorRole),
    effectiveRole: session.activeRole,
    identitySource: session.identitySource,
    identityVerified: session.identityVerified,
    authzDevMode: devBootstrap,
    allowUnverifiedRoleHeader: session.allowUnverifiedRoleHeader,
  };
}

export function buildFallbackAuthSession(
  actorId?: string,
  actorRole?: string,
  generatedAt = new Date().toISOString(),
): AuthSession {
  if (useLocalDevBootstrap()) {
    return {
      ...FALLBACK_SESSION,
      generatedAt,
      principal: normalizeActorId(actorId),
      sessionId: "localhost-dev-bootstrap",
      identitySource: "localhost_dev_bootstrap",
      authMode: "dev_override",
      grantedRoles: ["operator", "steward"],
      activeRole: resolveDevBootstrapRole(actorRole),
      allowRoleSwitch: true,
      allowUnverifiedRoleHeader: true,
    };
  }

  return {
    ...FALLBACK_SESSION,
    generatedAt,
    principal: normalizeActorId(actorId),
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
      ? useLocalDevBootstrap()
        ? "Gateway unavailable. Using localhost dev bootstrap shell."
        : "Gateway unavailable. Using local fallback shell."
      : useLocalDevBootstrap()
        ? "Identity endpoint unavailable. Using localhost dev bootstrap role."
        : "Identity endpoint unavailable. Using local fallback role.";
  return `${prefix}${targetSuffix} ${error}`.trim();
}

export function formatReadFallbackNotice(gatewayTarget?: string): string {
  const targetSuffix = gatewayTarget?.trim()
    ? ` Target: ${gatewayTarget.trim()}.`
    : "";
  return `Gateway is reachable, but no verified operator identity is attached to this browser session.${targetSuffix} Viewer-scoped heap data remains available; operator action plans and mutations stay gated until an operator session is verified.`;
}

export function formatReadOnlyObserverSummary(): string {
  return "Read-only observer mode";
}

export type ReadOnlyObserverGatewayState = "reachable" | "public_restricted";

interface ReadOnlyObserverDetailOptions {
  operatorLoginEnabled?: boolean;
}

export function formatReadOnlyObserverDetailLines(
  gatewayTarget?: string,
  gatewayState: ReadOnlyObserverGatewayState = "reachable",
  options: ReadOnlyObserverDetailOptions = {},
): string[] {
  const lines = gatewayState === "public_restricted"
    ? [
        "Gateway target is private or browser-restricted from this public session.",
        "Browser session has no verified operator identity.",
        "Viewer shell remains available; live heap data may be limited until a trusted operator session is used.",
        "Operator action plans and mutations remain gated.",
      ]
    : [
        "Gateway reachable.",
        "Browser session has no verified operator identity.",
        "Heap data is visible in viewer mode.",
        "Operator action plans and mutations remain gated.",
      ];
  if (!options.operatorLoginEnabled) {
    lines.push("Operator sign-in is disabled for this deployment; use a trusted local or operator-enabled environment to verify a session.");
  }
  const target = gatewayTarget?.trim();
  if (target) {
    lines.push(`Gateway target: ${target}.`);
  }
  return lines;
}

export function isPublicObserverGatewayBoundary(
  error: string,
  gatewayTarget?: string,
  publicHost = false,
): boolean {
  const target = gatewayTarget?.trim();
  if (!publicHost || !target || target === "same-origin /api proxy") {
    return false;
  }
  const normalizedError = error.toLowerCase();
  return (
    normalizedError.includes("failed to fetch")
    || normalizedError.includes("load failed")
    || normalizedError.includes("networkerror")
    || normalizedError.includes("cors")
    || normalizedError.includes("private network")
  );
}

export function describeAuthorityMode(session: AuthSession): string {
  if (session.authMode === "dev_override" || session.identitySource === "localhost_dev_bootstrap") {
    return "Local operator mode";
  }
  if (session.authMode === "read_fallback") {
    return "Read-only observer mode";
  }
  if (session.identityVerified && session.authMode === "principal_binding") {
    return "Verified operator mode";
  }
  if (session.identityVerified && session.authMode === "session_claims") {
    return "Verified operator mode";
  }
  return session.identityVerified ? "Verified" : "Unverified";
}

export function formatAuthorityStatus(session: AuthSession): string {
  const role = session.activeRole || "viewer";
  const mode = describeAuthorityMode(session);
  if (session.authMode === "read_fallback") {
    return `${role} / ${mode}: read-only observability; operator actions are gated.`;
  }
  if (session.authMode === "dev_override") {
    return `${role} / ${mode}: trusted local or preview-only authority; do not use as public production auth.`;
  }
  return `${role} / ${mode}: authority granted by gateway identity claims.`;
}
