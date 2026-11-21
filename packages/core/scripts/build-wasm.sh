#!/usr/bin/env bash
# Builds crates/ffi into packages/core/wasm/axonpdf.{mjs,wasm}: one WebAssembly module plus an ES
# module loader that runs in Node, browsers and Workers. Needs emsdk on PATH (see the README).
set -euo pipefail

EMSCRIPTEN_VERSION="7.0.0"
root="$(cd "$(dirname "$0")/../../.." && pwd)"
out="$root/packages/core/wasm"

if ! command -v emcc >/dev/null; then
  echo "emcc not found: run 'source <emsdk>/emsdk_env.sh' first (README, Development setup)" >&2
  exit 1
fi
if ! emcc --version | head -1 | grep -q " $EMSCRIPTEN_VERSION "; then
  echo "Emscripten $EMSCRIPTEN_VERSION is required, found: $(emcc --version | head -1)" >&2
  exit 1
fi

cargo build --manifest-path "$root/Cargo.toml" -p axonpdf-ffi --release --target wasm32-unknown-emscripten

exports=_axonpdf_new,_axonpdf_free,_axonpdf_alloc,_axonpdf_dealloc,_axonpdf_add_font,_axonpdf_add_image
exports+=,_axonpdf_set_render_callback,_axonpdf_render,_axonpdf_last_error

mkdir -p "$out"
em++ "$root/target/wasm32-unknown-emscripten/release/libaxonpdf_ffi.a" \
  -O3 -fwasm-exceptions \
  -o "$out/axonpdf.mjs" \
  -sMODULARIZE -sEXPORT_ES6 -sEXPORT_NAME=createModule \
  -sENVIRONMENT=web,worker,node \
  -sALLOW_MEMORY_GROWTH -sMAXIMUM_MEMORY=4GB \
  -sALLOW_TABLE_GROWTH \
  -sEXPORTED_FUNCTIONS="$exports" \
  -sEXPORTED_RUNTIME_METHODS=addFunction,UTF8ToString,HEAPU8

raw=$(wc -c <"$out/axonpdf.wasm" | tr -d ' ')
gz=$(gzip -9c "$out/axonpdf.wasm" | wc -c | tr -d ' ')
echo "axonpdf.wasm: $raw bytes, $gz gzipped"
