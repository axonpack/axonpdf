import { readFileSync } from "node:fs";

import type { ReactElement } from "react";
import { describe, expect, it } from "vitest";

import "../index.node.js";
import { mount } from "../pdf.js";
import { Document, Image, Page, Text, View } from "../primitives.js";
import { rootScope } from "../tree/dispatcher.js";
import { serializeDocument } from "../tree/serialize.js";
import { createSession } from "../tree/session.js";
import { walk } from "../tree/walk.js";

const badge = new URL("../../../../fixtures/assets/badge.png", import.meta.url);

/** The file, apart from the parts that change each run. */
async function file(element: ReactElement, reconciler: boolean) {
  const root = mount(element);
  // Reading the container makes the reconciler root, which the render then uses.
  if (reconciler) expect(root.container).toBeTruthy();
  const chunks: Uint8Array[] = [];
  await root.render((chunk) => chunks.push(chunk));
  return Buffer.concat(chunks)
    .toString("latin1")
    .replace(/\/CreationDate \([^)]*\)/g, "")
    .replace(/\/ID \[<[0-9a-f]+> <[0-9a-f]+>\]/gi, "");
}

// Read on each render, so each render's image is a new buffer.
const Badge = () => <Image src={readFileSync(badge)} style={{ width: 40 }} />;
const Row = ({ i }: { i: number }) => <Text>{`Row ${i}`}</Text>;
const list = (n: number) => Array.from({ length: n }, (_, i) => <Row key={i} i={i} />);
const footer = (
  <Text fixed render={({ pageNumber, totalPages }) => `Page ${pageNumber} of ${totalPages}`} />
);

describe("lazy children", () => {
  it("sends a long list's first children and leaves the rest to the engine", () => {
    const [root] = walk(
      <Document>
        <Page>
          <View>{list(300)}</View>
        </Page>
      </Document>,
      rootScope(),
    );
    const session = createSession();
    const view = (serializeDocument(root as any, session).children[0] as any).children[0];
    expect(view.children).toHaveLength(64);
    expect(session.rows.get(view.props.lazy)?.lazy).toBe(true);
  });

  it.each([
    [
      "a long list",
      <Document key="a">
        <Page>
          <View>{list(400)}</View>
          {footer}
        </Page>
      </Document>,
    ],
    [
      "many pages, with images made afresh on each render",
      <Document key="b">
        {[0, 1, 2].map((p) => (
          <Page key={p}>
            <Badge />
            <Image src={badge.pathname} style={{ width: 20 }} />
            {list(30)}
            {footer}
          </Page>
        ))}
      </Document>,
    ],
  ])("renders %s to the same file as the reconciler", async (_, doc) => {
    const [a, b] = [await file(doc, false), await file(doc, true)];
    if (a !== b) {
      let i = 0;
      while (a[i] === b[i]) i++;
      console.log(
        a.length,
        b.length,
        i,
        JSON.stringify(a.slice(i - 200, i + 50)),
        JSON.stringify(b.slice(i - 200, i + 50)),
      );
    }
    expect(a).toBe(b);
  });
});
