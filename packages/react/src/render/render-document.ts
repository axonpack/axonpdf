import { createEngine, type RenderArgs } from "@axonpdf/core";

import { addEmoji } from "../assets/emoji.js";
import { Font } from "../font/font-store.js";
import { addFonts } from "../font/load-fonts.js";
import { serializeDocument } from "../tree/serialize.js";
import { createSession, settle, type Session } from "../tree/session.js";
import type { Item } from "../types/instance.type.js";
import type { Recorder } from "../types/profile.type.js";
import type { TreeDocument } from "../types/tree.type.js";

/** Runs `fn` as the engine's callback: nothing async (images) may start inside it. */
function inCallback<T>(session: Session, fn: () => T): T {
  const outer = session.inCallback;
  session.inCallback = true;
  try {
    return fn();
  } finally {
    session.inCallback = outer;
  }
}

/** The root host item as the engine's tree; the root must be a `Document`. */
export function serializeRoot(document: Item | null | undefined, session: Session) {
  if (!document || document.type !== "DOCUMENT") {
    throw new Error("The root element must be a <Document>.");
  }
  return serializeDocument(document as Exclude<Item, { type: "TEXT_INSTANCE" }>, session);
}

/**
 * Renders the committed `Document`; `onChunk` gets each piece of the PDF in order. With an async
 * `Rows`, pages leave between batches; otherwise the engine runs in one go.
 */
export async function renderDocument(
  /** Builds the tree: from the reconciler's container, or walking the elements. */
  build: (session: Session) => TreeDocument,
  onChunk: (chunk: Uint8Array) => void,
  /** Awaited before each async batch: a stream's reader can hold the render back. */
  wait?: () => Promise<void>,
  /** Called once the tree is read, so a caller done with React can unmount it before the engine runs. */
  release?: () => void,
  /** Set only by `profile`. */
  recorder?: Recorder,
) {
  recorder?.mark("tree");
  const session = createSession();
  let tree: TreeDocument | undefined = build(session);
  release?.();
  recorder?.mark("assets");
  await settle(session);
  await addEmoji(tree, session);
  // Made inside the engine call: the tree and the string are garbage while the engine runs.
  const json = () => {
    recorder?.mark("tree");
    const text = JSON.stringify(tree);
    tree = undefined;
    recorder?.mark("engine");
    return text;
  };
  const engine = await createEngine();
  try {
    // ponytail: rows render after fonts are loaded, so with Rows every registered family loads.
    const families = session.rows.size
      ? new Set(Font.getRegisteredFontFamilies())
      : session.families;
    await addFonts(engine, families);
    const added = new Set<number>();
    const addImages = () => {
      for (const [id, bytes] of session.assets) {
        if (!added.has(id)) engine.addImage(id, bytes);
        added.add(id);
      }
    };
    addImages();
    recorder?.engine(engine, false);
    const builtin = (word: string) => engine.hyphenateBuiltin(word);
    const callbacks = {
      render: (id: number, args: RenderArgs) => session.callbacks.get(id)!(args),
      paint: (id: number, w?: number, h?: number) => session.paints.get(id)!(w, h),
      hyphenate: (id: number, word: string) =>
        JSON.stringify(session.hyphenators.get(id)!(word, builtin)),
    };
    if (![...session.rows.values()].some((source) => source.async)) {
      // Every Rows is an array or other sync iterable: the engine pulls batches as it goes.
      const rows = (id: number) =>
        inCallback(session, () => session.rows.get(id)!.next() as string);
      engine.render(json, onChunk, { ...callbacks, rows });
      return;
    }
    // An async Rows: the engine stops whenever it needs a batch, and carries on once it's pushed.
    // Batches are made between engine calls, so images in rows can load then.
    let id = engine.begin(json, onChunk, callbacks);
    while (id !== undefined) {
      recorder?.mark("rows");
      await wait?.();
      const batch = await session.rows.get(id)!.next();
      await settle(session);
      addImages();
      recorder?.mark("engine");
      id = engine.pushRows(id, batch);
    }
  } finally {
    recorder?.engine(engine, true);
    engine.free();
  }
}
