import { once } from "node:events";
import { PassThrough } from "node:stream";

import type { Sink } from "../live-stream.js";

/** A Node `Readable` sink that honours backpressure between engine steps. */
export function nodeSink(): Sink<PassThrough> {
  const stream = new PassThrough();
  return {
    stream,
    write: (chunk) => {
      stream.write(chunk);
    },
    async wait() {
      if (stream.writableNeedDrain && !stream.destroyed) {
        await Promise.race([once(stream, "drain"), once(stream, "close")]);
      }
      if (stream.destroyed) throw new Error("The PDF stream was closed.");
    },
    end: () => stream.end(),
    fail: (error) => stream.destroy(error as Error),
  };
}
