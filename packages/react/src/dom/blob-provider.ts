import { useEffect, type ReactElement, type ReactNode } from "react";

import type { DocumentProps } from "../types/props.type.js";
import { usePDFWorker } from "../worker.js";
import { usePDF } from "./use-pdf.js";

/** What `BlobProvider` gives its child function. */
export interface BlobProviderParams {
  blob: Blob | null;
  url: string | null;
  loading: boolean;
  error: Error | null;
}

export interface BlobProviderProps {
  document?: ReactElement<DocumentProps>;
  /** A Worker that called `expose`. When set, it renders the file from `props` and `document` is not used. */
  worker?: Worker | null;
  /** What the worker's document function gets. It must survive `postMessage`. */
  props?: unknown;
  children: (params: BlobProviderParams) => ReactNode;
}

/** Gives its child function the rendered Blob and URL without showing the document. */
export function BlobProvider({ document, worker, props, children }: BlobProviderProps) {
  const [instance, update] = usePDF();
  const fromWorker = usePDFWorker(worker, props);
  useEffect(() => {
    if (document && !worker) update(document);
  }, [document, worker, update]);
  return children(worker ? fromWorker : instance);
}
