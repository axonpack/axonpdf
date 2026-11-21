import { describe, expect, it } from "vitest";

import { Document, Page, Text, createRenderer } from "../index.node.js";
import type { Container } from "../types/instance.type.js";

describe("createRenderer", () => {
  it("takes createContainer(container) and updateContainer(doc, mountNode, parent, callback)", () => {
    const commits: Container[] = [];
    const renderer = createRenderer({ onChange: (container) => commits.push(container) });
    const container: Container = { document: null };
    const mountNode = renderer.createContainer(container);
    let called = false;
    renderer.updateContainer(
      <Document>
        <Page>
          <Text>Hello</Text>
        </Page>
      </Document>,
      mountNode,
      null,
      () => (called = true),
    );
    expect(called).toBe(true);
    expect(commits).toEqual([container]);
    expect(container.document).toMatchObject({
      type: "DOCUMENT",
      children: [{ type: "PAGE", children: [{ type: "TEXT" }] }],
    });
    renderer.updateContainer(null, mountNode, null);
    expect(container.document).toBeNull();
  });
});
