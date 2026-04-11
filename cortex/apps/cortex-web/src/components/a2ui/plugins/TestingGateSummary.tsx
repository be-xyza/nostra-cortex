import React from "react";
import type { A2UIComponentProps } from "../WidgetRegistry";
import { GateSummary as InnerGateSummary } from "../GateSummary";

export function TestingGateSummary(props: A2UIComponentProps) {
  return React.createElement(InnerGateSummary, {
    payload: (props.componentProperties.payload || props.componentProperties) as any
  });
}
