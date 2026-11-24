import { memo, useEffect, useState } from 'react';
import { createRoot } from 'react-dom/client';
import { ENGINES, run } from './measure.js';
import { CodeHighlight, domPrimitives } from '@axonpack/react-pretty-print';
import { DARK_THEME, LIGHT_THEME } from '@axonpack/react-pretty-print/themes';
import { PACKAGE, snippet } from './snippets.js';
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

function Numbers({ r }) {
  return (
    <dl>
      <div><dt>Time to PDF</dt><dd>{(r.ms / 1000).toFixed(2)} s</dd></div>
      <div><dt>Pages</dt><dd>{r.pages}</dd></div>
      <div><dt>File size</dt><dd>{mb(r.size)}</dd></div>
      <div><dt>Longest frame gap</dt><dd>{r.gap} ms</dd></div>
      <div><dt>Thread</dt><dd>{r.thread}</dd></div>
      {r.heap !== null && <div className="small"><dt>Page heap (excludes worker)</dt><dd>{mb(r.heap)}</dd></div>}
    </dl>
  );
}

const download = (r) => <a href={r.url} download={`ledger-${r.engine}-${r.pages}.pdf`}>Download PDF</a>;
// ponytail: older runs keep their PDF for preview and download; past this many, the oldest is
// dropped and its blob URL revoked.
const KEEP = 20;

// Its own component, so the once-a-second updates of the current run don't re-render it.
const Previous = memo(function Previous({ runs }) {
  const [open, setOpen] = useState(null);
  if (!runs.length) return null;
  return (
        <section className="history">
          <h2>Previous runs</h2>
          {runs.map((r) => (
            <details key={r.id} name="runs" className="run" onToggle={(e) => e.currentTarget.open ? setOpen(r.id) : open === r.id && setOpen(null)}>
              <summary>
                <span className={cls(r.engine)}>{ENGINES[r.engine].label}</span>
                <span>{r.thread}</span>
                <span>{r.pages} pages</span>
                <span>{r.layout === "flowing" ? "one flowing page" : "sections"}</span>
                <span>{(r.ms / 1000).toFixed(2)} s</span>
              </summary>
              {open === r.id && (
                <>
                  <Numbers r={r} />
                  <p className="dl">{download(r)}</p>
                  <iframe title={ENGINES[r.engine].label} src={r.url} />
                </>
              )}
            </details>
          ))}
        </section>
  );
});

function Live({ engine, live }) {
  return (
    <article className="card">
      <header>
        <h2 className={cls(engine)}>{ENGINES[engine].label}</h2>
        <span className="running">{live ? 'Running' : 'Waiting'}</span>
      </header>
      {live && (
        <dl>
          <div><dt>Stage</dt><dd>{live.stage}</dd></div>
          <div><dt>Elapsed</dt><dd>{(live.ms / 1000).toFixed(0)} s</dd></div>
          <div><dt>Longest freeze so far</dt><dd>{live.gap} ms</dd></div>
        </dl>
      )}
    </article>
  );
}

const dark = matchMedia('(prefers-color-scheme: dark)');
function useDark() {
  const [isDark, setDark] = useState(dark.matches);
  useEffect(() => {
    const on = (e) => setDark(e.matches);
    dark.addEventListener('change', on);
    return () => dark.removeEventListener('change', on);
  }, []);
  return isDark;
}

// The package name is marked, since it is the one thing that changes between the engines.
const matchers = Object.fromEntries(
  Object.entries(PACKAGE).map(([e, pkg]) => [e, { pattern: new RegExp(pkg.replace(/[/@-]/g, '\\$&'), 'g'), invalid: false }]),
);

function Code({ engine, text }) {
  const [copied, setCopied] = useState(false);
  const theme = useDark() ? DARK_THEME : LIGHT_THEME;
  const copy = () =>
    navigator.clipboard.writeText(text).then(
      () => {
        setCopied(true);
        setTimeout(() => setCopied(false), 1500);
      },
      () => {},
    );
  return (
    <figure className="code">
      <figcaption>
        <span className={cls(engine)}>{ENGINES[engine].label}</span>
        <button type="button" onClick={copy}>{copied ? 'Copied' : 'Copy'}</button>
      </figcaption>
      <pre>
        <CodeHighlight primitives={domPrimitives} code={text} language="tsx" format={false} theme={theme} matcher={matchers[engine]} />
      </pre>
    </figure>
  );
}

const cls = (e) => (e === 'axonpdf' ? 'axon' : 'rpdf');

function App() {
  const [preset, setPreset] = useState('100');
  const [custom, setCustom] = useState(250);
  const [engine, setEngine] = useState('both');
  const [layout, setLayout] = useState('sections');
  const [inWorker, setInWorker] = useState({ axonpdf: true, 'react-pdf': true });
  const [busy, setBusy] = useState('');
  // This run: one slot per engine: `null` while waiting, `{ live }` while running, then the result.
  const [current, setCurrent] = useState([]);
  const [previous, setPrevious] = useState([]);
  const [error, setError] = useState('');

  const pages = preset === 'custom' ? Math.max(1, Math.min(5000, Math.round(custom) || 1)) : +preset;
  // The main thread is allowed only up to 100 pages; above that both engines run in a worker.
  const forced = pages > 100;
  const worker = (e) => forced || inWorker[e];
  const [order, setOrder] = useState([]);
  const picked = engine === 'both' ? ['axonpdf', 'react-pdf'] : [engine];

  async function go() {
    setError('');
    const finished = current.filter((r) => r?.id);
    setPrevious((old) => {
      const all = [...finished, ...old];
      all.slice(KEEP).forEach((r) => URL.revokeObjectURL(r.url));
      return all.slice(0, KEEP);
    });
    setCurrent(picked.map(() => null));
    setOrder(picked);
    try {
      for (const [i, e] of picked.entries()) {
        setBusy(`Rendering with ${ENGINES[e].label}…`);
        const onTick = (live) => setCurrent((c) => c.map((x, j) => (j === i ? { live } : x)));
        const r = { ...(await run({ engine: e, pages, layout, useWorker: worker(e), onTick })), id: crypto.randomUUID() };
        setCurrent((c) => c.map((x, j) => (j === i ? r : x)));
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
          <span className="name">Layout</span>
          <Segmented label="Layout" value={layout} onChange={setLayout} options={[['sections', 'Sections'], ['flowing', 'One flowing page']]} />
          <span className="note" style={{ margin: 0 }}>react-pdf slows down sharply when one <code>&lt;Page&gt;</code> flows over hundreds of pages.</span>
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
          <p className="warn">{layout === 'flowing' ? 'At 500 pages on one flowing page, react-pdf had no PDF after 10 minutes in our run' : 'At 500 pages, react-pdf took about 46 s in our run'} ({results_link}).</p>
        ) : !worker('react-pdf') && (
          <p className="warn">react-pdf on the main thread freezes the tab until it finishes.</p>
        ))}
        <div className="row">
          <button className="go" onClick={go} disabled={!!busy}>{busy ? 'Rendering…' : 'Render'}</button>
          <Ticker />
        </div>
        {busy && <p className="note" style={{ margin: 0 }}>{busy}</p>}
      </section>
      <section className="codes" aria-label="Code for these choices">
        {picked.map((e) => (
          <Code key={e} engine={e} text={snippet({ engine: e, layout, worker: worker(e), pages })} />
        ))}
      </section>
      <p className="note">Page counts are approximate before the render; the results show the real count.</p>
      <section className="memory">
        <h2>Memory</h2>
        <p>A web page can't read how much memory a worker uses, so this page doesn't show it. To see it, open Chrome DevTools (<kbd>⌥⌘I</kbd> on macOS, <kbd>F12</kbd> on Windows and Linux), go to the <b>Memory</b> tab, and look under <b>Select JavaScript VM instance</b>. Each worker has its own row with its current heap size, and it updates while the render runs.</p>
        <img src="./devtools-memory.png" width="554" height="864" alt="Chrome DevTools Memory tab after a 100-page run: axonpdf.worker uses 17.8 MB and react-pdf.worker uses 271 MB" />
      </section>
      {error && <pre className="error">{error}</pre>}

      {current.length > 0 && (
        <div className="cards">
          {current.map((r, i) =>
            r?.id ? (
              <article key={r.id} className="card">
                <header>
                  <h2 className={cls(r.engine)}>{ENGINES[r.engine].label}</h2>
                  {download(r)}
                </header>
                <Numbers r={r} />
                <iframe title={ENGINES[r.engine].label} src={r.url} />
              </article>
            ) : (
              <Live key={i} engine={order[i]} live={r?.live} />
            ),
          )}
        </div>
      )}

      <Previous runs={previous} />
    </main>
  );
}

createRoot(document.getElementById('root')).render(<App />);
