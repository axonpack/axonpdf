import { describe, expect, it } from "vitest";

import { pdf } from "../pdf.js";
import { Canvas, Document, Image, Page, Text, View } from "../primitives.js";
import { serializeDocument } from "../tree/serialize.js";
import { createSession, settle } from "../tree/session.js";

const name = "world";
const png = new Uint8Array([137, 80, 78, 71]);

function treeOf(element: React.ReactElement<any>) {
  const session = createSession();
  const tree = serializeDocument(pdf(element).container.document!, session);
  return { tree, session };
}

describe("serializeDocument", () => {
  it("builds the tree from elements, styles passed through as written", () => {
    const style = [{ padding: "10 20" }, false, null, [{ border: "1pt solid red" }]];
    const { tree } = treeOf(
      <Document title="Invoice">
        <Page size="A4" style={style as any}>
          <View wrap={false} fixed>
            <Text>Hello {name}</Text>
          </View>
        </Page>
      </Document>,
    );
    expect(tree).toMatchObject({ v: 1, type: "DOCUMENT", props: { title: "Invoice" } });
    const page = tree.children[0] as any;
    expect(page).toMatchObject({ type: "PAGE", props: { size: "A4" }, style });
    expect(page.children[0]).toMatchObject({ type: "VIEW", props: { wrap: false, fixed: true } });
    expect(page.children[0].children[0].children).toEqual([
      { type: "TEXT_INSTANCE", value: "Hello " },
      { type: "TEXT_INSTANCE", value: "world" },
    ]);
  });

  it("keeps functions on the JS side under an id and reports reads of the total", () => {
    const { tree, session } = treeOf(
      <Document>
        <Page>
          <Text render={({ pageNumber, totalPages }) => `${pageNumber} / ${totalPages}`} />
          <Text render={({ pageNumber }) => <Text>{pageNumber}</Text>} />
        </Page>
      </Document>,
    );
    const [withTotal, withoutTotal] = (tree.children[0] as any).children;
    const call = (id: number) =>
      JSON.parse(
        session.callbacks.get(id)!({
          pageNumber: 2,
          totalPages: 5,
          subPageNumber: 1,
          subPageTotalPages: 1,
        }),
      );
    expect(call(withTotal.render)).toEqual({
      children: [{ type: "TEXT_INSTANCE", value: "2 / 5" }],
      readsTotal: true,
    });
    expect(call(withoutTotal.render)).toMatchObject({
      children: [{ type: "TEXT", children: [{ type: "TEXT_INSTANCE", value: "2" }] }],
      readsTotal: false,
    });
  });

  it("sends image bytes once as an asset and collects font families", async () => {
    const { tree, session } = treeOf(
      <Document>
        <Page style={{ fontFamily: ["Inter", "Helvetica"] }}>
          <Image src={png} />
          <Image source={png} />
        </Page>
      </Document>,
    );
    await settle(session);
    const [a, b] = (tree.children[0] as any).children;
    expect(a.props.src).toEqual(b.props.src);
    expect(session.assets.get(a.props.src.asset)).toBe(png);
    expect([...session.families]).toEqual(["Inter", "Helvetica"]);
  });
});

describe("Canvas", () => {
  it("records the painter calls of paint, in order, with the box size", () => {
    const { tree, session } = treeOf(
      <Document>
        <Page>
          <Canvas
            paint={(p, w, h) => {
              const g = p.linearGradient(0, 0, w, 0).stop(0, "red").stop(1, "blue", 0.5);
              p.moveTo(10, 10)
                .lineTo(w, h)
                .strokeColor("red")
                .stroke()
                .fillColor(g)
                .rect(0, 0, 5, 5)
                .fill();
              return null;
            }}
          />
        </Page>
      </Document>,
    );
    const canvas = (tree.children[0] as any).children[0];
    expect(JSON.parse(session.paints.get(canvas.paint)!(100, 50))).toEqual([
      ["linearGradient", 0, 0, 0, 100, 0],
      ["stop", 0, 0, "red"],
      ["stop", 0, 1, "blue", 0.5],
      ["moveTo", 10, 10],
      ["lineTo", 100, 50],
      ["strokeColor", "red"],
      ["stroke"],
      ["fillColor", { gradient: 0 }],
      ["rect", 0, 0, 5, 5],
      ["fill"],
    ]);
  });
});
