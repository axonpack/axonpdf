// The document both engines render. It takes the engine's exports as `pdf`, so the only
// difference between the two runs is where these components come from: `@axonpdf/react` or
// `@react-pdf/renderer`, which is the one line a project changes to switch.
import { ledger } from './data.js';

const money = (n) => n.toLocaleString('en-GB', { minimumFractionDigits: 2, maximumFractionDigits: 2 });
const COLS = [
  ['SKU', 70, 'left'],
  ['Item', 150, 'left'],
  ['Bin', 50, 'left'],
  ['Qty', 40, 'right'],
  ['Unit', 55, 'right'],
  ['Total', 70, 'right'],
];
const cell = (i) => ({ width: COLS[i][1], textAlign: COLS[i][2], paddingHorizontal: 3 });

export function LedgerDocument({ pdf, pages }) {
  const { Document, Page, View, Text, Svg, Rect, Path } = pdf;
  const shipments = ledger(pages);
  const units = shipments.reduce((s, sh) => s + sh.lines.reduce((t, l) => t + l.qty, 0), 0);
  return (
    <Document title="Warehouse ledger">
      <Page size="A4" style={{ paddingTop: 70, paddingBottom: 50, paddingHorizontal: 36, fontSize: 8, fontFamily: 'Helvetica' }}>
        <View fixed style={{ position: 'absolute', top: 24, left: 36, right: 36, flexDirection: 'row', alignItems: 'center', borderBottomWidth: 1, borderColor: '#1f4e79', paddingBottom: 6 }}>
          <Svg width={22} height={22} viewBox="0 0 24 24">
            <Rect x={2} y={6} width={20} height={14} fill="#1f4e79" />
            <Path d="M2 6 L12 1 L22 6 Z" fill="#4f81bd" />
          </Svg>
          <Text style={{ marginLeft: 8, fontSize: 13, fontFamily: 'Helvetica-Bold' }}>Warehouse ledger</Text>
          <Text style={{ marginLeft: 'auto', color: '#555' }}>Stock received, November</Text>
        </View>
        <Text fixed style={{ position: 'absolute', bottom: 20, left: 36, right: 36, textAlign: 'center', color: '#777' }} render={({ pageNumber, totalPages }) => `Page ${pageNumber} of ${totalPages}`} />

        <Text style={{ fontSize: 16, fontFamily: 'Helvetica-Bold', marginBottom: 8 }}>Summary</Text>
        <Text style={{ fontSize: 10, lineHeight: 1.4, marginBottom: 6 }}>
          {`${shipments.length} shipments, ${units.toLocaleString('en-GB')} units across four sites. Each shipment lists its lines with the bin they went to; notes from the person who checked them in wrap under the item.`}
        </Text>
        <View break />

        {shipments.map((sh) => (
          <View key={sh.ref}>
            <View wrap={false} minPresenceAhead={30} style={{ flexDirection: 'row', backgroundColor: '#dce6f1', paddingVertical: 3, paddingHorizontal: 3, marginTop: 6 }}>
              <Text style={{ fontFamily: 'Helvetica-Bold', width: 90 }}>{sh.ref}</Text>
              <Text style={{ width: 110 }}>{sh.site}</Text>
              <Text style={{ width: 130 }}>{sh.carrier}</Text>
              <Text>{`${sh.day} Nov`}</Text>
            </View>
            {sh.lines.map((l, i) => (
              <View key={i} wrap={false} style={{ flexDirection: 'row', paddingVertical: 2, borderBottomWidth: 0.5, borderColor: '#ccc' }}>
                <Text style={cell(0)}>{l.sku}</Text>
                <View style={cell(1)}>
                  <Text>{l.item}</Text>
                  {l.note ? <Text style={{ color: '#666', fontSize: 7 }}>{l.note}</Text> : null}
                </View>
                <Text style={cell(2)}>{l.bin}</Text>
                <Text style={cell(3)}>{l.qty}</Text>
                <Text style={cell(4)}>{money(l.unit)}</Text>
                <Text style={cell(5)}>{money(l.qty * l.unit)}</Text>
              </View>
            ))}
          </View>
        ))}
      </Page>
    </Document>
  );
}
