import { closeSync, createReadStream, openSync, writeSync } from "node:fs";
import { Readable } from "node:stream";
import type { ReactElement } from "react";

import { startStream, type Run } from "../live-stream.js";
import { mount, pdf } from "../pdf.js";
import type { DocumentProps } from "../types/props.type.js";
import { nodeSink } from "./node-sink.js";

type Element = ReactElement<DocumentProps>;

/** Renders, then unmounts so the root doesn't outlive the call. */
async function once<T>(element: Element, run: (instance: ReturnType<typeof pdf>) => Promise<T>) {
  const instance = pdf(element);
  try {
    return await run(instance);
  } finally {
    instance.updateContainer(null as unknown as ReactElement);
  }
}

/** A live stream: it resolves with the first chunk, and pages flow out as they are laid out. */
export function renderToStream(element: Element): Promise<NodeJS.ReadableStream> {
  const { container, updateContainer, render } = mount(element);
  const onRender = container.document?.props.onRender;
  const run: Run = (write, wait) =>
    render(write, wait)
      .then(() => onRender?.({}))
      .finally(() => updateContainer(null));
  return startStream(nodeSink(), run);
}

export const renderToBuffer = async (element: Element): Promise<Buffer> => {
  const stream = await renderToStream(element);
  const chunks: Buffer[] = [];
  for await (const chunk of stream) chunks.push(chunk as Buffer);
  return Buffer.concat(chunks);
};

/** @deprecated use `renderToBuffer`. */
export const renderToString = (element: Element): Promise<string> =>
  once(element, (instance) => instance.toString());

/** Writes each chunk to the file as the engine produces it, so memory stays flat. */
export async function renderToFile(
  element: Element,
  filePath: string,
  callback?: (output: NodeJS.ReadableStream, filePath: string) => any,
): Promise<NodeJS.ReadableStream> {
  const { container, updateContainer, render } = mount(element);
  const onRender = container.document?.props.onRender;
  const fd = openSync(filePath, "w");
  try {
    await render((chunk) => writeSync(fd, chunk));
  } finally {
    closeSync(fd);
    updateContainer(null);
  }
  onRender?.({});
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
