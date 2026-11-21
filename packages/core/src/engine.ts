import createModule, { type AxonpdfModule } from "../wasm/axonpdf.mjs";

/** What the engine knows about the page when it calls a `render` function. While pages are
 * being cut only `pageNumber` is known; the rest are `undefined` until the final call. */
export interface RenderArgs {
  pageNumber: number;
  /** Undefined until the page count is known. */
  totalPages: number | undefined;
  subPageNumber: number | undefined;
  subPageTotalPages: number | undefined;
}

/** Runs the JS function stored under `id` and returns `{"children":[...],"readsTotal":bool}`. */
export type RenderCallback = (id: number, args: RenderArgs) => string;

export interface Engine {
  addFont(family: string, weight: number, italic: boolean, bytes: Uint8Array): void;
  addImage(id: number, bytes: Uint8Array): void;
  /** Renders the tree; `onChunk` gets each piece of the PDF, in order, as it is written. */
  render(treeJson: string, onChunk: (chunk: Uint8Array) => void, onRender?: RenderCallback): void;
  /** Frees the engine's memory. The engine can't be used afterwards. */
  free(): void;
}

interface Active {
  onChunk?: (chunk: Uint8Array) => void;
  onRender?: RenderCallback;
  /** A JS error must not unwind through Rust, so it waits here until the call returns. */
  error?: unknown;
}

interface Loaded {
  module: AxonpdfModule;
  active: Map<number, Active>;
  output: number;
  callback: number;
}

let loading: Promise<Loaded> | undefined;

function load(): Promise<Loaded> {
  loading ??= createModule().then((module) => {
    const active = new Map<number, Active>();
    const encoder = new TextEncoder();
    const output = module.addFunction((handle, ptr, len) => {
      const run = active.get(handle)!;
      if (run.error === undefined) run.onChunk?.(module.HEAPU8.slice(ptr, ptr + len));
    }, "viii");
    const callback = module.addFunction((handle, id, page, total, subPage, subTotal) => {
      const run = active.get(handle)!;
      if (run.error !== undefined || !run.onRender) return 0;
      try {
        const json = encoder.encode(
          run.onRender(id, {
            pageNumber: page,
            totalPages: total < 0 ? undefined : total,
            subPageNumber: subPage < 0 ? undefined : subPage,
            subPageTotalPages: subTotal < 0 ? undefined : subTotal,
          }),
        );
        const ptr = module._axonpdf_alloc(json.length + 1);
        module.HEAPU8.set(json, ptr);
        module.HEAPU8[ptr + json.length] = 0;
        return ptr;
      } catch (error) {
        run.error = error;
        return 0;
      }
    }, "iiiiiii");
    return { module, active, output, callback };
  });
  return loading;
}

/** Loads the WebAssembly module (once per process) and creates an engine. */
export async function createEngine(): Promise<Engine> {
  const { module, active, output, callback } = await load();
  const handle = module._axonpdf_new();
  const run: Active = {};
  active.set(handle, run);
  module._axonpdf_set_render_callback(handle, callback, handle);

  const copy = (bytes: Uint8Array) => {
    const ptr = module._axonpdf_alloc(bytes.length);
    module.HEAPU8.set(bytes, ptr);
    return ptr;
  };
  const check = (status: number) => {
    const error = run.error;
    run.error = undefined;
    if (error !== undefined) throw error;
    if (status !== 0) throw new Error(module.UTF8ToString(module._axonpdf_last_error(handle)));
  };

  return {
    addFont(family, weight, italic, bytes) {
      const name = new TextEncoder().encode(family);
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
    render(treeJson, onChunk, onRender) {
      const json = new TextEncoder().encode(treeJson);
      const ptr = copy(json);
      run.onChunk = onChunk;
      run.onRender = onRender;
      try {
        check(module._axonpdf_render(handle, ptr, json.length, output, handle));
      } finally {
        module._axonpdf_dealloc(ptr, json.length);
        run.onChunk = undefined;
        run.onRender = undefined;
      }
    },
    free() {
      active.delete(handle);
      module._axonpdf_free(handle);
    },
  };
}
