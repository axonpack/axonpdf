# @axonpdf/react

Create PDF files with React components, in Node and in the browser. It follows the usual React PDF
renderer API (`Document`, `Page`, `View`, `Text`, `StyleSheet`, `Font`, `renderToFile`, `pdf`,
`PDFViewer` and the rest), so an existing project can switch by changing its import. The layout
and the PDF writing run in the axonpdf engine, compiled to WebAssembly.

On top of that API it adds components for long documents (`Rows`, `PageNumber`, `PageCount`,
`Table`), page header and footer bands, and a profiler.

## Install

```sh
npm install @axonpdf/react react
```

React 18 and 19 are supported. The packages aren't on npm yet. Until they are, build them from
this repo (see the root README).

## Example

```tsx
import { Document, Page, StyleSheet, Text, View, renderToFile } from "@axonpdf/react";

const styles = StyleSheet.create({
  page: { padding: 40, fontSize: 12 },
  title: { fontSize: 20, marginBottom: 12 },
});

function Invoice({ number }: { number: number }) {
  return (
    <Document title={`Invoice ${number}`}>
      <Page size="A4" style={styles.page}>
        <Text style={styles.title}>Invoice {number}</Text>
        <View style={{ flexDirection: "row", justifyContent: "space-between" }}>
          <Text>Total</Text>
          <Text>£120.00</Text>
        </View>
        <Text fixed render={({ pageNumber, totalPages }) => `Page ${pageNumber} of ${totalPages}`} />
      </Page>
    </Document>
  );
}

await renderToFile(<Invoice number={42} />, "invoice.pdf");
```

In the browser, show the same document with `<PDFViewer>` or offer it with `<PDFDownloadLink>`.
`packages/react/example` has both, from one invoice component.

Every export, component, prop and style key is in [REFERENCE.md](REFERENCE.md). The project's
[README](../../README.md) covers switching an existing project, long documents and the other
packages.
