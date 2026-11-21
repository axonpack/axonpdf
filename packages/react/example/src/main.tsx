import { PDFDownloadLink, PDFViewer } from "@axonpdf/react";
import { createRoot } from "react-dom/client";

import { Invoice } from "./invoice.js";

const invoice = <Invoice logo="/logo.png" />;

createRoot(document.getElementById("root")!).render(
  <div style={{ display: "flex", flexDirection: "column", height: "100vh" }}>
    <div style={{ padding: 12 }}>
      <PDFDownloadLink document={invoice} fileName="invoice.pdf">
        {({ loading }) => (loading ? "Rendering..." : "Download invoice.pdf")}
      </PDFDownloadLink>
    </div>
    <PDFViewer style={{ flex: 1, border: 0 }}>{invoice}</PDFViewer>
  </div>,
);
