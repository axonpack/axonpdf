// Renders each fixture with our engine and with a reference engine, then reports per fixture the
// page counts, the pixel difference of every page, `qpdf --check`, render time and peak memory.
//
//   node parity/compare.mjs [fixture...] [--reference <dir>] [--dpi 72] [--threshold 1]
//
// --reference is an engine folder as render.mjs takes it (default local/oracle). Output goes to
// local/diff/<fixture>/: both PDFs, and per page a PNG with the reference, ours and the differing
// pixels in red, side by side. A fixture passes when the page counts match, every page differs by
// at most --threshold percent of its pixels, and qpdf finds no errors in our PDF.
// Needs poppler (pdftoppm) and qpdf on the PATH.
import { spawnSync } from 'node:child_process';
import { existsSync, mkdirSync, readdirSync, rmSync, writeFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { parseArgs } from 'node:util';
import { diffPages, encodePng, rasterise } from './raster.mjs';

const ROOT = resolve(import.meta.dirname, '..');
const { values: opts, positionals } = parseArgs({
  allowPositionals: true,
  options: {
    reference: { type: 'string', default: join(ROOT, 'local/oracle') },
    dpi: { type: 'string', default: '72' },
    threshold: { type: 'string', default: '1' },
  },
});
// Fixtures of axonpdf's own additions: the reference cannot render them, so they are looked at.
const BY_EYE = new Set(['extras']);
const fixtures = positionals.length
  ? positionals
  : readdirSync(join(ROOT, 'fixtures')).filter((f) => existsSync(join(ROOT, 'fixtures', f, 'document.tsx')));

const warnings = new Set();
function render(name, engine, pdf) {
  const run = spawnSync(process.execPath, [join(import.meta.dirname, 'render.mjs'), name, pdf, '--engine', engine], { encoding: 'utf8' });
  for (const line of run.stderr.match(/^warning: .*$/gm) ?? []) warnings.add(line);
  if (run.status !== 0) return { error: (run.stderr.match(/^\w*Error.*$/m) ?? [run.stderr.trim().split('\n').at(-1)])[0] };
  return JSON.parse(run.stdout.trim().split('\n').at(-1));
}

function qpdf(pdf) {
  const { status } = spawnSync('qpdf', ['--check', pdf]);
  return status === 0 ? 'ok' : status === 3 ? 'warnings' : 'errors';
}

const mb = (r) => (r.maxRssKb ? `${Math.round(r.maxRssKb / 1024)} MB` : '-');
let failed = 0;

for (const name of fixtures) {
  const dir = join(ROOT, 'local/diff', name);
  rmSync(dir, { recursive: true, force: true });
  mkdirSync(dir, { recursive: true });
  const byEye = BY_EYE.has(name);
  const ours = render(name, ROOT, join(dir, 'ours.pdf'));
  const ref = byEye ? { error: 'not renderable by the reference' } : render(name, resolve(opts.reference), join(dir, 'reference.pdf'));

  console.log(`\n${name}`);
  console.log(`  ours       ${ours.error ? `FAILED: ${ours.error}` : `${ours.ms} ms, peak ${mb(ours)}, qpdf ${qpdf(join(dir, 'ours.pdf'))}`}`);
  console.log(`  reference  ${ref.error ? `${byEye ? '' : 'FAILED: '}${ref.error}` : `${ref.ms} ms, peak ${mb(ref)}, qpdf ${qpdf(join(dir, 'reference.pdf'))}`}`);
  if (ours.error) {
    failed++;
    continue;
  }

  const ourPages = rasterise(join(dir, 'ours.pdf'), +opts.dpi);
  if (ref.error) {
    ourPages.forEach((p, i) => writeFileSync(join(dir, `page-${i + 1}.png`), encodePng(p.width, p.height, p.rgb)));
    console.log(`  pages      ${ourPages.length}, written for review`);
    if (!byEye) failed++;
    continue;
  }

  const refPages = rasterise(join(dir, 'reference.pdf'), +opts.dpi);
  const blank = { width: 1, height: 1, rgb: Buffer.from([255, 255, 255]) };
  let pass = ourPages.length === refPages.length && qpdf(join(dir, 'ours.pdf')) !== 'errors';
  console.log(`  pages      ours ${ourPages.length}, reference ${refPages.length}`);
  for (let i = 0; i < Math.max(ourPages.length, refPages.length); i++) {
    const { percent, png } = diffPages(refPages[i] ?? blank, ourPages[i] ?? blank);
    writeFileSync(join(dir, `page-${i + 1}.png`), png);
    const ok = percent <= +opts.threshold;
    pass &&= ok;
    console.log(`  page ${String(i + 1).padEnd(5)} ${percent.toFixed(2)}% of pixels differ${ok ? '' : '  <- over threshold'}`);
  }
  console.log(`  result     ${pass ? 'pass' : 'FAIL'}`);
  if (!pass) failed++;
}

for (const w of warnings) console.log(`\n${w}`);
console.log(`\n${fixtures.length - failed} of ${fixtures.length} fixtures pass; diffs in local/diff/`);
process.exitCode = failed ? 1 : 0;
