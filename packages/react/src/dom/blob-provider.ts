import { useEffect, type ReactElement, type ReactNode } from "react";

import type { DocumentProps } from "../types/props.type.js";
import { usePDF, type UsePDFInstance } from "./use-pdf.js";

export interface BlobProviderProps {
  document: ReactElement<DocumentProps>;
  children: (params: UsePDFInstance) => ReactNode;
}

/** Gives its child function the rendered Blob and URL without showing the document. */
export function BlobProvider({ document, children }: BlobProviderProps) {
  const [instance, update] = usePDF();
  useEffect(() => {
    if (document) update(document);
  }, [document, update]);
  return children(instance);
}
