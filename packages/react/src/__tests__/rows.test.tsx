import { describe, expect, it } from "vitest";

import { Rows } from "../extras.js";
import { pdf, renderToBuffer } from "../index.node.js";
import { mount } from "../pdf.js";
import { Document, Page, Text } from "../primitives.js";
import { serializeDocument } from "../tree/serialize.js";
import { createSession, settle } from "../tree/session.js";

async function* letters() {
  yield* ["a", "b", "c"];
}

const doc = (rows: Iterable<string> | AsyncIterable<string>) => (
  <Document>
    <Page>
      <Rows rows={rows} batchSize={2}>
        {(row, i) => <Text>{`${i}:${row}`}</Text>}
      </Rows>
    </Page>
  </Document>
);

const texts = (json: string) => {
  const batch = JSON.parse(json);
  return { done: batch.done, rows: batch.children.map((c: any) => c.children[0].value) };
};

describe("Rows", () => {
  it.each([
    ["an array", () => ["a", "b", "c"]],
    ["an async generator, read once", letters],
  ])("hands out %s in batches as the engine pulls them", async (_, rows) => {
    const session = createSession();
    const tree = serializeDocument(pdf(doc(rows())).container.document!, session);
    await settle(session);
    const node = (tree.children[0] as any).children[0];
    expect(node).toMatchObject({ type: "ROWS", props: { batchSize: 2 }, children: [] });
    const source = session.rows.get(node.props.rows)!;
    expect(texts(await source.next())).toEqual({ done: false, rows: ["0:a", "1:b"] });
    expect(texts(await source.next())).toEqual({ done: true, rows: ["2:c"] });
  });

  it("gives page bands a placeholder total and isLast in a Rows document", async () => {
    const seen: { page: number; total: unknown; last: boolean }[] = [];
    const rows = Array.from({ length: 120 }, (_, i) => `row ${i}`);
    await renderToBuffer(
      <Document>
        <Page
          footer={({ pageNumber, totalPages, isLast }) => {
            seen.push({ page: pageNumber, total: totalPages, last: isLast });
            return <Text>{`${pageNumber} / ${totalPages}`}</Text>;
          }}
        >
          <Rows rows={rows}>{(row) => <Text>{row}</Text>}</Rows>
        </Page>
      </Document>,
    );
    const last = seen.filter((s) => s.last);
    expect(last).toHaveLength(1);
    expect(last[0].page).toBe(Math.max(...seen.map((s) => s.page)));
    expect(typeof last[0].total).toBe("string");
  });

  it("renders a PDF with rows pulled by the engine", async () => {
    const buffer = await renderToBuffer(doc(Array.from({ length: 250 }, (_, i) => `row ${i}`)));
    expect(buffer.subarray(0, 5).toString()).toBe("%PDF-");
  });

  it("streams an async source a batch at a time, with the same bytes as an array", async () => {
    const count = 2000;
    const batchSize = 100;
    let produced = 0;
    async function* source() {
      for (let i = 0; i < count; i++) {
        produced++;
        yield i;
      }
    }
    const render = async (rows: Iterable<number> | AsyncIterable<number>, onChunk: () => void) => {
      const chunks: Uint8Array[] = [];
      await mount(
        <Document creationDate={new Date(0)}>
          <Page>
            <Rows rows={rows} batchSize={batchSize}>
              {(n) => <Text>{`Row ${n}`}</Text>}
            </Rows>
          </Page>
        </Document>,
      ).render((chunk) => {
        chunks.push(chunk);
        onChunk();
      });
      return Buffer.concat(chunks);
    };
    // How many rows had been read from the source when each chunk of the PDF went out.
    const readAt: number[] = [];
    const streamed = await render(source(), () => readAt.push(produced));
    expect(produced).toBe(count);
    // The first page leaves after a couple of batches, not after the whole source...
    expect(readAt[1]).toBeLessThanOrEqual(3 * batchSize);
    // ...and most pages leave before the last batch is read.
    expect(readAt.filter((n) => n < count).length / readAt.length).toBeGreaterThan(0.75);
    const fromArray = await render(
      Array.from({ length: count }, (_, i) => i),
      () => {},
    );
    expect(streamed.equals(fromArray)).toBe(true);
  });
});
