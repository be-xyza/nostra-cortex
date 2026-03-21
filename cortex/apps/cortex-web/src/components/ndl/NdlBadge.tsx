import React from "react";

type BadgeVariant = "ok" | "warn" | "bad" | "neutral";

interface NdlBadgeProps {
    label: string;
    variant?: BadgeVariant;
    compact?: boolean;
}

const variantStyles: Record<BadgeVariant, string> = {
    ok: "bg-green-500/10 text-green-400",
    warn: "bg-yellow-500/10 text-yellow-400",
    bad: "bg-red-500/10 text-red-400",
    neutral: "bg-slate-500/30 text-slate-300",
};

export function NdlBadge({ label, variant = "neutral", compact = false }: NdlBadgeProps) {
    return (
        <span
            className={`${compact ? "px-1.5 text-[8px]" : "px-2.5 text-[10px]"} py-0.5 rounded-full uppercase font-bold tracking-widest ${variantStyles[variant]}`}
            role="status"
        >
            {label}
        </span>
    );
}
