// Renders one fixture to a PDF and prints {"ms", "maxRssKb"} as one JSON line.
//
//   node parity/render.mjs <fixture> <out.pdf> [--engine <dir>] [--layout <out.json>]
//
// --engine is a folder whose node_modules resolves `@axonpdf/react` and `react`; it defaults to
// the repo root, our own package through the workspace. Pointing it at a folder where
// `@axonpdf/react` is an npm alias for another engine renders the same fixture with that engine,
// unchanged, which is the drop-in check. Run each render in its own process: maxRssKb is the
// process's peak.
//
// --layout writes whatever the engine passes to the Document's onRender, as JSON.
//
// Fixtures are `fixtures/<name>/document.tsx`, default-exporting a component with no props that
// returns a <Document>. The working directory is `fixtures/` while it renders, so asset paths in
// fixtures read `assets/...`.
import { build } from 'esbuild';
import { execFileSync } from 'node:child_process';
import { existsSync, readdirSync, statSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { parseArgs } from 'node:util';
import { pathToFileURL } from 'node:url';

const ROOT = resolve(import.meta.dirname, '..');
const { values: opts, positionals: [name, out] } = parseArgs({
  allowPositionals: true,
  options: { engine: { type: 'string', default: ROOT }, layout: { type: 'string' } },
});
if (!name || !out) throw new Error('usage: render.mjs <fixture> <out.pdf> [--engine <dir>] [--layout <out.json>]');

// Our packages load from their build/ folders, so rebuild any whose src/ changed since. The WASM
// build is slow and left to `bun run build:wasm`; a stale one only gets a warning.
const times = (dir) => (existsSync(dir) ? readdirSync(dir, { recursive: true }).map((f) => statSync(resolve(dir, f)).mtimeMs) : []);
const stale = (src, out) => !times(out).length || Math.max(...times(src)) > Math.min(...times(out));
if (resolve(opts.engine) === ROOT) {
  for (const pkg of ['core', 'react']) {
    const dir = resolve(ROOT, 'packages', pkg);
    if (stale(resolve(dir, 'src'), resolve(dir, 'build'))) execFileSync('bun', ['run', 'build'], { cwd: dir, stdio: ['ignore', 'ignore', 'inherit'] });
  }
  if (Math.max(...['engine', 'ffi'].flatMap((c) => times(resolve(ROOT, 'crates', c, 'src')))) > Math.max(...times(resolve(ROOT, 'packages/core/wasm')))) {
    console.error('warning: crates/ changed after packages/core/wasm was built; run `bun run build:wasm` in packages/core');
  }
}

// The bundle goes inside the engine folder so Node resolves its imports from there. Its entry
// re-exports what this script needs, so those imports resolve from the engine folder too.
const engine = resolve(opts.engine);
const bundle = resolve(engine, 'node_modules/.cache/axonpdf-parity', `${name}.mjs`);
await build({
  stdin: {
    contents: `export { default as Doc } from './document.tsx';
      export { renderToFile } from '@axonpdf/react';
      export { createElement, cloneElement } from 'react';`,
    resolveDir: resolve(ROOT, 'fixtures', name),
    loader: 'tsx',
  },
  outfile: bundle,
  bundle: true,
  packages: 'external',
  platform: 'node',
  format: 'esm',
  jsx: 'automatic',
  logLevel: 'error',
});
const { Doc, renderToFile, createElement, cloneElement } = await import(pathToFileURL(bundle).href);

const outPath = resolve(out);
const layoutPath = opts.layout && resolve(opts.layout);
process.chdir(resolve(ROOT, 'fixtures'));

let doc = createElement(Doc);
if (layoutPath) {
  // Called as a function to reach its <Document>; fixtures use no hooks at the top level.
  doc = cloneElement(Doc(), { onRender: (data) => writeFileSync(layoutPath, toJson(data)) });
}

const start = performance.now();
await renderToFile(doc, outPath);
const ms = Math.round(performance.now() - start);
console.log(JSON.stringify({ ms, maxRssKb: process.resourceUsage().maxRSS }));

// Layout trees hold cycles, fonts and image bytes; keep the boxes, styles and text.
function toJson(data) {
  const seen = new WeakSet();
  return JSON.stringify(
    data,
    (key, value) => {
      if (key === 'parent' || key === 'font' || key === 'yogaNode') return undefined;
      if (ArrayBuffer.isView(value) || value?.type === 'Buffer') return `<${value.length ?? value.data?.length} bytes>`;
      if (typeof value === 'function') return undefined;
      if (value && typeof value === 'object') {
        if (seen.has(value)) return undefined;
        seen.add(value);
      }
      return value;
    },
    1,
  );
}
