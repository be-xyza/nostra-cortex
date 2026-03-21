import React, { ReactNode } from "react";
import { Link } from "react-router-dom";

export function ExecutionContainmentHeader({ surfaceName, children }: { surfaceName: string, children: ReactNode }) {
    return (
        <div className="flex flex-col h-full w-full bg-cortex-bg">
            <header className="flex items-center justify-between p-3 border-b border-cortex-line bg-cortex-bg-elev">
                <div className="flex items-center gap-3 text-sm font-medium uppercase tracking-wider text-cortex-ink-muted">
                    <span className="text-cortex-warn">⚠️ Execution Surface</span>
                    <span>-</span>
                    <span className="text-cortex-ink">{surfaceName}</span>
                </div>
                <Link to="/system" className="text-xs text-cortex-accent hover:underline no-underline uppercase tracking-wider font-bold">
                    Exit to Platform
                </Link>
            </header>
            <div className="flex-1 overflow-auto">
                {children}
            </div>
        </div>
    );
}
