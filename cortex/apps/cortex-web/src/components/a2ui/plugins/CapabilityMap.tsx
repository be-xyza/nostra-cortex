import React from "react";
import type { A2UIComponentProps } from "../WidgetRegistry";
import { CapabilityMapWidget } from "../WidgetRegistry";

export default function CapabilityMap(props: A2UIComponentProps) {
  return React.createElement(CapabilityMapWidget, props);
}
