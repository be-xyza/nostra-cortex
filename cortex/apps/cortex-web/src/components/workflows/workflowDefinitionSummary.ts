import type { WorkflowDefinitionResponse } from "../../contracts.ts";

export function summarizeWorkflowDefinition(
  response: WorkflowDefinitionResponse | null
): { motifKind: string | null; digest: string | null } {
  return {
    motifKind: response?.definition.motifKind ?? null,
    digest: response?.definition.digest ?? null,
  };
}
