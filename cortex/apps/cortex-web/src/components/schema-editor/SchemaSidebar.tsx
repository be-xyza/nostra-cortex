import React from 'react';

const CAPABILITY_TEMPLATES = [
  { type: 'domain_read', label: 'Domain Reader', icon: '📖', color: 'blue' },
  { type: 'domain_write', label: 'Domain Writer', icon: '📝', color: 'green' },
  { type: 'execution', label: 'Execution Worker', icon: '⚙️', color: 'purple' },
  { type: 'system', label: 'System Service', icon: '🛠️', color: 'orange' },
  { type: 'governance', label: 'Governance Gate', icon: '⚖️', color: 'red' },
];

export const SchemaSidebar = () => {
  const onDragStart = (event: React.DragEvent, nodeType: string) => {
    event.dataTransfer.setData('application/reactflow', nodeType);
    event.dataTransfer.effectAllowed = 'move';
  };

  return (
    <div className="w-64 bg-slate-900 border-r border-slate-800 p-4 flex flex-col gap-4 overflow-y-auto">
      <div className="flex flex-col gap-1 mb-2">
        <h3 className="font-bold text-slate-200 text-sm uppercase tracking-wider">Capability Library</h3>
        <p className="text-[10px] text-slate-500">Drag items to the canvas to extend the schema.</p>
      </div>

      <div className="flex flex-col gap-2">
        {CAPABILITY_TEMPLATES.map((tmpl) => (
          <div
            key={tmpl.type}
            className={`p-3 rounded-lg border border-slate-800 bg-slate-800/50 hover:bg-slate-800 cursor-grab active:cursor-grabbing transition-colors flex items-center gap-3 group`}
            onDragStart={(event) => onDragStart(event, tmpl.type)}
            draggable
          >
            <span className="text-xl group-hover:scale-110 transition-transform">{tmpl.icon}</span>
            <div className="flex flex-col">
              <span className="text-xs font-bold text-slate-300">{tmpl.label}</span>
              <span className="text-[9px] text-slate-500 uppercase font-mono">{tmpl.type}</span>
            </div>
          </div>
        ))}
      </div>

      <div className="mt-auto pt-4 border-t border-slate-800 flex flex-col gap-2">
        <div className="text-[10px] text-slate-500 font-mono">STABILITY: ALPHA</div>
        <div className="text-[10px] font-mono text-blue-400">SYNC: ENABLED</div>
      </div>
    </div>
  );
};
