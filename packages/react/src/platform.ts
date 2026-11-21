import { webSink, type Sink } from "./live-stream.js";

/** What differs between Node and the browser. The Node entry replaces these at import. */
export const platform = {
  /** Reads a file path or `file:` URL. Browsers have none, so everything is fetched. */
  readFile: undefined as ((path: string) => Promise<Uint8Array>) | undefined,
  /** Where live streams go: a Node `Readable` in Node, a web `ReadableStream` here. */
  sink: webSink as () => Sink<unknown>,
};
