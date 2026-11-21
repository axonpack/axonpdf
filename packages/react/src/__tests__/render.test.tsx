import { mkdtempSync, readFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

import { describe, expect, it } from "vitest";

import pkg from "../../package.json" with { type: "json" };
import {
  Document,
  Page,
  Text,
  renderToBuffer,
  renderToFile,
  usePDF,
  version,
} from "../index.node.js";

const calls: { pageNumber: number; totalPages?: number }[] = [];
const doc = (
  <Document>
    <Page>
      <Text>Invoice 42</Text>
      <Text
        render={({ pageNumber, totalPages }) => {
          calls.push({ pageNumber, totalPages });
          return `Page ${pageNumber} of ${totalPages}`;
        }}
        fixed
      />
    </Page>
  </Document>
);

describe("Node render functions", () => {
  it("writes a PDF file", async () => {
    const path = join(mkdtempSync(join(tmpdir(), "axonpdf-")), "out.pdf");
    await renderToFile(doc, path);
    const pdf = readFileSync(path, "latin1");
    expect(pdf.startsWith("%PDF-")).toBe(true);
    expect(pdf.trimEnd().endsWith("%%EOF")).toBe(true);
    expect(calls.at(-1)).toEqual({ pageNumber: 1, totalPages: 1 });
  });

  it("returns a Buffer", async () => {
    const buffer = await renderToBuffer(doc);
    expect(buffer.subarray(0, 5).toString()).toBe("%PDF-");
  });

  it("throws the web-only error for browser APIs", () => {
    expect(() => usePDF()).toThrow("usePDF is a web specific API");
  });

  it("reports the package version", () => {
    expect(version).toBe(pkg.version);
  });
});
