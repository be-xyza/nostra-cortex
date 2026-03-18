import json
import asyncio
from fastapi import FastAPI
from fastapi.responses import StreamingResponse
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel

app = FastAPI(title="AG-UI Prototype Backend")

# Enable CORS for the React/Vite frontend
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

import time
import uuid

# AG-UI Envelope wrapper
def agui_event(event_name: str, payload: dict) -> str:
    # Wrap in GlobalEvent envelope per shared/specs.md
    global_event = {
        "id": str(uuid.uuid4()),
        "source": {"Agent": "did:nostra:cortex-agent"},
        "type": event_name,
        "resource": "nostra://cortex-web/main_view",
        "payload": payload,
        "timestamp": int(time.time() * 1000)
    }
    data = json.dumps(global_event)
    return f"event: {event_name}\ndata: {data}\n\n"

# The A2UI stream generator
async def a2ui_stream_generator():
    """
    Simulates a generative UI agent producing A2UI components progressively.
    The `component` key maps to what model-processor.ts reads as componentData.component.
    """
    # 1. Root column layout + header text
    yield agui_event("a2uiUpdate", {
        "surfaceUpdate": {
            "surfaceId": "main_view",
            "components": [
                {
                    "id": "root",
                    "component": {
                        "Column": {
                            "alignment": "start",
                            "children": {
                                "explicitList": ["header_text", "main_card"]
                            }
                        }
                    }
                },
                {
                    "id": "header_text",
                    "component": {
                        "Text": {
                            "text": { "literalString": "AG-UI meets A2UI" },
                            "usageHint": "h1"
                        }
                    }
                }
            ]
        }
    })
    await asyncio.sleep(1)

    # 2. Card with description text and action button
    yield agui_event("a2uiUpdate", {
        "surfaceUpdate": {
            "surfaceId": "main_view",
            "components": [
                {
                    "id": "main_card",
                    "component": {
                        "Card": {
                            "child": "card_column"
                        }
                    }
                },
                {
                    "id": "card_column",
                    "component": {
                        "Column": {
                            "children": {
                                "explicitList": ["card_desc", "action_btn"]
                            }
                        }
                    }
                },
                {
                    "id": "card_desc",
                    "component": {
                        "Text": {
                            "text": { "literalString": "This UI was natively streamed from the Agent via an AG-UI SSE transport." }
                        }
                    }
                },
                {
                    "id": "btn_label",
                    "component": {
                        "Text": {
                            "text": { "literalString": "Trigger Agent Action" }
                        }
                    }
                },
                {
                    "id": "action_btn",
                    "component": {
                        "Button": {
                            "label": { "literalString": "Trigger Agent Action" },
                            "child": "btn_label",
                            "action": {
                                "name": "do_something",
                                "context": [
                                    {"key": "timestamp", "value": {"literalString": "now"}}
                                ]
                            }
                        }
                    }
                }
            ]
        }
    })

    await asyncio.sleep(0.5)

    # 3. Begin Rendering
    yield agui_event("a2uiUpdate", {
        "beginRendering": {
            "surfaceId": "main_view",
            "root": "root"
        }
    })

    # Keep stream open briefly
    await asyncio.sleep(5)
    
@app.get("/stream")
async def stream_ui():
    """Endpoint that the React shell consumes to get the UI graph dynamically."""
    return StreamingResponse(a2ui_stream_generator(), media_type="text/event-stream")


class A2UIUserAction(BaseModel):
    name: str
    surfaceId: str
    sourceComponentId: str
    timestamp: str
    context: dict

class A2UIEventWrapper(BaseModel):
    userAction: A2UIUserAction

@app.post("/action")
async def handle_action(action: A2UIEventWrapper):
    """
    Endpoint that receives the userAction emitted by the frontend when a button is clicked.
    This simulates human-in-the-loop or tool interrupts.
    """
    print(f"Received human-in-the-loop action from UI: {action.userAction.name}")
    print(f"Context: {action.userAction.context}")
    return {"status": "success", "message": "Backend acknowledged action."}

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8000)
