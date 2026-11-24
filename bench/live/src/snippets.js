// The code a project would write for the current choices, per engine. Only the package name
// differs between the two, apart from axonpdf's worker helpers.
export const PACKAGE = { axonpdf: '@axonpdf/react', 'react-pdf': '@react-pdf/renderer' };

const chrome = `          <Text fixed style={{ fontSize: 13 }}>Warehouse ledger</Text>
          <Text
            fixed
            style={{ position: "absolute", bottom: 20, left: 0, right: 0, textAlign: "center" }}
            render={({ pageNumber, totalPages }) => \`Page \${pageNumber} of \${totalPages}\`}
          />`;

const documents = {
  sections: (pkg) => `// ledger.tsx
import { Document, Page, Text } from "${pkg}";

type Row = { id: string; text: string };
export type LedgerProps = { sections: { day: number; title: string; rows: Row[] }[] };

export function Ledger({ sections }: LedgerProps) {
  return (
    <Document>
      {sections.map((s) => (
        <Page key={s.day} size="A4" style={{ padding: 40 }}>
${chrome}
          <Text>{s.title}</Text>
          {s.rows.map((r) => <Text key={r.id}>{r.text}</Text>)}
        </Page>
      ))}
    </Document>
  );
}`,
  flowing: (pkg) => `// ledger.tsx
import { Document, Page, Text } from "${pkg}";

export type LedgerProps = { rows: { id: string; text: string }[] };

export function Ledger({ rows }: LedgerProps) {
  return (
    <Document>
      <Page size="A4" style={{ padding: 40 }}>
${chrome.replace(/^ {2}/gm, '')}
        {rows.map((r) => <Text key={r.id}>{r.text}</Text>)}
      </Page>
    </Document>
  );
}`,
};

const run = {
  'axonpdf worker': (pkg, data) => `// axonpdf.worker.js
import { createElement } from "react";
import { expose } from "${pkg}/worker";
import { Ledger } from "./ledger";

expose((props) => createElement(Ledger, props));

// app.tsx
import { usePDFWorker, useWorker } from "${pkg}/worker";
import type { LedgerProps } from "./ledger";

function App({ ${data} }: LedgerProps) {
  const worker = useWorker(
    () => new Worker(new URL("./axonpdf.worker.js", import.meta.url), { type: "module" }),
  );
  const { url } = usePDFWorker(worker, { ${data} });
  return url ? <iframe src={url} /> : null;
}`,
  'react-pdf worker': (pkg, data) => `// react-pdf.worker.js
import { createElement } from "react";
import { pdf } from "${pkg}";
import { Ledger } from "./ledger";

addEventListener("message", async ({ data }) => {
  postMessage(await pdf(createElement(Ledger, data)).toBlob());
});

// app.ts
const worker = new Worker(new URL("./react-pdf.worker.js", import.meta.url), { type: "module" });
const blob = await new Promise<Blob>((resolve) => {
  worker.addEventListener("message", (e) => resolve(e.data), { once: true });
  worker.postMessage({ ${data} });
});`,
  main: (pkg, data) => `// app.tsx
import { pdf } from "${pkg}";
import { Ledger } from "./ledger";

const blob = await pdf(<Ledger ${data}={${data}} />).toBlob();`,
};

/** The snippet for one engine, given the page's choices. */
export function snippet({ engine, layout, worker, pages }) {
  const pkg = PACKAGE[engine];
  const data = layout === 'sections' ? 'sections' : 'rows';
  const key = !worker ? 'main' : `${engine} worker`;
  const note = layout === 'sections' ? `// ${pages} pages: 28 sections, one <Page> each` : `// ${pages} pages, all in one <Page>`;
  return `${documents[layout](pkg)}\n\n${note}\n${run[key](pkg, data)}`;
}
