import type { ReactElement } from "react";

import { startStream, type Run } from "./live-stream.js";
import { platform } from "./platform.js";
import { createRenderer, type Renderer } from "./reconciler/renderer.js";
import { renderDocument, serializeRoot } from "./render/render-document.js";
import { Fallback, rootScope } from "./tree/dispatcher.js";
import type { Session } from "./tree/session.js";
import { walk } from "./tree/walk.js";
import type { Container } from "./types/instance.type.js";
import type { Recorder } from "./types/profile.type.js";
import type { DocumentProps } from "./types/props.type.js";
import type { TreeDocument } from "./types/tree.type.js";

// React wants one renderer per process; each `pdf()` gets its own root in it.
let renderer: Renderer | undefined;
const listeners = new WeakMap<Container, (() => void)[]>();

/** The bytes as a string, one character per byte. */
export const latin1 = (chunks: Uint8Array[]) =>
  chunks.map((chunk) => Array.from(chunk, (byte) => String.fromCharCode(byte)).join("")).join("");

/** A reconciler root holding `element`, for the cases the one-shot walk leaves to it. */
function reconcile(element: ReactElement | null, changeListeners: (() => void)[]) {
  const container: Container = { document: null };
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
  const update = (document: ReactElement | null, callback?: () => void) =>
    renderer!.updateContainer(document, root, null, callback);
  if (element) update(element);
  const takeError = () => {
    const e = error;
    error = undefined;
    if (e !== undefined) throw e;
  };
  return { container, update, takeError };
}

/**
 * A document to render, with the reconciler's API. Rendering walks the elements once (see
 * `tree/walk.ts`); a reconciler root is made only when the walk can't render something, or when
 * the caller reads `container`.
 */
export function mount(initialValue?: ReactElement) {
  let element = initialValue ?? null;
  const changeListeners: (() => void)[] = [];
  let reconciled: ReturnType<typeof reconcile> | undefined;
  /** The `Document`'s props from the last render, for `onRender`. */
  let documentProps: Record<string, any> | undefined;
  const reconciler = () => (reconciled ??= reconcile(element, changeListeners));

  const updateContainer = (document: ReactElement | null, callback?: () => void) => {
    element = document;
    if (reconciled) return reconciled.update(document, callback);
    // What the reconciler's commit does: tell the listeners, then call back.
    changeListeners.slice().forEach((listener) => listener());
    callback?.();
  };

  /**
   * `release`: unmount once the tree is read, for a render that is the root's last. `compress`
   * false writes the streams undeflated.
   */
  const render = async (
    onChunk: (chunk: Uint8Array) => void,
    wait?: () => Promise<void>,
    release?: boolean,
    recorder?: Recorder,
    compress = true,
  ) => {
    const settings = (tree: TreeDocument) => {
      if (!compress) tree.props.compress = false;
      return tree;
    };
    if (!reconciled) {
      try {
        const build = (session: Session) => {
          const [root] = walk(element, rootScope());
          documentProps = root?.type === "DOCUMENT" ? root.props : undefined;
          return settings(serializeRoot(root, session));
        };
        return await renderDocument(build, onChunk, wait, undefined, recorder);
      } catch (error) {
        if (!(error instanceof Fallback)) throw error;
      }
    }
    recorder?.mark("react");
    const { container, update, takeError } = reconciler();
    takeError();
    const build = (session: Session) => {
      documentProps = container.document?.props;
      return settings(serializeRoot(container.document, session));
    };
    const unmount = release ? () => update(null) : undefined;
    await renderDocument(build, onChunk, wait, unmount, recorder);
  };
  return {
    get container() {
      return reconciler().container;
    },
    documentProps: () => documentProps,
    changeListeners,
    updateContainer,
    render,
  };
}

export function pdf(initialValue?: ReactElement<DocumentProps>) {
  const root = mount(initialValue);
  const { changeListeners, updateContainer, render: renderTo } = root;
  const render = async (compress?: boolean) => {
    const chunks: Uint8Array[] = [];
    await renderTo((chunk) => chunks.push(chunk), undefined, undefined, undefined, compress);
    return chunks;
  };
  const callOnRender = (params: { blob?: Blob } = {}) => root.documentProps()?.onRender?.(params);

  return {
    get container() {
      return root.container;
    },
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
    /** @deprecated use `toBuffer` or `toBlob`. Its streams are left undeflated. */
    async toString(): Promise<string> {
      if (globalThis.process?.env?.NODE_ENV === "development") {
        console.warn("`toString` is deprecated and will be removed in next major release");
      }
      const chunks = await render(false);
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
