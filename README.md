# axonpdf

A PDF engine written in Rust. Work in progress: nothing here is ready to use yet.

## Benchmarks

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
