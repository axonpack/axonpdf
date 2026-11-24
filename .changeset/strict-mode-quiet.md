---
'@axonpdf/core': patch
'@axonpdf/react': patch
---

`usePDF`, `PDFViewer`, `PDFDownloadLink` and `BlobProvider` no longer log "The root element must be a <Document>" under React StrictMode in development. The render thrown away by StrictMode's extra mount stays quiet.
