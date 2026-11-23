// react-pdf 4.9.0 in a Web Worker. It speaks the same messages as axonpdf's `expose`
// (`{ id, props }` in, `{ id, blob }` or `{ id, error }` out), so the page reuses `renderInWorker`.
import * as reactPdf from '@react-pdf/renderer';
import { createElement } from 'react';
import { LedgerDocument } from './ledger.jsx';

addEventListener('message', async ({ data: { id, props } }) => {
  try {
    const blob = await reactPdf.pdf(createElement(LedgerDocument, { pdf: reactPdf, ...props })).toBlob();
    postMessage({ id, blob });
  } catch (error) {
    postMessage({ id, error: String(error?.stack ?? error) });
  }
});
