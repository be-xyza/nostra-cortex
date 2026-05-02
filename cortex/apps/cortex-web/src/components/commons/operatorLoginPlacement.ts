import type { AuthSession } from "../../contracts.ts";

export type OperatorLoginPlacement = "hidden" | "authority_lane" | "observer_details";

interface OperatorLoginPlacementInput {
  enabled: boolean;
  session: Pick<AuthSession, "authMode"> | null;
  collapsed: boolean;
}

export function resolveOperatorLoginPlacement({
  enabled,
  session,
  collapsed,
}: OperatorLoginPlacementInput): OperatorLoginPlacement {
  if (!enabled || session?.authMode !== "read_fallback") {
    return "hidden";
  }
  return collapsed ? "observer_details" : "authority_lane";
}
