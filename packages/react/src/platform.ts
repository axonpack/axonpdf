/** What differs between Node and the browser. The Node entry replaces these at import. */
export const platform = {
  /** Reads a file path or `file:` URL. Browsers have none, so everything is fetched. */
  readFile: undefined as ((path: string) => Promise<Uint8Array>) | undefined,
  /** What `pdf().toBuffer()` returns: a Node `Readable` in Node, a `ReadableStream` here. */
  toStream: (chunks: Uint8Array[]): unknown => new Blob(chunks as BlobPart[]).stream(),
};
