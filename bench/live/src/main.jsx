import { useEffect, useState } from 'react';
import { createRoot } from 'react-dom/client';
import { ENGINES, run } from './measure.js';
import './style.css';

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
  const pos = (x / 2400) % 2;
  return (
    <div className="pulse" title="Stops moving while the main thread is busy">
      Main thread
      <div className="track">
        <div className="dot" style={{ left: `calc(${pos < 1 ? pos : 2 - pos} * (100% - 28px))` }} />
      </div>
    </div>
  );
}

function Segmented({ label, value, options, onChange }) {
  return (
    <div className="seg" role="group" aria-label={label}>
      {options.map(([v, text]) => (
        <button key={v} type="button" aria-pressed={value === v} onClick={() => onChange(v)}>
          {text}
        </button>
      ))}
    </div>
  );
}

const cls = (e) => (e === 'axonpdf' ? 'axon' : 'rpdf');

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
  // The main thread is allowed only up to 100 pages; above that both engines run in a worker.
  const forced = pages > 100;
  const worker = (e) => forced || inWorker[e];
  const picked = engine === 'both' ? ['axonpdf', 'react-pdf'] : [engine];

  async function go() {
    setError('');
    setShown([]);
    const done = [];
    try {
      for (const e of picked) {
        setBusy(`Rendering with ${ENGINES[e].label}…`);
        const r = await run({ engine: e, pages, useWorker: worker(e) });
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

  const results_link = <a href="https://github.com/axonpack/axonpdf/blob/main/bench/RESULTS.md">results</a>;

  return (
    <main>
      <h1>
        <span className="axon">axonpdf</span> and <span className="rpdf">react-pdf 4.9.0</span>, live in your browser
      </h1>
      <p className="lede">Both render the same warehouse ledger from the same components. Only the import differs.</p>

      <section className="panel" aria-label="Settings">
        <div className="row">
          <span className="name">Pages</span>
          <Segmented label="Pages" value={preset} onChange={setPreset} options={[['100', '100'], ['500', '500'], ['2000', '2000'], ['custom', 'Custom']]} />
          {preset === 'custom' && (
            <input className="custom" type="number" min={1} max={5000} value={custom} onChange={(e) => setCustom(+e.target.value)} name="custom" aria-label="Custom page count" />
          )}
        </div>
        <div className="row">
          <span className="name">Engine</span>
          <Segmented label="Engine" value={engine} onChange={setEngine} options={[['both', 'Both'], ['axonpdf', 'axonpdf'], ['react-pdf', 'react-pdf']]} />
        </div>
        <div className="row">
          <span className="name">Thread</span>
          {picked.map((e) => (
            <label key={e} className="pick">
              <span className={cls(e)}>{ENGINES[e].label}</span>
              <select value={worker(e) ? 'worker' : 'main'} onChange={(ev) => setInWorker({ ...inWorker, [e]: ev.target.value === 'worker' })} name={`thread-${e}`}>
                <option value="worker">Worker thread</option>
                <option value="main" disabled={forced}>Main thread</option>
              </select>
            </label>
          ))}
          {forced && <span className="note" style={{ margin: 0 }}>Over 100 pages runs in a worker.</span>}
        </div>
        {engine !== 'axonpdf' && (forced ? (
          <p className="warn">At 500 pages react-pdf took over 10 minutes in our run ({results_link}).</p>
        ) : !worker('react-pdf') && (
          <p className="warn">react-pdf on the main thread freezes the tab until it finishes.</p>
        ))}
        <div className="row">
          <button className="go" onClick={go} disabled={!!busy}>{busy ? 'Rendering…' : 'Render'}</button>
          <Ticker />
        </div>
        {busy && <p className="note" style={{ margin: 0 }}>{busy}</p>}
      </section>
      <p className="note">Page counts are approximate before the render; the results show the real count.</p>
      {error && <pre className="error">{error}</pre>}

      {shown.length > 0 && (
        <div className="cards">
          {shown.map((r) => (
            <article key={r.engine} className="card">
              <header>
                <h2 className={cls(r.engine)}>{ENGINES[r.engine].label}</h2>
                <a href={r.url} download={`ledger-${r.engine}-${r.pages}.pdf`}>Download PDF</a>
              </header>
              <dl>
                <div><dt>Time to PDF</dt><dd>{(r.ms / 1000).toFixed(2)} s</dd></div>
                <div><dt>Pages</dt><dd>{r.pages}</dd></div>
                <div><dt>File size</dt><dd>{mb(r.size)}</dd></div>
                <div><dt>Longest frame gap</dt><dd>{r.gap} ms</dd></div>
                <div><dt>Thread</dt><dd>{r.thread}</dd></div>
                <div><dt>JS heap peak</dt><dd>{r.heap === null ? 'not exposed' : mb(r.heap)}</dd></div>
              </dl>
              <iframe title={ENGINES[r.engine].label} src={r.url} />
            </article>
          ))}
        </div>
      )}

      {results.length > 0 && (
        <section className="history">
          <h2>All runs</h2>
          <table>
            <thead>
              <tr>{['Engine', 'Thread', 'Pages', 'Time to PDF', 'File size', 'Longest frame gap', 'JS heap peak'].map((h) => <th key={h}>{h}</th>)}</tr>
            </thead>
            <tbody>
              {results.map((r, i) => (
                <tr key={i}>
                  <td className={cls(r.engine)}>{ENGINES[r.engine].label}</td>
                  {[r.thread, r.pages, `${(r.ms / 1000).toFixed(2)} s`, mb(r.size), `${r.gap} ms`, r.heap === null ? 'not exposed' : mb(r.heap)].map((c, j) => <td key={j}>{c}</td>)}
                </tr>
              ))}
            </tbody>
          </table>
        </section>
      )}
    </main>
  );
}

createRoot(document.getElementById('root')).render(<App />);
