import type { ProviderFormState } from "./providerForm.ts";
import type { ProviderRegistryPanelState } from "./providerRegistryView.ts";

function normalizeMultiline(value: string): string {
  return value.replace(/\r\n/g, "\n").trimEnd();
}

export function serializeProviderFormSnapshot(form: ProviderFormState): string {
  return JSON.stringify({
    ...form,
    name: form.name.trim(),
    providerId: form.providerId.trim(),
    providerKind: form.providerKind.trim(),
    endpoint: form.endpoint.trim(),
    defaultModel: form.defaultModel.trim(),
    credentialBindingId: form.credentialBindingId.trim(),
    apiKey: form.apiKey,
    metadataJson: normalizeMultiline(form.metadataJson),
  });
}

export function shouldWarnBeforeClosingProviderPanel(input: {
  panelState: ProviderRegistryPanelState;
  isDirty: boolean;
  isSubmitting: boolean;
}): boolean {
  const isProviderPanel = input.panelState.kind === "provider" || input.panelState.kind === "create";
  return isProviderPanel && input.isDirty && !input.isSubmitting;
}
