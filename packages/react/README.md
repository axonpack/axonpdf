<div align="center">

# @axonpdf/react

**Create PDF files with React components, in Node, the browser and Web Workers.**

**[Documentation](https://axonpack.dev/docs/axonpdf/react/)** · [Getting started](https://axonpack.dev/docs/axonpdf/getting-started/) · [Reference](https://axonpack.dev/docs/axonpdf/react/reference/) · [Web Worker](https://axonpack.dev/docs/axonpdf/react/web-worker/)

[![npm version](https://img.shields.io/npm/v/@axonpdf/react.svg)](https://www.npmjs.com/package/@axonpdf/react)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://github.com/axonpack/axonpdf/blob/main/LICENSE)

</div>

## Install

```sh
npm install @axonpdf/react react
```

React 18 and 19 are supported.

## Example

```tsx
import { Document, Page, StyleSheet, Text, View, renderToFile } from "@axonpdf/react";

const styles = StyleSheet.create({
  page: { padding: 40, fontSize: 12 },
  row: { flexDirection: "row", justifyContent: "space-between" },
});

const Invoice = () => (
  <Document title="Invoice 42">
    <Page size="A4" style={styles.page}>
      <View style={styles.row}>
        <Text>Total</Text>
        <Text>£120.00</Text>
      </View>
      <Text fixed render={({ pageNumber, totalPages }) => `Page ${pageNumber} of ${totalPages}`} />
    </Page>
  </Document>
);

await renderToFile(<Invoice />, "invoice.pdf");
```

## Getting the PDF

`Invoice` is the component from the example above. `renderToFile` is one of several ways to get the PDF. In Node:

| API | Gives you |
| --- | --- |
| `renderToFile(<Invoice />, path)` | Writes the PDF to a file. |
| `renderToBuffer(<Invoice />)` | The whole PDF as a `Buffer`. |
| `renderToStream(<Invoice />)` | A stream that sends each page as soon as it is laid out, for example straight into an HTTP response. |

In the browser, and in Node:

| API | Gives you |
| --- | --- |
| `pdf(<Invoice />).toBlob()` | A `Blob` to upload, store or open. |
| `pdf(<Invoice />).toBuffer()` | A stream of the PDF's bytes. |
| `<PDFDownloadLink document={<Invoice />} fileName="invoice.pdf">` | A link that downloads the PDF. |
| `<PDFViewer><Invoice /></PDFViewer>` | The PDF shown in the page. |
| `<BlobProvider document={<Invoice />}>` and `usePDF({ document: <Invoice /> })` | The `blob`, a `url`, `loading` and `error`, to build your own preview or download button. |

```tsx
import { PDFDownloadLink } from "@axonpdf/react";

export function DownloadButton() {
  return (
    <PDFDownloadLink document={<Invoice />} fileName="invoice.pdf">
      {({ loading }) => (loading ? "Preparing…" : "Download PDF")}
    </PDFDownloadLink>
  );
}
```

[Rendering](https://axonpack.dev/docs/axonpdf/react/rendering/) has every option and return type,
streaming from a server, and updating a document after it has rendered.

[`packages/react/example`](https://github.com/axonpack/axonpdf/tree/main/packages/react/example) in the repository renders in Node and in the browser.

## In a Web Worker

The worker file calls `expose`. A plain `.js` file with `createElement` works with or without a
bundler:

```js
// axonpdf.worker.js
import { createElement } from "react";
import { expose } from "@axonpdf/react/worker";
import { Report } from "./report.js";

expose((props) => createElement(Report, props));
```

```jsx
import { usePDFWorker, useWorker } from "@axonpdf/react/worker";

const worker = useWorker(
  () => new Worker(new URL("./axonpdf.worker.js", import.meta.url), { type: "module" }),
);
const { url, loading } = usePDFWorker(worker, { rows });
```

The [Web Worker page](https://axonpack.dev/docs/axonpdf/react/web-worker/) covers loading a
prebuilt worker from `public/`, fonts, and what props can hold.

## Benchmarks

A shipping ledger with one `<Page>` per section, rendered in Chrome on an Apple M4 on 2026-11-23,
each engine in a Web Worker. Each cell is the time to the finished PDF and the worker's peak heap.

| Pages | axonpdf 0.0.1 | react-pdf 4.9.0 |
| --- | --- | --- |
| 100 | 0.90 s, 11.6 MB | 2.8 s, 192.5 MB |
| 500 | 4.1 s, 55.9 MB | 45.8 s, 2230 MB |
| 2000 | 15.9 s, 141.6 MB | no PDF after 10 min |

With the whole ledger in one `<Page>` that wraps onto every page:

| Pages | axonpdf 0.0.1 | react-pdf 4.9.0 |
| --- | --- | --- |
| 100 | 0.86 s | 32.8 s, 560.1 MB |
| 500 | 3.9 s, 39.2 MB | no PDF after 10 min |

axonpdf's 100-page flowing render finished before the first heap sample. The
[benchmarks page](https://axonpack.dev/docs/axonpdf/benchmarks/) has the method and the Node runs,
every run is in [bench/RESULTS.md](https://github.com/axonpack/axonpdf/blob/main/bench/RESULTS.md),
and [bench/live](https://github.com/axonpack/axonpdf/tree/main/bench/live) is the app to rerun it.

Coming from react-pdf 4.9.0? The [migration guide](https://axonpack.dev/docs/axonpdf/migrate-from-react-pdf/) shows the one import to change, or the npm alias that changes nothing in your code.

## Documentation

| Page | What it covers |
| --- | --- |
| [Migrate from react-pdf](https://axonpack.dev/docs/axonpdf/migrate-from-react-pdf/) | Changing one import or adding an npm alias |
| [Components](https://axonpack.dev/docs/axonpdf/react/components/) | Document, Page, View, Text, Link, Image, Canvas, SVG |
| [Styling](https://axonpack.dev/docs/axonpdf/react/styling/) | Units, flexbox, borders, text, media queries |
| [Fonts](https://axonpack.dev/docs/axonpdf/react/fonts/) | Registering fonts, weights, hyphenation, emoji |
| [Rendering](https://axonpack.dev/docs/axonpdf/react/rendering/) | Files, streams and buffers in Node; viewer, link and hooks in the browser |
| [Long documents](https://axonpack.dev/docs/axonpdf/react/long-documents/) | `<Rows>`, generators and memory |
| [axonpdf extras](https://axonpack.dev/docs/axonpdf/react/extras/) | `PageNumber`, `PageCount`, tables, header and footer bands |
| [Security and archiving](https://axonpack.dev/docs/axonpdf/react/security-and-archiving/) | Passwords, PDF/A, form fields |
| [Profiling](https://axonpack.dev/docs/axonpdf/react/profiling/) | Where a slow render spends its time |
| [Reference](https://axonpack.dev/docs/axonpdf/react/reference/) | Every export, prop, style key and error ([REFERENCE.md](REFERENCE.md) here) |

## License

MIT
