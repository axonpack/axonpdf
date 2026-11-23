import { afterEach, describe, expect, it, vi } from "vitest";

import { Document, Page, Text, createRenderer } from "../index.browser.js";
import type { Container } from "../types/instance.type.js";
import { expose, usePDFWorker, useWorker } from "../worker.js";
import type { UsePDFInstance } from "../dom/use-pdf.js";

type Props = { label: string; fail?: boolean; delay?: number };

/**
 * A Worker that runs `expose`'s handler in this thread. `delay` holds a request back, so a test
 * can make an older render finish after a newer one.
 */
function fakeWorker() {
  const inside = new EventTarget();
  const outside = new EventTarget();
  vi.stubGlobal("addEventListener", inside.addEventListener.bind(inside));
  vi.stubGlobal("postMessage", (data: unknown) =>
    outside.dispatchEvent(new MessageEvent("message", { data })),
  );
  expose(({ label, fail }: Props) => {
    if (fail) throw new Error("bad props");
    return (
      <Document>
        <Page>
          <Text>{label}</Text>
        </Page>
      </Document>
    );
  });
  return Object.assign(outside, {
    terminate: vi.fn(),
    postMessage: (data: { props: Props }) =>
      setTimeout(
        () => inside.dispatchEvent(new MessageEvent("message", { data })),
        data.props.delay ?? 0,
      ),
  }) as unknown as Worker & { terminate: ReturnType<typeof vi.fn> };
}

function mountHost(element: React.ReactElement) {
  const container: Container = { document: null };
  const renderer = createRenderer({ onChange: () => {} });
  const root = renderer.createContainer(container, (e) => {
    throw e;
  });
  renderer.updateContainer(element, root, null);
  return {
    get node() {
      return container.document!;
    },
    update: (next: React.ReactElement | null) => renderer.updateContainer(next, root, null),
  };
}

const text = async (blob: Blob | null) =>
  new TextDecoder("latin1").decode(await blob!.arrayBuffer());

afterEach(() => {
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
});

describe("usePDFWorker", () => {
  const states: UsePDFInstance[] = [];
  function Probe({ worker, props }: { worker: Worker; props: Props }) {
    states.push(usePDFWorker(worker, props));
    return null;
  }
  afterEach(() => {
    states.length = 0;
  });

  it("is loading, then gives a URL and the PDF blob", async () => {
    mountHost(<Probe worker={fakeWorker()} props={{ label: "One" }} />);
    expect(states[0]).toMatchObject({ loading: true, url: null, blob: null, error: null });
    await vi.waitFor(() => expect(states.at(-1)!.url).toMatch(/^blob:/));
    expect(states.at(-1)!.loading).toBe(false);
    expect(await text(states.at(-1)!.blob)).toMatch(/^%PDF-/);
  });

  it("gives the error the worker sent back", async () => {
    mountHost(<Probe worker={fakeWorker()} props={{ label: "x", fail: true }} />);
    await vi.waitFor(() => expect(states.at(-1)!.error).toBeInstanceOf(Error));
    expect(states.at(-1)).toMatchObject({ loading: false, url: null });
  });

  it("renders again when props change, and drops a result for older props", async () => {
    const worker = fakeWorker();
    const revoke = vi.spyOn(URL, "revokeObjectURL");
    const host = mountHost(<Probe worker={worker} props={{ label: "First" }} />);
    await vi.waitFor(() => expect(states.at(-1)!.url).toMatch(/^blob:/));
    const first = states.at(-1)!.url!;

    host.update(<Probe worker={worker} props={{ label: "Slow", delay: 50 }} />);
    host.update(<Probe worker={worker} props={{ label: "Fast" }} />);
    await vi.waitFor(() => expect(states.at(-1)!.url).not.toBe(first));
    expect(revoke).toHaveBeenCalledWith(first);
    const fast = states.at(-1)!;
    await new Promise((resolve) => setTimeout(resolve, 100));
    expect(states.at(-1)!.url).toBe(fast.url);
  });
});

describe("useWorker", () => {
  it("creates the worker once and terminates it on unmount, revoking the URL", async () => {
    const worker = fakeWorker();
    const create = vi.fn(() => worker);
    const revoke = vi.spyOn(URL, "revokeObjectURL");
    let state!: UsePDFInstance;
    function App() {
      state = usePDFWorker(useWorker(create), { label: "Mine" });
      return null;
    }
    const host = mountHost(<App />);
    await vi.waitFor(() => expect(state.url).toMatch(/^blob:/));
    expect(create).toHaveBeenCalledTimes(1);
    host.update(null);
    expect(worker.terminate).toHaveBeenCalled();
    expect(revoke).toHaveBeenCalledWith(state.url);
  });
});
