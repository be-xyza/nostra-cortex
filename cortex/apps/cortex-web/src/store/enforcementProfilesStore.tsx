import { useMemo } from "react";
import { Eye, ShieldCheck, ShieldAlert } from "lucide-react";

export type EnforcementId = "observe" | "softgate" | "hardgate";

export interface EnforcementProfile {
  id: EnforcementId;
  label: string;
  description: string;
  icon: React.ReactNode;
  color: string;
}

// Initial dataset simulated from the NDL Graph
const ENFORCEMENT_CATALOG: Record<EnforcementId, Omit<EnforcementProfile, "id">> = {
  observe: {
    label: "Audit Only",
    description: "System behavior is monitored and logged. No actions are blocked, but all deviations from protocol are recorded for later audit.",
    color: "text-sky-400",
    icon: null, // Icons are added in the hook to avoid React component serialization issues in raw data
  },
  softgate: {
    label: "Flexible Guard",
    description: "Protection is active but adaptive. Specific protocol deviations trigger warnings or require additional manual approval from a Steward.",
    color: "text-amber-400",
    icon: null,
  },
  hardgate: {
    label: "Strict Guard",
    description: "Maximum enforcement. All actions must strictly adhere to the space's constitutional framework. Non-compliant activities are blocked immediately.",
    color: "text-red-400",
    icon: null,
  },
};

/**
 * Hook to fetch Enforcement Profiles from the 'graph' (Simulated).
 * This ensures that descriptions and labels are dynamic and can be upgraded 
 * globally without view-layer changes.
 */
export function useEnforcementProfiles() {
  return useMemo(() => {
    const iconMap = {
      observe: Eye,
      softgate: ShieldCheck,
      hardgate: ShieldAlert,
    };

    const profiles: Record<EnforcementId, EnforcementProfile> = {} as any;
    
    (Object.entries(ENFORCEMENT_CATALOG) as [EnforcementId, any][]).forEach(([id, data]) => {
      const Icon = iconMap[id];
      profiles[id] = {
        ...data,
        id,
        icon: Icon ? <Icon className="w-3 h-3" /> : null,
      };
    });

    return profiles;
  }, []);
}
