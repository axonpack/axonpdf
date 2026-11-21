// The benchmark report as JSX. axonpdf renders it as written; react-pdf 4.9.0 renders the same
// file with `@axonpdf/react` aliased to `@react-pdf/renderer`, which is the drop-in switch.
import { Document, Image, Page, Text, View } from '@axonpdf/react';
import { COLS, HEAD_H, PAD, PARAGRAPHS, ROW_H, TITLE, row } from './layout.mjs';

const cell = (c, extra) => ({ width: c.width, paddingHorizontal: 4, paddingTop: 4, textAlign: c.align, borderRightWidth: 0.5, borderColor: '#555', ...extra });

export function Row({ i }) {
  const r = row(i);
  return (
    <View wrap={false} style={{ flexDirection: 'row', height: ROW_H, borderBottomWidth: 0.5, borderColor: '#555' }}>
      {COLS.map((c) => (
        <Text key={c.key} style={cell(c)}>{r[c.key]}</Text>
      ))}
    </View>
  );
}

// The page around the table rows, shared with report-rows.jsx. `logo` comes in as a prop: this
// file is bundled, so it can't find the image from its own URL.
export function ReportPage({ logo, children }) {
  return (
    <Document title={TITLE}>
      <Page size="A4" style={{ paddingTop: PAD.top, paddingBottom: PAD.bottom, paddingHorizontal: PAD.x, fontFamily: 'Helvetica', fontSize: 9 }}>
        <View fixed style={{ position: 'absolute', top: 25, left: PAD.x, right: PAD.x, height: 30, flexDirection: 'row', alignItems: 'center' }}>
          <Image src={logo} style={{ width: 30, height: 30 }} />
          <Text style={{ marginLeft: 10, fontSize: 14, fontFamily: 'Helvetica-Bold' }}>{TITLE}</Text>
        </View>
        {PARAGRAPHS.map((p, i) => (
          <Text key={i} style={{ fontSize: 10, lineHeight: 1.3, marginBottom: 8 }}>{p}</Text>
        ))}
        <View style={{ borderTopWidth: 0.5, borderLeftWidth: 0.5, borderColor: '#555' }}>
          <View fixed style={{ flexDirection: 'row', height: HEAD_H, backgroundColor: '#dde3ea', borderBottomWidth: 0.5, borderColor: '#555' }}>
            {COLS.map((c) => (
              <Text key={c.key} style={cell(c, { fontFamily: 'Helvetica-Bold', paddingTop: 5 })}>{c.title}</Text>
            ))}
          </View>
          {children}
        </View>
        <Text fixed style={{ position: 'absolute', bottom: 25, left: PAD.x, right: PAD.x, textAlign: 'center', fontSize: 8, color: '#555' }} render={({ pageNumber, totalPages }) => `Page ${pageNumber} of ${totalPages}`} />
      </Page>
    </Document>
  );
}

export default function Report({ rows, logo }) {
  return (
    <ReportPage logo={logo}>
      {Array.from({ length: rows }, (_, i) => (
        <Row key={i} i={i} />
      ))}
    </ReportPage>
  );
}
