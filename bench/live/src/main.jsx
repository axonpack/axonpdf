import { useEffect, useState } from 'react';
import { createRoot } from 'react-dom/client';
import { ENGINES, run } from './measure.js';

const mb = (n) => `${(n / 2 ** 20).toFixed(1)} MB`;

// Moves every frame from JavaScript, so it freezes whenever the main thread is blocked.
function Ticker() {
  const [x, setX] = useState(0);
  useEffect(() => {
    let id = requestAnimationFrame(function step(t) {
      setX(t);
      id = requestAnimationFrame(step);
    });
    return () => cancelAnimationFrame(id);
  }, []);
  return (
    <div style={{ position: 'relative', height: 8, background: '#eee', width: 240, borderRadius: 4 }} title="Stops moving while the main thread is busy">
      <div style={{ position: 'absolute', left: ((x / 4) % 230), width: 10, height: 8, background: '#1f4e79', borderRadius: 4 }} />
    </div>
  );
}

function App() {
  const [preset, setPreset] = useState('100');
  const [custom, setCustom] = useState(250);
  const [engine, setEngine] = useState('both');
  const [inWorker, setInWorker] = useState({ axonpdf: true, 'react-pdf': true });
  const [busy, setBusy] = useState('');
  const [results, setResults] = useState([]);
  const [shown, setShown] = useState([]);
  const [error, setError] = useState('');

  const pages = preset === 'custom' ? Math.max(1, Math.min(5000, Math.round(custom) || 1)) : +preset;

  async function go() {
    setError('');
    setShown([]);
    const order = engine === 'both' ? ['axonpdf', 'react-pdf'] : [engine];
    const done = [];
    try {
      for (const e of order) {
        setBusy(`Rendering with ${ENGINES[e].label}…`);
        const r = await run({ engine: e, pages, useWorker: inWorker[e] });
        done.push(r);
        setShown([...done]);
        setResults((rs) => [r, ...rs]);
        console.log('RESULT ' + JSON.stringify({ ...r, url: undefined }));
      }
    } catch (e) {
      setError(String(e?.stack ?? e));
      console.log('ERROR ' + String(e));
    }
    setBusy('');
  }

  return (
    <main style={{ fontFamily: 'system-ui, sans-serif', margin: '16px auto', maxWidth: 1200, padding: '0 16px', color: '#222' }}>
      <h1 style={{ fontSize: 22, marginBottom: 4 }}>axonpdf and react-pdf 4.9.0, live in your browser</h1>
      <p style={{ marginTop: 0, color: '#555' }}>Both render the same warehouse ledger from the same components. Only the import differs.</p>

      <fieldset style={{ display: 'flex', flexWrap: 'wrap', gap: 16, alignItems: 'center', border: '1px solid #ccc', padding: 12 }}>
        <label>
          Pages{' '}
          <select value={preset} onChange={(e) => setPreset(e.target.value)} name="pages">
            <option value="100">100</option>
            <option value="500">500</option>
            <option value="2000">2000</option>
            <option value="custom">custom</option>
          </select>
        </label>
        {preset === 'custom' && <input type="number" min={1} max={5000} value={custom} onChange={(e) => setCustom(+e.target.value)} style={{ width: 80 }} name="custom" />}
        <label>
          Engine{' '}
          <select value={engine} onChange={(e) => setEngine(e.target.value)} name="engine">
            <option value="both">both, side by side</option>
            <option value="axonpdf">axonpdf</option>
            <option value="react-pdf">react-pdf 4.9.0</option>
          </select>
        </label>
        {(engine === 'both' ? ['axonpdf', 'react-pdf'] : [engine]).map((e) => (
          <label key={e}>
            <input type="checkbox" checked={inWorker[e]} onChange={(ev) => setInWorker({ ...inWorker, [e]: ev.target.checked })} name={`worker-${e}`} /> {ENGINES[e].label} in a Web Worker
          </label>
        ))}
        <button onClick={go} disabled={!!busy}>{busy ? 'Rendering…' : 'Render'}</button>
        <Ticker />
        <span style={{ color: '#555' }}>{busy}</span>
        {engine !== 'axonpdf' && pages > 100 && (
          <span style={{ color: '#a15c00' }}>
            {inWorker['react-pdf']
              ? 'At 500 pages react-pdf took over 10 minutes in our run ('
              : 'react-pdf runs on the main thread. At 500 pages it froze the tab for over 10 minutes in our run ('}
            <a href="https://github.com/axonpack/axonpdf/blob/main/bench/RESULTS.md">results</a>).
          </span>
        )}
      </fieldset>
      <p style={{ color: '#777', fontSize: 13 }}>Page counts are approximate before the render; the table shows the real count.</p>
      {error && <pre style={{ color: '#b00', whiteSpace: 'pre-wrap' }}>{error}</pre>}

      {results.length > 0 && (
        <table style={{ borderCollapse: 'collapse', width: '100%', fontSize: 14 }}>
          <thead>
            <tr>{['Engine', 'Thread', 'Pages', 'Time to PDF', 'File size', 'Longest frame gap', 'JS heap peak'].map((h) => <th key={h} style={{ textAlign: 'left', borderBottom: '2px solid #999', padding: 4 }}>{h}</th>)}</tr>
          </thead>
          <tbody>
            {results.map((r, i) => (
              <tr key={i}>
                {[ENGINES[r.engine].label, r.thread, r.pages, `${(r.ms / 1000).toFixed(2)} s`, mb(r.size), `${r.gap} ms`, r.heap === null ? 'not exposed' : mb(r.heap)].map((c, j) => <td key={j} style={{ borderBottom: '1px solid #ddd', padding: 4 }}>{c}</td>)}
              </tr>
            ))}
          </tbody>
        </table>
      )}

      <div style={{ display: 'grid', gridTemplateColumns: `repeat(auto-fit, minmax(320px, 1fr))`, gap: 16, marginTop: 16 }}>
        {shown.map((r) => (
          <section key={r.engine}>
            <h2 style={{ fontSize: 16 }}>
              {ENGINES[r.engine].label} <a href={r.url} download={`ledger-${r.engine}-${r.pages}.pdf`} style={{ fontSize: 14, marginLeft: 8 }}>Download PDF</a>
            </h2>
            <iframe title={ENGINES[r.engine].label} src={r.url} style={{ width: '100%', height: '70vh', border: '1px solid #ccc' }} />
          </section>
        ))}
      </div>
    </main>
  );
}

createRoot(document.getElementById('root')).render(<App />);
