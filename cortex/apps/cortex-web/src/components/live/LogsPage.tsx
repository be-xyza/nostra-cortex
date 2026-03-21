import { useEffect, useState, useRef } from "react";
import { Terminal, Copy, Trash2, PowerOff, Power } from "lucide-react";

export function LogsPage() {
  const [logs, setLogs] = useState<string[]>([]);
  const [connected, setConnected] = useState(false);
  const [autoScroll, setAutoScroll] = useState(true);
  const wsRef = useRef<WebSocket | null>(null);
  const endRef = useRef<HTMLDivElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const gatewayUrl = import.meta.env.VITE_CORTEX_GATEWAY_URL || "http://127.0.0.1:3000";
    const wsUrl = gatewayUrl.replace(/^http/, "ws") + "/api/system/logs/stream";
    
    let reconnectTimer: number;

    const connect = () => {
      try {
        const ws = new WebSocket(wsUrl);
        wsRef.current = ws;

        ws.onopen = () => {
          setConnected(true);
          setLogs(prev => [...prev, `[system] Connected to live telemetry stream: ${wsUrl}`]);
        };

        ws.onmessage = (event) => {
          try {
            // Logs are broadcast as plain strings
            setLogs(prev => {
              const newLogs = [...prev, event.data];
              // Keep maximum 2000 lines in memory to prevent DOM lag
              if (newLogs.length > 2000) return newLogs.slice(newLogs.length - 2000);
              return newLogs;
            });
          } catch (e) {
            console.error("Failed to parse log message", e);
          }
        };

        ws.onclose = () => {
          setConnected(false);
          setLogs(prev => [...prev, `[system] Disconnected from telemetry stream. Reconnecting in 3s...`]);
          reconnectTimer = window.setTimeout(connect, 3000);
        };

        ws.onerror = (err) => {
          console.error("WebSocket error:", err);
          ws.close();
        };
      } catch (err) {
        console.error("Failed to setup WebSocket:", err);
        setLogs(prev => [...prev, `[system] Failed to connect: ${err}`]);
      }
    };

    connect();

    return () => {
      clearTimeout(reconnectTimer);
      if (wsRef.current) {
        // Prevent onclose handle from firing and triggering reconnect
        wsRef.current.onclose = null;
        wsRef.current.close();
      }
    };
  }, []);

  useEffect(() => {
    if (autoScroll && endRef.current) {
      endRef.current.scrollIntoView({ behavior: "smooth" });
    }
  }, [logs, autoScroll]);

  const handleScroll = () => {
    if (!containerRef.current) return;
    const { scrollTop, scrollHeight, clientHeight } = containerRef.current;
    
    // If we're within 50px of the bottom, turn auto-scroll back on
    const isAtBottom = scrollHeight - scrollTop - clientHeight < 50;
    if (isAtBottom !== autoScroll) {
      setAutoScroll(isAtBottom);
    }
  };

  const clearLogs = () => setLogs([]);

  const copyLogs = () => {
    navigator.clipboard.writeText(logs.join("\n")).then(() => {
      // Could show a toast here
    });
  };

  const toggleConnection = () => {
    if (connected && wsRef.current) {
      wsRef.current.close(); // Will trigger auto-reconnect currently, but let's just close it
      // Actually we need to alter the reconnect logic to support manual disconnect, 
      // but for simplicity, closing it will just auto-reconnect in 3s
    }
  };

  return (
    <div className="flex flex-col h-full bg-cortex-bg border-l border-cortex-line">
      {/* Header toolbar */}
      <div className="flex items-center justify-between px-4 py-3 border-b border-cortex-line bg-cortex-surface">
        <div className="flex items-center gap-2 text-cortex-ink">
          {connected ? (
            <Power className="w-4 h-4 text-green-500 animate-pulse" />
          ) : (
            <PowerOff className="w-4 h-4 text-red-500" />
          )}
          <h2 className="font-semibold text-sm flex items-center gap-2">
            <Terminal className="w-4 h-4" /> Live Telemetry Pulse
          </h2>
          <span className="text-xs text-cortex-subtle ml-2">
            {connected ? "Streaming" : "Offline"} • {logs.length} events
          </span>
        </div>
        <div className="flex items-center gap-2">
          <button 
            onClick={copyLogs}
            className="p-1.5 text-cortex-subtle hover:text-cortex-ink hover:bg-cortex-line rounded transition-colors"
            title="Copy all logs"
          >
            <Copy className="w-4 h-4" />
          </button>
          <button 
            onClick={clearLogs}
            className="p-1.5 text-cortex-subtle hover:text-red-400 hover:bg-cortex-line rounded transition-colors"
            title="Clear logs"
          >
            <Trash2 className="w-4 h-4" />
          </button>
        </div>
      </div>

      {/* Terminal View */}
      <div 
        ref={containerRef}
        onScroll={handleScroll}
        className="flex-1 p-4 overflow-y-auto font-mono text-xs bg-[#0D0D0D] text-green-400 select-text break-all"
        style={{ scrollBehavior: 'smooth' }}
      >
        {logs.length === 0 ? (
          <div className="text-gray-600 italic">Waiting for Eudaemon Alpha telemetry...</div>
        ) : (
          logs.map((log, i) => (
            <div key={i} className="mb-1 leading-relaxed whitespace-pre-wrap">
              {log}
            </div>
          ))
        )}
        <div ref={endRef} className="h-4" />
      </div>
      
      {/* Auto-scroll indicator/toggle */}
      {!autoScroll && (
        <div className="absolute bottom-6 right-6">
          <button 
            onClick={() => {
              setAutoScroll(true);
              if (endRef.current) endRef.current.scrollIntoView({ behavior: 'smooth' });
            }}
            className="px-3 py-1.5 bg-cortex-surface border border-cortex-line text-xs font-medium rounded shadow-lg hover:bg-cortex-line transition-colors flex items-center gap-2"
          >
            Resume Auto-scroll ↓
          </button>
        </div>
      )}
    </div>
  );
}
