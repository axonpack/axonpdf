<div align="center">

# axonpdf

**Create PDF files from React components, in Node, the browser and Web Workers.**

**[Documentation](https://axonpack.dev/docs/axonpdf/)** · [Getting started](https://axonpack.dev/docs/axonpdf/getting-started/) · [Reference](https://axonpack.dev/docs/axonpdf/react/reference/) · [Benchmarks](https://axonpack.dev/docs/axonpdf/benchmarks/)

[![npm version](https://img.shields.io/npm/v/@axonpdf/react.svg)](https://www.npmjs.com/package/@axonpdf/react)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

</div>

axonpdf's engine is written in Rust, uses Yoga for flexbox, and is compiled to WebAssembly. It
writes each page as soon as it is finished, so documents of thousands of pages render in little memory. The
packages are at an early 0.0.x version, so expect changes.

## Install

```sh
npm install @axonpdf/react react
```

## Example

```tsx
import { Document, Page, Text, renderToFile } from "@axonpdf/react";

const Report = () => (
  <Document title="Hello">
    <Page size="A4" style={{ padding: 40 }}>
      <Text>Hello from axonpdf.</Text>
      <Text fixed render={({ pageNumber, totalPages }) => `Page ${pageNumber} of ${totalPages}`} />
    </Page>
  </Document>
);

await renderToFile(<Report />, "report.pdf");
```

## Getting the PDF

`Report` is the component from the example above. `renderToFile` is one of several ways to get the PDF. In Node:

| API | Gives you |
| --- | --- |
| `renderToFile(<Report />, path)` | Writes the PDF to a file. |
| `renderToBuffer(<Report />)` | The whole PDF as a `Buffer`. |
| `renderToStream(<Report />)` | A stream that sends each page as soon as it is laid out, for example straight into an HTTP response. |

In the browser, and in Node:

| API | Gives you |
| --- | --- |
| `pdf(<Report />).toBlob()` | A `Blob` to upload, store or open. |
| `pdf(<Report />).toBuffer()` | A stream of the PDF's bytes. |
| `<PDFDownloadLink document={<Report />} fileName="report.pdf">` | A link that downloads the PDF. |
| `<PDFViewer><Report /></PDFViewer>` | The PDF shown in the page. |
| `<BlobProvider document={<Report />}>` and `usePDF({ document: <Report /> })` | The `blob`, a `url`, `loading` and `error`, to build your own preview or download button. |

```tsx
import { PDFDownloadLink } from "@axonpdf/react";

export function DownloadButton() {
  return (
    <PDFDownloadLink document={<Report />} fileName="report.pdf">
      {({ loading }) => (loading ? "Preparing…" : "Download PDF")}
    </PDFDownloadLink>
  );
}
```

[Rendering](https://axonpack.dev/docs/axonpdf/react/rendering/) has every option and return type,
streaming from a server, and updating a document after it has rendered.

## What it does

- [Components](https://axonpack.dev/docs/axonpdf/react/components/) for pages, text, links, images, SVG, canvas and form fields, styled with [flexbox and CSS-like styles](https://axonpack.dev/docs/axonpdf/react/styling/).
- [Streams and files](https://axonpack.dev/docs/axonpdf/react/rendering/) that get each page as soon as it is laid out.
- [Long documents](https://axonpack.dev/docs/axonpdf/react/long-documents/) with `<Rows>`, which reads an array or an async iterable such as a database cursor in batches.
- [Page numbers, tables and page bands](https://axonpack.dev/docs/axonpdf/react/extras/): `PageNumber`, `PageCount`, tables with a repeating header, and a header and footer on every page.
- [Rendering in a Web Worker](https://axonpack.dev/docs/axonpdf/react/web-worker/), so the page stays responsive.
- [Passwords, PDF/A and forms](https://axonpack.dev/docs/axonpdf/react/security-and-archiving/), and a [profiler](https://axonpack.dev/docs/axonpdf/react/profiling/) for slow renders.

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
[benchmarks page](https://axonpack.dev/docs/axonpdf/benchmarks/) has the method and the Node runs. Every run is in [bench/RESULTS.md](bench/RESULTS.md), and
[bench/live](bench/live) is the benchmark app.

Coming from react-pdf 4.9.0? The [migration guide](https://axonpack.dev/docs/axonpdf/migrate-from-react-pdf/) shows the one import to change, or the npm alias that changes nothing in your code.

## Documentation

| Page | What it covers |
| --- | --- |
| [Getting started](https://axonpack.dev/docs/axonpdf/getting-started/) | Install, a first document, Node and the browser |
| [Migrate from react-pdf](https://axonpack.dev/docs/axonpdf/migrate-from-react-pdf/) | Changing one import or adding an npm alias |
| [@axonpdf/react](https://axonpack.dev/docs/axonpdf/react/) | Components, styles, fonts, rendering, Web Workers |
| [Reference](https://axonpack.dev/docs/axonpdf/react/reference/) | Every export, prop, style key and error |
| [@axonpdf/core](https://axonpack.dev/docs/axonpdf/core/) | The engine on its own, for bindings to other frameworks |

## Packages

| Package | What it is |
| --- | --- |
| [`@axonpdf/react`](packages/react) | The components and render functions, for Node and the browser. Most apps need only this. |
| [`@axonpdf/core`](packages/core) | The engine as WebAssembly, for Node, browsers and Workers. |

`packages/font`, `packages/stylesheet` and `packages/types` are internal and aren't published.
`@axonpdf/react` builds in the parts it uses.

## Development setup

These steps are for macOS and Linux.

### Rust

Install Rust with [rustup](https://rustup.rs), then add the target that the WebAssembly build uses:

```sh
rustup target add wasm32-unknown-emscripten
```

### C++ compiler

The engine includes Yoga, a C++20 library, and compiles it during the build. On macOS, the Xcode
Command Line Tools provide the compiler (`xcode-select --install`). On Linux, install clang or a
recent GCC.

### Emscripten

Emscripten builds the engine and Yoga into one WebAssembly file for browsers and Node. The project
uses Emscripten 7.0.0, installed with emsdk. One copy can serve every checkout, so it can live
anywhere; `.tools/emsdk` inside this repo is ignored by git.

```sh
git clone https://github.com/emscripten-core/emsdk.git .tools/emsdk
cd .tools/emsdk
./emsdk install 7.0.0
./emsdk activate 7.0.0
```

Then, in each new shell, run `source .tools/emsdk/emsdk_env.sh` to put `emcc` on your `PATH`.
`./emsdk activate --permanent` does this for you, but it edits your shell profile.

### Node

Install Node 24 or newer for the JavaScript side of the project.

### Check

```sh
rustc --version
rustup target list --installed   # should include wasm32-unknown-emscripten
emcc --version
node --version
```

## Build and test

The JavaScript packages, from the repo root:

```sh
source .tools/emsdk/emsdk_env.sh
bun install
bun run build:wasm    # the engine, into packages/core/wasm
bun run build         # every package's build/ folder
bun run test
bun run check-types && bun run lint && bun run format:check
```

`node parity/compare.mjs` renders each document in `fixtures/` and compares its pages with stored
reference renders (they live in the ignored `local/oracle/`; it needs poppler and qpdf).

The engine:

```sh
cargo test                                      # native
source .tools/emsdk/emsdk_env.sh
cargo test --target wasm32-unknown-emscripten   # WebAssembly, run in Node
```

The layout tests in `crates/engine/tests/yoga.rs` compare the engine with `yoga-layout` 3.2.1,
Meta's own build of the same Yoga release: every box, and every call Yoga makes to measure a leaf,
must be identical. `crates/engine/tests/fixtures/yoga.json` has one case per Yoga setting, and
`yoga-random.json` has 300 random trees from a fixed seed. After adding or changing a case in
`yoga.json`, write the expected results again:

```sh
cd parity && npm ci && npm run yoga
```

## License

[MIT](LICENSE)
