# Initiative 131: OpenResponses LLM Adapter

This initiative activates agent runs in `cortex-eudaemon` by integrating a local `open-responses-server` sidecar (Responses API + SSE) and projecting planner progress to `cortex-web` via repeated A2UI `surface_update` events.
