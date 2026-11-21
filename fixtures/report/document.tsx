// A three-page report in Helvetica: reusable components fed by props, a bordered table that breaks
// across pages under a repeating header row, fixed header and footer, page numbers with the total,
// nested bookmarks, internal and external links, metadata, and the pagination props
// (break, wrap={false}, minPresenceAhead, orphans, widows).
import { Document, Link, Page, StyleSheet, Text, View } from '@axonpdf/react';

type Line = { sku: string; name: string; qty: number; price: number };

const LINES: Line[] = Array.from({ length: 48 }, (_, i) => ({
  sku: `SKU-${String(1000 + i * 7)}`,
  name: ['Gauze roll', 'Saline 500 ml', 'Nitrile gloves (M)', 'Thermometer covers', 'Syringe 5 ml', 'Alcohol swabs'][i % 6],
  qty: (i * 13) % 40 + 1,
  price: ((i * 37) % 900) / 10 + 0.5,
}));

const PARAGRAPH =
  'Stock is counted at the end of every shift and the counts are checked against the order list. ' +
  'Where a count differs from the list by more than two units, the shift lead writes the reason in ' +
  'the log and the pharmacy is told before the next delivery window. Items close to their expiry ' +
  'date are moved to the front of the shelf and marked, and anything past its date is recorded and ' +
  'returned. The table above lists the lines ordered this month with their unit price.';

const s = StyleSheet.create({
  page: { paddingTop: 70, paddingBottom: 60, paddingHorizontal: 40, fontSize: 10 },
  header: { position: 'absolute', top: 24, left: 40, right: 40, flexDirection: 'row', justifyContent: 'space-between', borderBottom: '1pt solid #999', paddingBottom: 6 },
  footer: { position: 'absolute', bottom: 24, left: 40, right: 40, fontSize: 8, color: '#666', textAlign: 'center' },
  h1: { fontSize: 22, fontFamily: 'Helvetica-Bold', marginBottom: 12 },
  h2: { fontSize: 14, fontFamily: 'Helvetica-Bold', marginTop: 16, marginBottom: 8 },
  table: { borderWidth: 1, borderColor: '#333', borderStyle: 'solid' },
  row: { flexDirection: 'row', borderBottomWidth: 0.5, borderBottomColor: '#bbb' },
  head: { backgroundColor: '#1f3a5f', color: 'white', fontFamily: 'Helvetica-Bold' },
  cell: { padding: 4, borderRightWidth: 0.5, borderRightColor: '#bbb' },
  para: { marginBottom: 8, lineHeight: 1.4, textAlign: 'justify' },
});

const money = (n: number) => `£${n.toFixed(2)}`;

function Field({ label, value }: { label: string; value: string }) {
  return (
    <View style={{ flexDirection: 'row', marginBottom: 4 }}>
      <Text style={{ width: 110, color: '#555' }}>{label}</Text>
      <Text style={{ fontFamily: 'Helvetica-Bold' }}>{value}</Text>
    </View>
  );
}

const COLS = [
  { key: 'sku', title: 'SKU', width: '18%' },
  { key: 'name', title: 'Item', width: '42%' },
  { key: 'qty', title: 'Qty', width: '12%', align: 'right' },
  { key: 'price', title: 'Unit', width: '14%', align: 'right' },
  { key: 'total', title: 'Total', width: '14%', align: 'right' },
] as const;

function Row({ line, index }: { line: Line; index: number }) {
  const values = { ...line, price: money(line.price), total: money(line.qty * line.price) };
  return (
    <View style={[s.row, index % 2 ? { backgroundColor: '#f3f6fa' } : {}]} wrap={false}>
      {COLS.map((c) => (
        <Text key={c.key} style={[s.cell, { width: c.width, textAlign: c.align ?? 'left' }]}>
          {String(values[c.key])}
        </Text>
      ))}
    </View>
  );
}

export default function Report() {
  const total = LINES.reduce((sum, l) => sum + l.qty * l.price, 0);
  return (
    <Document
      title="Monthly stock report"
      author="Fixture author"
      subject="Parity fixture"
      keywords="stock, report, fixture"
      creator="fixtures/report"
      producer="fixtures/report"
      language="en-GB"
      creationDate={new Date('2026-11-21T09:00:00Z')}
      modificationDate={new Date('2026-11-21T09:00:00Z')}
      pageMode="useOutlines"
    >
      <Page size="A4" style={s.page} bookmark="Stock report">
        <View style={s.header} fixed>
          <Text style={{ fontFamily: 'Helvetica-Bold' }}>Willow House</Text>
          <Text render={({ pageNumber }) => (pageNumber > 1 ? 'Monthly stock report (continued)' : 'Monthly stock report')} />
        </View>

        <Text style={s.h1} bookmark={{ title: 'Summary', fit: true }}>Summary</Text>
        <Link src="https://example.com/homes/willow" hitSlop={{ top: 3, left: 6, bottom: 1 }}>
          <Field label="Home" value="Willow House" />
        </Link>
        <Field label="Period" value="1 to 30 November 2026" />
        <Field label="Lines" value={String(LINES.length)} />
        <Field label="Order total" value={money(total)} />
        <Text style={{ marginTop: 8 }}>
          Jump to the <Link src="#totals">totals</Link> or the <Link src="#notes">notes</Link>, or read the{' '}
          <Link src="https://example.com/policy">stock policy</Link>.
        </Text>

        <Text style={s.h2} minPresenceAhead={60} bookmark="Order lines">Order lines</Text>
        <View style={s.table}>
          <View style={[s.row, s.head]} fixed>
            {COLS.map((c) => (
              <Text key={c.key} style={[s.cell, { width: c.width, textAlign: c.align ?? 'left' }]}>{c.title}</Text>
            ))}
          </View>
          {LINES.map((line, i) => (
            <Row key={line.sku} line={line} index={i} />
          ))}
        </View>
        <View id="totals" wrap={false} style={{ alignSelf: 'flex-end', width: 200, marginTop: 10, padding: 8, border: '1pt solid #1f3a5f' }} bookmark="Totals">
          <Field label="Net" value={money(total)} />
          <Field label="VAT 20%" value={money(total * 0.2)} />
          <Field label="Due" value={money(total * 1.2)} />
        </View>

        <View break id="notes" bookmark="Notes">
          <Text style={s.h2}>Notes</Text>
          {Array.from({ length: 9 }, (_, i) => (
            <Text key={i} style={s.para} orphans={3} widows={3}>
              {i + 1}. {PARAGRAPH}
            </Text>
          ))}
        </View>

        <Text style={s.footer} fixed render={({ pageNumber, totalPages }) => `Page ${pageNumber} of ${totalPages}`} />
      </Page>
    </Document>
  );
}
