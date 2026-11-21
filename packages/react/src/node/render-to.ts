import { closeSync, createReadStream, openSync, writeSync } from "node:fs";
import { Readable } from "node:stream";
import type { ReactElement } from "react";

import { startStream, type Run } from "../live-stream.js";
import { latin1, mount } from "../pdf.js";
import type { DocumentProps } from "../types/props.type.js";
import { nodeSink } from "./node-sink.js";

type Element = ReactElement<DocumentProps>;

/** A live stream: it resolves with the first chunk, and pages flow out as they are laid out. */
export function renderToStream(element: Element): Promise<NodeJS.ReadableStream> {
  const { documentProps, updateContainer, render } = mount(element);
  const run: Run = (write, wait) =>
    render(write, wait, true)
      .then(() => documentProps()?.onRender?.({}))
      .finally(() => updateContainer(null));
  return startStream(nodeSink(), run);
}

export const renderToBuffer = async (element: Element): Promise<Buffer> => {
  const stream = await renderToStream(element);
  const chunks: Buffer[] = [];
  for await (const chunk of stream) chunks.push(chunk as Buffer);
  return Buffer.concat(chunks);
};

/** @deprecated use `renderToBuffer`. The PDF as text, its streams undeflated as in `toString`. */
export async function renderToString(element: Element): Promise<string> {
  if (process.env.NODE_ENV === "development") {
    console.warn(
      "`renderToString` is deprecated and will be removed in next major release, use `renderToBuffer` instead",
    );
  }
  const { documentProps, updateContainer, render } = mount(element);
  const chunks: Uint8Array[] = [];
  try {
    await render((chunk) => chunks.push(chunk), undefined, true, undefined, false);
  } finally {
    updateContainer(null);
  }
  documentProps()?.onRender?.({});
  return latin1(chunks);
}

/** Writes each chunk to the file as the engine produces it, so memory stays flat. */
export async function renderToFile(
  element: Element,
  filePath: string,
  callback?: (output: NodeJS.ReadableStream, filePath: string) => any,
): Promise<NodeJS.ReadableStream> {
  const { documentProps, updateContainer, render } = mount(element);
  const fd = openSync(filePath, "w");
  try {
    await render((chunk) => writeSync(fd, chunk), undefined, true);
  } finally {
    closeSync(fd);
    updateContainer(null);
  }
  documentProps()?.onRender?.({});
  // Opened only if someone reads it.
  const output = Readable.from(
    (async function* () {
      yield* createReadStream(filePath);
    })(),
  );
  callback?.(output, filePath);
  return output;
}

export const render = renderToFile;
