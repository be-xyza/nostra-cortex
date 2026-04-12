# Terminal Surface Handoff Contract

## Purpose

Define how operator-side terminal hosts classify surfaces and when they must hand off to `cortex-web`.

## Authority Boundary

- Terminal hosts are Cortex execution hosts.
- They do not define authority.
- They consume existing heap, workflow, and A2UI payload contracts.
- They must not introduce terminal-only authority schemas.
- They must validate terminal-renderable documents against `terminal_document_v1` before rendering.

## Input Contract

The terminal host consumes an artifact envelope with:

```json
{
  "artifactId": "optional-artifact-id",
  "title": "optional-title",
  "routeHint": "explore | artifacts | workflows | labs",
  "workflowHref": "optional workflow/api path",
  "surfaceJson": {
    "payload_type": "a2ui | rich_text | note | media | structured_data | pointer | task | chart | telemetry"
  }
}
```

## Classification Modes

### `terminal_render`
Use when:
- the payload is terminal-safe A2UI
- all widgets belong to the supported terminal subset
- the normalized terminal document passes `terminal_document_v1`

### `terminal_summary`
Use when:
- the payload is readable in terminal without richer navigation

### `terminal_summary_with_handoff`
Use when:
- terminal can summarize the content
- but richer inspection belongs in `cortex-web`

### `web_handoff`
Use when:
- the surface exceeds terminal fidelity
- interaction depends on browser-native or richer workbench features
- the A2UI tree or generated terminal document fails strict validation

## Supported Terminal Widget Subset

- `Container`
- `Box`
- `Text`
- `Spacer`
- `SelectList`

Any other widget should be treated as unsupported unless explicitly added to this contract.

## Validation Rule

- `terminal_document_v1` is the strict gate for terminal rendering.
- Unsupported or malformed documents fail closed into `web_handoff` or `terminal_summary_with_handoff`.
- Terminal hosts must not attempt best-effort rendering for invalid documents.

## Default Web-Required Families

- workflow inspector widgets
- spatial/tldraw widgets
- capability maps and matrices
- schema editors
- contribution graph viewers
- evaluator DAG viewers

## Handoff Routes

### Artifact route
- `/explore?artifact_id=<artifact_id>`

### Workflow route
- `/workflows?node_id=<url_encoded_gateway_api_path>`

## Non-Goals

- browser parity inside terminal
- media-native fidelity in terminal
- terminal-specific payload schema forks
- redefining `cortex-web` as optional for rich workbench flows

## Host Rollout

- desktop-first remains the intended promotion target through the existing ACP terminal endpoints
- current live proof is ACP-compatible and was exercised through `cortex-eudaemon`
- desktop service parity is partially wired in this branch, but desktop is not yet packaged here as a runnable gateway binary
- eudaemon parity remains useful as a compatible host proof, not a substitute for eventual desktop packaging
