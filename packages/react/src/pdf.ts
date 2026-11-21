import type { ReactElement } from "react";

import { startStream, type Run } from "./live-stream.js";
import { platform } from "./platform.js";
import { createRenderer, type Renderer } from "./reconciler/renderer.js";
import { renderDocument } from "./render/render-document.js";
import type { Container } from "./types/instance.type.js";
import type { Recorder } from "./types/profile.type.js";
import type { DocumentProps } from "./types/props.type.js";

// React wants one renderer per process; each `pdf()` gets its own root in it.
let renderer: Renderer | undefined;
const listeners = new WeakMap<Container, (() => void)[]>();

const latin1 = (chunks: Uint8Array[]) =>
  chunks.map((chunk) => Array.from(chunk, (byte) => String.fromCharCode(byte)).join("")).join("");

/** A root in the shared renderer, with a render that streams chunks. */
export function mount(initialValue?: ReactElement) {
  const container: Container = { document: null };
  const changeListeners: (() => void)[] = [];
  listeners.set(container, changeListeners);
  let error: unknown;
  renderer ??= createRenderer({
    onChange: (root) =>
      listeners
        .get(root)
        ?.slice()
        .forEach((listener) => listener()),
  });
  const root = renderer.createContainer(container, (e) => {
    error = e;
  });
  const updateContainer = (document: ReactElement | null, callback?: () => void) =>
    renderer!.updateContainer(root, document, callback);
  if (initialValue) updateContainer(initialValue);

  /** `release`: unmount once the tree is read, for a render that is the root's last. */
  const render = async (
    onChunk: (chunk: Uint8Array) => void,
    wait?: () => Promise<void>,
    release?: boolean,
    recorder?: Recorder,
  ) => {
    if (error !== undefined) {
      const e = error;
      error = undefined;
      throw e;
    }
    await renderDocument(
      container,
      onChunk,
      wait,
      release ? () => updateContainer(null) : undefined,
      recorder,
    );
  };
  return { container, changeListeners, updateContainer, render };
}

export function pdf(initialValue?: ReactElement<DocumentProps>) {
  const { container, changeListeners, updateContainer, render: renderTo } = mount(initialValue);
  const render = async () => {
    const chunks: Uint8Array[] = [];
    await renderTo((chunk) => chunks.push(chunk));
    return chunks;
  };
  const callOnRender = (params: { blob?: Blob } = {}) =>
    container.document?.props.onRender?.(params);

  return {
    container,
    updateContainer: updateContainer as (document: ReactElement, callback?: () => void) => void,
    async toBlob(): Promise<Blob> {
      const blob = new Blob((await render()) as BlobPart[], { type: "application/pdf" });
      callOnRender({ blob });
      return blob;
    },
    /** A live stream of the PDF (a Node `Readable` in Node), as the name has always meant. */
    toBuffer(): Promise<NodeJS.ReadableStream> {
      const run: Run = (write, wait) => renderTo(write, wait).then(() => callOnRender());
      return startStream(platform.sink(), run) as Promise<NodeJS.ReadableStream>;
    },
    /** @deprecated use `toBuffer` or `toBlob`. */
    async toString(): Promise<string> {
      if (globalThis.process?.env?.NODE_ENV === "development") {
        console.warn("`toString` is deprecated and will be removed in next major release");
      }
      const chunks = await render();
      callOnRender();
      return latin1(chunks);
    },
    on(event: "change", listener: () => void) {
      if (event === "change") changeListeners.push(listener);
    },
    removeListener(event: "change", listener: () => void) {
      const index = changeListeners.indexOf(listener);
      if (event === "change" && index !== -1) changeListeners.splice(index, 1);
    },
  };
}

export type PDFInstance = ReturnType<typeof pdf>;
