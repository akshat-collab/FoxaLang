import { useMemo, useState } from 'react';
import { CodeEditor } from '../components/CodeEditor';
import { FileUploader } from '../components/FileUploader';
import { StatusChip } from '../components/StatusChip';
import { runFoxa, type RunResult } from '../lib/foxaInterpreter';
import {
  compileFoxaFile,
  formatShowReport,
  scaffoldFoxaFn,
  type CompileReport,
} from '../lib/foxaCompile';
import { SAMPLES } from '../lib/samples';
import './Compiler.css';

type FoxaFile = { name: string; content: string };
type RunState = 'idle' | 'running' | 'ok' | 'err';

export function Compiler() {
  const [files, setFiles] = useState<FoxaFile[]>([{ name: 'main.foxa', content: SAMPLES.hello }]);
  const [active, setActive] = useState(0);
  const [result, setResult] = useState<RunResult | null>(null);
  const [showLines, setShowLines] = useState<string[] | null>(null);
  const [compile, setCompile] = useState<CompileReport | null>(null);
  const [runState, setRunState] = useState<RunState>('idle');
  const [fnOpen, setFnOpen] = useState(false);
  const [fnName, setFnName] = useState('greet');
  const [fnParams, setFnParams] = useState('name: String');
  const [fnRet, setFnRet] = useState('String');

  const current = files[active] ?? files[0];
  const filename = current?.name ?? 'main.foxa';
  const source = current?.content ?? '';

  const setSource = (content: string) => {
    setFiles((prev) => prev.map((f, i) => (i === active ? { ...f, content } : f)));
  };

  const setFilename = (name: string) => {
    setFiles((prev) => prev.map((f, i) => (i === active ? { ...f, name } : f)));
  };

  const consoleLines = useMemo(() => {
    if (showLines) return showLines;
    if (!result) return null;
    const lines = [...result.output];
    if (result.error) lines.push(`error: ${result.error}`);
    return lines;
  }, [result, showLines]);

  const run = () => {
    setRunState('running');
    setShowLines(null);
    requestAnimationFrame(() => {
      const report = compileFoxaFile(filename, source);
      const r = runFoxa(source);
      setCompile(report);
      setResult(r);
      setRunState(r.ok ? 'ok' : 'err');
    });
  };

  const show = () => {
    setRunState('running');
    requestAnimationFrame(() => {
      const report = compileFoxaFile(filename, source);
      if (!report.ok) {
        setResult(null);
        setCompile(report);
        setShowLines([
          `=== foxa show: ${filename} ===`,
          'compile: failed',
          '--- diagnostics ---',
          ...report.diagnostics.map((d) => `${d.severity}${d.line ? `:${d.line}` : ''}: ${d.message}`),
        ]);
        setRunState('err');
        return;
      }
      const r = runFoxa(source);
      setResult(r);
      setCompile(report);
      setShowLines(formatShowReport(filename, report, r.output, r.error));
      setRunState(r.ok ? 'ok' : 'err');
    });
  };

  const checkOnly = () => {
    const report = compileFoxaFile(filename, source);
    setCompile(report);
    setResult(null);
    setShowLines([
      `=== foxa check: ${filename} ===`,
      ...report.diagnostics.map((d) => `${d.severity}${d.line ? `:${d.line}` : ''}: ${d.message}`),
    ]);
    setRunState(report.ok ? 'ok' : 'err');
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

  const insertFn = () => {
    const stub = scaffoldFoxaFn({ name: fnName, params: fnParams, ret: fnRet || undefined });
    setSource(source.trimEnd() + '\n\n' + stub);
    setFnOpen(false);
  };

  const meta =
    result && runState !== 'running'
      ? `${result.elapsedMs.toFixed(1)}ms`
      : compile
        ? `${compile.functions.length} fn`
        : undefined;

  return (
    <div className="playground">
      <div className="toolbar">
        <button
          type="button"
          className="btn btn-run"
          data-state={runState === 'running' ? 'running' : undefined}
          onClick={run}
          disabled={runState === 'running'}
        >
          ▶ Run
        </button>
        <button type="button" className="btn btn-ghost btn-sm" onClick={show} disabled={runState === 'running'}>
          Show
        </button>
        <button type="button" className="btn btn-ghost btn-sm" onClick={checkOnly}>
          Check
        </button>
        <span className="toolbar-sep" />
        <StatusChip state={runState} detail={meta} />
        <span className="toolbar-sep" />
        <select
          className="pg-select mono"
          defaultValue=""
          aria-label="Load sample"
          onChange={(e) => {
            const key = e.target.value as keyof typeof SAMPLES;
            if (key && SAMPLES[key]) {
              setSource(SAMPLES[key]);
              setFilename(`${key}.foxa`);
              setResult(null);
              setShowLines(null);
              setRunState('idle');
            }
            e.target.value = '';
          }}
        >
          <option value="" disabled>
            Sample…
          </option>
          <option value="hello">hello.foxa</option>
          <option value="features">features.foxa</option>
          <option value="loops">loops.foxa</option>
          <option value="functions">functions.foxa</option>
        </select>
        <button type="button" className="btn btn-ghost btn-sm" onClick={() => setFnOpen((v) => !v)}>
          New fn
        </button>
        <FileUploader
          accept=".foxa,.txt"
          onLoad={(name, content) => {
            const fname = name.endsWith('.foxa') ? name : `${name}.foxa`;
            setFiles((prev) => {
              const idx = prev.findIndex((f) => f.name === fname);
              if (idx >= 0) {
                const next = [...prev];
                next[idx] = { name: fname, content };
                setActive(idx);
                return next;
              }
              setActive(prev.length);
              return [...prev, { name: fname, content }];
            });
            setResult(null);
            setShowLines(null);
            setRunState('idle');
          }}
        />
        <button type="button" className="btn btn-ghost btn-sm" onClick={download}>
          Save
        </button>
        <button
          type="button"
          className="btn btn-ghost btn-sm"
          onClick={() => {
            setFiles([{ name: 'main.foxa', content: SAMPLES.hello }]);
            setActive(0);
            setResult(null);
            setShowLines(null);
            setCompile(null);
            setRunState('idle');
          }}
        >
          Reset
        </button>
      </div>

      <div className="pg-tabs">
        {files.map((f, i) => (
          <button
            key={`${f.name}-${i}`}
            type="button"
            className={i === active ? 'pg-tab active' : 'pg-tab'}
            onClick={() => {
              setActive(i);
              setResult(null);
              setShowLines(null);
              setRunState('idle');
            }}
          >
            {f.name}
          </button>
        ))}
        <button
          type="button"
          className="pg-tab add"
          onClick={() => {
            const name = `file${files.length + 1}.foxa`;
            setFiles((prev) => [...prev, { name, content: `fn main() {\n    show("new file");\n}\n` }]);
            setActive(files.length);
          }}
        >
          +
        </button>
        <input
          className="pg-filename mono"
          value={filename}
          onChange={(e) => setFilename(e.target.value)}
          aria-label="Filename"
        />
      </div>

      {fnOpen && (
        <div className="pg-fnbar">
          <div className="field">
            <label htmlFor="fn-name">name</label>
            <input id="fn-name" className="mono" value={fnName} onChange={(e) => setFnName(e.target.value)} />
          </div>
          <div className="field">
            <label htmlFor="fn-params">params</label>
            <input id="fn-params" className="mono" value={fnParams} onChange={(e) => setFnParams(e.target.value)} />
          </div>
          <div className="field">
            <label htmlFor="fn-ret">ret</label>
            <input id="fn-ret" className="mono" value={fnRet} onChange={(e) => setFnRet(e.target.value)} />
          </div>
          <button type="button" className="btn btn-primary btn-sm" onClick={insertFn}>
            Insert fn
          </button>
        </div>
      )}

      <div className="pg-split">
        <section className={`pg-editor run-rail`} data-state={runState === 'idle' ? 'active' : runState}>
          <div className="panel-head">
            <span>Editor</span>
            <span className="mono" style={{ textTransform: 'none', letterSpacing: 0, fontWeight: 400 }}>
              {filename}
            </span>
          </div>
          <CodeEditor value={source} onChange={setSource} minHeight="100%" className="pg-code" />
        </section>

        <section className="pg-console">
          <div className="panel-head">
            <span>Output</span>
            <span className="mono" style={{ textTransform: 'none', letterSpacing: 0, fontWeight: 400 }}>
              {runState === 'running' ? 'executing…' : compile?.ok === false ? 'diagnostics' : 'console'}
            </span>
          </div>
          <pre className="console-body pg-out">
            {!consoleLines && <span className="dim">▶ Run or Show — executes fn main()</span>}
            {consoleLines?.map((line, i) => (
              <div key={i} className={line.startsWith('error') || line.includes('failed') ? 'err-line' : undefined}>
                {line}
              </div>
            ))}
          </pre>
        </section>
      </div>
    </div>
  );
}
