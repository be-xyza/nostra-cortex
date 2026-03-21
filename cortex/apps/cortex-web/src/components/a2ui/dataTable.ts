export type DataTableRow = Record<string, unknown>;

export interface DataTableProjection {
  columns: string[];
  rows: DataTableRow[];
  rowHrefField: string;
  rowKeyField: string;
  hiddenColumns: Set<string>;
}

export const DEFAULT_DATA_TABLE_ROW_HREF_FIELD = "_href";
export const DEFAULT_DATA_TABLE_ROW_KEY_FIELD = "_row_id";

export function normalizeDataTableRows(
  rawRows: unknown,
  explicitColumns: string[],
): DataTableRow[] {
  if (!Array.isArray(rawRows)) return [];
  return rawRows
    .map((row) => {
      if (row && typeof row === "object" && !Array.isArray(row)) {
        return row as DataTableRow;
      }
      if (Array.isArray(row)) {
        const mapped: DataTableRow = {};
        row.forEach((cell, index) => {
          const columnName = explicitColumns[index] ?? `Column ${index + 1}`;
          mapped[columnName] = cell;
        });
        return mapped;
      }
      return null;
    })
    .filter((row): row is DataTableRow => row !== null);
}

export function projectDataTable(
  props: Record<string, unknown>,
): DataTableProjection {
  const explicitColumns = Array.isArray(props.columns)
    ? props.columns.map((column) => String(column))
    : [];
  const rows = normalizeDataTableRows(props.rows ?? props.data, explicitColumns);
  const rowHrefField =
    typeof props.rowHrefField === "string" && props.rowHrefField.trim()
      ? props.rowHrefField.trim()
      : DEFAULT_DATA_TABLE_ROW_HREF_FIELD;
  const rowKeyField =
    typeof props.rowKeyField === "string" && props.rowKeyField.trim()
      ? props.rowKeyField.trim()
      : DEFAULT_DATA_TABLE_ROW_KEY_FIELD;
  const hiddenColumns = new Set<string>(
    Array.isArray(props.hiddenColumns)
      ? props.hiddenColumns.map((column) => String(column))
      : [],
  );
  const inferredColumns = rows[0] ? Object.keys(rows[0]) : [];
  const columns = (explicitColumns.length > 0 ? explicitColumns : inferredColumns).filter(
    (column) => !hiddenColumns.has(column) && !column.startsWith("_"),
  );

  return {
    columns,
    rows,
    rowHrefField,
    rowKeyField,
    hiddenColumns,
  };
}
