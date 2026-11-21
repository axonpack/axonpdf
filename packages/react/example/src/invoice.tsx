import { Document, Image, Page, StyleSheet, Text, View } from "@axonpdf/react";

const lines = [
  { item: "Design review", qty: 2, price: 120 },
  { item: "Layout fixes", qty: 5, price: 45 },
  { item: "PDF export", qty: 1, price: 300 },
  { item: "Support, monthly", qty: 3, price: 80 },
];

const money = (n: number) => `$${n.toFixed(2)}`;

const s = StyleSheet.create({
  page: { padding: 40, fontSize: 11, fontFamily: "Helvetica" },
  header: { flexDirection: "row", justifyContent: "space-between", alignItems: "center" },
  logo: { width: 48, height: 48 },
  title: { fontSize: 24, fontFamily: "Helvetica-Bold" },
  meta: { marginTop: 16, marginBottom: 24, color: "#555" },
  row: { flexDirection: "row", borderBottom: "0.5pt solid #ccc", paddingVertical: 6 },
  head: { fontFamily: "Helvetica-Bold", borderBottom: "1pt solid #333" },
  item: { flex: 3 },
  cell: { flex: 1, textAlign: "right" },
  total: { marginTop: 12, textAlign: "right", fontSize: 14, fontFamily: "Helvetica-Bold" },
  footer: {
    position: "absolute",
    bottom: 24,
    left: 40,
    right: 40,
    textAlign: "center",
    fontSize: 9,
    color: "#888",
  },
});

/** The same invoice for the web page and the Node script. `logo` is a URL or a file path. */
export function Invoice({ logo }: { logo: string }) {
  const total = lines.reduce((sum, l) => sum + l.qty * l.price, 0);
  return (
    <Document title="Invoice 2026-042" author="axonpdf example">
      <Page size="A4" style={s.page}>
        <View style={s.header}>
          <Text style={s.title}>Invoice</Text>
          <Image src={logo} style={s.logo} />
        </View>
        <Text style={s.meta}>No. 2026-042, issued 21 November 2026</Text>
        <View style={[s.row, s.head]}>
          <Text style={s.item}>Item</Text>
          <Text style={s.cell}>Qty</Text>
          <Text style={s.cell}>Price</Text>
          <Text style={s.cell}>Amount</Text>
        </View>
        {lines.map((l) => (
          <View key={l.item} style={s.row} wrap={false}>
            <Text style={s.item}>{l.item}</Text>
            <Text style={s.cell}>{l.qty}</Text>
            <Text style={s.cell}>{money(l.price)}</Text>
            <Text style={s.cell}>{money(l.qty * l.price)}</Text>
          </View>
        ))}
        <Text style={s.total}>Total {money(total)}</Text>
        <Text
          style={s.footer}
          fixed
          render={({ pageNumber, totalPages }) => `Page ${pageNumber} of ${totalPages}`}
        />
      </Page>
    </Document>
  );
}
