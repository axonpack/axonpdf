// Runs the benchmark report with axonpdf, react-pdf 4.9.0 and pdfkit, one process per run and one
// run at a time, and appends every result to bench/RESULTS.md.
//
//   node bench/run.mjs [--sizes 10,100,1000,5000] [--engines axonpdf,axonpdf-rows,react-pdf,pdfkit] [--label text]
//
// --label is added to the batch heading in RESULTS.md, for example "baseline, not for publishing".
//
// A run is stopped past 15 minutes or 8 GB RSS. An engine skips the bigger sizes once a run took
// over 5 minutes or was stopped. PDFs go to local/bench/ (gitignored). Needs poppler and qpdf.
import { execFileSync, spawn } from 'node:child_process';
import { appendFileSync, existsSync, readFileSync, statSync } from 'node:fs';
import { cpus, totalmem } from 'node:os';
import { resolve } from 'node:path';
import { parseArgs } from 'node:util';
import { freshBuild } from '../parity/fresh-build.mjs';

const HERE = import.meta.dirname;
const ROOT = resolve(HERE, '..');
const RESULTS = resolve(HERE, 'RESULTS.md');
const LIMIT_MS = 15 * 60e3;
const LIMIT_KB = 8 * 2 ** 20;
const SKIP_AFTER_MS = 5 * 60e3;

const { values: o } = parseArgs({
  options: { label: { type: 'string' }, sizes: { type: 'string', default: '10,100,1000,5000' }, engines: { type: 'string', default: 'axonpdf,axonpdf-rows,react-pdf,pdfkit' } },
});
const sizes = o.sizes.split(',').map(Number);
const engines = o.engines.split(',');

const sh = (cmd, args) => execFileSync(cmd, args, { encoding: 'utf8' }).trim();
const version = (path) => JSON.parse(readFileSync(path, 'utf8')).version;
const VERSIONS = {
  axonpdf: `${version(resolve(ROOT, 'packages/react/package.json'))} (dev build)`,
  'axonpdf-rows': `${version(resolve(ROOT, 'packages/react/package.json'))} (dev build, <Rows>)`,
  'react-pdf': version(resolve(HERE, 'node_modules/@react-pdf/renderer/package.json')),
  pdfkit: version(resolve(HERE, 'node_modules/pdfkit/package.json')),
};

const wasmStale = engines.some((e) => e.startsWith('axonpdf')) && freshBuild();
const dirty = sh('git', ['-C', ROOT, 'status', '--porcelain', '--', 'crates', 'packages']) ? ' with uncommitted changes in crates/ or packages/' : '';
const wasm = resolve(ROOT, 'packages/core/wasm/axonpdf.wasm');
const header = [
  `\n## ${new Date().toISOString().slice(0, 16).replace('T', ' ')} UTC, benchmark report${o.label ? ` (${o.label})` : ''}`,
  '',
  `- Commit: \`${sh('git', ['-C', ROOT, 'rev-parse', '--short', 'HEAD'])}\`${dirty}`,
  `- WASM: ${existsSync(wasm) ? `${statSync(wasm).size} bytes` : 'missing'}${wasmStale ? ', **older than crates/ (stale)**' : ''}`,
  `- Machine: ${sh('sysctl', ['-n', 'machdep.cpu.brand_string'])}, ${cpus().length} cores, ${Math.round(totalmem() / 2 ** 30)} GB RAM, macOS ${sh('sw_vers', ['-productVersion'])}, Node ${process.version}`,
  `- Engines: axonpdf ${VERSIONS.axonpdf}, react-pdf ${VERSIONS['react-pdf']}, pdfkit ${VERSIONS.pdfkit}`,
  '- React in production mode (`NODE_ENV=production`). Time is the render call alone (Node start-up and bundling excluded); peak RSS is the whole process.',
  '- Document: bench/report.jsx (axonpdf-rows: bench/report-rows.jsx, pdfkit: bench/report-pdfkit.mjs), A4, a header with a logo, three paragraphs, a 5-column bordered table, "Page n of N" footer',
  '',
  '| Engine | Target pages | Pages | Time | Peak RSS | Output | qpdf | Rerun |',
  '| --- | --- | --- | --- | --- | --- | --- | --- |',
].join('\n');
appendFileSync(RESULTS, header + '\n');

function runOne(engine, size) {
  const out = resolve(ROOT, 'local/bench', `report-${size}-${engine}.pdf`);
  const child = spawn(process.execPath, [resolve(HERE, 'render-one.mjs'), engine, String(size), out], {
    stdio: ['ignore', 'pipe', 'pipe'],
    env: { ...process.env, NODE_ENV: 'production' }, // React's production build for both React engines
  });
  let stdout = '';
  let stderr = '';
  child.stdout.on('data', (d) => (stdout += d));
  child.stderr.on('data', (d) => (stderr = (stderr + d).slice(-4000)));
  const start = Date.now();
  let stopped = null;
  const timer = setInterval(() => {
    try {
      if (+sh('ps', ['-o', 'rss=', '-p', String(child.pid)]) > LIMIT_KB) stopped = 'stopped at 8 GB';
    } catch {}
    if (Date.now() - start > LIMIT_MS) stopped = 'stopped at 15 min';
    if (stopped) child.kill('SIGKILL');
  }, 1000);
  return new Promise((done) =>
    child.on('exit', (code) => {
      clearInterval(timer);
      if (stopped) return done({ stopped, wallMs: Date.now() - start });
      if (code !== 0) return done({ stopped: `failed: ${stderr.trim().split('\n').find((l) => /Error/.test(l)) ?? stderr.slice(-200)}`, wallMs: Date.now() - start });
      let qpdf = 'ok';
      try {
        execFileSync('qpdf', ['--check', out], { stdio: 'ignore' });
      } catch (e) {
        qpdf = e.status === 3 ? 'warnings' : 'errors';
      }
      done({ ...JSON.parse(stdout.trim().split('\n').at(-1)), qpdf, wallMs: Date.now() - start });
    }),
  );
}

const bytes = (b) => (b < 2 ** 20 ? `${Math.round(b / 1024)} KB` : `${(b / 2 ** 20).toFixed(1)} MB`);
const fmt = (ms) => (ms < 1000 ? `${ms} ms` : ms < 60e3 ? `${(ms / 1000).toFixed(1)} s` : `${Math.floor(ms / 60e3)} min ${Math.round((ms % 60e3) / 1000)} s`);
for (const engine of engines) {
  let skip = null;
  for (const size of sizes) {
    const rerun = `\`node bench/run.mjs --engines ${engine} --sizes ${size}\``;
    if (skip) {
      appendFileSync(RESULTS, `| ${engine} | ${size} | skipped (${skip}) | | | | | ${rerun} |\n`);
      continue;
    }
    const r = await runOne(engine, size);
    const line = r.stopped
      ? `| ${engine} | ${size} | ${r.stopped} | ${fmt(r.wallMs)} | | | | ${rerun} |`
      : `| ${engine} | ${size} | ${r.pages} | ${fmt(r.ms)} | ${r.maxRssMb} MB | ${bytes(r.bytes)} | ${r.qpdf} | ${rerun} |`;
    console.log(line);
    appendFileSync(RESULTS, line + '\n');
    if (r.stopped) skip = 'the size before was stopped';
    else if (r.ms > SKIP_AFTER_MS) skip = 'the size before took over 5 min';
  }
}
