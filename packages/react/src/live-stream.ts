/** Renders by writing chunks; `wait` resolves when the reader is ready for more. */
export type Run = (write: (chunk: Uint8Array) => void, wait: () => Promise<void>) => Promise<void>;

/** Where a live render's chunks go. */
export interface Sink<S> {
  stream: S;
  write(chunk: Uint8Array): void;
  /** Resolves when the reader wants more; throws when it has gone away. */
  wait(): Promise<void>;
  end(): void;
  fail(error: unknown): void;
}

/**
 * Starts `run` into `sink` and resolves with the stream as soon as the first chunk is written,
 * so the PDF flows out while the rest is laid out. An error before any byte rejects; one after
 * that errors the stream.
 */
export function startStream<S>(sink: Sink<S>, run: Run): Promise<S> {
  return new Promise((resolve, reject) => {
    let started = false;
    const write = (chunk: Uint8Array) => {
      if (!started) resolve(sink.stream);
      started = true;
      sink.write(chunk);
    };
    run(write, () => sink.wait()).then(
      () => {
        sink.end();
        resolve(sink.stream);
      },
      (error) => (started ? sink.fail(error) : reject(error)),
    );
  });
}

/** A web `ReadableStream` sink, for browsers and Workers. */
export function webSink(): Sink<ReadableStream<Uint8Array>> {
  let controller!: ReadableStreamDefaultController<Uint8Array>;
  let cancelled = false;
  let pulled: (() => void) | undefined;
  const stream = new ReadableStream<Uint8Array>({
    start: (c) => {
      controller = c;
    },
    pull: () => pulled?.(),
    cancel: () => {
      cancelled = true;
      pulled?.();
    },
  });
  return {
    stream,
    write: (chunk) => controller.enqueue(chunk),
    async wait() {
      if (!cancelled && (controller.desiredSize ?? 1) <= 0) {
        await new Promise<void>((resolve) => (pulled = resolve));
      }
      if (cancelled) throw new Error("The PDF stream was cancelled.");
    },
    end: () => controller.close(),
    fail: (error) => controller.error(error),
  };
}
