import { createElement, type ComponentType, type ReactElement } from "react";

import type {
  NodeProps,
  PageNumberProps,
  RowsProps,
  TableProps,
  TableRowProps,
} from "./types/props.type.js";

// axonpdf's own components, on top of the standard set.

/** Long content: rows from an array or an async iterable, drawn by the child function. */
export function Rows<T>({ rows, batchSize = 100, children }: RowsProps<T>): ReactElement {
  return createElement("ROWS", { rows, batchSize, renderRow: children });
}

/** The current page number, drawn like a `Text`. */
export const PageNumber = "PAGE_NUMBER" as unknown as ComponentType<PageNumberProps>;
/** The total page count, drawn like a `Text`, with no second layout pass. */
export const PageCount = "PAGE_COUNT" as unknown as ComponentType<PageNumberProps>;

/** A table whose `TableHeader` repeats at the top of every page it spans. */
export const Table = "TABLE" as unknown as ComponentType<TableProps>;
export const TableHeader = "TABLE_HEADER" as unknown as ComponentType<NodeProps>;
export const TableRow = "TABLE_ROW" as unknown as ComponentType<TableRowProps>;
export const TableCell = "TABLE_CELL" as unknown as ComponentType<NodeProps>;
