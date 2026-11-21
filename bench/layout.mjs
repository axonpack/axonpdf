// The benchmark report's numbers and data, shared by the JSX version (report.jsx) and the
// hand-written pdfkit version (report-pdfkit.mjs) so both draw the same pages.
export const PAGE = { width: 595.28, height: 841.89 };
export const PAD = { top: 80, bottom: 50, x: 40 };
export const ROW_H = 18;
export const HEAD_H = 20;
export const COLS = [
  { key: 'n', title: 'No.', width: 50, align: 'right' },
  { key: 'date', title: 'Date', width: 80, align: 'left' },
  { key: 'item', title: 'Item', width: 215.28, align: 'left' },
  { key: 'qty', title: 'Qty', width: 60, align: 'right' },
  { key: 'amount', title: 'Amount', width: 110, align: 'right' },
];
export const LOGO = new URL('../fixtures/assets/badge.png', import.meta.url).pathname;
export const TITLE = 'Quarterly supply report';
export const PARAGRAPHS = [
  'This report lists every supply line delivered to the home over the quarter, in the order the deliveries were booked. Each line shows the date it arrived, what it was, how many units came and what they cost.',
  'Quantities are counted on arrival and checked against the order. Where the count differed, the line shows the counted figure and the difference is followed up with the supplier before the next order goes out.',
  'Amounts include VAT. Totals per month and for the quarter are on the summary page that goes out with this report.',
];
const ITEMS = ['Gauze roll', 'Saline 500 ml', 'Nitrile gloves (M)', 'Thermometer covers', 'Syringe 5 ml', 'Alcohol swabs', 'Incontinence pads', 'Hand gel 500 ml'];

export const row = (i) => ({
  n: String(i + 1),
  date: `${String(1 + (i % 28)).padStart(2, '0')}/${String(1 + ((i >> 5) % 12)).padStart(2, '0')}/2026`,
  item: ITEMS[i % ITEMS.length],
  qty: String(1 + ((i * 7) % 40)),
  amount: `£${(((i * 37) % 9000) / 10 + 0.5).toFixed(2)}`,
});

// Rows for about `pages` pages: the first page loses roughly ten rows to the paragraphs.
export const rowsFor = (pages) => Math.floor((PAGE.height - PAD.top - PAD.bottom - HEAD_H) / ROW_H) * pages - 10;
