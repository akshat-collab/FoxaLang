import { useState } from 'react';
import { Link } from 'react-router-dom';
import { CodeEditor } from '../components/CodeEditor';
import { runFoxa } from '../lib/foxaInterpreter';
import { SAMPLES } from '../lib/samples';
import './Home.css';

const PILLARS = [
  {
    title: 'Memory-safe by default',
    body: 'Unique ownership plus explicit ARC (shared T). No tracing GC in v1 — latency stays predictable.',
  },
  {
    title: 'Static types, no null',
    body: 'Structs, enums, Option / Result, and local inference. Public APIs stay explicitly annotated.',
  },
  {
    title: 'Systems performance',
    body: 'Built for services and tooling: interpreter for fast feedback, Cranelift JIT on the path to native codegen.',
  },
  {
    title: 'Batteries-included toolchain',
    body: 'foxa show, foxa fn, check, run, and a browser playground with a Colab-style ML Lab.',
  },
];

export function Home() {
  const [code, setCode] = useState(SAMPLES.hello);
  const [out, setOut] = useState<string[]>([]);

  return (
    <div className="home">
      <section className="home-hero" aria-label="foxaLang brand">
        <div className="home-logo-wrap home-anim home-anim-1">
          <img
            src="/foxalang-logo.png"
            alt="foxaLang — fox circuit logo"
            className="home-logo"
            width={685}
            height={511}
          />
          <div className="home-logo-glow" aria-hidden />
        </div>
        <p className="home-tagline home-anim home-anim-2">
          A modern systems language
          <span className="home-caret" aria-hidden />
        </p>
        <p className="home-lead home-anim home-anim-3">
          <strong>foxaLang</strong> is Foxa’s public face: safe by default, fast to iterate, and honest about systems
          work. Write <span className="mono">.foxa</span> sources, run <span className="mono">fn main</span>, and train
          notebook cells in the browser — the same ideas as <span className="mono">foxa show</span> and{' '}
          <span className="mono">foxa fn</span> on the CLI.
        </p>
        <div className="home-links home-anim home-anim-4">
          <Link to="/compiler" className="btn btn-primary">
            Open playground
          </Link>
          <Link to="/learn" className="btn btn-ghost">
            Learn Foxa
          </Link>
          <Link to="/lab" className="btn btn-ghost">
            ML Lab
          </Link>
        </div>
      </section>

      <section className="home-about home-anim home-anim-5" aria-labelledby="about-foxa">
        <h2 id="about-foxa">About the language</h2>
        <p>
          Foxa aims at the same problem space as Rust, Go, and Swift: memory-safe systems code with a sharp developer
          experience. Values move by default; sharing is explicit; errors travel through{' '}
          <span className="mono">Result</span> and <span className="mono">?</span>, not null.
        </p>
        <ul className="home-pillars">
          {PILLARS.map((p) => (
            <li key={p.title}>
              <h3>{p.title}</h3>
              <p>{p.body}</p>
            </li>
          ))}
        </ul>
      </section>

      <section className="home-embed run-rail home-anim home-anim-6" data-state="active">
        <div className="panel-head">
          <span>Try foxaLang</span>
          <button
            type="button"
            className="btn btn-run btn-sm"
            onClick={() => {
              const r = runFoxa(code);
              setOut(r.ok ? r.output : [...r.output, `error: ${r.error}`]);
            }}
          >
            ▶ Run
          </button>
        </div>
        <div className="home-embed-grid">
          <CodeEditor value={code} onChange={setCode} minHeight={220} />
          <pre className="console-body">
            {out.length === 0 ? (
              <span className="dim">Output appears here</span>
            ) : (
              out.map((l, i) => <div key={i}>{l}</div>)
            )}
          </pre>
        </div>
      </section>
    </div>
  );
}
