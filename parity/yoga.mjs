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
const doc = JSON.parse(readFileSync(FILE, 'utf8'));

for (const c of doc.cases) {
  const config = Yoga.Config.create();
  config.setPointScaleFactor(0); // no pixel-grid rounding, as the engine sets it
  let calls = 0;
  const all = [];
  const build = (spec) => {
    const node = Yoga.Node.createWithConfig(config);
    all.push(node);
    for (const [k, v] of Object.entries(spec.style ?? {})) apply(node, k, v);
    if (spec.measure) {
      const [mw, mh] = spec.measure;
      const fit = (offer, mode, own) => (mode === 1 ? offer : mode === 2 ? Math.min(offer, own) : own);
      node.setMeasureFunc((w, wm, h, hm) => {
        calls++;
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

writeFileSync(FILE, JSON.stringify(doc, null, 2) + '\n');
console.log(`yoga-layout ${JSON.parse(readFileSync(new URL('node_modules/yoga-layout/package.json', import.meta.url))).version}: ${doc.cases.length} cases written`);
