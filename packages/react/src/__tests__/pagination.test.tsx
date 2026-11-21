import { describe, expect, it, vi } from "vitest";

import { Document, Page, Text, View, renderToBuffer } from "../index.node.js";

const pages = (pdf: Buffer) => pdf.toString("latin1").match(/\/Type\s*\/Page\b/g)?.length;

describe("pagination", () => {
  it("calls a View's render function on every page it lands on", async () => {
    const calls: { pageNumber: number; subPageNumber?: number }[] = [];
    const pdf = await renderToBuffer(
      <Document>
        <Page size="A6">
          <View
            fixed
            render={({ pageNumber, subPageNumber }) => {
              calls.push({ pageNumber, subPageNumber });
              return <Text>{`Page ${pageNumber}`}</Text>;
            }}
          />
          <Text>First</Text>
          <Text break>Second</Text>
        </Page>
      </Document>,
    );
    expect(pages(pdf)).toBe(2);
    expect(calls).toContainEqual({ pageNumber: 1, subPageNumber: 1 });
    expect(calls).toContainEqual({ pageNumber: 2, subPageNumber: 2 });
  });

  it("warns about an unbreakable node taller than the page, and keeps it on one page", async () => {
    const warn = vi.spyOn(console, "warn").mockImplementation(() => {});
    const pdf = await renderToBuffer(
      <Document>
        <Page size="A6">
          <Text>Before</Text>
          <View wrap={false} style={{ height: 1000 }} />
          <Text>After</Text>
        </Page>
      </Document>,
    );
    expect(warn).toHaveBeenCalledWith(
      "Node of type VIEW can't wrap between pages and it's bigger than available page height",
    );
    warn.mockRestore();
    expect(pages(pdf)).toBe(2);
  });
});
