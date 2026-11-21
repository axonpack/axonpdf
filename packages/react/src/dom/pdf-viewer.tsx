import {
  useEffect,
  type CSSProperties,
  type IframeHTMLAttributes,
  type ReactElement,
  type Ref,
} from "react";

import type { DocumentProps } from "../types/props.type.js";
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
}

/** An iframe showing the document; it renders again when `children` changes. */
export function PDFViewer({
  title,
  style,
  className,
  children,
  innerRef,
  showToolbar = true,
  ...props
}: PDFViewerProps) {
  const [instance, update] = usePDF();
  useEffect(() => {
    if (children) update(children);
  }, [children, update]);
  const src = instance.url ? `${instance.url}#toolbar=${showToolbar ? 1 : 0}` : undefined;
  return (
    <iframe src={src} title={title} ref={innerRef} style={style} className={className} {...props} />
  );
}
