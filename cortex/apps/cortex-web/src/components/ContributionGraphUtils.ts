export function statusColor(status: string): string {
  switch (status) {
    case "active":
      return "#42c0ff";
    case "completed":
      return "#49d18f";
    case "superseded":
      return "#ff7a70";
    case "deferred":
      return "#f0ad42";
    default:
      return "#94a3b8";
  }
}

export function layerColor(layer: string): string {
  switch (layer) {
    case "protocol":
      return "#9b8bf7";
    case "runtime":
      return "#58d4b5";
    case "host":
      return "#f792bf";
    case "adapter":
      return "#ffd166";
    default:
      return "#8aa2c2";
  }
}
