import { Component, lazy, useState } from "react";
import { describe, expect, it } from "vitest";

import { mount } from "../pdf.js";
import { Document, Page, Text } from "../primitives.js";
import { Fallback } from "../tree/dispatcher.js";
import { walked } from "./walk.fixture.js";

function UpdatesWhileRendering() {
  const [n, setN] = useState(0);
  if (n < 2) setN(n + 1);
  return <Text>{`renders ${n}`}</Text>;
}

class LegacyContext extends Component {
  static contextTypes = {};
  render() {
    return <Text>legacy</Text>;
  }
}

const Lazy = lazy(async () => ({ default: () => <Text>lazy</Text> }));

const doc = (child: React.ReactNode) => (
  <Document>
    <Page>{child}</Page>
  </Document>
);

describe("falling back to the reconciler", () => {
  it.each([
    ["a state update during render", <UpdatesWhileRendering key="a" />],
    ["legacy context", <LegacyContext key="b" />],
    ["React.lazy", <Lazy key="c" />],
  ])("the walk hands %s to the reconciler", (_, child) => {
    expect(() => walked(doc(child))).toThrow(Fallback);
  });

  it("renders what the reconciler renders, with no error", async () => {
    const root = mount(doc(<UpdatesWhileRendering />));
    const chunks: Uint8Array[] = [];
    await root.render((chunk) => chunks.push(chunk));
    expect(Buffer.concat(chunks).subarray(0, 5).toString()).toBe("%PDF-");
    // The reconciler ran: it rendered again after each update.
    expect(JSON.stringify(root.container.document)).toContain("renders 2");
  });
});
