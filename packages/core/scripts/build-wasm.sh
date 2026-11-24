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

# Size over speed for the web build only; native builds keep the workspace profile.
CARGO_PROFILE_RELEASE_LTO=true CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_PROFILE_RELEASE_OPT_LEVEL=s \
  cargo build --manifest-path "$root/Cargo.toml" -p axonpdf-ffi --release --target wasm32-unknown-emscripten

exports=_axonpdf_new,_axonpdf_free,_axonpdf_alloc,_axonpdf_dealloc,_axonpdf_add_font,_axonpdf_add_image
exports+=,_axonpdf_set_render_callback,_axonpdf_render,_axonpdf_last_error
exports+=,_axonpdf_set_paint_callback,_axonpdf_set_hyphenation_callback,_axonpdf_hyphenate_builtin
exports+=,_axonpdf_set_rows_callback,_axonpdf_total_pages_marker,_axonpdf_render_begin,_axonpdf_push_rows
exports+=,_axonpdf_profile,_axonpdf_profile_report,_axonpdf_page_width,_axonpdf_pages

# $1: the environments, $2: the loader to write; the .wasm lands beside it.
link() {
  em++ "$root/target/wasm32-unknown-emscripten/release/libaxonpdf_ffi.a" \
    -Oz -fwasm-exceptions \
    -o "$2" \
    -sMODULARIZE -sEXPORT_ES6 -sEXPORT_NAME=createModule \
    -sENVIRONMENT="$1" \
    -sALLOW_MEMORY_GROWTH -sMAXIMUM_MEMORY=4GB \
    -sALLOW_TABLE_GROWTH \
    -sEXPORTED_FUNCTIONS="$exports" \
    -sEXPORTED_RUNTIME_METHODS=addFunction,UTF8ToString,HEAPU8
}

mkdir -p "$out"
link web,worker,node "$out/axonpdf.mjs"
# A loader without the Node branch for bundlers that build for browsers (package.json
# `imports`), so they don't meet Node's built-in modules. It loads the same axonpdf.wasm.
web=$(mktemp -d)
link web,worker "$web/axonpdf.mjs"
if ! cmp -s "$web/axonpdf.wasm" "$out/axonpdf.wasm"; then
  echo "the browser loader's WebAssembly differs from the shared one" >&2
  exit 1
fi
mv "$web/axonpdf.mjs" "$out/axonpdf.web.mjs"
rm -rf "$web"

# Turbopack turns `new URL("axonpdf.wasm", import.meta.url).href` into a bare path such as
# /_next/static/media/axonpdf.<hash>.wasm. A worker started from a blob: URL can't fetch a bare path,
# so resolve it against the page's origin there. The `new URL(..., import.meta.url)` stays as it is,
# because that is the pattern bundlers look for to copy the .wasm.
for loader in "$out/axonpdf.mjs" "$out/axonpdf.web.mjs"; do
  find='return new URL("axonpdf.wasm",import.meta.url).href'
  if [ "$(grep -o "$find" "$loader" | wc -l | tr -d ' ')" != 1 ]; then
    echo "$loader: the wasm lookup Emscripten writes has changed; update build-wasm.sh" >&2
    exit 1
  fi
  base='globalThis.location&&(location.href.startsWith("blob:")?location.origin:location.href)'
  perl -pi -e "s/\Q$find\E/return new URL(new URL(\"axonpdf.wasm\",import.meta.url).href,$base).href/" "$loader"
done

raw=$(wc -c <"$out/axonpdf.wasm" | tr -d ' ')
gz=$(gzip -9c "$out/axonpdf.wasm" | wc -c | tr -d ' ')
echo "axonpdf.wasm: $raw bytes, $gz gzipped"
