import type { ProviderRegistryPanelState } from "./providerRegistryView.ts";

export interface ProviderWorkbenchChromeState {
  compactRegistryChrome: boolean;
  compactRegistryRows: boolean;
  contentPaddingClass: string;
  headerLayoutClass: string;
  titleClass: string;
  description: string;
}

export interface ProviderDefaultModelControlState {
  control: "combobox" | "input";
  helperText: string;
}

export function buildProviderWorkbenchChromeState(_panelState: ProviderRegistryPanelState): ProviderWorkbenchChromeState {
  return {
    compactRegistryChrome: false,
    compactRegistryRows: false,
    contentPaddingClass: "",
    headerLayoutClass: "flex flex-col gap-4 xl:flex-row xl:items-end xl:justify-between",
    titleClass: "text-2xl",
    description:
      "Scan provider readiness at a glance, keep detail in a docked rail, and use discovery only when you need runtime catalog insight.",
  };
}

export function buildDefaultModelControlState(catalogSize: number): ProviderDefaultModelControlState {
  if (catalogSize > 0) {
    return {
      control: "combobox",
      helperText: `Choose from ${catalogSize} loaded models, or refine the catalog in Models.`,
    };
  }

  return {
    control: "input",
    helperText: "No catalog is loaded yet. Use Models to refresh, or type a custom model manually.",
  };
}
