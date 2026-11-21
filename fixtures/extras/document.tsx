// axonpdf's own additions, so there is no reference render: checked by eye. Page header and footer
// bands with their page info, PageNumber and PageCount, a Table whose header repeats on every
// page, Rows fed by an async iterable in batches, and the CSS break props on View.
import { Document, Page, PageCount, PageNumber, Rows, Table, TableCell, TableHeader, TableRow, Text, View } from '@axonpdf/react';

type Dose = { time: string; drug: string; dose: string; by: string };

const DOSES: Dose[] = Array.from({ length: 40 }, (_, i) => ({
  time: `${String(6 + (i % 16)).padStart(2, '0')}:${i % 2 ? '30' : '00'}`,
  drug: ['Paracetamol', 'Amlodipine', 'Metformin', 'Omeprazole', 'Sertraline'][i % 5],
  dose: `${(i % 4) + 1} tablet${i % 4 ? 's' : ''}`,
  by: ['AB', 'CD', 'EF'][i % 3],
}));

async function* entries(count: number) {
  for (let i = 0; i < count; i++) yield { n: i + 1, text: `Entry ${i + 1}: checked, signed and filed.` };
}

const cell = { padding: 4, borderRight: '0.5pt solid #bbb' };

type Info = { pageNumber: number; totalPages: number; isFirst: boolean; isLast: boolean; isOdd: boolean };

function Header({ pageNumber, isFirst, isOdd }: Info) {
  return (
    <View style={{ flexDirection: isOdd ? 'row' : 'row-reverse', justifyContent: 'space-between', padding: '16 30 6', borderBottom: '1pt solid #2a9d8f' }}>
      <Text style={{ fontFamily: 'Helvetica-Bold' }}>{isFirst ? 'Medication round' : 'Medication round (continued)'}</Text>
      <Text>{isOdd ? 'odd' : 'even'} page {pageNumber}</Text>
    </View>
  );
}

export default function Extras() {
  return (
    <Document title="axonpdf extras">
      <Page
        size="A4"
        style={{ paddingHorizontal: 30, fontSize: 9 }}
        header={(info: Info) => <Header {...info} />}
        footer={({ isLast }: Info) => (
          <View style={{ flexDirection: 'row', justifyContent: 'center', padding: '6 30 16', color: '#666' }}>
            <Text>
              Page <PageNumber /> of <PageCount />
              {isLast ? ' (end)' : ''}
            </Text>
          </View>
        )}
      >
        <Text style={{ fontSize: 16, marginVertical: 8 }}>Table with a repeating header</Text>
        <Table style={{ border: '1pt solid #333' }}>
          <TableHeader style={{ backgroundColor: '#264653', color: 'white' }}>
            <TableCell style={[cell, { width: 60 }]}><Text>Time</Text></TableCell>
            <TableCell style={[cell, { flex: 1 }]}><Text>Medicine</Text></TableCell>
            <TableCell style={[cell, { width: 80 }]}><Text>Dose</Text></TableCell>
            <TableCell style={{ padding: 4, width: 40 }}><Text>By</Text></TableCell>
          </TableHeader>
          {DOSES.map((d, i) => (
            <TableRow key={i} wrap={false} style={{ borderTop: '0.5pt solid #bbb', backgroundColor: i % 2 ? '#f1f5f4' : 'white' }}>
              <TableCell style={[cell, { width: 60 }]}><Text>{d.time}</Text></TableCell>
              <TableCell style={[cell, { flex: 1 }]}><Text>{d.drug}</Text></TableCell>
              <TableCell style={[cell, { width: 80 }]}><Text>{d.dose}</Text></TableCell>
              <TableCell style={{ padding: 4, width: 40 }}><Text>{d.by}</Text></TableCell>
            </TableRow>
          ))}
        </Table>

        <View breakBefore="page">
          <Text style={{ fontSize: 16, marginVertical: 8 }}>Rows from an async iterable, batches of 20</Text>
        </View>
        <Rows rows={entries(60)} batchSize={20}>
          {(row: { n: number; text: string }, index: number) => (
            <Text style={{ paddingVertical: 2, color: index % 20 === 0 ? '#e76f51' : 'black' }}>{row.text}</Text>
          )}
        </Rows>

        <View breakInside="avoid" breakAfter="page" style={{ marginTop: 12, padding: 10, height: 200, backgroundColor: '#e9c46a' }}>
          <Text>breakInside avoid: this box moves whole to the next page rather than split, and breakAfter page ends the page after it.</Text>
        </View>
        <Text>After the break.</Text>
      </Page>
    </Document>
  );
}
