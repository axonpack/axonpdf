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

const invoice = (
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

await renderToFile(invoice, "invoice.pdf");
```

## Getting the PDF

`renderToFile` is one of several ways to get the PDF. In Node:

| API | Gives you |
| --- | --- |
| `renderToFile(doc, path)` | Writes the PDF to a file. |
| `renderToBuffer(doc)` | The whole PDF as a `Buffer`. |
| `renderToStream(doc)` | A stream that sends each page as soon as it is laid out, for example straight into an HTTP response. |

In the browser, and in Node:

| API | Gives you |
| --- | --- |
| `pdf(doc).toBlob()` | A `Blob` to upload, store or open. |
| `pdf(doc).toBuffer()` | A stream of the PDF's bytes. |
| `<PDFDownloadLink document={doc} fileName="report.pdf">` | A link that downloads the PDF. |
| `<PDFViewer>` | The PDF shown in the page. |
| `<BlobProvider document={doc}>` and `usePDF({ document })` | The `blob`, a `url`, `loading` and `error`, to build your own preview or download button. |

```tsx
import { PDFDownloadLink } from "@axonpdf/react";

<PDFDownloadLink document={doc} fileName="report.pdf">
  {({ loading }) => (loading ? "Preparing…" : "Download PDF")}
</PDFDownloadLink>;
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

## Documentation

| Page | What it covers |
| --- | --- |
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
