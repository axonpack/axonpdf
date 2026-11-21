# Benchmark results

Every benchmark run, appended as it finishes. Rerun one with the command in its row; rerun a whole
batch with `node bench/run.mjs` from the repo root (it needs poppler and qpdf, and `bun run
build:wasm` in packages/core first if the crates changed). Each run is its own Node process, and runs
go one at a time.
