import { useState } from 'react';
import { Play, RotateCcw, Download, Eraser } from 'lucide-react';
import { CodeEditor } from '../components/CodeEditor';
import { FileUploader } from '../components/FileUploader';
import { runFoxa, type RunResult } from '../lib/foxaInterpreter';
import { SAMPLES } from '../lib/samples';
import './Compiler.css';

export function Compiler() {
  const [filename, setFilename] = useState('main.foxa');
  const [source, setSource] = useState(SAMPLES.hello);
  const [result, setResult] = useState<RunResult | null>(null);
  const [running, setRunning] = useState(false);

  const run = () => {
    setRunning(true);
    requestAnimationFrame(() => {
      const r = runFoxa(source);
      setResult(r);
      setRunning(false);
    });
  };

  const download = () => {
    const blob = new Blob([source], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = filename.endsWith('.foxa') ? filename : `${filename}.foxa`;
    a.click();
    URL.revokeObjectURL(url);
  };

  return (
    <div className="compiler container-wide">
      <header className="page-head anim-fade-up">
        <h1>Online compiler</h1>
        <p>Edit Foxa in the browser, upload files, and run with the playground interpreter.</p>
      </header>

      <div className="compiler-toolbar anim-fade-up anim-delay-1">
        <input
          className="filename-input mono"
          value={filename}
          onChange={(e) => setFilename(e.target.value)}
          aria-label="Filename"
        />
        <div className="toolbar-actions">
          <select
            className="sample-select"
            defaultValue=""
            onChange={(e) => {
              const key = e.target.value as keyof typeof SAMPLES;
              if (key && SAMPLES[key]) {
                setSource(SAMPLES[key]);
                setFilename(`${key}.foxa`);
                setResult(null);
              }
              e.target.value = '';
            }}
            aria-label="Load sample"
          >
            <option value="" disabled>
              Samples…
            </option>
            <option value="hello">hello.foxa</option>
            <option value="features">features.foxa</option>
            <option value="loops">loops.foxa</option>
            <option value="functions">functions.foxa</option>
          </select>
          <button type="button" className="btn btn-primary btn-sm" onClick={run} disabled={running}>
            <Play size={14} /> {running ? 'Running…' : 'Run'}
          </button>
          <button
            type="button"
            className="btn btn-ghost btn-sm"
            onClick={() => {
              setSource(SAMPLES.hello);
              setFilename('main.foxa');
              setResult(null);
            }}
          >
            <RotateCcw size={14} /> Reset
          </button>
          <button type="button" className="btn btn-ghost btn-sm" onClick={download}>
            <Download size={14} /> Save
          </button>
          <button type="button" className="btn btn-ghost btn-sm" onClick={() => setResult(null)}>
            <Eraser size={14} /> Clear out
          </button>
        </div>
      </div>

      <div className="compiler-upload anim-fade-up anim-delay-1">
        <FileUploader
          onLoad={(name, content) => {
            setFilename(name);
            setSource(content);
            setResult(null);
          }}
        />
      </div>

      <div className="compiler-grid anim-fade-up anim-delay-2">
        <CodeEditor value={source} onChange={setSource} minHeight={420} />
        <div className={`console ${result && !result.ok ? 'err' : ''}`}>
          <div className="console-head">
            <span>Output</span>
            {result && (
              <span className="console-meta">
                {result.ok ? 'ok' : 'error'} · {result.elapsedMs.toFixed(1)} ms
              </span>
            )}
          </div>
          <pre className="console-body mono">
            {!result && <span className="dim">Press Run to execute fn main()</span>}
            {result?.output.map((line, i) => (
              <div key={i}>{line}</div>
            ))}
            {result?.error && <div className="err-line">error: {result.error}</div>}
          </pre>
        </div>
      </div>
    </div>
  );
}
