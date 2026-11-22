import { useEffect, type ReactElement, type ReactNode } from "react";

import type { DocumentProps } from "../types/props.type.js";
import { usePDF } from "./use-pdf.js";

/** What `BlobProvider` gives its child function. */
export interface BlobProviderParams {
  blob: Blob | null;
  url: string | null;
  loading: boolean;
  error: Error | null;
}

export interface BlobProviderProps {
  document: ReactElement<DocumentProps>;
  children: (params: BlobProviderParams) => ReactNode;
}

/** Gives its child function the rendered Blob and URL without showing the document. */
export function BlobProvider({ document, children }: BlobProviderProps) {
  const [instance, update] = usePDF();
  useEffect(() => {
    if (document) update(document);
  }, [document, update]);
  return children(instance);
}
