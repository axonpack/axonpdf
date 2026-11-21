import { describe, expect, it } from "vitest";

import { Rows } from "../extras.js";
import { pdf, renderToBuffer } from "../index.node.js";
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
    const next = session.rows.get(node.props.rows)!;
    expect(texts(next())).toEqual({ done: false, rows: ["0:a", "1:b"] });
    expect(texts(next())).toEqual({ done: true, rows: ["2:c"] });
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
});
