---
'@axonpdf/core': patch
'@axonpdf/react': patch
---

Add `usePDFWorker` and `useWorker` to `@axonpdf/react/worker`, and a `worker` prop on `PDFViewer`, `PDFDownloadLink` and `BlobProvider`, so a page can render in a Web Worker without calling `renderInWorker` itself. Both packages now ship their changelog and link to the axonpdf docs.
