// Lays out every case in crates/engine/tests/fixtures/yoga.json with yoga-layout 3.2.1, Meta's own
// build of the Yoga release in vendor/yoga, and writes the boxes back as each case's `expected`.
import { readFileSync, writeFileSync } from 'node:fs';
import { loadYoga } from 'yoga-layout/load';

const FILE = new URL('../crates/engine/tests/fixtures/yoga.json', import.meta.url);

// Same order as YGEnums.h, so a name's index is Yoga's enum value.
const ENUMS = {
  flex_direction: ['column', 'column-reverse', 'row', 'row-reverse'],
  justify_content: ['flex-start', 'center', 'flex-end', 'space-between', 'space-around', 'space-evenly'],
  align_items: ['auto', 'flex-start', 'center', 'flex-end', 'stretch', 'baseline', 'space-between', 'space-around', 'space-evenly'],
  position_type: ['static', 'relative', 'absolute'],
  flex_wrap: ['no-wrap', 'wrap', 'wrap-reverse'],
  overflow: ['visible', 'hidden', 'scroll'],
  display: ['flex', 'none', 'contents'],
};
ENUMS.align_self = ENUMS.align_content = ENUMS.align_items;
const EDGES = ['left', 'top', 'right', 'bottom', 'start', 'end', 'horizontal', 'vertical', 'all'];
const GUTTERS = ['column', 'row', 'all'];
const camel = (key) => key.replace(/(^|_)(.)/g, (_, __, c) => c.toUpperCase());

// Calls the explicit point, percent or auto setter, as the Rust bindings do.
function setLength(node, method, args, value) {
  if (value === 'auto') return node[`set${method}Auto`](...args);
  if (typeof value === 'string') return node[`set${method}Percent`](...args, parseFloat(value));
  return node[`set${method}`](...args, value);
}

function apply(node, key, value) {
  const method = camel(key);
  if (ENUMS[key]) return node[`set${method}`](ENUMS[key].indexOf(value));
  if (key === 'gap') {
    for (const [g, v] of Object.entries(value)) setLength(node, 'Gap', [GUTTERS.indexOf(g)], v);
    return;
  }
  if (['margin', 'padding', 'border', 'position'].includes(key)) {
    for (const [e, v] of Object.entries(value)) setLength(node, method, [EDGES.indexOf(e)], v);
    return;
  }
  setLength(node, method, [], value);
}

const Yoga = await loadYoga();

function layOut(c) {
  const config = Yoga.Config.create();
  config.setPointScaleFactor(0); // no pixel-grid rounding, as the engine sets it
  const calls = []; // every measure call's [width, width mode, height, height mode], in order
  const all = [];
  const build = (spec) => {
    const node = Yoga.Node.createWithConfig(config);
    all.push(node);
    for (const [k, v] of Object.entries(spec.style ?? {})) apply(node, k, v);
    if (spec.measure) {
      const [mw, mh] = spec.measure;
      const fit = (offer, mode, own) => (mode === 1 ? offer : mode === 2 ? Math.min(offer, own) : own);
      node.setMeasureFunc((w, wm, h, hm) => {
        calls.push([w, wm, h, hm]);
        return { width: fit(w, wm, mw), height: fit(h, hm, mh) };
      });
    }
    (spec.children ?? []).forEach((child, i) => node.insertChild(build(child), i));
    return node;
  };
  const root = build(c.root);
  const [w, h] = c.available ?? [null, null];
  root.calculateLayout(w ?? undefined, h ?? undefined, Yoga.DIRECTION_LTR);
  const boxes = all.map((n) => {
    const l = n.getComputedLayout();
    return [l.left, l.top, l.width, l.height];
  });
  c.expected = { boxes, measure_calls: calls };
  root.freeRecursive();
  config.free();
}

const doc = JSON.parse(readFileSync(FILE, 'utf8'));
doc.cases.forEach(layOut);
writeFileSync(FILE, JSON.stringify(doc, null, 2) + '\n');

// Random trees exercise the bindings in combination. The seed is fixed, so a failing case can be
// regenerated and read.
const SEED = 1;
const COUNT = 300;
let state = SEED;
const rand = () => {
  // mulberry32
  state = (state + 0x6d2b79f5) | 0;
  let t = Math.imul(state ^ (state >>> 15), 1 | state);
  t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
  return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
};
const int = (lo, hi) => lo + Math.floor(rand() * (hi - lo + 1));
const pickOne = (xs) => xs[int(0, xs.length - 1)];
const pt = (lo, hi) => int(lo, hi);
const pct = () => `${int(0, 100)}%`;
const lenValue = () => (rand() < 0.7 ? pt(0, 120) : pct());
const dimValue = () => { const r = rand(); return r < 0.6 ? pt(0, 200) : r < 0.85 ? pct() : 'auto'; };
const edgesOf = (value) => Object.fromEntries(Array.from({ length: int(1, 2) }, () => [pickOne(EDGES), value()]));
const STYLE = {
  ...Object.fromEntries(Object.entries(ENUMS).map(([k, names]) => [k, () => pickOne(names)])),
  flex_grow: () => int(0, 30) / 10,
  flex_shrink: () => int(0, 30) / 10,
  aspect_ratio: () => int(5, 30) / 10,
  flex_basis: dimValue,
  width: dimValue,
  height: dimValue,
  min_width: lenValue,
  min_height: lenValue,
  max_width: lenValue,
  max_height: lenValue,
  margin: () => edgesOf(() => (rand() < 0.15 ? -pt(1, 30) : dimValue())),
  padding: () => edgesOf(lenValue),
  border: () => edgesOf(() => pt(0, 10)),
  position: () => edgesOf(dimValue),
  gap: () => Object.fromEntries([[pickOne(GUTTERS), lenValue()]]),
};
const KEYS = Object.keys(STYLE);
const randomNode = (depth) => {
  const style = Object.fromEntries(Array.from({ length: int(0, 5) }, () => { const k = pickOne(KEYS); return [k, STYLE[k]()]; }));
  const n = { style };
  const kids = depth < 3 ? int(0, 4) : 0;
  if (kids === 0 && rand() < 0.4) n.measure = [pt(5, 200), pt(5, 80)];
  if (kids) n.children = Array.from({ length: kids }, () => randomNode(depth + 1));
  return n;
};
const RANDOM = new URL('../crates/engine/tests/fixtures/yoga-random.json', import.meta.url);
const randomCases = Array.from({ length: COUNT }, (_, i) => {
  const root = randomNode(0);
  root.style.width = pt(50, 500);
  root.style.height = pt(50, 500);
  return { name: `random_${i}`, root };
});
randomCases.forEach(layOut);
writeFileSync(RANDOM, JSON.stringify({ yoga: '3.2.1', seed: SEED, how: 'Generated by parity/yoga.mjs; same format as yoga.json.', cases: randomCases }) + '\n');
console.log(`yoga-layout ${JSON.parse(readFileSync(new URL('node_modules/yoga-layout/package.json', import.meta.url))).version}: ${doc.cases.length} cases, ${COUNT} random cases written`);
