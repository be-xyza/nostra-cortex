import { FIXTURES } from "./fixtures.js";
import { A2UINode } from "./types.js";

export { A2UINode } from "./types.js";

export const MOCK_A2UI_PAYLOAD: A2UINode =
    (FIXTURES["terminal-approval"].surfaceJson.a2ui?.tree as A2UINode | undefined) ?? {
        id: "empty-root",
        type: "Container",
        componentProperties: {},
        children: { explicitList: [] },
    };
