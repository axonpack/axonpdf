import { useCallback, useEffect, useRef, useState, type ReactElement } from "react";

import { pdf, type PDFInstance } from "../pdf.js";
import type { DocumentProps } from "../types/props.type.js";

export interface UsePDFInstance {
  loading: boolean;
  blob: Blob | null;
  url: string | null;
  error: any;
}

/**
 * Renders a document to a Blob and an object URL, again on every `update`. One render runs at a
 * time; an update during a render waits, and a newer one replaces it. Old URLs are revoked.
 */
export function usePDF({ document }: { document?: ReactElement<DocumentProps> } = {}): [
  UsePDFInstance,
  (newDocument: ReactElement<DocumentProps>) => void,
] {
  const [state, setState] = useState<UsePDFInstance>({
    url: null,
    blob: null,
    error: null,
    loading: !!document,
  });
  const instance = useRef<PDFInstance | null>(null);

  useEffect(() => {
    const current = pdf();
    instance.current = current;
    let running = false;
    let queued = false;
    let unmounted = false;
    let url: string | null = null;

    const render = async () => {
      if (running) {
        queued = true;
        return;
      }
      running = true;
      setState((prev) => ({ ...prev, loading: true }));
      try {
        const blob = await current.toBlob();
        if (!unmounted) {
          if (url) URL.revokeObjectURL(url);
          url = URL.createObjectURL(blob);
          setState({ blob, url, error: null, loading: false });
        }
      } catch (error) {
        console.error(error);
        if (!unmounted) setState((prev) => ({ ...prev, error, loading: false }));
      }
      running = false;
      if (queued && !unmounted) {
        queued = false;
        render();
      }
    };

    current.on("change", render);
    if (document) current.updateContainer(document);
    return () => {
      unmounted = true;
      current.removeListener("change", render);
      current.updateContainer(null as unknown as ReactElement);
      if (url) URL.revokeObjectURL(url);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps -- `document` is only the first value.
  }, []);

  const update = useCallback(
    (newDocument: ReactElement<DocumentProps>) => instance.current?.updateContainer(newDocument),
    [],
  );
  return [state, update];
}
