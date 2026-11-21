import type { Engine } from "./engine.type.js";
import { encoder, load, type Active } from "./module.js";

/** The UTF-8 size of `text`, as `TextEncoder` writes it (a lone surrogate becomes U+FFFD). */
function utf8Length(text: string) {
  let len = text.length;
  for (let i = 0; i < text.length; i++) {
    const c = text.charCodeAt(i);
    if (c < 0x80) continue;
    if (c < 0x800) len += 1;
    else if (c >= 0xd800 && c <= 0xdbff && (text.charCodeAt(i + 1) & 0xfc00) === 0xdc00) {
      len += 2;
      i++;
    } else len += 2;
  }
  return len;
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
  // Only `render` pulls rows; without the callback the engine waits for pushes.
  let rowsCallbackSet = false;
  const rowsId = module._axonpdf_alloc(4);

  const copy = (bytes: Uint8Array) => {
    const ptr = module._axonpdf_alloc(bytes.length);
    module.HEAPU8.set(bytes, ptr);
    return ptr;
  };
  /**
   * Writes the text as UTF-8 straight into engine memory, with no JS copy of the bytes. Given a
   * function, the string it makes is garbage once this returns, before the engine runs.
   */
  const copyText = (source: string | (() => string)) => {
    const text = typeof source === "function" ? source() : source;
    const len = utf8Length(text);
    const ptr = module._axonpdf_alloc(len);
    encoder.encodeInto(text, module.HEAPU8.subarray(ptr, ptr + len));
    return { ptr, len };
  };
  const check = (status: number) => {
    const error = current.error;
    current.error = undefined;
    if (error !== undefined) throw error;
    if (status !== 0) throw new Error(module.UTF8ToString(module._axonpdf_last_error(handle)));
  };

  // 1: the engine needs the rows whose id it wrote; 0: done; -1: failed.
  const step = (status: number) => {
    if (status === 1) return new DataView(module.HEAPU8.buffer).getUint32(rowsId, true);
    current.onChunk = undefined;
    current.callbacks = {};
    check(status);
    return undefined;
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
      const json = copyText(treeJson);
      current.onChunk = onChunk;
      current.callbacks = callbacks;
      if (callbacks.rows && !rowsCallbackSet) {
        module._axonpdf_set_rows_callback(handle, pointers.rows, handle);
        rowsCallbackSet = true;
      }
      try {
        check(module._axonpdf_render(handle, json.ptr, json.len, pointers.output, handle));
      } finally {
        module._axonpdf_dealloc(json.ptr, json.len);
        current.onChunk = undefined;
        current.callbacks = {};
      }
    },
    begin(treeJson, onChunk, callbacks = {}) {
      current.onChunk = onChunk;
      current.callbacks = callbacks;
      const json = copyText(treeJson);
      try {
        return step(
          module._axonpdf_render_begin(handle, json.ptr, json.len, pointers.output, handle, rowsId),
        );
      } finally {
        module._axonpdf_dealloc(json.ptr, json.len);
      }
    },
    pushRows(id, batchJson) {
      const json = copyText(batchJson);
      try {
        return step(
          module._axonpdf_push_rows(
            handle,
            id,
            json.ptr,
            json.len,
            pointers.output,
            handle,
            rowsId,
          ),
        );
      } finally {
        module._axonpdf_dealloc(json.ptr, json.len);
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
      module._axonpdf_dealloc(rowsId, 4);
      module._axonpdf_free(handle);
    },
  };
}
