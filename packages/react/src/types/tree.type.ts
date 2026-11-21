/**
 * The document tree this package sends to the engine. The engine's own description is in
 * `crates/engine/src/tree/mod.rs`; this file adds what the React side puts in it.
 *
 * - `props` holds every prop except `style`, `children` and functions. `undefined` is left out.
 *   Dates (`creationDate`, `modificationDate`) travel as ISO strings; `creationDate` defaults to
 *   the time of the render.
 * - `IMAGE` and `IMAGE_BACKGROUND`: `src` (or `source`, renamed to `src`) is `{ "asset": id }`,
 *   the id given to the engine's `add_image` for the bytes. `srcSet` stays a string.
 * - `render` (Text and View): the callback id. The engine calls back with the page numbers.
 * - `PAGE`: `layout` is the id of its layout template. The engine calls it back like `render`;
 *   the template's `children` serialize as one `{"type":"SLOT"}` node where the content goes.
 * - `CANVAS`: `paint` is the id of its `paint` function. The engine calls back with the box's
 *   width and height after layout and gets the painter calls in order, as a JSON array of
 *   `[method, ...args]` (see `tree/canvas-painter.ts`).
 * - `PAGE` `header` and `footer` (axonpdf's bands): a node, or `{ "render": id }` when the band
 *   is a function. The function gets `{ pageNumber, totalPages, isFirst, isLast, isOdd }`;
 *   reading `totalPages` or `isLast` sets `readsTotal`.
 * - `breakBefore` / `breakAfter` (`"page" | "auto"`) and `breakInside` (`"avoid" | "auto"`) are
 *   props on any node: `breakBefore: "page"` means `break`, `breakInside: "avoid"` means
 *   `wrap: false`, and `breakAfter: "page"` starts a new page after the node.
 * - `ROWS`: `props.rows` is the id the engine pulls batches by (`{"children":[...],"done":bool}`),
 *   `props.batchSize` (default 100) how many rows a batch holds; `children` is empty.
 * - `DOCUMENT` `encryptionSeed`: 32 random bytes (numbers) whenever `userPassword` or
 *   `ownerPassword` is set, for the encryption keys, salts and IVs.
 * - `PAGE_NUMBER` and `PAGE_COUNT`: no children; the engine draws the number as text in `style`.
 * - `TABLE`, `TABLE_HEADER`, `TABLE_ROW`, `TABLE_CELL`: a table is a column of rows, a row lays
 *   out its cells like a View row (`width` or `flex` in the cell's style), borders between cells
 *   collapse, and `TABLE_HEADER` repeats at the top of every page the table spans.
 */
export interface TreeNode {
  type: string;
  props: Record<string, unknown>;
  style?: unknown;
  render?: number;
  /** `PAGE` only: the id of its `layout` template, answered like `render` with one `SLOT`. */
  layout?: number;
  /** `CANVAS` only: the id of its `paint` function. */
  paint?: number;
  children: (TreeNode | TreeText)[];
}

export interface TreeText {
  type: "TEXT_INSTANCE";
  value: string;
}

export interface TreeDocument extends TreeNode {
  v: 1;
  type: "DOCUMENT";
}

/** What a render callback returns to the engine. */
export interface RenderResult {
  children: (TreeNode | TreeText)[];
  readsTotal: boolean;
}
