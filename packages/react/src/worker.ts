import { useEffect, useRef, useState, type ReactElement } from "react";

import type { UsePDFInstance } from "./dom/use-pdf.js";
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

/**
 * Renders `props` in `worker` (one that called `expose`) and again whenever `props` change. Gives
 * the same `{ loading, blob, url, error }` as `usePDF`. A result for older props is dropped, and
 * old object URLs are revoked. With no `worker` yet, it waits.
 */
export function usePDFWorker<P>(worker: Worker | null | undefined, props: P): UsePDFInstance {
  const [state, setState] = useState<UsePDFInstance>({
    url: null,
    blob: null,
    error: null,
    loading: !!worker,
  });
  const url = useRef<string | null>(null);
  // ponytail: props are compared by their JSON, so values JSON can't tell apart (a Date and its
  // ISO string, a Map) don't trigger a render. They must survive `postMessage` anyway.
  const key = JSON.stringify(props);

  useEffect(() => {
    if (!worker) return;
    let stale = false;
    setState((prev) => ({ ...prev, loading: true }));
    renderInWorker(worker, props).then(
      (blob) => {
        if (stale) return;
        if (url.current) URL.revokeObjectURL(url.current);
        url.current = URL.createObjectURL(blob);
        setState({ blob, url: url.current, error: null, loading: false });
      },
      (error) => {
        if (!stale) setState((prev) => ({ ...prev, error, loading: false }));
      },
    );
    return () => {
      stale = true;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps -- `key` stands for `props`.
  }, [worker, key]);

  useEffect(
    () => () => {
      if (url.current) URL.revokeObjectURL(url.current);
    },
    [],
  );
  return state;
}

/** Creates a Worker once, on mount, and terminates it on unmount. It is `null` until then. */
export function useWorker(create: () => Worker): Worker | null {
  const [worker, setWorker] = useState<Worker | null>(null);
  useEffect(() => {
    const created = create();
    setWorker(created);
    return () => created.terminate();
    // eslint-disable-next-line react-hooks/exhaustive-deps -- created once.
  }, []);
  return worker;
}
