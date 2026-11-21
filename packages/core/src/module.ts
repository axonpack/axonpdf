import createModule, { type AxonpdfModule } from "#loader";
import type { Callbacks } from "./engine.type.js";

export interface Active {
  onChunk?: (chunk: Uint8Array) => void;
  callbacks: Callbacks;
  /** A JS error must not unwind through Rust, so it waits here until the call returns. */
  error?: unknown;
}

interface Loaded {
  module: AxonpdfModule;
  active: Map<number, Active>;
  pointers: Record<"output" | "render" | "paint" | "hyphenate" | "rows", number>;
}

export const encoder = new TextEncoder();
const decoder = new TextDecoder();
let loading: Promise<Loaded> | undefined;

export function load(): Promise<Loaded> {
  // The engine writes only warnings to stderr; they go out as warnings, as the components give them.
  loading ??= createModule({ printErr: (text: string) => console.warn(text) }).then((module) => {
    const active = new Map<number, Active>();
    const cString = (text: string) => {
      const bytes = encoder.encode(text);
      const ptr = module._axonpdf_alloc(bytes.length + 1);
      module.HEAPU8.set(bytes, ptr);
      module.HEAPU8[ptr + bytes.length] = 0;
      return ptr;
    };
    // Runs one callback for the engine `handle`; any JS error is held until the engine returns.
    // Not JSON: after a JS error, the engine stops at its next read of an answer.
    const stop = "!";
    const call = (handle: number, run: (callbacks: Callbacks) => string | undefined) => {
      const current = active.get(handle)!;
      if (current.error !== undefined) return cString(stop);
      try {
        const json = run(current.callbacks);
        return json === undefined ? 0 : cString(json);
      } catch (error) {
        current.error = error;
        return cString(stop);
      }
    };
    const marker = module.UTF8ToString(module._axonpdf_total_pages_marker());
    const optional = (n: number) => (n < 0 || Number.isNaN(n) ? undefined : n);
    const pointers = {
      output: module.addFunction((handle, ptr, len) => {
        const current = active.get(handle)!;
        if (current.error === undefined) current.onChunk?.(module.HEAPU8.slice(ptr, ptr + len));
      }, "viii"),
      render: module.addFunction(
        (handle, id, page, total, subPage, subTotal, isLast, totalPending) =>
          call(handle, (c) =>
            c.render?.(id, {
              pageNumber: page,
              totalPages: totalPending ? marker : optional(total),
              subPageNumber: optional(subPage),
              subPageTotalPages: optional(subTotal),
              isLast: isLast < 0 ? undefined : isLast === 1,
            }),
          ),
        "iiiiiiiii",
      ),
      paint: module.addFunction(
        (handle, id, width, height) =>
          call(handle, (c) => c.paint?.(id, optional(width), optional(height))),
        "iiiff",
      ),
      hyphenate: module.addFunction(
        (handle, id, word, len) =>
          call(handle, (c) =>
            c.hyphenate?.(id, decoder.decode(module.HEAPU8.subarray(word, word + len))),
          ),
        "iiiii",
      ),
      rows: module.addFunction((handle, id) => call(handle, (c) => c.rows?.(id)), "iii"),
    };
    return { module, active, pointers };
  });
  return loading;
}
