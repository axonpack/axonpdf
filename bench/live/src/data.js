// A made-up warehouse ledger: shipments, each with its lines. Seeded, so every run builds the
// same data.
const ROWS_PER_PAGE = 33; // measured: about this many lines and shipment headings fit a page

const SITES = ['North Dock', 'River Yard', 'Hill Depot', 'East Annex'];
const CARRIERS = ['Bluewater Freight', 'Pine Haulage', 'Arc Logistics', 'Meadow Couriers'];
const ITEMS = [
  'Steel shelving bracket, galvanised',
  'Pallet wrap, 500 mm clear roll',
  'Cardboard carton, double wall, large',
  'Hand truck wheel, solid rubber',
  'Label printer ribbon, wax resin',
  'Safety gloves, cut level C, size 9',
  'Hi-vis vest, orange, XL',
  'Forklift battery water, 5 L',
  'Strapping seal, 13 mm',
  'Corner protector, 50 mm',
];
const NOTES = [
  'Counted twice at receipt; one carton dented but contents fine.',
  'Partial delivery, the rest follows on the next run.',
  'Moved to overflow racking until the bay is cleared.',
  '',
  '',
];

export function ledger(pages) {
  let seed = 7;
  const rand = () => (seed = (seed * 16807) % 2147483647) / 2147483647;
  const pick = (list) => list[Math.floor(rand() * list.length)];
  const target = Math.max(1, pages - 1) * ROWS_PER_PAGE; // the first page holds the summary
  const shipments = [];
  for (let n = 0, rows = 0; rows < target; n++) {
    const count = 4 + Math.floor(rand() * 9);
    const lines = Array.from({ length: count }, (_, i) => {
      const qty = 1 + Math.floor(rand() * 240);
      const unit = Math.round(50 + rand() * 9000) / 100;
      return { sku: `SKU-${String(10000 + ((n * 13 + i * 7) % 90000))}`, item: pick(ITEMS), bin: `${pick('ABCDEFGH')}${1 + Math.floor(rand() * 40)}-${1 + Math.floor(rand() * 6)}`, qty, unit, note: pick(NOTES) };
    });
    shipments.push({ ref: `SHP-${String(2026000 + n)}`, site: pick(SITES), carrier: pick(CARRIERS), day: 1 + (n % 28), lines });
    rows += count + 1;
  }
  return shipments;
}
