import { useState } from 'react';
import { Link } from 'react-router-dom';
import { CodeEditor } from '../components/CodeEditor';
import { runFoxa } from '../lib/foxaInterpreter';
import { SAMPLES } from '../lib/samples';
import './Home.css';

export function Home() {
  const [code, setCode] = useState(SAMPLES.hello);
  const [out, setOut] = useState<string[]>([]);

  return (
    <div className="home">
      <div className="home-intro">
        <h1>Foxa playground</h1>
        <p>
          Browser toolchain for the Foxa language — edit <span className="mono">.foxa</span> files, run{' '}
          <span className="mono">fn main</span>, and train notebook cells. Same surface as{' '}
          <span className="mono">foxa show</span> / <span className="mono">foxa fn</span> on the CLI.
        </p>
        <div className="home-links">
          <Link to="/compiler" className="btn btn-primary">
            Playground
          </Link>
          <Link to="/lab" className="btn btn-ghost">
            ML Lab
          </Link>
          <Link to="/learn" className="btn btn-ghost">
            Learn
          </Link>
        </div>
      </div>

      <section className="home-embed run-rail" data-state="active">
        <div className="panel-head">
          <span>main.foxa</span>
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
            {out.length === 0 ? <span className="dim">Output</span> : out.map((l, i) => <div key={i}>{l}</div>)}
          </pre>
        </div>
      </section>
    </div>
  );
}
