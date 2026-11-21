# @axonpdf/react example

One invoice, defined once in `src/invoice.tsx`, rendered in two places:

- `src/main.tsx` is a web page with a live preview (`PDFViewer`) and a download link
  (`PDFDownloadLink`).
- `scripts/render.tsx` is a Node script that writes `out.pdf`.

The invoice has text, an image (`public/logo.png`), a table and page numbers.

## Run it

From the repo root, build the engine and the packages first:

```sh
source <emsdk>/emsdk_env.sh   # see the root README
bun install
bun run build:wasm
bun run build
```

Then, in this folder:

```sh
bun run dev   # the web page, at the address Vite prints
bun run pdf   # writes out.pdf
```
