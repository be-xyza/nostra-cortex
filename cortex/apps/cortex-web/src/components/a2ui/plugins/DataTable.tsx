import React from "react";
import { useNavigate } from "react-router-dom";
import { readProps } from "../WidgetRegistry";
import { useWorkflowArtifactInspector } from "../../workflows/WorkflowArtifactInspectorContext";
import { classifyWorkbenchHref } from "../../workflows/artifactRouting";
import { openGatewayApiArtifact } from "../../../api";
import { projectDataTable } from "../dataTable";
import type { A2UIComponentProps } from "../WidgetRegistry";

export default function DataTable({ componentProperties }: A2UIComponentProps) {
  const props = readProps(componentProperties, "DataTable");
  const navigate = useNavigate();
  const workflowInspector = useWorkflowArtifactInspector();
  const { columns, rows, rowHrefField, rowKeyField } = projectDataTable(props);

  if (columns.length === 0 || rows.length === 0) {
    return React.createElement("div", { className: "text-sm text-slate-500 italic p-6 text-center border border-white/5 rounded-2xl bg-slate-950/20" }, "No data to display");
  }

  const handleRowNavigation = async (href: string) => {
    const hrefKind = classifyWorkbenchHref(href);
    if (hrefKind === "gateway_api") {
      if (workflowInspector) {
        await workflowInspector.openArtifact(href);
        return;
      }
      await openGatewayApiArtifact(href, "new_tab");
      return;
    }
    if (hrefKind === "internal_workbench") {
      navigate(href);
      return;
    }
    window.location.assign(href);
  };

  return React.createElement("div", { className: "w-full overflow-x-auto min-h-0 border border-white/10 rounded-2xl bg-slate-950/30 shadow-2xl backdrop-blur-xl custom-scrollbar" },
    React.createElement("table", { className: "w-full text-sm text-left text-slate-300" }, [
      React.createElement("thead", { key: "thead", className: "text-[10px] font-black uppercase tracking-[0.15em] text-slate-400 bg-slate-900/60 border-b border-white/10 backdrop-blur-md sticky top-0 z-10" },
        React.createElement("tr", {},
          columns.map((col): React.ReactNode => React.createElement("th", { key: col, className: "px-6 py-4 whitespace-nowrap" }, col))
        )
      ),
      React.createElement("tbody", { key: "tbody", className: "divide-y divide-white/5 bg-transparent" },
        rows.map((row, idx): React.ReactNode => {
          const href = typeof row[rowHrefField] === "string" ? String(row[rowHrefField]) : "";
          const key = typeof row[rowKeyField] === "string" && String(row[rowKeyField]).trim()
            ? String(row[rowKeyField])
            : `row-${idx}`;
          const clickable = href.trim().length > 0;
          return React.createElement("tr", {
            key,
            className: `group transition-all duration-200 ${clickable ? "cursor-pointer hover:bg-slate-800/50 hover:shadow-inner" : "hover:bg-slate-900/30"}`,
            onClick: clickable ? () => void handleRowNavigation(href) : undefined,
            role: clickable ? "link" : undefined,
            tabIndex: clickable ? 0 : undefined,
            onKeyDown: clickable
              ? (event: React.KeyboardEvent<HTMLTableRowElement>) => {
                  if (event.key === "Enter" || event.key === " ") {
                    event.preventDefault();
                    void handleRowNavigation(href);
                  }
                }
              : undefined,
          },
            columns.map((col): React.ReactNode => {
              const cellValue = String(row[col] || "");
              if (col === "Title" && (row["Artifact ID"] || row["_row_id"])) {
                const idValue = String(row["Artifact ID"] || row["_row_id"]);
                return React.createElement("td", { key: `${key}-${col}`, className: "px-6 py-3.5 transition-colors group-hover:text-slate-200" }, [
                  React.createElement("div", { key: "title", className: "font-medium" }, cellValue),
                  React.createElement("div", { key: "id", className: "text-[10px] text-slate-500 font-mono mt-0.5" }, idValue)
                ]);
              }
              return React.createElement("td", { key: `${key}-${col}`, className: "px-6 py-3.5 transition-colors group-hover:text-slate-200" }, cellValue);
            })
          );
        })
      )
    ])
  );
}
