import FontStore, { sharedFontStore } from "@axonpdf/font";
import { describe, expect, it } from "vitest";

import { Document, Font, Page, Text, renderToBuffer } from "../index.node.js";

const lato = new URL("../../../../fixtures/assets/Lato-Regular.ttf", import.meta.url).pathname;

describe("the font store", () => {
  it("is the one @axonpdf/font shares", () => {
    expect(Font).toBe(sharedFontStore);
    expect(Font).toBeInstanceOf(FontStore);
  });

  it("renders with a font registered through @axonpdf/font", async () => {
    sharedFontStore.register({ family: "Lato", src: lato });
    const pdf = await renderToBuffer(
      <Document>
        <Page>
          <Text style={{ fontFamily: "Lato" }}>Registered elsewhere</Text>
        </Page>
      </Document>,
    );
    const text = pdf.toString("latin1");
    expect(text).toMatch(/\/BaseFont \/[A-Z]{6}\+Lato/);
    expect(text).toContain("/FontFile2");
  });
});
