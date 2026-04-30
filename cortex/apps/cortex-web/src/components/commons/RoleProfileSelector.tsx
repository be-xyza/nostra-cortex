import React, { useState, useRef, useEffect } from "react";
import { Shield, Crown, PenTool, Eye, Check, ChevronDown, Terminal } from "lucide-react";
import type { AuthSession } from "../../contracts.ts";
import { describeAuthorityMode, formatAuthorityStatus } from "./shellBootstrapFallback.ts";

interface RoleProfileSelectorProps {
    session: AuthSession;
    onRoleChange: (role: string) => void | Promise<void>;
    className?: string;
    collapsed?: boolean;
    isCentered?: boolean;
}

const ROLE_CONFIG: Record<string, { 
    icon: React.ReactNode, 
    color: string, 
    ringColor: string, 
    label: string,
    authorityBadge: string
}> = {
    admin: {
        icon: <Shield className="w-3.5 h-3.5" />,
        color: "text-amber-400",
        ringColor: "ring-amber-500/50",
        label: "Administrator",
        authorityBadge: "Level 5"
    },
    steward: {
        icon: <Crown className="w-3.5 h-3.5" />,
        color: "text-purple-400",
        ringColor: "ring-purple-500/50",
        label: "Steward",
        authorityBadge: "Level 4"
    },
    operator: {
        icon: <Terminal className="w-3.5 h-3.5" />,
        color: "text-blue-400",
        ringColor: "ring-blue-500/50",
        label: "Operator",
        authorityBadge: "Level 3"
    },
    editor: {
        icon: <PenTool className="w-3.5 h-3.5" />,
        color: "text-emerald-400",
        ringColor: "ring-emerald-500/50",
        label: "Editor",
        authorityBadge: "Level 2"
    },
    viewer: {
        icon: <Eye className="w-3.5 h-3.5" />,
        color: "text-slate-400",
        ringColor: "ring-slate-500/50",
        label: "Viewer",
        authorityBadge: "Level 1"
    }
};

export const RoleProfileSelector: React.FC<RoleProfileSelectorProps> = ({ 
    session,
    onRoleChange,
    className = "",
    collapsed = false,
    isCentered = false
}) => {
    const [isOpen, setIsOpen] = useState(false);
    const dropdownRef = useRef<HTMLDivElement>(null);
    const activeRole = session.activeRole || "viewer";
    const config = ROLE_CONFIG[activeRole] || ROLE_CONFIG.viewer;
    const authModeLabel = describeAuthorityMode(session);
    const authorityStatus = formatAuthorityStatus(session);

    useEffect(() => {
        const handleClickOutside = (event: MouseEvent) => {
            if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
                setIsOpen(false);
            }
        };
        document.addEventListener("mousedown", handleClickOutside);
        return () => document.removeEventListener("mousedown", handleClickOutside);
    }, []);

    const handleRoleChange = (role: string) => {
        void onRoleChange(role);
        setIsOpen(false);
    };

    return (
        <div className={`relative ${className}`} ref={dropdownRef}>
            <button
                onClick={() => session.allowRoleSwitch && setIsOpen(!isOpen)}
                className={`flex items-center transition-all duration-300 ${session.allowRoleSwitch ? "hover:bg-white/5" : "cursor-default"} 
                    ${collapsed ? "justify-center p-1 rounded-full w-10 h-10" : "gap-2.5 p-1 rounded-xl pr-3"}
                    ${isCentered ? "mx-auto" : ""}`}
                aria-label={`Current role: ${activeRole}. ${authorityStatus}`}
                aria-haspopup="listbox"
                aria-expanded={isOpen}
                title={authorityStatus}
            >
                {/* Circular Profile Avatar */}
                <div className={`relative w-8 h-8 shrink-0 flex items-center justify-center rounded-full bg-cortex-900 border border-white/10 ring-2 ${config.ringColor} ring-offset-2 ring-offset-cortex-surface-base shadow-lg transition-transform duration-300 ${isOpen ? "scale-110" : ""}`}>
                    <div className={`${config.color}`}>
                        {config.icon}
                    </div>
                    {/* Tiny Status indicator */}
                    <div className="absolute -bottom-0.5 -right-0.5 w-2.5 h-2.5 rounded-full bg-emerald-500 border-2 border-cortex-900 shadow-sm" />
                </div>

                {!collapsed && (
                    <div className="flex flex-col items-start -translate-y-px">
                        <div className="flex items-center gap-1">
                            <span className="text-[10px] font-black uppercase tracking-wider text-cortex-100">
                                {activeRole}
                            </span>
                            {session.allowRoleSwitch && (
                                <ChevronDown className={`w-3 h-3 text-cortex-500 transition-transform ${isOpen ? "rotate-180" : ""}`} />
                            )}
                        </div>
                        <span className="text-[8px] font-bold text-cortex-400 tracking-tight leading-none uppercase">
                            {authModeLabel || config.authorityBadge}
                        </span>
                    </div>
                )}
            </button>

            {isOpen && (
                <div className="absolute top-full left-0 w-48 mt-2 bg-slate-950/98 backdrop-blur-2xl border border-white/10 rounded-xl shadow-[0_8px_32px_-8px_rgba(0,0,0,0.8)] p-1 z-50 animate-in fade-in zoom-in-95 duration-100">
                    <div className="text-[9px] font-black text-cortex-ink-faint px-3 py-2 uppercase tracking-widest border-b border-white/5 mb-1">
                        {session.authMode === "dev_override" ? "Switch Dev Role" : "Switch Authority Role"}
                    </div>
                    {session.grantedRoles.map((role) => {
                        const roleConfig = ROLE_CONFIG[role] || ROLE_CONFIG.viewer;
                        const isSelected = activeRole === role;
                        return (
                            <button
                                key={role}
                                onClick={() => handleRoleChange(role)}
                                className={`w-full flex items-center justify-between px-3 py-2.5 rounded-lg text-[11px] transition-all group ${isSelected ? "bg-white/5 text-blue-400" : "text-cortex-400 hover:bg-white/5 hover:text-white"}`}
                            >
                                <div className="flex items-center gap-3">
                                    <div className={`w-6 h-6 flex items-center justify-center rounded-full bg-cortex-900 border border-white/5 shadow-inner ${roleConfig.color}`}>
                                        {roleConfig.icon}
                                    </div>
                                    <div className="flex flex-col items-start leading-tight">
                                        <span className={`font-bold uppercase tracking-wide ${isSelected ? "text-blue-400" : "text-cortex-200"}`}>
                                            {role}
                                        </span>
                                        <span className="text-cortex-500 text-[8px] font-medium tracking-tight">
                                            {roleConfig.authorityBadge}
                                        </span>
                                    </div>
                                </div>
                                {isSelected && <Check className="w-3.5 h-3.5 text-blue-500" />}
                            </button>
                        );
                    })}
                </div>
            )}
        </div>
    );
};
