import type { Engine } from "@axonpdf/core";

import { fetchBytes } from "../assets/fetch-bytes.js";
import { Font, isStandardFont, type FontSource } from "./font-store.js";

export async function loadFontSource(source: FontSource): Promise<Uint8Array | null> {
  if (isStandardFont(source.src)) return null;
  const { method, headers, body } = source;
  source.data ??= await fetchBytes(source.src, { method, headers, body });
  return source.data;
}

/** Loads every registered font of the families the document names and hands them to the engine. */
export async function addFonts(engine: Engine, families: Set<string>) {
  const registered = Font.getRegisteredFonts();
  const sources = [...families].flatMap((family) => registered[family]?.sources ?? []);
  const loaded = await Promise.all(sources.map(loadFontSource));
  sources.forEach((source, i) => {
    const data = loaded[i];
    if (data) engine.addFont(source.family, source.fontWeight, source.fontStyle !== "normal", data);
  });
}
