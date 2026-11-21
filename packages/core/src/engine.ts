import createModule, { type AxonpdfModule } from "../wasm/axonpdf.mjs";

/** What the engine knows about the page when it calls a `render` function. While pages are
 * being cut only `pageNumber` is known; the rest are `undefined` until the final call. */
export interface RenderArgs {
  pageNumber: number;
  /**
   * Undefined until the page count is known. In documents with `Rows` the total is written in
   * when the file is finished, so the final call gets a marker string that text can show (the
   * engine swaps it for the number) but that can't be computed with.
   */
  totalPages: number | string | undefined;
  subPageNumber: number | undefined;
  subPageTotalPages: number | undefined;
  /** Whether this is the document's last page; known only in the final call for a page. */
  isLast: boolean | undefined;
}

/** Runs the JS function stored under `id` and returns `{"children":[...],"readsTotal":bool}`. */
export type RenderCallback = (id: number, args: RenderArgs) => string;

/** The functions the engine calls by id while it renders. Each answers JSON. */
export interface Callbacks {
  render?: RenderCallback;
  /** A Canvas `paint` at its content-box size (`undefined` when only measuring): the calls. */
  paint?: (id: number, width: number | undefined, height: number | undefined) => string;
  /** A hyphenation function on one word: the parts, as a JSON list of strings. */
  hyphenate?: (id: number, word: string) => string;
  /** The next batch of a `Rows`: `{"children":[...],"done":bool}`. */
  rows?: (id: number) => string;
}

export interface Engine {
  addFont(family: string, weight: number, italic: boolean, bytes: Uint8Array): void;
  addImage(id: number, bytes: Uint8Array): void;
  /** Renders the tree; `onChunk` gets each piece of the PDF, in order, as it is written. */
  render(treeJson: string, onChunk: (chunk: Uint8Array) => void, callbacks?: Callbacks): void;
  /** The built-in split of a word: at its soft hyphens, else by the English patterns. */
  hyphenateBuiltin(word: string): string[];
  /** Frees the engine's memory. The engine can't be used afterwards. */
  free(): void;
}

interface Active {
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

const encoder = new TextEncoder();
const decoder = new TextDecoder();
let loading: Promise<Loaded> | undefined;

function load(): Promise<Loaded> {
  loading ??= createModule().then((module) => {
    const active = new Map<number, Active>();
    const cString = (text: string) => {
      const bytes = encoder.encode(text);
      const ptr = module._axonpdf_alloc(bytes.length + 1);
      module.HEAPU8.set(bytes, ptr);
      module.HEAPU8[ptr + bytes.length] = 0;
      return ptr;
    };
    // Runs one callback for the engine `handle`; any JS error is held until the engine returns.
    const call = (handle: number, run: (callbacks: Callbacks) => string | undefined) => {
      const current = active.get(handle)!;
      if (current.error !== undefined) return 0;
      try {
        const json = run(current.callbacks);
        return json === undefined ? 0 : cString(json);
      } catch (error) {
        current.error = error;
        return 0;
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

/** Loads the WebAssembly module (once per process) and creates an engine. */
export async function createEngine(): Promise<Engine> {
  const { module, active, pointers } = await load();
  const handle = module._axonpdf_new();
  const current: Active = { callbacks: {} };
  active.set(handle, current);
  module._axonpdf_set_render_callback(handle, pointers.render, handle);
  module._axonpdf_set_paint_callback(handle, pointers.paint, handle);
  module._axonpdf_set_hyphenation_callback(handle, pointers.hyphenate, handle);
  module._axonpdf_set_rows_callback(handle, pointers.rows, handle);

  const copy = (bytes: Uint8Array) => {
    const ptr = module._axonpdf_alloc(bytes.length);
    module.HEAPU8.set(bytes, ptr);
    return ptr;
  };
  const check = (status: number) => {
    const error = current.error;
    current.error = undefined;
    if (error !== undefined) throw error;
    if (status !== 0) throw new Error(module.UTF8ToString(module._axonpdf_last_error(handle)));
  };

  return {
    addFont(family, weight, italic, bytes) {
      const name = encoder.encode(family);
      const namePtr = copy(name);
      // The engine takes the font bytes; the name is only borrowed.
      const status = module._axonpdf_add_font(
        handle,
        namePtr,
        name.length,
        weight,
        italic ? 1 : 0,
        copy(bytes),
        bytes.length,
      );
      module._axonpdf_dealloc(namePtr, name.length);
      check(status);
    },
    addImage(id, bytes) {
      check(module._axonpdf_add_image(handle, id, copy(bytes), bytes.length));
    },
    render(treeJson, onChunk, callbacks = {}) {
      const json = encoder.encode(treeJson);
      const ptr = copy(json);
      current.onChunk = onChunk;
      current.callbacks = callbacks;
      try {
        check(module._axonpdf_render(handle, ptr, json.length, pointers.output, handle));
      } finally {
        module._axonpdf_dealloc(ptr, json.length);
        current.onChunk = undefined;
        current.callbacks = {};
      }
    },
    hyphenateBuiltin(word) {
      const bytes = encoder.encode(word);
      const wordPtr = copy(bytes);
      const out = module._axonpdf_hyphenate_builtin(wordPtr, bytes.length);
      module._axonpdf_dealloc(wordPtr, bytes.length);
      const json = module.UTF8ToString(out);
      module._axonpdf_dealloc(out, encoder.encode(json).length + 1);
      return JSON.parse(json);
    },
    free() {
      active.delete(handle);
      module._axonpdf_free(handle);
    },
  };
}
