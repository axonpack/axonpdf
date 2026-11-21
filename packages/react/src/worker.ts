import type { ReactElement } from "react";

import { pdf } from "./pdf.js";
import type { DocumentProps } from "./types/props.type.js";

type Request<P> = { id: number; props: P };
type Response = { id: number; blob?: Blob; error?: unknown };

/**
 * Call inside a Worker: renders `document(props)` there, React and engine included, for every
 * `renderInWorker` call from the page. Props must survive `postMessage` (no functions).
 */
export function expose<P>(document: (props: P) => ReactElement<DocumentProps>) {
  addEventListener("message", async (event) => {
    const { id, props } = (event as MessageEvent<Request<P>>).data;
    try {
      const blob = await pdf(document(props)).toBlob();
      postMessage({ id, blob } satisfies Response);
    } catch (error) {
      postMessage({ id, error } satisfies Response);
    }
  });
}

let nextId = 0;

/** Asks a Worker that called `expose` to render with `props`, and resolves with the PDF. */
export function renderInWorker<P>(worker: Worker, props: P): Promise<Blob> {
  const id = nextId++;
  return new Promise((resolve, reject) => {
    const onMessage = ({ data }: MessageEvent<Response>) => {
      if (data.id !== id) return;
      worker.removeEventListener("message", onMessage);
      if (data.blob) resolve(data.blob);
      else reject(data.error);
    };
    worker.addEventListener("message", onMessage);
    worker.postMessage({ id, props } satisfies Request<P>);
  });
}
