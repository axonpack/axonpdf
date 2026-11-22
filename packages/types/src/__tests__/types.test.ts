import { describe, expect, expectTypeOf, it } from "vitest";

import * as components from "@axonpdf/react/components";
import FontStore from "@axonpdf/font";

import {
  Primitive,
  type Bookmark,
  type FontStore as FontStoreType,
  type FontWeight,
  type PageSize,
  type SourceObject,
  type Style,
  type StyleProp,
} from "../index.js";

describe("@axonpdf/types", () => {
  it("types what a document's code writes", () => {
    const style: Style = {
      padding: "10 20",
      fontWeight: "bold",
      "@media max-width: 400": { fontSize: 8 },
    };
    const list: StyleProp = [style, false, [{ margin: 2 }]];
    const size: PageSize = [200, "10in"];
    const bookmark: Bookmark = { title: "Intro", fit: true };
    const source: SourceObject = async () => ({ uri: "https://example.com/a.png", method: "GET" });
    const weight: FontWeight = 600;
    expect([style, list, size, bookmark, source, weight]).toHaveLength(6);
    expectTypeOf<FontStoreType>().toEqualTypeOf<FontStore>();
  });

  it("names each primitive as the components do", () => {
    expect(Primitive.View).toBe(components.View as unknown as string);
    expect(Primitive.TextInstance).toBe(components.TextInstance);
    expect(Primitive.ClipPath).toBe(components.ClipPath as unknown as string);
  });
});
