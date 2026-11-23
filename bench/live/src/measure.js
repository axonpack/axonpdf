import * as axonpdf from '@axonpdf/react';
import { renderInWorker } from '@axonpdf/react/worker';
import * as reactPdf from '@react-pdf/renderer';
import { createElement } from 'react';
import { LedgerDocument } from './ledger.jsx';

export const ENGINES = { axonpdf: { label: 'axonpdf 0.0.1', pdf: axonpdf }, 'react-pdf': { label: 'react-pdf 4.9.0', pdf: reactPdf } };

const heap = () => performance.memory?.usedJSHeapSize;
// Both engines write each page as a plain `/Type /Page` object, so counting them gives the pages.
const countPages = (bytes) => (new TextDecoder('latin1').decode(bytes).match(/\/Type\s*\/Page\b(?!s)/g) ?? []).length;

const workers = {};
const newWorker = {
  axonpdf: () => new Worker(new URL('./worker.js', import.meta.url), { type: 'module' }),
  'react-pdf': () => new Worker(new URL('./react-pdf.worker.js', import.meta.url), { type: 'module' }),
};

// Page plus its workers. Needs cross-origin isolation (the COOP/COEP headers); null without it.
const canMeasure = () => globalThis.crossOriginIsolated && !!performance.measureUserAgentSpecificMemory;

/**
 * Renders the ledger once and measures it. The frame watcher runs on the main thread throughout.
 * `onTick` gets the numbers so far about once a second. The ledger data is built inside the
 * render, so the stages are only `rendering` and `reading the PDF`.
 */
export async function run({ engine, pages, useWorker, onTick = () => {} }) {
  let peak = heap() ?? 0;
  let gap = 0;
  let last = performance.now();
  let on = true;
  const frame = () => {
    const now = performance.now();
    gap = Math.max(gap, now - last);
    last = now;
    peak = Math.max(peak, heap() ?? 0);
    if (on) requestAnimationFrame(frame);
  };
  requestAnimationFrame(frame);
  let memory = null;
  let memoryNow = null;
  let stage = 'rendering';
  // ponytail: one sample about every second; the browser may answer late, so short renders can
  // get a single sample.
  const sampler = (async () => {
    while (on && canMeasure()) {
      const { bytes } = await performance.measureUserAgentSpecificMemory();
      memoryNow = bytes;
      memory = Math.max(memory ?? 0, bytes);
      await new Promise((r) => setTimeout(r, 1000));
    }
  })().catch(() => {}); // a browser can still refuse (SecurityError); then it's n/a
  await new Promise((r) => setTimeout(r, 100)); // let the watcher settle before starting

  const start = performance.now();
  const tick = () => onTick({ stage, ms: performance.now() - start, gap: Math.round(gap), memory, memoryNow });
  const ticker = setInterval(tick, 1000);
  tick();
  let blob;
  if (useWorker) {
    workers[engine] ??= newWorker[engine]();
    blob = await renderInWorker(workers[engine], { pages });
  } else {
    blob = await ENGINES[engine].pdf.pdf(createElement(LedgerDocument, { pdf: ENGINES[engine].pdf, pages })).toBlob();
  }
  const ms = performance.now() - start;
  frame();
  on = false;
  stage = 'reading the PDF';
  tick();
  await sampler;
  clearInterval(ticker);
  const bytes = new Uint8Array(await blob.arrayBuffer());
  return {
    engine,
    thread: useWorker ? 'worker' : 'main thread',
    asked: pages,
    pages: countPages(bytes),
    ms: Math.round(ms),
    size: blob.size,
    gap: Math.round(gap),
    heap: heap() === undefined ? null : peak,
    memory,
    url: URL.createObjectURL(blob),
  };
}
