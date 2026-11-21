// The benchmark report with its table rows passed as a generator child instead of an array. A
// generator is an ordinary iterable child in React, so react-pdf renders this file unchanged too.
import { ReportPage, Row } from './report.jsx';

function* rowElements(count) {
  for (let i = 0; i < count; i++) yield <Row key={i} i={i} />;
}

export default function ReportGenerator({ rows, logo }) {
  return <ReportPage logo={logo}>{rowElements(rows)}</ReportPage>;
}
