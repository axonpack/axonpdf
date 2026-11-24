---
'@axonpdf/core': patch
'@axonpdf/react': patch
---

A Web Worker bundled by Turbopack (`next dev --turbopack`, `next build --turbopack`) finds `axonpdf.wasm` and renders. Before, it stayed on loading.
