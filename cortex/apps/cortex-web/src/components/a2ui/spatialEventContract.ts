export const A2UI_EVENT_TYPES = [
  "button_click",
  "approval",
  "spatial_shape_click",
  "spatial_shape_move",
  "spatial_edge_connect",
  "spatial_adapter_loaded",
  "spatial_adapter_fallback",
  "spatial_adapter_replay",
  "spatial_adapter_replay_failed"
] as const;

export type A2UIEventType = (typeof A2UI_EVENT_TYPES)[number];

export function emitA2uiEvent(eventType: A2UIEventType, detail: Record<string, unknown>) {
  window.dispatchEvent(new CustomEvent("cortex:a2ui:event", { detail: { eventType, ...detail } }));
}
