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
  /**
   * Renders the tree; `onChunk` gets each piece of the PDF, in order, as it is written. Pass a
   * function that makes the JSON to let a large string be freed while the engine runs.
   */
  render(
    treeJson: string | (() => string),
    onChunk: (chunk: Uint8Array) => void,
    callbacks?: Callbacks,
  ): void;
  /**
   * Starts a render that stops when it needs rows: returns the id of the `Rows` whose next batch
   * it needs (give it with `pushRows`), or `undefined` when the file is complete. Finished
   * pages go to `onChunk` during each call. Leave `callbacks.rows` out: rows are pushed.
   */
  begin(
    treeJson: string | (() => string),
    onChunk: (chunk: Uint8Array) => void,
    callbacks?: Callbacks,
  ): number | undefined;
  /** Pushes a batch `{"children":[...],"done":bool}` to the `Rows` with `id`; returns as `begin`. */
  pushRows(id: number, batchJson: string): number | undefined;
  /** The built-in split of a word: at its soft hyphens, else by the English patterns. */
  hyphenateBuiltin(word: string): string[];
  /** Frees the engine's memory. The engine can't be used afterwards. */
  free(): void;
}
