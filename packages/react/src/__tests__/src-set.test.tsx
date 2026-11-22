import { resolve } from "node:path";

import { describe, expect, it } from "vitest";

import { imageSource } from "../assets/src-set.js";
import { Document, Image, Page, renderToBuffer } from "../index.node.js";

const ASSETS = resolve(import.meta.dirname, "../../../../fixtures/assets");
const SET = "small.jpg 300w, large.jpg 900w, medium.jpg 600w";

describe("srcSet", () => {
  it("picks the narrowest candidate at least as wide as the image is shown, else the widest", () => {
    expect(imageSource({ srcSet: SET, src: "src.jpg" }, 595.28)).toBe("medium.jpg");
    expect(imageSource({ srcSet: SET, sizes: 200 }, 595.28)).toBe("small.jpg");
    expect(imageSource({ srcSet: SET, sizes: "1000" }, 595.28)).toBe("large.jpg");
    expect(imageSource({ srcSet: SET, sizes: "25vw" }, 1000)).toBe("small.jpg");
    expect(imageSource({ srcSet: SET, sizes: "3in" }, 1000)).toBe("small.jpg");
  });

  it("matches sizes conditions against the page width, in order", () => {
    const sizes =
      "(orientation: landscape) 900px, (max-width: 400px) 200px, (min-width: 800pt) 80%, 500";
    expect(imageSource({ srcSet: SET, sizes }, 300)).toBe("small.jpg");
    expect(imageSource({ srcSet: SET, sizes }, 1000)).toBe("large.jpg");
    expect(imageSource({ srcSet: SET, sizes }, 600)).toBe("medium.jpg");
  });

  it("falls back to src or source without a width candidate", () => {
    expect(imageSource({ srcSet: "a.jpg 2x", src: "src.jpg" }, 600)).toBe("src.jpg");
    expect(imageSource({ source: "source.jpg" }, 600)).toBe("source.jpg");
  });

  it("loads only the picked image over src, against the page it is on", async () => {
    const image = (
      <Image
        src={`${ASSETS}/photo.jpg`}
        srcSet={`${ASSETS}/photo.jpg 300w, ${ASSETS}/badge.png 600w`}
        sizes="(max-width: 400px) 250px, 100vw"
        style={{ width: 50 }}
      />
    );
    const jpegs = async (size: "A4" | "A6") => {
      const file = (
        await renderToBuffer(
          <Document>
            <Page size={size}>{image}</Page>
          </Document>,
        )
      ).toString("latin1");
      return file.split("/DCTDecode").length - 1;
    };
    expect(await jpegs("A6")).toBe(1);
    expect(await jpegs("A4")).toBe(0);
  });
});
