// The benchmark report written by hand with pdfkit: same page, fonts, sizes and table as
// report.jsx. Page breaks are worked out up front so "Page n of N" can be drawn as each page is
// made and pages stream out (no bufferPages).
import PDFDocument from 'pdfkit';
import { createWriteStream } from 'node:fs';
import { COLS, HEAD_H, LOGO, PAD, PAGE, PARAGRAPHS, ROW_H, TITLE, row } from './layout.mjs';

export async function renderPdfkit(rows, out) {
  const doc = new PDFDocument({ size: 'A4', margin: 0, autoFirstPage: false, info: { Title: TITLE } });
  const done = new Promise((resolve, reject) => doc.pipe(createWriteStream(out)).on('finish', resolve).on('error', reject));
  const width = PAGE.width - 2 * PAD.x;
  const bottom = PAGE.height - PAD.bottom;

  doc.font('Helvetica').fontSize(10);
  const paraH = PARAGRAPHS.map((p) => doc.heightOfString(p, { width, lineGap: 3 }) + 8);
  const firstTop = PAD.top + paraH.reduce((a, b) => a + b, 0);
  const perPage = (top) => Math.floor((bottom - top - HEAD_H - 0.5) / ROW_H);
  const firstRows = perPage(firstTop);
  const restRows = perPage(PAD.top);
  const total = rows <= firstRows ? 1 : 1 + Math.ceil((rows - firstRows) / restRows);

  const cells = (y, h, values, bold) => {
    let x = PAD.x;
    doc.font(bold ? 'Helvetica-Bold' : 'Helvetica').fontSize(9).fillColor('black');
    for (const c of COLS) {
      doc.text(values[c.key], x + 4, y + (bold ? 5 : 4), { width: c.width - 8, align: c.align, lineBreak: false });
      x += c.width;
      doc.moveTo(x, y).lineTo(x, y + h);
    }
    doc.moveTo(PAD.x, y + h).lineTo(PAD.x + width, y + h).lineWidth(0.5).strokeColor('#555').stroke();
  };

  let i = 0;
  for (let n = 1; n <= total; n++) {
    doc.addPage({ size: 'A4', margin: 0 });
    doc.image(LOGO, PAD.x, 25, { width: 30, height: 30 });
    doc.font('Helvetica-Bold').fontSize(14).fillColor('black').text(TITLE, PAD.x + 40, 33, { lineBreak: false });
    doc.font('Helvetica').fontSize(8).fillColor('#555').text(`Page ${n} of ${total}`, PAD.x, PAGE.height - 25 - 10, { width, align: 'center', lineBreak: false });

    let y = PAD.top;
    if (n === 1) {
      doc.font('Helvetica').fontSize(10).fillColor('black');
      PARAGRAPHS.forEach((p, k) => {
        doc.text(p, PAD.x, y, { width, lineGap: 3 });
        y += paraH[k];
      });
    }
    const top = y;
    doc.rect(PAD.x, y, width, HEAD_H).fill('#dde3ea');
    cells(y, HEAD_H, Object.fromEntries(COLS.map((c) => [c.key, c.title])), true);
    y += HEAD_H;
    for (let k = n === 1 ? firstRows : restRows; k > 0 && i < rows; k--, i++) {
      cells(y, ROW_H, row(i), false);
      y += ROW_H;
    }
    doc.moveTo(PAD.x, top).lineTo(PAD.x + width, top).moveTo(PAD.x, top).lineTo(PAD.x, y).lineWidth(0.5).strokeColor('#555').stroke();
  }
  doc.end();
  await done;
}
