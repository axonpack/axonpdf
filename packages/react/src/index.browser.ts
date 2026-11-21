import * as components from "./components.js";
import { BlobProvider } from "./dom/blob-provider.js";
import { PDFDownloadLink } from "./dom/pdf-download-link.js";
import { PDFViewer } from "./dom/pdf-viewer.js";
import { usePDF } from "./dom/use-pdf.js";
import type * as node from "./node/render-to.js";
import { Font, environmentError, pdf, version } from "./shared.js";

export * from "./shared.js";
export { BlobProvider, PDFDownloadLink, PDFViewer, usePDF };
export type { BlobProviderProps } from "./dom/blob-provider.js";
export type { PDFDownloadLinkProps } from "./dom/pdf-download-link.js";
export type { PDFViewerProps } from "./dom/pdf-viewer.js";
export type { UsePDFInstance } from "./dom/use-pdf.js";

export const renderToStream = environmentError(
  "renderToStream",
  "Node",
) as typeof node.renderToStream;
export const renderToBuffer = environmentError(
  "renderToBuffer",
  "Node",
) as typeof node.renderToBuffer;
export const renderToString = environmentError(
  "renderToString",
  "Node",
) as typeof node.renderToString;
export const renderToFile = environmentError("renderToFile", "Node") as typeof node.renderToFile;
export const render = environmentError("render", "Node") as typeof node.render;

/** @deprecated import what you use by name; this object keeps every export from tree-shaking. */
export default {
  pdf,
  usePDF,
  Font,
  version,
  PDFViewer,
  BlobProvider,
  PDFDownloadLink,
  renderToStream,
  renderToString,
  renderToFile,
  render,
  ...components,
};
