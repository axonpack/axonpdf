import { describe, expect, it } from "vitest";

import { createEngine } from "../index.js";

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
    engine.render(
      JSON.stringify(tree),
      (chunk) => chunks.push(chunk),
      (id, args) => {
        calls.push({ id, ...args });
        return JSON.stringify({
          children: [{ type: "TEXT_INSTANCE", value: "1" }],
          readsTotal: false,
        });
      },
    );
    engine.free();

    const pdf = Buffer.concat(chunks).toString("latin1");
    expect(pdf.startsWith("%PDF-")).toBe(true);
    expect(pdf.trimEnd().endsWith("%%EOF")).toBe(true);
    expect(calls[0]).toMatchObject({ id: 3, pageNumber: 1 });
  });

  it("rethrows an error from a render function after the engine returns", async () => {
    const engine = await createEngine();
    const render = () =>
      engine.render(
        JSON.stringify(tree),
        () => {},
        () => {
          throw new Error("boom");
        },
      );
    expect(render).toThrow("boom");
    engine.free();
  });

  it("throws the engine's message for bad input", async () => {
    const engine = await createEngine();
    expect(() => engine.render("{", () => {})).toThrow();
    engine.free();
  });
});
