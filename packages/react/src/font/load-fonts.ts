import type { Engine } from "@axonpdf/core";

import { Font } from "./font-store.js";

/** Loads every registered font of the families the document names and hands them to the engine. */
export async function addFonts(engine: Engine, families: Set<string>) {
  const registered = Font.getRegisteredFonts();
  const sources = [...families].flatMap((family) => registered[family]?.sources ?? []);
  const loaded = await Promise.all(sources.map(Font.loadSource));
  sources.forEach((source, i) => {
    const data = loaded[i];
    if (data) engine.addFont(source.family, source.fontWeight, source.fontStyle !== "normal", data);
  });
}
