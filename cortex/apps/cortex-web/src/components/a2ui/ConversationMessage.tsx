import React from "react";

export type ConversationMessageProps = {
  role: "user" | "assistant" | "system";
  content: string;
  name?: string;
  timestamp?: string;
};

export const ConversationMessage: React.FC<{ componentProperties: Record<string, unknown> }> = ({ componentProperties }) => {
  const props = (componentProperties["ConversationMessage"] || componentProperties) as Record<string, unknown>;
  
  const role = (typeof props.role === "string" ? props.role.toLowerCase() : "user") as "user" | "assistant" | "system";
  const content = typeof props.content === "string" ? props.content : "";
  const name = typeof props.name === "string" ? props.name : undefined;
  
  // Style configurations based on role for premium glassmorphic chat feel
  const isAgent = role === "assistant" || role === "system";
  const bgClass = isAgent 
    ? "bg-blue-900/10 border-blue-500/20" 
    : "bg-slate-800/20 border-white/5";
    
  const alignClass = isAgent ? "mr-12" : "ml-12";
  const avatarClass = isAgent 
    ? "bg-blue-500/20 text-blue-400 border border-blue-500/30" 
    : "bg-slate-700/50 text-slate-300 border border-white/10";
    
  const label = name || (role === "assistant" ? "Cortex" : role === "system" ? "System" : "You");

  // Basic markdown text splitting
  const paragraphs = content.split("\n\n").filter(p => p.trim().length > 0);

  return (
    <div className={`flex flex-col gap-1.5 ${alignClass} animate-in fade-in slide-in-from-bottom-2 duration-500 pb-2`}>
      <div className="flex items-center gap-2 mb-0.5">
        <div className={`w-6 h-6 rounded-full flex items-center justify-center text-[10px] font-bold tracking-widest uppercase shadow-inner ${avatarClass}`}>
          {label.charAt(0)}
        </div>
        <span className="text-xs font-semibold text-slate-400 tracking-wide">
          {label}
        </span>
      </div>
      
      <div className={`p-4 rounded-2xl border backdrop-blur-md shadow-xl ${bgClass} text-slate-200 text-sm leading-relaxed max-w-none`}>
        {paragraphs.length > 0 ? (
          paragraphs.map((p, i) => (
            <p key={i} className={i > 0 ? "mt-3" : ""}>
              {p.split("\\n").map((line, j) => (
                <React.Fragment key={j}>
                  {j > 0 && <br />}
                  {line}
                </React.Fragment>
              ))}
            </p>
          ))
        ) : (
          <span className="italic text-slate-500 text-xs">No content provided</span>
        )}
      </div>
    </div>
  );
};
