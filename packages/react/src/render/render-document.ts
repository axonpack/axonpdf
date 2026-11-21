import { createEngine } from "@axonpdf/core";

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
  const engine = await createEngine();
  try {
    await addFonts(engine, session.families);
    for (const [id, bytes] of session.assets) engine.addImage(id, bytes);
    engine.render(JSON.stringify(tree), onChunk, (id, args) => session.callbacks.get(id)!(args));
  } finally {
    engine.free();
  }
}
