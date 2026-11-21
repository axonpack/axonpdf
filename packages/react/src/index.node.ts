// The Node entry: every export by name, and all of them again as the default export.
import * as ReactPDF from "./api.node.js";

export * from "./api.node.js";

/**
 * Every export, values and types, under one name: `ReactPDF.pdf(...)` and
 * `ReactPDF.DocumentProps` both work.
 * @deprecated import what you use by name; this object keeps every export from tree-shaking.
 */
export default ReactPDF;
