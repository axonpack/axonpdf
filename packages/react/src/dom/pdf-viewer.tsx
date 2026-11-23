import {
  useEffect,
  type CSSProperties,
  type IframeHTMLAttributes,
  type ReactElement,
  type Ref,
} from "react";

import type { DocumentProps } from "../types/props.type.js";
import { usePDFWorker } from "../worker.js";
import { usePDF } from "./use-pdf.js";

export interface PDFViewerProps extends Omit<
  IframeHTMLAttributes<HTMLIFrameElement>,
  "children" | "style" | "src"
> {
  width?: number | string;
  height?: number | string;
  style?: CSSProperties;
  className?: string;
  children?: ReactElement<DocumentProps>;
  innerRef?: Ref<HTMLIFrameElement>;
  showToolbar?: boolean;
  /** A Worker that called `expose`. When set, it renders the file from `props` and `children` is not used. */
  worker?: Worker | null;
  /** What the worker's document function gets. It must survive `postMessage`. */
  props?: unknown;
}

/** An iframe showing the document; it renders again when `children` changes. */
export function PDFViewer({
  title,
  style,
  className,
  children,
  innerRef,
  showToolbar = true,
  worker,
  props: workerProps,
  ...props
}: PDFViewerProps) {
  const [rendered, update] = usePDF();
  const fromWorker = usePDFWorker(worker, workerProps);
  useEffect(() => {
    if (children && !worker) update(children);
  }, [children, worker, update]);
  const instance = worker ? fromWorker : rendered;
  const src = instance.url ? `${instance.url}#toolbar=${showToolbar ? 1 : 0}` : undefined;
  return (
    <iframe src={src} title={title} ref={innerRef} style={style} className={className} {...props} />
  );
}
