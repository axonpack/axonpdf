// The benchmark report with its table rows in axonpdf's <Rows>: the rows come from a generator and
// are laid out and written in batches, so memory stays flat however long the table is. axonpdf only.
import { Rows } from '@axonpdf/react';
import { ReportPage, Row } from './report.jsx';

function* numbers(count) {
  for (let i = 0; i < count; i++) yield i;
}

export default function ReportRows({ rows, logo }) {
  return (
    <ReportPage logo={logo}>
      <Rows rows={numbers(rows)} batchSize={200}>
        {(i) => <Row i={i} />}
      </Rows>
    </ReportPage>
  );
}
