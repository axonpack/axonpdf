import { Readable } from "node:stream";

import { describe, expect, it } from "vitest";

import { Document, Page, Text, pdf } from "../index.node.js";

const doc = (label: string, onRender?: (params: { blob?: Blob }) => void) => (
  <Document onRender={onRender}>
    <Page>
      <Text>{label}</Text>
    </Page>
  </Document>
);

const read = async (stream: NodeJS.ReadableStream) => {
  const chunks: Buffer[] = [];
  for await (const chunk of stream) chunks.push(chunk as Buffer);
  return Buffer.concat(chunks).toString("latin1");
};

describe("pdf() instance", () => {
  it("toBuffer returns a Node readable stream of the PDF", async () => {
    const stream = await pdf(doc("Stream")).toBuffer();
    expect(stream).toBeInstanceOf(Readable);
    const file = await read(stream);
    expect(file.startsWith("%PDF-")).toBe(true);
    expect(file.trimEnd().endsWith("%%EOF")).toBe(true);
  });

  it("calls change listeners on every update, until they are removed", () => {
    const instance = pdf();
    let changes = 0;
    const listener = () => changes++;
    instance.on("change", listener);
    instance.updateContainer(doc("One"));
    instance.updateContainer(doc("Two"));
    expect(changes).toBe(2);
    instance.removeListener("change", listener);
    instance.updateContainer(doc("Three"));
    expect(changes).toBe(2);
  });

  it("calls the Document's onRender with the blob after toBlob, and after toBuffer", async () => {
    const calls: { blob?: Blob }[] = [];
    const instance = pdf(doc("Callback", (params) => calls.push(params)));
    const blob = await instance.toBlob();
    expect(calls).toHaveLength(1);
    expect(calls[0].blob).toBe(blob);
    await read(await instance.toBuffer());
    expect(calls).toHaveLength(2);
    expect(calls[1].blob).toBeUndefined();
  });

  it("compresses every stream", async () => {
    const file = await read(await pdf(doc("Compressed ".repeat(50))).toBuffer());
    const streams = file.match(/<<(?:[^<>]|<<[^<>]*>>)*>>\s*stream\r?\n/g) ?? [];
    expect(streams.length).toBeGreaterThan(0);
    for (const dict of streams) expect(dict).toContain("/FlateDecode");
  });

  it("toString gives the PDF as text with its streams undeflated", async () => {
    const file = await pdf(doc("Plain words")).toString();
    expect(file.startsWith("%PDF-")).toBe(true);
    expect(file).not.toContain("/FlateDecode");
    expect(file).toMatch(/\[\(Plain w\) [\d.]+ \(ords\)\] TJ/);
  });
});
