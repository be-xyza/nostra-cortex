import { WorkbenchSurfaceView } from "../commons/WorkbenchSurfaceView";
import { EXECUTION_CANVAS_ROUTE } from "./spaceStudioRoutes";

export function ExecutionCanvasPage() {
  return <WorkbenchSurfaceView routeOverride={EXECUTION_CANVAS_ROUTE} />;
}
