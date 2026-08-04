import { useMemo, useState } from 'react';
import { Link } from 'react-router-dom';
import { Play, Clock } from 'lucide-react';
import { LEARN_LESSONS } from '../lib/samples';
import { CodeEditor } from '../components/CodeEditor';
import { runFoxa } from '../lib/foxaInterpreter';
import './Learn.css';

export function Learn() {
  const [activeId, setActiveId] = useState(LEARN_LESSONS[0].id);
  const lesson = useMemo(() => LEARN_LESSONS.find((l) => l.id === activeId)!, [activeId]);
  const [code, setCode] = useState(lesson.code);
  const [output, setOutput] = useState<string[]>([]);

  const selectLesson = (id: string) => {
    const next = LEARN_LESSONS.find((l) => l.id === id)!;
    setActiveId(id);
    setCode(next.code);
    setOutput([]);
  };

  const run = () => {
    const result = runFoxa(code);
    setOutput(result.ok ? result.output : [...result.output, `error: ${result.error}`]);
  };

  return (
    <div className="learn container-wide">
      <header className="page-head anim-fade-up">
        <h1>Learn Foxa</h1>
        <p>Short lessons with runnable examples. Open the compiler anytime for a full workspace.</p>
      </header>

      <div className="learn-grid">
        <aside className="lesson-list anim-fade-up anim-delay-1">
          {LEARN_LESSONS.map((l, i) => (
            <button
              key={l.id}
              type="button"
              className={l.id === activeId ? 'lesson-item active' : 'lesson-item'}
              onClick={() => selectLesson(l.id)}
            >
              <span className="lesson-num">{String(i + 1).padStart(2, '0')}</span>
              <span className="lesson-meta">
                <strong>{l.title}</strong>
                <span>
                  <Clock size={12} /> {l.minutes} min
                </span>
              </span>
            </button>
          ))}
          <Link to="/compiler" className="btn btn-ghost" style={{ marginTop: '0.75rem' }}>
            Full compiler →
          </Link>
        </aside>

        <div className="lesson-panel anim-fade-up anim-delay-2">
          <h2>{lesson.title}</h2>
          <div className="lesson-body">
            {lesson.body.split('\n\n').map((para, i) => (
              <p key={i}>{para.replace(/`([^`]+)`/g, '$1')}</p>
            ))}
          </div>
          <CodeEditor value={code} onChange={setCode} minHeight={240} />
          <div className="lesson-actions">
            <button type="button" className="btn btn-primary" onClick={run}>
              <Play size={16} /> Run example
            </button>
          </div>
          {output.length > 0 && (
            <pre className="lesson-out mono">
              {output.map((line, i) => (
                <div key={i}>{line}</div>
              ))}
            </pre>
          )}
        </div>
      </div>
    </div>
  );
}
