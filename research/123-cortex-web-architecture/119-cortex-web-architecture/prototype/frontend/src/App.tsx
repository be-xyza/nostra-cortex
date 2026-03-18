import React, { useEffect } from "react";
import { useA2uiProcessor } from "./a2ui/useA2uiProcessor";
import { A2uiRoot } from "./a2ui/A2uiRoot";
import "./App.css";

const App: React.FC = () => {
  const { surfaces, processMessage } = useA2uiProcessor();

  useEffect(() => {
    // 1. Connect to the mock AG-UI backend streaming SSE events
    const eventSource = new EventSource("http://localhost:8000/stream");

    // 2. Listen for 'a2uiUpdate' AG-UI envelopes
    eventSource.addEventListener("a2uiUpdate", (event: MessageEvent) => {
      try {
        const globalEvent = JSON.parse(event.data);
        console.log("AG-UI GlobalEvent received: ", globalEvent);
        const payload = globalEvent.payload;
        // 3. De-envelope AG-UI and feed the pure A2UI payload to our React renderer hook
        processMessage(payload);
      } catch (err) {
        console.error("Error parsing AG-UI event payload:", err);
      }
    });

    eventSource.onerror = (err) => {
      console.error("SSE stream error", err);
      eventSource.close();
    };

    return () => {
      eventSource.close();
    };
  }, [processMessage]);

  // Forward the action back to the AG-UI backend proxy
  const handleAction = async (actionName: string, context?: any) => {
    console.log("Intercepted human-in-the-loop action:", actionName, context);
    try {
      await fetch("http://localhost:8000/action", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ actionName, context }),
      });
    } catch (err) {
      console.error("Failed to forward AG-UI action", err);
    }
  };

  return (
    <div className="flex flex-col h-screen w-full bg-slate-50">
      <header className="p-4 bg-blue-600 text-white shadow-md flex justify-between items-center">
        <h2 className="text-xl font-bold">Cortex React Agent: Native UI Demo</h2>
        <span className="text-sm bg-blue-700 px-3 py-1 rounded-full">
          {surfaces.size > 0 ? "Connected (Receiving UI Stream...)" : "Awaiting Stream"}
        </span>
      </header>

      <main className="flex-1 p-8 flex justify-center overflow-auto">
        <div className="w-full max-w-2xl bg-white shadow-lg rounded-2xl p-6 border border-gray-100 min-h-[400px]">
          {surfaces.size === 0 ? (
            <div className="flex items-center justify-center h-full text-gray-400">
              No AG-UI surfaces active.
            </div>
          ) : (
            Array.from(surfaces.entries()).map(([surfaceId, surface]) => (
              <A2uiRoot
                key={surfaceId}
                surface={surface}
                onAction={handleAction}
              />
            ))
          )}
        </div>
      </main>
    </div>
  );
};

export default App;
