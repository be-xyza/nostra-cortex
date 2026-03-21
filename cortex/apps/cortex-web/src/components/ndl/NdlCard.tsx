import React, { ReactNode } from "react";

interface NdlCardProps {
    children: ReactNode;
    headerText?: string;
    className?: string;
}

export function NdlCard({ children, headerText, className = "" }: NdlCardProps) {
    return (
        <div className={`p-4 border border-cortex-line rounded-cortex bg-cortex-bg-elev shadow-cortex flex flex-col gap-3 ${className}`} role="region">
            {headerText && <div className="text-cortex-ink font-semibold border-b border-cortex-line pb-2">{headerText}</div>}
            {children}
        </div>
    );
}
