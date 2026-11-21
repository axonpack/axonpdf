import { setTimeout as sleep } from "node:timers/promises";

import { describe, expect, it } from "vitest";

import { Rows } from "../extras.js";
import { startStream, webSink } from "../live-stream.js";
import { renderToStream } from "../index.node.js";
import { Document, Image, Page, Text } from "../primitives.js";

function counted(count: number) {
  const state = { produced: 0 };
  async function* rows() {
    for (let i = 0; i < count; i++) {
      state.produced++;
      yield i;
    }
  }
  const doc = (
    <Document>
      <Page>
        <Rows rows={rows()}>{(n) => <Text>{`Row ${n}`}</Text>}</Rows>
      </Page>
    </Document>
  );
  return { state, doc };
}

async function readAll(stream: NodeJS.ReadableStream) {
  const chunks: Buffer[] = [];
  for await (const chunk of stream) chunks.push(chunk as Buffer);
  return Buffer.concat(chunks).toString("latin1");
}

describe("renderToStream", () => {
  it("hands out the stream before the last batch is read, then finishes the PDF", async () => {
    const { state, doc } = counted(2000);
    const stream = await renderToStream(doc);
    expect(state.produced).toBeLessThan(2000);
    const pdf = await readAll(stream);
    expect(state.produced).toBe(2000);
    expect(pdf.startsWith("%PDF-")).toBe(true);
    expect(pdf.trimEnd().endsWith("%%EOF")).toBe(true);
  });

  it("waits for the reader: unread output holds the render back", async () => {
    const { state, doc } = counted(10000);
    const stream = await renderToStream(doc);
    await sleep(300);
    expect(state.produced).toBeLessThan(10000);
    await readAll(stream);
    expect(state.produced).toBe(10000);
  });

  it("rejects for an error before the first byte", async () => {
    const doc = (
      <Document>
        <Page>
          <Image src={42 as any} />
        </Page>
      </Document>
    );
    await expect(renderToStream(doc)).rejects.toThrow("Invalid image source");
  });

  it("streams into a web ReadableStream for browsers, waiting for its reader", async () => {
    let written = 0;
    const stream = await startStream(webSink(), async (write, wait) => {
      for (let i = 0; i < 5; i++) {
        await wait();
        write(new Uint8Array([i]));
        written++;
      }
    });
    await sleep(50);
    expect(written).toBeLessThan(5);
    const bytes: number[] = [];
    const reader = stream.getReader();
    for (let r = await reader.read(); !r.done; r = await reader.read()) bytes.push(...r.value);
    expect(bytes).toEqual([0, 1, 2, 3, 4]);
  });
});
