import { createEngine } from "@axonpdf/core";

import { addEmoji } from "../assets/emoji.js";
import { addFonts } from "../font/load-fonts.js";
import { serializeDocument } from "../tree/serialize.js";
import { createSession, settle } from "../tree/session.js";
import type { Instance } from "../types/instance.type.js";

/** Renders a committed `Document`; `onChunk` gets each piece of the PDF in order. */
export async function renderDocument(
  document: Instance | null,
  onChunk: (chunk: Uint8Array) => void,
) {
  if (!document || document.type !== "DOCUMENT") {
    throw new Error("The root element must be a <Document>.");
  }
  const session = createSession();
  const tree = serializeDocument(document, session);
  await settle(session);
  await addEmoji(tree, session);
  const engine = await createEngine();
  try {
    await addFonts(engine, session.families);
    for (const [id, bytes] of session.assets) engine.addImage(id, bytes);
    const builtin = (word: string) => engine.hyphenateBuiltin(word);
    engine.render(JSON.stringify(tree), onChunk, {
      render: (id, args) => session.callbacks.get(id)!(args),
      paint: (id, width, height) => session.paints.get(id)!(width, height),
      hyphenate: (id, word) => JSON.stringify(session.hyphenators.get(id)!(word, builtin)),
    });
  } finally {
    engine.free();
  }
}
