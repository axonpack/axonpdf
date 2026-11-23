# Live benchmark

A web page where you pick a page count and watch axonpdf and react-pdf 4.9.0 render the same
document in your browser. The document is a made-up warehouse ledger: a fixed header with a small
SVG logo, a "Page n of N" footer, and shipments, each a heading row followed by its lines, some
with a note that wraps under the item. The data comes from a seeded generator, so every run builds
the same document.

Both engines render one set of components (`src/ledger.jsx`). The page passes in either
`@axonpdf/react` or `@react-pdf/renderer`, and nothing else changes. In a project, switching is
that one import.

It uses the published packages, `@axonpdf/react` 0.0.1 and `@react-pdf/renderer` 4.9.0, so it
needs Node and npm but no Rust toolchain.

## Run it

```sh
cd bench/live
npm install
npm run dev
```

Open the URL Vite prints. Pick a page count (100, 500, 2000, or a custom number), an engine or
both side by side, and whether each engine runs in a Web Worker (the default for both) or on the
main thread. The main thread is only offered up to 100 pages; above that both engines run in a
worker. react-pdf's worker (`src/react-pdf.worker.js`) calls its `pdf().toBlob()` there and
answers the same messages as axonpdf's `expose`, so the page sends both the same way.

## Build and deploy

```sh
cd bench/live
npm ci
npm run build
```

`dist/` is a static site. Upload it to any static host (GitHub Pages, Netlify, Cloudflare Pages,
an S3 bucket). Paths are relative, so it also works from a subfolder such as
`example.com/bench/live/`. The host must serve `.wasm` files as `application/wasm`, which the
common hosts already do. It must also send `Cross-Origin-Opener-Policy: same-origin` and
`Cross-Origin-Embedder-Policy: require-corp`: they turn on cross-origin isolation, which the
page-plus-worker memory figure needs. `public/_headers` sets them on Cloudflare Pages and Netlify,
and `vite.config.js` sets them for `npm run dev` and `npm run preview`; on other hosts, add them
in its config. To check the build before uploading, run `npm run preview`.

## What the numbers mean

- Time to PDF: from the click (after a 100 ms pause) until the PDF exists as a Blob. It
  includes building the ledger data, the React render, layout and writing the file.
- Pages: counted in the finished PDF. The page count you pick is a target, so the real count
  can be off by a page or two.
- File size: the size of the PDF.
- Longest frame gap: the longest time between two animation frames on the main thread during
  the render. Around 16 ms means the page stayed smooth; a gap close to the render time means the
  page froze for the whole render. The moving bar next to the Render button shows the same thing.
- Peak memory (page + worker): the highest memory the page and its workers used during the
  render, from `performance.measureUserAgentSpecificMemory()`, sampled about once a second. It
  needs cross-origin isolation (see above); without it the page shows "n/a". The browser can answer
  late, so a short render may get only one sample.
- Page heap (excludes worker): the main thread's JavaScript heap peak from `performance.memory`
  (Chromium only). A worker's memory isn't in it.