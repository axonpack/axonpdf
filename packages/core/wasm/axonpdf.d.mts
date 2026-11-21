// Types for the Emscripten loader that scripts/build-wasm.sh writes next to this file.
export interface AxonpdfModule {
  HEAPU8: Uint8Array;
  addFunction(fn: (...args: number[]) => number | void, signature: string): number;
  UTF8ToString(ptr: number): string;
  _axonpdf_new(): number;
  _axonpdf_free(handle: number): void;
  _axonpdf_alloc(len: number): number;
  _axonpdf_dealloc(ptr: number, len: number): void;
  _axonpdf_add_font(
    handle: number,
    family: number,
    familyLen: number,
    weight: number,
    italic: number,
    bytes: number,
    len: number,
  ): number;
  _axonpdf_add_image(handle: number, id: number, bytes: number, len: number): number;
  _axonpdf_set_render_callback(handle: number, callback: number, user: number): void;
  _axonpdf_render(handle: number, json: number, jsonLen: number, output: number, user: number): number;
  _axonpdf_last_error(handle: number): number;
}

export default function createModule(options?: object): Promise<AxonpdfModule>;
