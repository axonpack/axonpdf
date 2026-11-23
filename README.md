# axonpdf

A PDF engine written in Rust, with React components on top. `@axonpdf/react` follows the usual
React PDF renderer API, so an existing project can switch by changing its imports. Pages are laid
out with flexbox and written as soon as each one is finished, which keeps memory low on documents
of thousands of pages.

Work in progress: the packages are on npm at an early 0.0.x version, so expect changes. To work
on axonpdf itself, see Development setup and Build and test.

The docs are at [axonpack.dev/docs/axonpdf](https://axonpack.dev/docs/axonpdf/).

## Packages

| Package | What it is |
| --- | --- |
| [`@axonpdf/react`](packages/react) | The React components and render functions, for Node and the browser. |
| [`@axonpdf/core`](packages/core) | The engine as WebAssembly, for Node, browsers and Workers. Bindings for other frameworks build on it. |
Each package has a README and a REFERENCE.md that lists everything it exports.

`packages/font`, `packages/stylesheet` and `packages/types` are internal: the font store, a style
resolver and the shared types. They aren't published. `@axonpdf/react` builds in the parts it
uses and exports `Font`, `Style` and the other types itself.

## Getting started

```sh
npm install @axonpdf/react react
```

### Node

```tsx
import { Document, Page, Text, renderToFile } from "@axonpdf/react";

const doc = (
  <Document title="Hello">
    <Page size="A4" style={{ padding: 40 }}>
      <Text>Hello from axonpdf.</Text>
      <Text fixed render={({ pageNumber, totalPages }) => `Page ${pageNumber} of ${totalPages}`} />
    </Page>
  </Document>
);

await renderToFile(doc, "hello.pdf");
```

`renderToStream` gives a stream that starts before the last page is laid out, and
`renderToBuffer` gives the whole file as a `Buffer`.

### Browser

```tsx
import { PDFDownloadLink, PDFViewer } from "@axonpdf/react";

function Preview() {
  return (
    <>
      <PDFViewer style={{ width: "100%", height: 600 }}>{doc}</PDFViewer>
      <PDFDownloadLink document={doc} fileName="hello.pdf">
        {({ loading }) => (loading ? "Preparing..." : "Download")}
      </PDFDownloadLink>
    </>
  );
}
```

`usePDF` and `pdf(doc).toBlob()` give you the Blob yourself, and `@axonpdf/react/worker` renders
in a Web Worker. Bundlers pick the browser build through the package's `browser` field and
`exports` conditions. `packages/react/example` is a small app with a web page and a Node script.

## Switching an existing project

Replace the renderer's package name with `@axonpdf/react` in your imports. Nothing else in the
document code has to change.

To leave the imports alone, alias the renderer's package name in `package.json` instead:

```json
{
  "dependencies": {
    "<the renderer package>": "npm:@axonpdf/react"
  }
}
```

If your code also imports the renderer's separate font, stylesheet or types packages, import
from `@axonpdf/react` there instead: it exports `Font`, `StyleSheet`, `Style` and the other
types. Those packages have no axonpdf counterpart to alias.

What behaves differently:

- React 18 and 19 are supported, not earlier versions.
- A Document without `creator` or `producer` writes `axonpdf` in both.
- `renderToStream`, `renderToFile` and `toBuffer` send pages as they are finished instead of
  laying the whole document out first. `toBuffer` waits for the last page only when the Document
  has `onRender`.
- These drawing bugs are fixed: a bookmark's `left` is measured from the page's left edge; a
  radial gradient runs from its focal point to its outer circle; `gradientTransform` angles are
  degrees and `skewX` and `skewY` act on the right axes; an unchecked checkbox is blank; floats sit
  inside their parent's padding and push later content down when their text wraps.

## Long documents

A document written the usual way already renders in little memory: long lists of children, and
each `<Page>` of a document with several, are built only when the engine reaches them, and each
page is written and dropped as soon as it is finished. Two things help more on very large
documents:

- Pass long lists as a generator instead of an array. An array of 190,000 row elements stays alive
  for the whole render; a generator never holds them all. In the 5000-page benchmark this lowered
  the peak from 471 MB to 215 MB.

  ```tsx
  function* rows(data) {
    for (const item of data) yield <Row key={item.id} item={item} />;
  }

  <View>{rows(data)}</View>
  ```

- Use `<Rows>` for rows from an array or an async iterable such as a database cursor. It renders
  and writes them in batches; with an async source, pages go out between batches.

  ```tsx
  import { Rows } from "@axonpdf/react";

  <Rows rows={cursor} batchSize={200}>
    {(row) => <Row item={row} />}
  </Rows>
  ```

  In a document with `Rows` the page total isn't known ahead, so `totalPages` in `render`
  functions is a placeholder the engine fills in at the end: show it, but don't compute with it.
  `<PageCount />` works as usual. `Rows` is axonpdf's own component.

`profile(element)` reports where a slow render spends its time, and `formatProfile` prints it.

## Benchmarks

### In the browser (2026-11-23)

The live benchmark page (`bench/live`) renders a warehouse ledger with @axonpdf/react 0.0.1 and
react-pdf 4.9.0, both in a Web Worker, from the same components. The ledger is laid out the way
reports usually are: a summary page, then one `<Page>` per day of shipments. Measured in Chrome
for Testing 156 on an Apple M4 (24 GB RAM).

| Pages | axonpdf | react-pdf 4.9.0 |
| --- | --- | --- |
| about 100 | 0.9 s, 382 KB, 11.6 MB worker heap | 2.8 s, 563 KB, 193 MB worker heap |
| about 500 | 4.1 s, 1.8 MB, 56 MB worker heap | 45.8 s, 2.7 MB, 2.2 GB worker heap |
| about 2000 | 15.9 s, 7.1 MB, 142 MB worker heap | no PDF after 10 min |

Each cell is the time to the finished PDF, the file size and the worker's peak JS heap. The
longest main-thread freeze was 19 ms or less in every run, since both engines ran in a worker.
With the whole ledger in one flowing `<Page>`, react-pdf took 32.8 s at 100 pages and gave no PDF
after 10 minutes at 500, while axonpdf took 0.86 s and 3.9 s. Every run is in
[bench/RESULTS.md](bench/RESULTS.md).

### In Node (2026-11-21)

One A4 report, rendered at 100, 1000 and 5000 pages: a header with a logo, three paragraphs, and a
bordered five-column table that runs across pages, with "Page n of N" in the footer. axonpdf and
react-pdf 4.9.0 render the same JSX file. The pdfkit 0.21.0 version draws the same pages by hand.
Measured on an Apple M4 (24 GB RAM, macOS 27.2.1, Node 24.11.0) with a development build of axonpdf
at commit `ecb0699`, React in production mode, one Node process per run.

| Engine | 100 pages | 1000 pages | 5000 pages |
| --- | --- | --- | --- |
| axonpdf, rows as an array | 0.8 s, 117 MB | 7.8 s, 154 MB | 39.2 s, 471 MB |
| axonpdf, rows from a generator | 0.9 s, 117 MB | 7.8 s, 130 MB | 39.2 s, 215 MB |
| axonpdf, rows in `<Rows>` | 0.5 s, 129 MB | 4.8 s, 147 MB | 24.1 s, 259 MB |
| react-pdf 4.9.0 | 36.1 s, 1.3 GB | stopped after 15 min | not run |
| pdfkit 0.21.0 | 0.1 s, 97 MB | 1.1 s, 175 MB | 5.3 s, 347 MB |

Each cell is the render time and the peak memory (RSS) of the whole process. pdfkit is faster
because it has no layout step: you place every line and every page break yourself, while the two
React engines lay the document out with flexbox and break the pages for you.

A document written the usual way needs no changes: axonpdf builds long lists of children, and each
page's content, only when it reaches them. How the rows are passed still matters at large sizes.
An array of 190,000 row elements stays alive for the whole render, which makes V8 keep the elements
each row creates in its old generation until a full garbage collection, so the array version peaks
at 471 MB. Passing the rows as a generator child avoids that and peaked at 215 MB, and react-pdf
renders that code unchanged.

`<Rows>` is axonpdf's own component for rows from an array or an async iterable, such as a database
cursor. It renders and writes them in batches, and it was the fastest of the three. A document that
uses it no longer renders in react-pdf.

Every run, with its date, commit, machine and the command that reruns it, is in
[bench/RESULTS.md](bench/RESULTS.md). To run them all again: `cd bench && npm ci && cd .. && node
bench/run.mjs` (it needs poppler and qpdf).

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

MIT
