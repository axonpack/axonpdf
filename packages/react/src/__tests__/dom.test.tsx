import { describe, expect, it, vi } from "vitest";

import {
  BlobProvider,
  Document,
  PDFDownloadLink,
  PDFViewer,
  Page,
  Text,
  createRenderer,
  usePDF,
} from "../index.browser.js";
import type { Container } from "../types/instance.type.js";

const doc = (label: string) => (
  <Document>
    <Page>
      <Text>{label}</Text>
    </Page>
  </Document>
);

/** Mounts `element` in a bare React root and returns the instance tree it renders. */
function mountHost(element: React.ReactElement) {
  const container: Container = { document: null };
  const renderer = createRenderer({ onChange: () => {} });
  const root = renderer.createContainer(container, (e) => {
    throw e;
  });
  renderer.updateContainer(root, element);
  return {
    get node() {
      return container.document!;
    },
    update: (next: React.ReactElement | null) => renderer.updateContainer(root, next),
  };
}

const isPdf = async (blob: Blob | null) =>
  new TextDecoder("latin1").decode(await blob!.arrayBuffer()).startsWith("%PDF-");

describe("browser components", () => {
  it("usePDF renders the document to a Blob and an object URL, and again on update", async () => {
    const states: ReturnType<typeof usePDF>[0][] = [];
    let update!: ReturnType<typeof usePDF>[1];
    function Probe() {
      const [instance, set] = usePDF({ document: doc("First") });
      states.push(instance);
      update = set;
      return null;
    }
    mountHost(<Probe />);
    expect(states[0]).toMatchObject({ loading: true, blob: null, url: null, error: null });
    await vi.waitFor(() => expect(states.at(-1)!.url).toMatch(/^blob:/));
    const first = states.at(-1)!;
    expect(first.loading).toBe(false);
    expect(await isPdf(first.blob)).toBe(true);

    update(doc("Second"));
    await vi.waitFor(() => expect(states.at(-1)!.url).not.toBe(first.url));
    expect(await isPdf(states.at(-1)!.blob)).toBe(true);
  });

  it("PDFViewer shows the document in an iframe, with the toolbar switch on its URL", async () => {
    const host = mountHost(
      <PDFViewer width={300} height={200} title="Preview" showToolbar={false}>
        {doc("Viewer")}
      </PDFViewer>,
    );
    expect(host.node.type).toBe("iframe");
    await vi.waitFor(() => expect(host.node.props.src).toMatch(/^blob:.*#toolbar=0$/));
    expect(host.node.props).toMatchObject({ width: 300, height: 200, title: "Preview" });
  });

  it("PDFDownloadLink links to the rendered file under its name, and passes the instance on", async () => {
    const clicks: unknown[] = [];
    const host = mountHost(
      <PDFDownloadLink
        document={doc("Link")}
        fileName="report.pdf"
        onClick={(_, instance) => clicks.push(instance)}
      >
        {/* A host renderer keeps strings only inside text, so the label is a Text. */}
        {({ loading }) => <Text>{loading ? "Loading" : "Download"}</Text>}
      </PDFDownloadLink>,
    );
    expect(host.node).toMatchObject({ type: "a", props: { download: "report.pdf" } });
    await vi.waitFor(() => expect(host.node.props.href).toMatch(/^blob:/));
    expect(host.node.children).toMatchObject([
      { type: "TEXT", children: [{ type: "TEXT_INSTANCE", value: "Download" }] },
    ]);
    host.node.props.onClick({});
    expect(clicks[0]).toMatchObject({ loading: false, url: host.node.props.href });
  });

  it("BlobProvider gives its child function the Blob and URL", async () => {
    const seen: { url: string | null; blob: Blob | null }[] = [];
    mountHost(
      <BlobProvider document={doc("Blob")}>
        {({ url, blob }) => {
          seen.push({ url, blob });
          return null;
        }}
      </BlobProvider>,
    );
    await vi.waitFor(() => expect(seen.at(-1)!.url).toMatch(/^blob:/));
    expect(await isPdf(seen.at(-1)!.blob)).toBe(true);
  });
});
