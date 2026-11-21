import {
  forwardRef,
  useEffect,
  type AnchorHTMLAttributes,
  type FC,
  type MouseEvent,
  type ReactElement,
  type ReactNode,
} from "react";

import type { DocumentProps } from "../types/props.type.js";
import { usePDF, type UsePDFInstance } from "./use-pdf.js";

export interface PDFDownloadLinkProps extends Omit<
  AnchorHTMLAttributes<HTMLAnchorElement>,
  "href" | "children" | "onClick"
> {
  /** The file name for the download. Default `document.pdf`. */
  fileName?: string;
  document: ReactElement<DocumentProps>;
  children?: ReactNode | FC<UsePDFInstance>;
  onClick?: (event: MouseEvent<HTMLAnchorElement>, instance: UsePDFInstance) => void;
}

/** A link that downloads the document, rendered ahead of the click. */
export const PDFDownloadLink = forwardRef<HTMLAnchorElement, PDFDownloadLinkProps>(
  function PDFDownloadLink(
    { fileName = "document.pdf", document, children, onClick, ...props },
    ref,
  ) {
    const [instance, update] = usePDF();
    useEffect(() => update(document), [document, update]);
    if (!document) {
      console.warn("You should pass a valid document to PDFDownloadLink");
      return null;
    }
    return (
      <a
        href={instance.url ?? undefined}
        download={fileName}
        ref={ref}
        onClick={(event) => onClick?.(event, instance)}
        {...props}
      >
        {typeof children === "function" ? (children(instance) as ReactNode) : children}
      </a>
    );
  },
);
