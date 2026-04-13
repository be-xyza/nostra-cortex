import React from "react";
import { HardDrive, Server, Activity } from "lucide-react";

export const MediaArtifact: React.FC<{ componentProperties: Record<string, unknown> }> = ({ componentProperties }) => {
  const props = (componentProperties["MediaArtifact"] || componentProperties) as Record<string, unknown>;
  const payload = (props.payload || props) as Record<string, unknown>;
  
  const title = typeof payload.title === "string" ? payload.title : "Media Asset";
  const desc = typeof payload.description === "string" ? payload.description : typeof payload.text === "string" ? payload.text : "Standard binary asset storage block";
  const size = typeof payload.file_size === "number" ? Math.round(payload.file_size / 1024) : null;
  const mime = typeof payload.mime_type === "string" ? payload.mime_type : typeof payload.mimeType === "string" ? payload.mimeType : "Unknown Type";

  return (
    <div className="flex flex-col gap-4 animate-in fade-in slide-in-from-bottom-2 duration-500">
      <div className="flex items-center gap-3 mb-2">
        <div className="flex h-10 w-10 items-center justify-center rounded-xl bg-slate-800/80 border border-white/5 shadow-inner">
          <HardDrive className="h-5 w-5 text-indigo-400" />
        </div>
        <div>
           <h2 className="text-sm font-semibold text-slate-200 tracking-wide">
             {title}
           </h2>
           <div className="text-xs text-slate-500 mt-0.5">{mime} {size ? `• ${size} KB` : ""}</div>
        </div>
      </div>
      
      <div className="p-4 rounded-xl border border-white/5 bg-white/2 backdrop-blur-md shadow-sm text-slate-400 text-sm flex flex-col gap-2">
         {desc && <p className="leading-relaxed">{desc}</p>}
         <div className="flex items-center gap-4 mt-3 pt-3 border-t border-white/5 text-xs text-slate-500">
           <div className="flex items-center gap-1.5"><Server className="w-3.5 h-3.5" /> Gateway Node</div>
           <div className="flex items-center gap-1.5"><Activity className="w-3.5 h-3.5" /> Indexed Status: Active</div>
         </div>
      </div>
    </div>
  );
};
