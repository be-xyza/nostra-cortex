import React from "react";
import { FileText } from "lucide-react";

export const NoteArtifact: React.FC<{ componentProperties: Record<string, unknown> }> = ({ componentProperties }) => {
  const props = (componentProperties["NoteArtifact"] || componentProperties) as Record<string, unknown>;
  const payload = (props.payload || props) as Record<string, unknown>;
  
  const title = typeof payload.title === "string" ? payload.title : "Untitled Note";
  const markdownSource = typeof payload.markdownSource === "string" 
    ? payload.markdownSource 
    : typeof payload.text === "string" 
      ? payload.text 
      : "";

  // Basic markdown text splitting
  const paragraphs = markdownSource.split("\n\n").filter(p => p.trim().length > 0);

  return (
    <div className="flex flex-col gap-4 animate-in fade-in slide-in-from-bottom-2 duration-500">
      <div className="flex items-center gap-3 mb-2">
        <div className="flex h-10 w-10 items-center justify-center rounded-xl bg-slate-800/80 border border-white/5 shadow-inner">
          <FileText className="h-5 w-5 text-emerald-400" />
        </div>
        <h2 className="text-sm font-semibold text-slate-200 tracking-wide">
          {title}
        </h2>
      </div>
      
      <div className="p-5 rounded-2xl border border-white/5 bg-white/2 backdrop-blur-md shadow-xl text-slate-300 text-sm leading-relaxed">
        {paragraphs.length > 0 ? (
          paragraphs.map((p, i) => (
            <p key={i} className={i > 0 ? "mt-4" : ""}>
              {p.split("\n").map((line, j) => (
                <React.Fragment key={j}>
                  {j > 0 && <br />}
                  {line}
                </React.Fragment>
              ))}
            </p>
          ))
        ) : (
          <span className="italic text-slate-500 text-xs">No markdown content provided</span>
        )}
      </div>
    </div>
  );
};
