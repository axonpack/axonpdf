// The report on the newer pagination engine, its fixed footer written before the content, and a
// last page whose `layout` template draws a header with the page count and a line below.
import { Document, Page, Text, View } from '@axonpdf/react';
import type { ReactNode } from 'react';

import { META, PARAGRAPH, StockPage, s } from '../report/document';

function NotesLayout({ children, pageNumber, totalPages }: { children?: ReactNode; pageNumber?: number; totalPages?: number }) {
  return (
    <>
      <View style={{ flexDirection: 'row', justifyContent: 'space-between', borderBottom: '0.5pt solid #999', paddingBottom: 4, marginBottom: 8 }}>
        <Text style={{ fontFamily: 'Helvetica-Bold' }}>Delivery notes</Text>
        <Text>{`Page ${pageNumber} of ${totalPages}`}</Text>
      </View>
      {children}
      <Text style={{ marginTop: 8, fontSize: 7, color: '#666' }}>Willow House, stock room</Text>
    </>
  );
}

export default function ReportNext() {
  return (
    <Document {...META}>
      <StockPage next />
      <Page size="A6" style={{ padding: 24, fontSize: 9 }} bookmark="Delivery notes" layout={NotesLayout}>
        {Array.from({ length: 5 }, (_, i) => (
          <Text key={i} style={s.para}>
            {i + 1}. {PARAGRAPH}
          </Text>
        ))}
      </Page>
    </Document>
  );
}
