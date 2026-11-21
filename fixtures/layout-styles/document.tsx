// Three pages of box styles: flex direction, wrap, justify and align, gap, margin and padding
// shorthands, borders and radii, colours, every unit, absolute positioning, zIndex, opacity,
// overflow, transforms, aspect ratio, min and max sizes, display none, debug boxes, floats with
// shapeOutside and clear. Page sizes
// and orientations differ per page, and one style block switches through media queries. A last
// page with a page number holds rows whose `flex: 1` and `minHeight` parts come out smaller once
// the page has a height, so all three fit on it.
import { Document, Page, StyleSheet, Text, View } from '@axonpdf/react';

const s = StyleSheet.create({
  page: { padding: '20 24', fontSize: 9 },
  label: { fontSize: 8, color: '#444', marginBottom: 3, marginTop: 8 },
  box: { width: 40, height: 24, backgroundColor: '#4f7cac', margin: 2 },
  lane: { backgroundColor: '#eef1f5', height: 40 },
  // Narrow pages and landscape pages get their own colours and direction.
  responsive: {
    flexDirection: 'row',
    backgroundColor: '#d8f3dc',
    padding: 6,
    '@media max-width: 400': { flexDirection: 'column', backgroundColor: '#ffe5d9' },
    '@media orientation: landscape': { backgroundColor: '#e0e7ff' },
  },
});

function Boxes({ n = 3, color }: { n?: number; color?: string }) {
  return (
    <>
      {Array.from({ length: n }, (_, i) => (
        <View key={i} style={[s.box, color ? { backgroundColor: color } : {}]} />
      ))}
    </>
  );
}

const WRAP = 'Text flows around the float beside it, one line at a time, then takes the whole width.';
const FLOATS = [
  ['left', undefined],
  ['left', 'circle(50%)'],
  ['right', 'ellipse(28 20 at 50% 40%)'],
  ['right', 'polygon(100% 0, 100% 100%, 0 100%)'],
  ['left', 'inset(0 20 10 0)'],
] as const;

const JUSTIFY = ['flex-start', 'center', 'flex-end', 'space-between', 'space-around', 'space-evenly'] as const;
const ALIGN = ['flex-start', 'center', 'flex-end', 'stretch'] as const;

// One unbreakable row: `n` slots down two stretched columns, each closed by a `flex: 1` row.
function Slots({ n }: { n: number }) {
  const last = { flexDirection: 'row', flex: 1, minHeight: 20 } as const;
  const pad = { alignSelf: 'stretch', paddingVertical: 10, justifyContent: 'center' } as const;
  return (
    <View wrap={false} style={{ flexDirection: 'row', borderBottomWidth: 1, borderLeftWidth: 1 }}>
      <View style={{ width: 70, justifyContent: 'center' }}>
        <Text>{`${n} slots`}</Text>
      </View>
      <View style={{ alignSelf: 'stretch' }}>
        {Array.from({ length: n }, (_, i) => (
          <View key={i} style={{ width: 40, height: 36, justifyContent: 'center', borderBottomWidth: 1 }}>
            <Text>AM</Text>
          </View>
        ))}
        <View style={last}>
          <View style={[pad, { width: 40 }]}>
            <Text>Now</Text>
          </View>
        </View>
      </View>
      <View style={{ flex: 1 }}>
        {Array.from({ length: n }, (_, i) => (
          <View key={i} style={{ height: 36, borderBottomWidth: 1 }} />
        ))}
        <View style={last}>
          <View style={[pad, { flex: 1 }]} />
        </View>
      </View>
    </View>
  );
}

function Responsive() {
  return (
    <View style={s.responsive}>
      <Text style={{ padding: 4 }}>Media query block</Text>
      <Boxes n={2} color="#2d6a4f" />
    </View>
  );
}

export default function LayoutStyles() {
  return (
    <Document title="Layout and box styles">
      <Page size="A4" style={s.page}>
        <Text style={s.label}>justifyContent, row</Text>
        {JUSTIFY.map((j) => (
          <View key={j} style={[s.lane, { flexDirection: 'row', justifyContent: j, height: 28, marginBottom: 2 }]}>
            <Boxes />
          </View>
        ))}

        <Text style={s.label}>alignItems, then alignSelf and flexGrow / flexShrink / flexBasis</Text>
        <View style={{ flexDirection: 'row', gap: 6 }}>
          {ALIGN.map((a) => (
            <View key={a} style={[s.lane, { width: 80, height: 60, alignItems: a }]}>
              <View style={{ width: '50%', minWidth: 20, height: 14, backgroundColor: '#e07a5f' }} />
            </View>
          ))}
          <View style={[s.lane, { flex: 1, height: 60, flexDirection: 'row' }]}>
            <View style={{ flexGrow: 2, backgroundColor: '#81b29a', alignSelf: 'flex-end', height: 20 }} />
            <View style={{ flexGrow: 1, flexBasis: 30, backgroundColor: '#3d405b' }} />
            <View style={{ flexShrink: 1, width: 300, maxWidth: 60, backgroundColor: '#f2cc8f' }} />
          </View>
        </View>

        <Text style={s.label}>flexWrap with rowGap (flexFlow is ignored) / columnGap, alignContent, column-reverse and row-reverse</Text>
        <View style={{ flexDirection: 'row', gap: 6 }}>
          <View style={[s.lane, { width: 230, height: 90, flexDirection: 'row', flexWrap: 'wrap', flexFlow: 'column nowrap', rowGap: 4, columnGap: 8, alignContent: 'space-between' }]}>
            <Boxes n={9} color="#9c6644" />
          </View>
          <View style={[s.lane, { width: 60, height: 90, flexDirection: 'column-reverse' }]}>
            <Boxes n={2} />
          </View>
          <View style={[s.lane, { flex: 1, height: 90, flexDirection: 'row-reverse', flexWrap: 'wrap-reverse' }]}>
            <Boxes n={5} color="#bc4749" />
          </View>
        </View>

        <Text style={s.label}>margin and padding shorthands, margin auto, borders and radii</Text>
        <View style={{ flexDirection: 'row', flexWrap: 'wrap', backgroundColor: '#f7f7f7' }}>
          <View style={{ margin: '4 8 12 16', padding: '2 6', border: '2pt solid #333', backgroundColor: '#ffd166' }}>
            <Text>m 4 8 12 16</Text>
          </View>
          <View style={{ marginVertical: 6, marginHorizontal: 'auto', paddingVertical: 8, paddingHorizontal: 4, borderWidth: 3, borderStyle: 'dashed', borderColor: '#ef476f' }}>
            <Text>dashed, auto</Text>
          </View>
          <View style={{ margin: 6, padding: 6, borderTop: '4pt dotted #118ab2', borderRight: '1pt solid red', borderBottomWidth: 2, borderBottomColor: 'green', borderLeftWidth: 6, borderLeftColor: '#073b4c' }}>
            <Text>per side</Text>
          </View>
          <View style={{ margin: 6, width: 60, height: 40, backgroundColor: '#06d6a0', borderRadius: 10 }} />
          <View style={{ margin: 6, width: 60, height: 40, backgroundColor: '#118ab2', borderTopLeftRadius: 20, borderBottomRightRadius: '50%', border: '2pt solid #073b4c' }} />
          <View style={{ margin: 6, width: 40, height: 40, backgroundColor: '#ef476f', borderRadius: 20 }} />
        </View>

        <Text style={s.label}>colours: hex, short hex, rgb, rgba, hsl, hsla, names; opacity</Text>
        <View style={{ flexDirection: 'row' }}>
          {['#2a9d8f', '#e76', 'rgb(233, 196, 106)', 'rgba(244, 162, 97, 0.5)', 'hsl(12, 76%, 61%)', 'hsla(200, 60%, 40%, 0.4)', 'tomato', 'rebeccapurple'].map((c) => (
            <View key={c} style={{ width: 40, height: 20, backgroundColor: c, marginRight: 4 }} />
          ))}
          <View style={{ width: 40, height: 20, backgroundColor: 'black', opacity: 0.25 }} />
        </View>

        <Text style={s.label}>units: pt, in, mm, cm, %, vw, vh, px, rem</Text>
        {[72, '1in', '25.4mm', '2.54cm', '20%', '20vw', '5vh', '96px', '4rem'].map((w) => (
          <View key={String(w)} style={{ width: w, height: 6, backgroundColor: '#264653', marginBottom: 2 }} />
        ))}

        <Text style={s.label}>absolute positioning, zIndex, overflow hidden, display none, debug</Text>
        <View style={{ height: 90, backgroundColor: '#eef1f5', position: 'relative' }} debug>
          <View style={{ position: 'absolute', top: 10, left: 10, width: 60, height: 60, backgroundColor: '#e63946', zIndex: 2 }} />
          <View style={{ position: 'absolute', top: 25, left: 40, width: 60, height: 60, backgroundColor: '#457b9d', zIndex: 1 }} />
          <View style={{ position: 'absolute', right: 10, bottom: 10, width: 80, height: 40, overflow: 'hidden', backgroundColor: '#a8dadc' }}>
            <View style={{ width: 120, height: 70, marginLeft: 20, backgroundColor: '#1d3557' }} />
          </View>
          <View style={{ display: 'none', width: 500, height: 500, backgroundColor: 'red' }} />
          <View style={{ marginLeft: 140, marginTop: 10, width: 120, aspectRatio: 2, backgroundColor: '#f1faee', padding: 6, borderWidth: 1 }} debug>
            <Text>aspectRatio 2</Text>
          </View>
        </View>
      </Page>

      <Page size="LETTER" orientation="landscape" style={s.page}>
        <Text style={s.label}>transforms (rotate, scale, translate, skew, matrix) with transformOrigin</Text>
        <View style={{ flexDirection: 'row', height: 120, alignItems: 'center', justifyContent: 'space-around', backgroundColor: '#fafafa' }}>
          {[
            { transform: 'rotate(20deg)' },
            { transform: 'scale(1.4, 0.7)' },
            { transform: 'translate(10, -20)' },
            { transform: 'skew(15deg, 5deg)' },
            { transform: 'rotate(-30deg) scale(0.8)', transformOrigin: 'top left' },
            { transform: 'matrix(1, 0.2, 0, 1, 0, 0)', transformOrigin: '20% 80%' },
          ].map((t, i) => (
            <View key={i} style={[{ width: 60, height: 40, backgroundColor: '#6d597a', justifyContent: 'center', alignItems: 'center' }, t]}>
              <Text style={{ color: 'white' }}>{i + 1}</Text>
            </View>
          ))}
        </View>
        <Text style={s.label}>this page is landscape, so the media query picks the landscape colour</Text>
        <Responsive />
        <Text style={[s.label, { marginTop: 40 }]}>floats: the plain box, then circle, ellipse, polygon and inset shapes; the last line of each clears</Text>
        <View style={{ flexDirection: 'row', justifyContent: 'space-between' }}>
          {FLOATS.map(([float, shapeOutside], i) => (
            <View key={i} style={{ width: 140 }}>
              <View style={{ float, shapeOutside, width: 56, height: 56, backgroundColor: '#e9d8a6', [float === 'left' ? 'marginRight' : 'marginLeft']: 6, marginBottom: 4 }} />
              <Text>{WRAP}</Text>
              <Text style={{ clear: float, color: '#9b2226' }}>Cleared below the {float} float.</Text>
            </View>
          ))}
        </View>
      </Page>

      <Page size={[300, 400]} style={s.page}>
        <Text style={s.label}>300 x 400 page: max-width 400 matches, the block stacks</Text>
        <Responsive />
      </Page>

      <Page size={{ width: 300 }} style={s.page}>
        <Text style={s.label}>width-only page: height grows to its content</Text>
        <Boxes n={4} color="#b5838d" />
      </Page>

      <Page size={[400, 300]} style={{ padding: 20, fontSize: 8 }}>
        {[3, 2, 1].map((n) => (
          <Slots key={n} n={n} />
        ))}
        <Text fixed style={{ position: 'absolute', bottom: 6, left: 20 }} render={({ pageNumber }) => `Page ${pageNumber}`} />
      </Page>
    </Document>
  );
}
