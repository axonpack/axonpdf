# Benchmark results

Every benchmark run, appended as it finishes. Rerun one with the command in its row; rerun a whole
batch with `node bench/run.mjs` from the repo root (it needs poppler and qpdf, and `bun run
build:wasm` in packages/core first if the crates changed). Each run is its own Node process, and runs
go one at a time.

## 2026-11-21 13:05 UTC, benchmark report (baseline, before the memory work, not for publishing)

- Commit: `c7d98b8` with uncommitted changes in crates/ or packages/
- WASM: 2845241 bytes, **older than crates/ (stale)**
- Machine: Apple M4, 10 cores, 24 GB RAM, macOS 27.2.1, Node v24.11.0
- Engines: axonpdf 0.0.0 (dev build), react-pdf 4.9.0, pdfkit 0.21.0
- Time is the render call alone (Node start-up and bundling excluded); peak RSS is the whole process.
- Document: bench/report.jsx (pdfkit: bench/report-pdfkit.mjs), A4, a header with a logo, three paragraphs, a 5-column bordered table, "Page n of N" footer

| Engine | Target pages | Pages | Time | Peak RSS | Output | qpdf | Rerun |
| --- | --- | --- | --- | --- | --- | --- | --- |
| axonpdf | 10 | 10 | 169 ms | 130 MB | 48 KB | ok | `node bench/run.mjs --engines axonpdf --sizes 10` |
| axonpdf | 100 | 100 | 1.2 s | 259 MB | 442 KB | ok | `node bench/run.mjs --engines axonpdf --sizes 100` |
| react-pdf | 10 | 10 | 815 ms | 311 MB | 68 KB | ok | `node bench/run.mjs --engines react-pdf --sizes 10` |
| react-pdf | 100 | 100 | 42.0 s | 1420 MB | 643 KB | ok | `node bench/run.mjs --engines react-pdf --sizes 100` |
| pdfkit | 10 | 10 | 29 ms | 89 MB | 39 KB | ok | `node bench/run.mjs --engines pdfkit --sizes 10` |
| pdfkit | 100 | 100 | 152 ms | 98 MB | 357 KB | ok | `node bench/run.mjs --engines pdfkit --sizes 100` |

## 2026-11-21, real-world templates (baseline before the memory work, with the axonpdf eMAR rows at the four smaller scales rerun after it)

- Commit `bfe8fce`, WASM 2648396 bytes built at that commit, React in development mode: every react-pdf row, the rota rows, and the axonpdf eMAR rows at scales 180 and 140. **Baseline, before the memory work, not for publishing.**
- Commit `1e6a592` (engine at `62ed1ce`), WASM 2870555 bytes, React in production mode (`NODE_ENV=production`), rerun on 2026-11-21: the axonpdf eMAR rows at scales 9, 7, 36 and 28.
- Machine: Apple M4, 10 cores, 24 GB RAM, macOS 27.2.1, Node v24.11.0
- Engines: axonpdf 0.0.0 (dev build) and react-pdf 4.9.0, rendering the same template code (react-pdf through its own import, axonpdf through an alias of that import). pdfkit is not run here: these templates are flexbox layouts of several hundred lines each, with no pdfkit version to compare.
- The templates and their data generator are private and not in this repo; the data is generated (fake residents, medications and administrations over 28 days, or fake staff). Scale is residents for the eMAR reports and staff for the rota.
- Time is the render call alone; peak RSS is the whole process, including about 100 MB of generated data.

| Document | Engine | Scale | Pages | Time | Peak RSS | Output | qpdf |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Real-world care-home medication report (eMAR), one resident per section | axonpdf | 9 | 46 | 18.3 s | 764 MB | 927 KB | ok |
| Real-world care-home medication report (eMAR), one resident per section | react-pdf | 9 | 46 | 23.4 s | 1773 MB | 1.6 MB | ok |
| Real-world care-home medication report (eMAR), one medication across residents | axonpdf | 7 | 51 | 13.9 s | 725 MB | 881 KB | ok |
| Real-world care-home medication report (eMAR), one medication across residents | react-pdf | 7 | 51 | 17.8 s | 1667 MB | 1.4 MB | ok |
| Weekly staff rota (one long page of staff rows) | axonpdf | 1000 | 48 | 1.5 s | 350 MB | 888 KB | ok |
| Weekly staff rota (one long page of staff rows) | react-pdf | 1000 | 48 | 32.7 s | 1596 MB | 681 KB | ok |
| Real-world care-home medication report (eMAR), one resident per section | axonpdf | 36 | 181 | 1 min 15 s | 1581 MB | 3.6 MB | ok |
| Real-world care-home medication report (eMAR), one resident per section | react-pdf | 36 | 181 | 2 min 3 s | 3761 MB | 6.3 MB | ok |
| Real-world care-home medication report (eMAR), one medication across residents | axonpdf | 28 | 201 | 55.2 s | 1456 MB | 3.4 MB | ok |
| Real-world care-home medication report (eMAR), one medication across residents | react-pdf | 28 | 201 | 1 min 35 s | 3406 MB | 5.6 MB | ok |
| Weekly staff rota (one long page of staff rows) | axonpdf | 4000 | 191 | 5.1 s | 747 MB | 3.5 MB | ok |
| Weekly staff rota (one long page of staff rows) | react-pdf | 4000 | 191 | 7 min 54 s | 3030 MB | 2.6 MB | ok |
| Real-world care-home medication report (eMAR), one resident per section | axonpdf | 180 | 1119 | 6 min 41 s | 3159 MB | 20.6 MB | ok |
| Real-world care-home medication report (eMAR), one medication across residents | axonpdf | 140 | 997 | 4 min 43 s | 2607 MB | 16.7 MB | ok |
| Weekly staff rota (one long page of staff rows) | axonpdf | 20000 | 953 | 27.0 s | 2679 MB | 17.2 MB | ok |

Notes on this batch:
- Page counts: axonpdf now gives the same page count as react-pdf on every eMAR row that was rerun (46, 51, 181 and 201). The scale-180 resident row (1119 pages) is from the baseline, before the page-break fix.
- react-pdf was not run at about 1000 pages: at about 200 pages it took over 2 minutes on the resident eMAR and the rota, and the medication eMAR run was held back to keep the machine free for profiling.
- The 3000-page axonpdf runs were stopped before they started, to be redone after the memory work.

## 2026-11-21 17:24 UTC, benchmark report (final, engine at 62ed1ce)

- Commit: `d1db145`
- WASM: 2870555 bytes
- Machine: Apple M4, 10 cores, 24 GB RAM, macOS 27.2.1, Node v24.11.0
- Engines: axonpdf 0.0.0 (dev build), react-pdf 4.9.0, pdfkit 0.21.0
- React in production mode (`NODE_ENV=production`). Time is the render call alone (Node start-up and bundling excluded); peak RSS is the whole process.
- Document: bench/report.jsx (axonpdf-rows: bench/report-rows.jsx, pdfkit: bench/report-pdfkit.mjs), A4, a header with a logo, three paragraphs, a 5-column bordered table, "Page n of N" footer

| Engine | Target pages | Pages | Time | Peak RSS | Output | qpdf | Rerun |
| --- | --- | --- | --- | --- | --- | --- | --- |
| axonpdf | 100 | 100 | 924 ms | 195 MB | 442 KB | ok | `node bench/run.mjs --engines axonpdf --sizes 100` |
| axonpdf | 1000 | 1000 | 8.2 s | 735 MB | 4.3 MB | ok | `node bench/run.mjs --engines axonpdf --sizes 1000` |
| axonpdf | 5000 | 5000 | 40.4 s | 2489 MB | 21.5 MB | ok | `node bench/run.mjs --engines axonpdf --sizes 5000` |
| axonpdf-rows | 100 | 100 | 529 ms | 129 MB | 447 KB | ok | `node bench/run.mjs --engines axonpdf-rows --sizes 100` |
| axonpdf-rows | 1000 | 1000 | 4.7 s | 152 MB | 4.3 MB | ok | `node bench/run.mjs --engines axonpdf-rows --sizes 1000` |
| axonpdf-rows | 5000 | 5000 | 23.0 s | 224 MB | 21.7 MB | ok | `node bench/run.mjs --engines axonpdf-rows --sizes 5000` |
| react-pdf | 100 | 100 | 36.3 s | 1359 MB | 643 KB | ok | `node bench/run.mjs --engines react-pdf --sizes 100` |
| react-pdf | 1000 | did not finish in 15 min | 15 min 1 s | | | | `node bench/run.mjs --engines react-pdf --sizes 1000` |
| react-pdf | 5000 | skipped (the size before was stopped) | | | | | `node bench/run.mjs --engines react-pdf --sizes 5000` |
| pdfkit | 100 | 100 | 142 ms | 97 MB | 357 KB | ok | `node bench/run.mjs --engines pdfkit --sizes 100` |
| pdfkit | 1000 | 1000 | 1.2 s | 175 MB | 3.5 MB | ok | `node bench/run.mjs --engines pdfkit --sizes 1000` |
| pdfkit | 5000 | 5000 | 6.0 s | 343 MB | 17.3 MB | ok | `node bench/run.mjs --engines pdfkit --sizes 5000` |

Notes on this batch:
- `d1db145` is 62ed1ce plus the benchmark's own changes; the engine is as of 62ed1ce, with the WASM built there.
- react-pdf 4.9.0 at 1000 pages was stopped at the 15-minute limit; its RSS was 3.5 to 3.7 GB while it ran (sampled with `ps`). The 5000-page size was not run after that.
- Output check: at 100 pages, axonpdf's first and last pages differ from react-pdf's by 0.13% and 0.12% of pixels (react-pdf draws no left border on the split table; axonpdf does). pdfkit's differ by 16% and 10%, because the hand-written version wraps the paragraphs differently and starts the table about 8pt lower. At 5000 pages the `<Rows>` output differs from the plain axonpdf output by 0.02% on the first and last pages.

## 2026-11-21 18:46 UTC, benchmark report (P7 one-shot renderer)

- Commit: `a19e38b`
- WASM: 2876611 bytes
- Machine: Apple M4, 10 cores, 24 GB RAM, macOS 27.2.1, Node v24.11.0
- Engines: axonpdf 0.0.0 (dev build), react-pdf 4.9.0, pdfkit 0.21.0
- React in production mode (`NODE_ENV=production`). Time is the render call alone (Node start-up and bundling excluded); peak RSS is the whole process.
- Document: bench/report.jsx (axonpdf-rows: bench/report-rows.jsx, pdfkit: bench/report-pdfkit.mjs), A4, a header with a logo, three paragraphs, a 5-column bordered table, "Page n of N" footer

| Engine | Target pages | Pages | Time | Peak RSS | Output | qpdf | Rerun |
| --- | --- | --- | --- | --- | --- | --- | --- |
| axonpdf | 1000 | 1000 | 7.7 s | 183 MB | 4.3 MB | ok | `node bench/run.mjs --engines axonpdf --sizes 1000` |
| axonpdf | 5000 | 5000 | 38.4 s | 594 MB | 21.5 MB | ok | `node bench/run.mjs --engines axonpdf --sizes 5000` |
| axonpdf-rows | 1000 | 1000 | 4.7 s | 150 MB | 4.3 MB | ok | `node bench/run.mjs --engines axonpdf-rows --sizes 1000` |
| axonpdf-rows | 5000 | 5000 | 23.3 s | 298 MB | 21.7 MB | ok | `node bench/run.mjs --engines axonpdf-rows --sizes 5000` |

## 2026-11-21 19:51 UTC, benchmark report (final after the memory work)

- Commit: `f843e84`
- WASM: 2879400 bytes
- Machine: Apple M4, 10 cores, 24 GB RAM, macOS 27.2.1, Node v24.11.0
- Engines: axonpdf 0.0.0 (dev build), react-pdf 4.9.0, pdfkit 0.21.0
- React in production mode (`NODE_ENV=production`). Time is the render call alone (Node start-up and bundling excluded); peak RSS is the whole process.
- Document: bench/report.jsx (axonpdf-generator: bench/report-generator.jsx, axonpdf-rows: bench/report-rows.jsx, pdfkit: bench/report-pdfkit.mjs), A4, a header with a logo, three paragraphs, a 5-column bordered table, "Page n of N" footer

| Engine | Target pages | Pages | Time | Peak RSS | Output | qpdf | Rerun |
| --- | --- | --- | --- | --- | --- | --- | --- |
| axonpdf | 100 | 100 | 817 ms | 117 MB | 442 KB | ok | `node bench/run.mjs --engines axonpdf --sizes 100` |
| axonpdf | 1000 | 1000 | 7.8 s | 154 MB | 4.3 MB | ok | `node bench/run.mjs --engines axonpdf --sizes 1000` |
| axonpdf | 5000 | 5000 | 39.2 s | 471 MB | 21.5 MB | ok | `node bench/run.mjs --engines axonpdf --sizes 5000` |
| axonpdf-generator | 100 | 100 | 863 ms | 117 MB | 442 KB | ok | `node bench/run.mjs --engines axonpdf-generator --sizes 100` |
| axonpdf-generator | 1000 | 1000 | 7.8 s | 130 MB | 4.3 MB | ok | `node bench/run.mjs --engines axonpdf-generator --sizes 1000` |
| axonpdf-generator | 5000 | 5000 | 39.2 s | 215 MB | 21.5 MB | ok | `node bench/run.mjs --engines axonpdf-generator --sizes 5000` |
| axonpdf-rows | 100 | 100 | 527 ms | 129 MB | 447 KB | ok | `node bench/run.mjs --engines axonpdf-rows --sizes 100` |
| axonpdf-rows | 1000 | 1000 | 4.8 s | 147 MB | 4.3 MB | ok | `node bench/run.mjs --engines axonpdf-rows --sizes 1000` |
| axonpdf-rows | 5000 | 5000 | 24.1 s | 259 MB | 21.7 MB | ok | `node bench/run.mjs --engines axonpdf-rows --sizes 5000` |
| pdfkit | 100 | 100 | 145 ms | 97 MB | 357 KB | ok | `node bench/run.mjs --engines pdfkit --sizes 100` |
| pdfkit | 1000 | 1000 | 1.1 s | 175 MB | 3.5 MB | ok | `node bench/run.mjs --engines pdfkit --sizes 1000` |
| pdfkit | 5000 | 5000 | 5.3 s | 347 MB | 17.3 MB | ok | `node bench/run.mjs --engines pdfkit --sizes 5000` |

## 2026-11-21 19:54 UTC, benchmark report (final after the memory work, react-pdf at 100 pages only)

- Commit: `f843e84`
- WASM: 2879400 bytes
- Machine: Apple M4, 10 cores, 24 GB RAM, macOS 27.2.1, Node v24.11.0
- Engines: axonpdf 0.0.0 (dev build), react-pdf 4.9.0, pdfkit 0.21.0
- React in production mode (`NODE_ENV=production`). Time is the render call alone (Node start-up and bundling excluded); peak RSS is the whole process.
- Document: bench/report.jsx (axonpdf-generator: bench/report-generator.jsx, axonpdf-rows: bench/report-rows.jsx, pdfkit: bench/report-pdfkit.mjs), A4, a header with a logo, three paragraphs, a 5-column bordered table, "Page n of N" footer

| Engine | Target pages | Pages | Time | Peak RSS | Output | qpdf | Rerun |
| --- | --- | --- | --- | --- | --- | --- | --- |
| react-pdf | 100 | 100 | 36.1 s | 1289 MB | 643 KB | ok | `node bench/run.mjs --engines react-pdf --sizes 100` |

Notes on the two batches above:
- `f843e84` is `ecb0699` plus the generator variant of the benchmark; the engine is as of `ecb0699`, with the WASM and the JS packages rebuilt there.
- react-pdf 4.9.0 was rerun at 100 pages only. Its larger sizes carry forward from the 2026-11-21 17:24 UTC batch (`d1db145`, same react-pdf version and machine): 1000 pages did not finish in 15 min (3.5 to 3.7 GB RSS while it ran), and 5000 pages was not run.
- axonpdf-generator is the same report with its rows passed as a generator child (`bench/report-generator.jsx`). react-pdf renders that file unchanged too.
- Why the generator uses less memory than the array at 5000 pages (215 MB against 471 MB): the array of 190,000 `<Row>` elements stays alive for the whole render, so V8 starts placing the elements that `Row` creates straight into its old generation, where they wait for a full garbage collection. With a generator, no such array exists.
- Output check at 100 pages: axonpdf's first and last pages differ from react-pdf's by 0.13% and 0.12% of pixels, pdfkit's by 16% and 10% (as before). The generator output matches the plain axonpdf output pixel for pixel; `<Rows>` differs from it by 0.02% and 0.03%. qpdf passes on every output.

## 2026-11-23 UTC, live benchmark page in the browser (bench/live)

- Commit: `33d9cbd` (the page as built from bench/live), engines from npm: @axonpdf/react 0.0.1 and react-pdf 4.9.0
- Machine: Apple M4, 10 cores, 24 GB RAM, macOS 27.2.1, Chrome for Testing 156.0.8078.4 (headless, through Playwright), with `--enable-precise-memory-info`
- Document: bench/live/src/ledger.jsx, A4, a fixed header with an SVG logo, a "Page n of N" footer, shipments of 4 to 12 lines each with wrapped notes; pages are counted in the output PDF
- Time is from the click to the finished PDF Blob, including building the data. Frame gap is the longest gap between main-thread animation frames during the render. JS heap is the page's main-thread peak from `performance.memory`; a worker's heap can't be read there (`performance.memory` is not defined in a worker), so the worker rows show only the page's heap.
- One run per cell, each in a fresh tab. A run that had no PDF after 10 minutes was stopped.
- Rerun: `cd bench/live && npm ci && npm run build && npm run preview`, open the printed URL in Chromium, choose custom pages, the engine and the worker box, then Render, one fresh tab per cell.

| Engine | Thread | Target pages | Pages | Time to PDF | File size | Longest frame gap | Page JS heap peak |
| --- | --- | --- | --- | --- | --- | --- | --- |
| axonpdf | worker | 100 | 98 | 0.90 s | 362 KB | 18 ms | 20 MB |
| axonpdf | main thread | 100 | 98 | 0.83 s | 362 KB | 832 ms | 42 MB |
| react-pdf | main thread | 100 | 98 | 33.7 s | 539 KB | 33.6 s | 160 MB |
| axonpdf | worker | 500 | 487 | 3.9 s | 1.8 MB | 17 ms | 20 MB |
| axonpdf | main thread | 500 | 487 | 3.8 s | 1.8 MB | 3.8 s | 66 MB |
| react-pdf | main thread | 500 | - | no PDF after 10 min (stopped at 601 s) | - | - | - |
| axonpdf | worker | 2000 | 1951 | 16.2 s | 7.1 MB | 19 ms | 21 MB |
| axonpdf | main thread | 2000 | 1951 | 15.1 s | 7.1 MB | 15.1 s | 174 MB |
| react-pdf | main thread | 2000 | - | no PDF after 10 min (stopped at 601 s) | - | - | - |
