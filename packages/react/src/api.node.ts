import { readFile } from "node:fs/promises";

import type * as dom from "./api.browser.js";
import { cpuSampler } from "./node/cpu-sampler.js";
import { nodeSink } from "./node/node-sink.js";
import { platform } from "./platform.js";
import { runProfile } from "./profile/run-profile.js";
import { environmentError } from "./shared.js";

platform.readFile = async (path) =>
  new Uint8Array(await readFile(path.startsWith("file:") ? new URL(path) : path));
platform.sink = nodeSink;

export * from "./shared.js";
export * from "./node/render-to.js";

/**
 * Renders the document to nothing and reports where the time went: React, the tree, assets and
 * the engine's phases, each page, peak memory, and the CPU by package and function.
 */
export const profile = (element: Parameters<typeof runProfile>[0]) =>
  runProfile(element, cpuSampler());
export type {
  BlobProviderParams,
  BlobProviderProps,
  PDFDownloadLinkProps,
  PDFViewerProps,
  UsePDFInstance,
} from "./api.browser.js";

// Typed like the browser versions, so both entries share one set of types.
export const usePDF = environmentError("usePDF", "web") as typeof dom.usePDF;
export const PDFViewer = environmentError("PDFViewer", "web") as typeof dom.PDFViewer;
export const PDFDownloadLink = environmentError(
  "PDFDownloadLink",
  "web",
) as unknown as typeof dom.PDFDownloadLink;
export const BlobProvider = environmentError("BlobProvider", "web") as typeof dom.BlobProvider;
