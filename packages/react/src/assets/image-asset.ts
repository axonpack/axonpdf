import type { Session } from "../tree/session.js";
import { imageBytes } from "./image-bytes.js";

/** Bytes a source holds already: a buffer, or `{ data }`. */
function heldBytes(source: any): Uint8Array | undefined {
  const data = source?.data ?? source;
  if (data instanceof Uint8Array) return data;
  if (data instanceof ArrayBuffer) return new Uint8Array(data);
  return undefined;
}

/** What an image source is known by from one try to the next: a URL by its text, bytes by
 * their content (a source made afresh in each render is still the same image). */
function sourceKey(source: any, held: Uint8Array | undefined): unknown {
  if (held) {
    let hash = 0x811c9dc5;
    for (const byte of held) hash = Math.imul(hash ^ byte, 0x01000193);
    return `bytes:${held.length}:${hash >>> 0}`;
  }
  return typeof source === "object" && source && "uri" in source ? `uri:${source.uri}` : source;
}

/** An image source as the engine's `{ "asset": id }`, its bytes loaded (or kept) for the engine. */
export function asset(source: unknown, useCache: boolean, session: Session) {
  let id = session.assetIds.get(source);
  if (id !== undefined) return { asset: id };
  const held = heldBytes(source);
  const key = sourceKey(source, held);
  // Loaded on an earlier try, and given to the engine before this one started. The n-th source
  // with this key gets the n-th one: a walk meets them in the same order each time.
  const used = session.carriedUsed.get(key) ?? 0;
  id = session.carriedIds.get(key)?.[used];
  if (id !== undefined) {
    session.carriedUsed.set(key, used + 1);
    session.assetIds.set(source, id);
    return { asset: id };
  }
  if (session.inCallback && !session.inBatch) {
    // ponytail: the engine can't take new images mid-render; preload them up front if needed.
    throw new Error("An Image inside a render function must also be used outside it.");
  }
  const assetId = session.nextId++;
  session.assetIds.set(source, assetId);
  const carried: { key: unknown; bytes?: Uint8Array; held: boolean } = { key, held: !!held };
  session.carried.images.push(carried);
  const keep = (bytes: Uint8Array) => {
    // Lazy children are built while the engine runs, too late to hand it an image: the image
    // is kept for the next try, and this one is made again.
    if (!session.inCallback) session.assets.set(assetId, bytes);
    carried.bytes = bytes;
  };
  if (held) keep(held);
  else session.pending.push(imageBytes(source, useCache).then(keep));
  if (session.inCallback) session.restart = true;
  return { asset: assetId };
}
