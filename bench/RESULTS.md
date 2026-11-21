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

## 2026-11-21, real-world templates (baseline, before the memory work, not for publishing)

- Commit `bfe8fce`, WASM 2648396 bytes built at that commit. **Baseline, before the memory work, not for publishing.**
- Machine: Apple M4, 10 cores, 24 GB RAM, macOS 27.2.1, Node v24.11.0
- Engines: axonpdf 0.0.0 (dev build) and react-pdf 4.9.0, rendering the same template code (react-pdf through its own import, axonpdf through an alias of that import). pdfkit is not run here: these templates are flexbox layouts of several hundred lines each, with no pdfkit version to compare.
- The templates and their data generator are private and not in this repo; the data is generated (fake residents, medications and administrations over 28 days, or fake staff). Scale is residents for the eMAR reports and staff for the rota.
- Time is the render call alone; peak RSS is the whole process, including about 100 MB of generated data.

| Document | Engine | Scale | Pages | Time | Peak RSS | Output | qpdf |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Real-world care-home medication report (eMAR), one resident per section | axonpdf | 9 | 56 | 18.7 s | 1327 MB | 1.0 MB | ok |
| Real-world care-home medication report (eMAR), one resident per section | react-pdf | 9 | 46 | 23.4 s | 1773 MB | 1.6 MB | ok |
| Real-world care-home medication report (eMAR), one medication across residents | axonpdf | 7 | 51 | 15.9 s | 1290 MB | 881 KB | ok |
| Real-world care-home medication report (eMAR), one medication across residents | react-pdf | 7 | 51 | 17.8 s | 1667 MB | 1.4 MB | ok |
| Weekly staff rota (one long page of staff rows) | axonpdf | 1000 | 48 | 1.5 s | 350 MB | 888 KB | ok |
| Weekly staff rota (one long page of staff rows) | react-pdf | 1000 | 48 | 32.7 s | 1596 MB | 681 KB | ok |
| Real-world care-home medication report (eMAR), one resident per section | axonpdf | 36 | 224 | 1 min 22 s | 1681 MB | 4.1 MB | ok |
| Real-world care-home medication report (eMAR), one resident per section | react-pdf | 36 | 181 | 2 min 3 s | 3761 MB | 6.3 MB | ok |
| Real-world care-home medication report (eMAR), one medication across residents | axonpdf | 28 | 201 | 54.7 s | 1594 MB | 3.4 MB | ok |
| Real-world care-home medication report (eMAR), one medication across residents | react-pdf | 28 | 201 | 1 min 35 s | 3406 MB | 5.6 MB | ok |
| Weekly staff rota (one long page of staff rows) | axonpdf | 4000 | 191 | 5.1 s | 747 MB | 3.5 MB | ok |
| Weekly staff rota (one long page of staff rows) | react-pdf | 4000 | 191 | 7 min 54 s | 3030 MB | 2.6 MB | ok |
| Real-world care-home medication report (eMAR), one resident per section | axonpdf | 180 | 1119 | 6 min 41 s | 3159 MB | 20.6 MB | ok |
| Real-world care-home medication report (eMAR), one medication across residents | axonpdf | 140 | 997 | 4 min 43 s | 2607 MB | 16.7 MB | ok |
| Weekly staff rota (one long page of staff rows) | axonpdf | 20000 | 953 | 27.0 s | 2679 MB | 17.2 MB | ok |

Notes on this batch:
- The resident-per-section eMAR has more pages with axonpdf (56 against 46, 224 against 181) because axonpdf moves some unbreakable medication rows to the next page when they would fit. That is a known bug being fixed, so these rows don't compare like for like.
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
