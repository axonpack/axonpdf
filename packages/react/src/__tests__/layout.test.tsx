import { describe, expect, it } from "vitest";

import { renderToBuffer } from "../index.node.js";
import { Document, Page, Text, View } from "../primitives.js";
import type { PageLayoutProps } from "../types/props.type.js";

const content = (
  <>
    <Text>First page body</Text>
    <Text break>Second page body</Text>
  </>
);

describe("Page layout", () => {
  it("renders a template around the content of every page", async () => {
    const calls: { pageNumber?: number; totalPages?: number }[] = [];
    const layout = ({ children, pageNumber, totalPages }: PageLayoutProps) => {
      calls.push({ pageNumber, totalPages });
      return (
        <View style={{ flex: 1 }}>
          <Text>Header</Text>
          <View style={{ flex: 1 }}>{children}</View>
          <Text>{`Page ${pageNumber} of ${totalPages}`}</Text>
        </View>
      );
    };
    const pdf = (
      await renderToBuffer(
        <Document>
          <Page layout={layout}>{content}</Page>
        </Document>,
      )
    ).toString("latin1");
    expect(pdf.trimEnd().endsWith("%%EOF")).toBe(true);
    expect(pdf.match(/\/Type\s*\/Page\b/g)).toHaveLength(2);
    expect(calls.map((c) => c.pageNumber)).toContain(2);
    expect(calls.at(-1)).toEqual({ pageNumber: 2, totalPages: 2 });
  });

  it("rejects a template that drops its children", async () => {
    const doc = (
      <Document>
        <Page layout={() => <Text>No content</Text>}>{content}</Page>
      </Document>
    );
    await expect(renderToBuffer(doc)).rejects.toThrow(
      "[layout] A page layout must render its children exactly once (found 0).",
    );
  });
});
