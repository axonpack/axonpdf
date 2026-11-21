// One benchmark render in this process; prints one JSON line with pages, time, peak RSS and bytes.
// run.mjs starts one of these per run, so the peak RSS belongs to that render alone.
//
//   node bench/render-one.mjs <axonpdf|axonpdf-rows|react-pdf|pdfkit> <pages> <out.pdf>
//
// axonpdf-rows is the same report with its table rows in axonpdf's <Rows> (report-rows.jsx).
//
// <pages> is the target page count; the row count comes from layout.mjs rowsFor.
import { build } from 'esbuild';
import { execFileSync } from 'node:child_process';
import { mkdirSync, statSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import { LOGO, rowsFor } from './layout.mjs';

const HERE = import.meta.dirname;
const ROOT = resolve(HERE, '..');
const [engine, pages, outArg] = process.argv.slice(2);
if (!['axonpdf', 'axonpdf-rows', 'react-pdf', 'pdfkit'].includes(engine) || !(+pages > 0) || !outArg) {
  throw new Error('usage: render-one.mjs <axonpdf|axonpdf-rows|react-pdf|pdfkit> <pages> <out.pdf>');
}
const rows = rowsFor(+pages);
const out = resolve(outArg);
mkdirSync(dirname(out), { recursive: true });

let render;
if (engine === 'pdfkit') {
  const { renderPdfkit } = await import('./report-pdfkit.mjs');
  render = () => renderPdfkit(rows, out);
} else {
  // The bundle sits where its imports should resolve: the repo root for axonpdf (the workspace
  // package), bench/ for react-pdf, with `@axonpdf/react` aliased to `@react-pdf/renderer`.
  const dir = engine === 'react-pdf' ? HERE : ROOT;
  const bundle = resolve(dir, 'node_modules/.cache/axonpdf-bench', `${engine}.mjs`);
  await build({
    stdin: {
      contents: `export { default as Report } from './${engine === 'axonpdf-rows' ? 'report-rows' : 'report'}.jsx';
        export { renderToFile } from '@axonpdf/react';
        export { createElement } from 'react';`,
      resolveDir: HERE,
      loader: 'jsx',
    },
    outfile: bundle,
    bundle: true,
    packages: 'external',
    alias: engine === 'react-pdf' ? { '@axonpdf/react': '@react-pdf/renderer' } : {},
    platform: 'node',
    format: 'esm',
    jsx: 'automatic',
    logLevel: 'error',
  });
  const { Report, renderToFile, createElement } = await import(pathToFileURL(bundle).href);
  render = () => renderToFile(createElement(Report, { rows, logo: LOGO }), out);
}

const start = performance.now();
await render();
const ms = Math.round(performance.now() - start);
const count = +execFileSync('pdfinfo', [out], { encoding: 'utf8' }).match(/^Pages:\s+(\d+)/m)[1];
console.log(JSON.stringify({ engine, rows, pages: count, ms, maxRssMb: Math.round(process.resourceUsage().maxRSS / 1024), bytes: statSync(out).size }));
