import React from "react";
import { HeapBlockGrid } from "../heap/HeapBlockGrid";
import type { A2UIComponentProps } from "./WidgetRegistry";

export function HeapBoard({ componentProperties }: A2UIComponentProps) {
    const props = componentProperties["HeapBoard"] as Record<string, unknown> || {};
    const spaceId = String(props.spaceId || "");
    return React.createElement(HeapBlockGrid, {
        filterDefaults: { spaceId },
        showFilterSidebar: false,
    });
}
