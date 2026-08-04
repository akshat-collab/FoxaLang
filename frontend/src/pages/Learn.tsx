import { useMemo, useState } from 'react';
import { Link } from 'react-router-dom';
import { LEARN_LESSONS } from '../lib/samples';
import { CodeEditor } from '../components/CodeEditor';
import { StatusChip } from '../components/StatusChip';
import { runFoxa } from '../lib/foxaInterpreter';
import './Learn.css';

export function Learn() {
  const [activeId, setActiveId] = useState(LEARN_LESSONS[0].id);
  const lesson = useMemo(() => LEARN_LESSONS.find((l) => l.id === activeId)!, [activeId]);
  const [code, setCode] = useState(lesson.code);
  const [output, setOutput] = useState<string[]>([]);
  const [state, setState] = useState<'idle' | 'running' | 'ok' | 'err'>('idle');

  const selectLesson = (id: string) => {
    const next = LEARN_LESSONS.find((l) => l.id === id)!;
    setActiveId(id);
    setCode(next.code);
    setOutput([]);
    setState('idle');
  };

  const run = () => {
    setState('running');
    requestAnimationFrame(() => {
      const result = runFoxa(code);
      setOutput(result.ok ? result.output : [...result.output, `error: ${result.error}`]);
      setState(result.ok ? 'ok' : 'err');
    });
  };

  return (
    <div className="learn">
      <aside className="learn-toc" aria-label="Lessons">
        <div className="learn-toc-head">Learn</div>
        <nav className="learn-toc-list">
          {LEARN_LESSONS.map((l) => (
            <button
              key={l.id}
              type="button"
              className={l.id === activeId ? 'learn-toc-item active' : 'learn-toc-item'}
              onClick={() => selectLesson(l.id)}
            >
              <span className="learn-toc-title">{l.title}</span>
              <span className="learn-toc-meta mono">{l.minutes}m</span>
            </button>
          ))}
        </nav>
        <Link to="/compiler" className="learn-toc-link">
          Open playground →
        </Link>
      </aside>

      <article className="learn-doc">
        <header className="learn-doc-head">
          <h1>{lesson.title}</h1>
          <p className="learn-lede">
            {lesson.body
              .split('\n\n')[0]
              ?.replace(/`([^`]+)`/g, '$1')}
          </p>
        </header>

        <div className="learn-prose">
          {lesson.body
            .split('\n\n')
            .slice(1)
            .map((para, i) => (
              <p key={i}>{para.replace(/`([^`]+)`/g, '$1')}</p>
            ))}
        </div>

        <section className="learn-try run-rail" data-state={state === 'idle' ? 'active' : state}>
          <div className="panel-head">
            <span>Try it</span>
            <StatusChip state={state} />
          </div>
          <CodeEditor value={code} onChange={setCode} minHeight={220} />
          <div className="learn-try-bar">
            <button type="button" className="btn btn-run" data-state={state === 'running' ? 'running' : undefined} onClick={run}>
              ▶ Run
            </button>
          </div>
          {output.length > 0 && (
            <pre className="console-body">
              {output.map((line, i) => (
                <div key={i} className={line.startsWith('error') ? 'err-line' : 'ok-line'}>
                  {line}
                </div>
              ))}
            </pre>
          )}
        </section>
      </article>
    </div>
  );
}
