import { fetchBytes } from "./fetch-bytes.js";

const cache = new Map<string, Promise<Uint8Array>>();

/** Resolves every image source form to bytes: URL or path, bytes, Blob, objects, factories. */
export async function imageBytes(source: unknown, useCache = true): Promise<Uint8Array> {
  if (typeof source === "function") return imageBytes(await source(), useCache);
  if (source instanceof Promise) return imageBytes(await source, useCache);
  if (typeof source === "string") {
    if (!useCache) return fetchBytes(source);
    let bytes = cache.get(source);
    if (!bytes) {
      bytes = fetchBytes(source);
      cache.set(source, bytes);
      bytes.catch(() => cache.delete(source));
    }
    return bytes;
  }
  if (source instanceof Uint8Array) return source;
  if (source instanceof ArrayBuffer) return new Uint8Array(source);
  if (typeof Blob !== "undefined" && source instanceof Blob) {
    return new Uint8Array(await source.arrayBuffer());
  }
  if (source && typeof source === "object") {
    if ("data" in source) return imageBytes(source.data, useCache);
    if ("uri" in source) {
      const { uri, ...options } = source as { uri: string };
      return fetchBytes(uri, options);
    }
  }
  throw new Error(`Invalid image source: ${String(source)}`);
}
