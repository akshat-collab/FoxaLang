import { useMemo, useState } from 'react';
import { Play, RotateCcw, Download, Eraser, Eye, Plus, FileCode2, CheckCircle2 } from 'lucide-react';
import { CodeEditor } from '../components/CodeEditor';
import { FileUploader } from '../components/FileUploader';
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

export function Compiler() {
  const [files, setFiles] = useState<FoxaFile[]>([{ name: 'main.foxa', content: SAMPLES.hello }]);
  const [active, setActive] = useState(0);
  const [result, setResult] = useState<RunResult | null>(null);
  const [showLines, setShowLines] = useState<string[] | null>(null);
  const [compile, setCompile] = useState<CompileReport | null>(null);
  const [running, setRunning] = useState(false);
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
    setRunning(true);
    setShowLines(null);
    requestAnimationFrame(() => {
      const report = compileFoxaFile(filename, source);
      setCompile(report);
      const r = runFoxa(source);
      setResult(r);
      setRunning(false);
    });
  };

  const show = () => {
    setRunning(true);
    requestAnimationFrame(() => {
      const report = compileFoxaFile(filename, source);
      setCompile(report);
      if (!report.ok) {
        setResult(null);
        setShowLines([
          `=== foxa show: ${filename} ===`,
          'compile: failed',
          '--- diagnostics ---',
          ...report.diagnostics.map((d) => `${d.severity}${d.line ? `:${d.line}` : ''}: ${d.message}`),
        ]);
        setRunning(false);
        return;
      }
      const r = runFoxa(source);
      setResult(r);
      setShowLines(formatShowReport(filename, report, r.output, r.error));
      setRunning(false);
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
    const next = source.trimEnd() + '\n\n' + stub;
    setSource(next);
    setFnOpen(false);
    setShowLines([`Added fn ${fnName} — like \`foxa fn ${fnName}\``]);
  };

  return (
    <div className="compiler container-wide">
      <header className="page-head anim-fade-up">
        <h1>Online compiler</h1>
        <p>
          File-based Foxa compiler: create <code className="mono">fn</code> with Foxa syntax,{' '}
          <code className="mono">show</code> program output, upload <code className="mono">.foxa</code>{' '}
          files.
        </p>
      </header>

      <div className="file-tabs anim-fade-up anim-delay-1">
        {files.map((f, i) => (
          <button
            key={`${f.name}-${i}`}
            type="button"
            className={i === active ? 'file-tab active' : 'file-tab'}
            onClick={() => {
              setActive(i);
              setResult(null);
              setShowLines(null);
            }}
          >
            <FileCode2 size={14} />
            {f.name}
          </button>
        ))}
        <button
          type="button"
          className="file-tab add"
          onClick={() => {
            const name = `file${files.length + 1}.foxa`;
            setFiles((prev) => [
              ...prev,
              {
                name,
                content: `fn main() {\n    show("new file");\n}\n`,
              },
            ]);
            setActive(files.length);
          }}
        >
          <Plus size={14} /> File
        </button>
      </div>

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
                setShowLines(null);
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
          <button type="button" className="btn btn-mint btn-sm" onClick={show} disabled={running}>
            <Eye size={14} /> Show
          </button>
          <button type="button" className="btn btn-primary btn-sm" onClick={run} disabled={running}>
            <Play size={14} /> {running ? 'Running…' : 'Run'}
          </button>
          <button type="button" className="btn btn-ghost btn-sm" onClick={checkOnly}>
            <CheckCircle2 size={14} /> Check
          </button>
          <button type="button" className="btn btn-ghost btn-sm" onClick={() => setFnOpen((v) => !v)}>
            <Plus size={14} /> New fn
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
            }}
          >
            <RotateCcw size={14} /> Reset
          </button>
          <button type="button" className="btn btn-ghost btn-sm" onClick={download}>
            <Download size={14} /> Save
          </button>
          <button
            type="button"
            className="btn btn-ghost btn-sm"
            onClick={() => {
              setResult(null);
              setShowLines(null);
            }}
          >
            <Eraser size={14} /> Clear out
          </button>
        </div>
      </div>

      {fnOpen && (
        <div className="fn-panel anim-fade-up">
          <div className="field">
            <label htmlFor="fn-name">fn name</label>
            <input id="fn-name" className="mono" value={fnName} onChange={(e) => setFnName(e.target.value)} />
          </div>
          <div className="field">
            <label htmlFor="fn-params">params</label>
            <input
              id="fn-params"
              className="mono"
              value={fnParams}
              onChange={(e) => setFnParams(e.target.value)}
              placeholder="a: Int, b: Int"
            />
          </div>
          <div className="field">
            <label htmlFor="fn-ret">return type</label>
            <input
              id="fn-ret"
              className="mono"
              value={fnRet}
              onChange={(e) => setFnRet(e.target.value)}
              placeholder="Int (optional)"
            />
          </div>
          <button type="button" className="btn btn-primary btn-sm" onClick={insertFn}>
            Insert Foxa fn
          </button>
        </div>
      )}

      <div className="compiler-upload anim-fade-up anim-delay-1">
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
            {!consoleLines && <span className="dim">Press Show (foxa show) or Run to execute fn main()</span>}
            {consoleLines?.map((line, i) => (
              <div key={i}>{line}</div>
            ))}
          </pre>
          {compile && (
            <div className="compile-footer mono">
              {compile.ok ? 'compile ok' : 'compile errors'} · {compile.functions.length} fn
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
