# @axonpdf/react

## 0.0.4

### Patch Changes

- 327d2c6: The `@axonpdf/react` README lists every way to get the PDF (file, buffer, stream, blob, download link, viewer, `BlobProvider`, `usePDF`), links to the Rendering docs, and shows the browser benchmark.
- Updated dependencies [327d2c6]
  - @axonpdf/core@0.0.4

## 0.0.3

### Patch Changes

- 5314daf: Rewrite the package READMEs: a short first example, a `.js` worker example with a link to the Web Worker guide, and links to the docs.
- Updated dependencies [5314daf]
  - @axonpdf/core@0.0.3

## 0.0.2

### Patch Changes

- 31302db: Add `usePDFWorker` and `useWorker` to `@axonpdf/react/worker`, and a `worker` prop on `PDFViewer`, `PDFDownloadLink` and `BlobProvider`, so a page can render in a Web Worker without calling `renderInWorker` itself. Both packages now ship their changelog and link to the axonpdf docs.
- Updated dependencies [31302db]
  - @axonpdf/core@0.0.2

## 0.0.1

### Patch Changes

- e57fd9f: First release of axonpdf: the PDF engine and the React renderer.
- Updated dependencies [e57fd9f]
  - @axonpdf/core@0.0.1
