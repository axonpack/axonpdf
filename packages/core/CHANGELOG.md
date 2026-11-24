# @axonpdf/core

## 0.0.6

### Patch Changes

- 0c20b45: A Next.js app built with webpack (`next build`) compiles with `@axonpdf/react` again. The build stopped on a React internal that React 19 doesn't export.
- 74b4dbc: `usePDF`, `PDFViewer`, `PDFDownloadLink` and `BlobProvider` no longer log "The root element must be a <Document>" under React StrictMode in development. The render thrown away by StrictMode's extra mount stays quiet.
- 2a68a29: A Web Worker bundled by Turbopack (`next dev --turbopack`, `next build --turbopack`) finds `axonpdf.wasm` and renders. Before, it stayed on loading.

## 0.0.5

### Patch Changes

- cf8fe66: The README links the browser benchmark at axonpdf-demo.axonpack.dev.

## 0.0.4

### Patch Changes

- 327d2c6: The `@axonpdf/react` README lists every way to get the PDF (file, buffer, stream, blob, download link, viewer, `BlobProvider`, `usePDF`), links to the Rendering docs, and shows the browser benchmark.

## 0.0.3

### Patch Changes

- 5314daf: Rewrite the package READMEs: a short first example, a `.js` worker example with a link to the Web Worker guide, and links to the docs.

## 0.0.2

### Patch Changes

- 31302db: Add `usePDFWorker` and `useWorker` to `@axonpdf/react/worker`, and a `worker` prop on `PDFViewer`, `PDFDownloadLink` and `BlobProvider`, so a page can render in a Web Worker without calling `renderInWorker` itself. Both packages now ship their changelog and link to the axonpdf docs.

## 0.0.1

### Patch Changes

- e57fd9f: First release of axonpdf: the PDF engine and the React renderer.
