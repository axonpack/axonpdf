import { platform } from "../platform.js";

export interface FetchOptions {
  method?: string;
  headers?: any;
  body?: any;
  credentials?: RequestCredentials;
}

/** Bytes from a URL (`http`, `https`, `data`, `blob`) or, in Node, a file path or `file:` URL. */
export async function fetchBytes(src: string, options?: FetchOptions): Promise<Uint8Array> {
  if (platform.readFile && !/^(https?|data|blob):/i.test(src)) return platform.readFile(src);
  const response = await fetch(src, options);
  if (!response.ok)
    throw new Error(`Failed to fetch ${src}: ${response.status} ${response.statusText}`);
  return new Uint8Array(await response.arrayBuffer());
}
