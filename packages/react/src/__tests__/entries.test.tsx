import { existsSync } from "node:fs";
import { resolve } from "node:path";

import { describe, expect, it } from "vitest";

import pkg from "../../package.json" with { type: "json" };
import * as browser from "../index.browser.js";
import * as node from "../index.node.js";

const ROOT = resolve(import.meta.dirname, "../..");
// The primitives every element name comes from, as the components' default export lists them.
const PRIMITIVES =
  "G Svg View Text Link Page Note Path Rect Line FieldSet TextInput Select Checkbox List Stop Defs Image ImageBackground Tspan Canvas Circle Ellipse Polygon Document Polyline ClipPath TextInstance Fragment LinearGradient RadialGradient Marker".split(
    " ",
  );

describe("package entries", () => {
  it("maps the Node entry to the browser one for bundlers, and both files exist", () => {
    expect(pkg.browser).toEqual({ [`./${pkg.main}`]: pkg.exports["."].browser });
    expect(pkg.exports["."]).toMatchObject({
      default: `./${pkg.main}`,
      worker: pkg.exports["."].browser,
    });
    for (const file of [pkg.main, pkg.exports["."].browser, pkg.types]) {
      expect(existsSync(resolve(ROOT, file)), file).toBe(true);
    }
  });

  it("throws the Node-only APIs' environment error in the browser entry", () => {
    for (const name of [
      "renderToStream",
      "renderToBuffer",
      "renderToString",
      "renderToFile",
      "render",
    ] as const) {
      expect(() => (browser[name] as () => unknown)()).toThrow(
        `${name} is a Node specific API. You're either using this method in a browser, or your bundler is not loading axonpdf from the appropriate web build.`,
      );
    }
  });

  it("throws the web-only APIs' environment error in the Node entry", () => {
    for (const name of ["usePDF", "PDFViewer", "PDFDownloadLink", "BlobProvider"] as const) {
      expect(() => (node[name] as () => unknown)()).toThrow(
        `${name} is a web specific API. You're either using this component on Node, or your bundler is not loading axonpdf from the appropriate web build.`,
      );
    }
  });

  it("keeps every API and primitive on the default export of both entries", () => {
    const apis = [
      "pdf",
      "Font",
      "version",
      "StyleSheet",
      "usePDF",
      "PDFViewer",
      "BlobProvider",
      "PDFDownloadLink",
      "renderToStream",
      "renderToString",
      "renderToFile",
      "render",
    ];
    for (const entry of [node, browser]) {
      for (const name of [...apis, ...PRIMITIVES]) {
        expect(entry.default, name).toHaveProperty(name, (entry as Record<string, unknown>)[name]);
      }
    }
  });
});
