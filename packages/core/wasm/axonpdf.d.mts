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
  _axonpdf_set_paint_callback(handle: number, callback: number, user: number): void;
  _axonpdf_set_hyphenation_callback(handle: number, callback: number, user: number): void;
  _axonpdf_set_rows_callback(handle: number, callback: number, user: number): void;
  _axonpdf_hyphenate_builtin(word: number, wordLen: number): number;
  _axonpdf_render(handle: number, json: number, jsonLen: number, output: number, user: number): number;
  _axonpdf_render_begin(
    handle: number,
    json: number,
    jsonLen: number,
    output: number,
    user: number,
    rowsId: number,
  ): number;
  _axonpdf_push_rows(
    handle: number,
    id: number,
    batch: number,
    batchLen: number,
    output: number,
    user: number,
    rowsId: number,
  ): number;
  _axonpdf_last_error(handle: number): number;
  _axonpdf_total_pages_marker(): number;
  _axonpdf_profile(on: boolean): void;
  _axonpdf_profile_report(): number;
  _axonpdf_page_width(props: number, len: number): number;
}

export default function createModule(options?: object): Promise<AxonpdfModule>;
