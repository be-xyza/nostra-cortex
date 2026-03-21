export type HeapActionZone = "page" | "selection" | "detail" | "detailHeader" | "cardMenu";

export type HeapActionId =
  | "create"
  | "regenerate"
  | "refine_selection"
  | "export"
  | "history"
  | "publish"
  | "synthesize"
  | "pin"
  | "delete"
  | "discussion"
  | "edit";

export interface HeapActionDescriptor {
  id: HeapActionId;
  zone: HeapActionZone;
  label: string;
  title: string;
  icon?: string;
  emphasis?: "default" | "primary" | "danger";
  enabled: boolean;
  disabledReason?: string;
}

export interface HeapActionPlan {
  page: HeapActionDescriptor[];
  selection: HeapActionDescriptor[];
  detail: HeapActionDescriptor[];
  detailHeader: HeapActionDescriptor[];
  cardMenu: HeapActionDescriptor[];
}

export interface BuildHeapActionPlanOptions {
  selectionCount: number;
  heapCreateFlowEnabled: boolean;
  heapParityEnabled: boolean;
}

function disabledByParity(heapParityEnabled: boolean): string | undefined {
  return heapParityEnabled ? undefined : "Heap parity features are disabled.";
}

function buildSelectionAction(
  options: BuildHeapActionPlanOptions,
  config: Omit<HeapActionDescriptor, "zone" | "enabled" | "disabledReason"> & {
    minSelected?: number;
    maxSelected?: number;
  },
): HeapActionDescriptor | null {
  const { selectionCount, heapParityEnabled } = options;
  const parityDisabledReason = disabledByParity(heapParityEnabled);

  if (selectionCount === 0) {
    return null;
  }

  let enabled = !parityDisabledReason;
  let disabledReason = parityDisabledReason;

  if (enabled && typeof config.minSelected === "number" && selectionCount < config.minSelected) {
    enabled = false;
    disabledReason = `Requires at least ${config.minSelected} blocks selected.`;
  }
  if (enabled && typeof config.maxSelected === "number" && selectionCount > config.maxSelected) {
    return null;
  }

  return {
    id: config.id,
    zone: "selection",
    label: config.label,
    title: config.title,
    icon: config.icon,
    emphasis: config.emphasis,
    enabled,
    disabledReason,
  };
}

export function buildHeapActionPlan(options: BuildHeapActionPlanOptions): HeapActionPlan {
  const page: HeapActionDescriptor[] = [];
  const selection = [
    buildSelectionAction(options, {
      id: "regenerate",
      label: "Regen",
      title: "Regenerate",
      icon: "✦",
      maxSelected: 1,
    }),
    buildSelectionAction(options, {
      id: "refine_selection",
      label: "Refine Selection",
      title: "Refine selected context bundle",
      icon: "⊞",
      minSelected: 1,
    }),
    buildSelectionAction(options, {
      id: "export",
      label: "Export",
      title: "Export",
      icon: "⤓",
      minSelected: 1,
    }),
    buildSelectionAction(options, {
      id: "history",
      label: "History",
      title: "History",
      icon: "🕘",
      maxSelected: 1,
    }),
    buildSelectionAction(options, {
      id: "publish",
      label: "Publish",
      title: "Publish via Steward Gate",
      icon: "⇪",
      emphasis: "primary",
      maxSelected: 1,
    }),
    buildSelectionAction(options, {
      id: "synthesize",
      label: "Synthesize",
      title: "Synthesize selected blocks into a new summary",
      icon: "✨",
      minSelected: 3,
    }),
    buildSelectionAction(options, {
      id: "pin",
      label: "Pin",
      title: "Toggle Pin",
      icon: "📌",
      minSelected: 1,
    }),
    buildSelectionAction(options, {
      id: "delete",
      label: "Delete",
      title: "Delete Block",
      icon: "🗑",
      emphasis: "danger",
      minSelected: 1,
    }),
  ].filter((action): action is HeapActionDescriptor => action !== null);

  if (options.heapCreateFlowEnabled) {
    page.push({
      id: "create",
      zone: "page",
      label: "Create",
      title: "Open create block panel",
      icon: "+",
      emphasis: "primary",
      enabled: true,
    });
  }

  const detailDisabledReason = disabledByParity(options.heapParityEnabled);
  const detail: HeapActionDescriptor[] = [
    {
      id: "discussion",
      zone: "detail",
      label: "Discussion",
      title: "View discussion",
      enabled: true,
    },
    {
      id: "edit",
      zone: "detail",
      label: "Edit",
      title: "Edit block",
      enabled: false,
      disabledReason: "Edit flow is not implemented in the web heap yet.",
    },
    {
      id: "regenerate",
      zone: "detail",
      label: "Regenerate",
      title: "Regenerate",
      emphasis: "primary",
      enabled: !detailDisabledReason,
      disabledReason: detailDisabledReason,
    },
  ];

  const detailHeader = detail.map((action) => ({
    ...action,
    zone: "detailHeader" as const,
  }));

  const cardDisabledReason = disabledByParity(options.heapParityEnabled);
  const cardMenu: HeapActionDescriptor[] = [
    {
      id: "discussion",
      zone: "cardMenu",
      label: "Discussion",
      title: "Open discussion for this block",
      enabled: true,
    },
    {
      id: "history",
      zone: "cardMenu",
      label: "History",
      title: "Inspect version history",
      enabled: !cardDisabledReason,
      disabledReason: cardDisabledReason,
    },
    {
      id: "pin",
      zone: "cardMenu",
      label: "Pin",
      title: "Pin this block",
      enabled: !cardDisabledReason,
      disabledReason: cardDisabledReason,
    },
    {
      id: "delete",
      zone: "cardMenu",
      label: "Delete",
      title: "Delete this block",
      emphasis: "danger",
      enabled: !cardDisabledReason,
      disabledReason: cardDisabledReason,
    },
  ];

  return {
    page,
    selection,
    detail,
    detailHeader,
    cardMenu,
  };
}
