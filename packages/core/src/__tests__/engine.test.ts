import { describe, expect, it } from "vitest";

import { createEngine, loadPageWidth } from "../index.js";

const tree = {
  v: 1,
  type: "DOCUMENT",
  props: {},
  children: [
    {
      type: "PAGE",
      props: {},
      children: [{ type: "TEXT", props: {}, render: 3, children: [] }],
    },
  ],
};

describe("createEngine", () => {
  it("streams a PDF and calls render functions by id", async () => {
    const engine = await createEngine();
    const chunks: Uint8Array[] = [];
    const calls: unknown[] = [];
    engine.render(JSON.stringify(tree), (chunk) => chunks.push(chunk), {
      render: (id, args) => {
        calls.push({ id, ...args });
        return JSON.stringify({
          children: [{ type: "TEXT_INSTANCE", value: "1" }],
          readsTotal: false,
        });
      },
    });
    engine.free();

    const pdf = Buffer.concat(chunks).toString("latin1");
    expect(pdf.startsWith("%PDF-")).toBe(true);
    expect(pdf.trimEnd().endsWith("%%EOF")).toBe(true);
    expect(calls[0]).toMatchObject({ id: 3, pageNumber: 1 });
  });

  it("takes the JSON from a function, sized exactly for any UTF-8", async () => {
    const engine = await createEngine();
    const text = { type: "TEXT_INSTANCE", value: "£ • é 😀 \ud800 x" };
    const json = JSON.stringify({
      ...tree,
      children: [
        { type: "PAGE", props: {}, children: [{ type: "TEXT", props: {}, children: [text] }] },
      ],
    });
    const chunks: Uint8Array[] = [];
    // One byte short would cut the closing braces and fail to parse.
    engine.render(
      () => json,
      (chunk) => chunks.push(chunk),
    );
    engine.free();
    expect(Buffer.concat(chunks).toString("latin1").trimEnd().endsWith("%%EOF")).toBe(true);
  });

  it("rethrows an error from a render function after the engine returns", async () => {
    const engine = await createEngine();
    const render = () =>
      engine.render(JSON.stringify(tree), () => {}, {
        render: () => {
          throw new Error("boom");
        },
      });
    expect(render).toThrow("boom");
    engine.free();
  });

  it("throws the engine's message for bad input", async () => {
    const engine = await createEngine();
    expect(() => engine.render("{", () => {})).toThrow();
    engine.free();
  });

  it("splits a word with the built-in hyphenation", async () => {
    const engine = await createEngine();
    expect(engine.hyphenateBuiltin("hyphenation").join("")).toBe("hyphenation");
    expect(engine.hyphenateBuiltin("hyphenation").length).toBeGreaterThan(1);
    expect(engine.hyphenateBuiltin("co\u00adop")).toEqual(["co", "op"]);
    engine.free();
  });

  it("stops for rows and carries on when they are pushed", async () => {
    const engine = await createEngine();
    const rowsTree = {
      v: 1,
      type: "DOCUMENT",
      props: {},
      children: [
        {
          type: "PAGE",
          props: {},
          children: [{ type: "ROWS", props: { rows: 9, batchSize: 1 }, children: [] }],
        },
      ],
    };
    const row = (n: number) => ({
      type: "TEXT",
      props: {},
      children: [{ type: "TEXT_INSTANCE", value: `row ${n}` }],
    });
    const chunks: Uint8Array[] = [];
    expect(engine.begin(JSON.stringify(rowsTree), (c) => chunks.push(c))).toBe(9);
    expect(engine.pushRows(9, JSON.stringify({ children: [row(1)], done: false }))).toBe(9);
    expect(engine.pushRows(9, JSON.stringify({ children: [row(2)], done: true }))).toBeUndefined();
    engine.free();
    expect(Buffer.concat(chunks).toString("latin1").trimEnd().endsWith("%%EOF")).toBe(true);
  });
});

describe("loadPageWidth", () => {
  it("measures a page from its size, orientation and dpi", async () => {
    const width = await loadPageWidth();
    expect(width({})).toBe(595.28);
    expect(width({ size: "LETTER", orientation: "landscape" })).toBe(792);
    expect(width({ size: ["4in", 300] })).toBe(288);
    expect(width({ size: { width: 192 }, dpi: 144 })).toBe(192);
    expect(width({ size: "B12" })).toBeNaN();
  });
});
