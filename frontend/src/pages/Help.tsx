import { Link } from 'react-router-dom';
import './Help.css';

const FAQS = [
  {
    q: 'How do I run Foxa code?',
    a: 'Open Playground, write or upload a .foxa file with fn main(), then Run or Show. Show mirrors foxa show on the CLI.',
  },
  {
    q: 'What is foxa show?',
    a: 'CLI: foxa show path/to/file.foxa compiles, runs main, and prints output. Playground Show button matches that report.',
  },
  {
    q: 'How do I create a function?',
    a: 'Use Foxa fn syntax: fn name(params) -> Ret { ... }. CLI: foxa fn … — Playground: New fn.',
  },
  {
    q: 'What is ML Lab?',
    a: 'Notebook cells for code and training. Each cell has its own run control and Out[n] panel.',
  },
  {
    q: 'File support?',
    a: 'Upload .foxa tabs in Playground. Lab accepts .foxa / .md into new cells.',
  },
];

export function Help() {
  return (
    <div className="help">
      <header className="help-head">
        <h1>Help</h1>
        <p>Playground, Lab, and CLI quick reference.</p>
      </header>

      <div className="help-links">
        <Link to="/learn">Learn</Link>
        <Link to="/compiler">Playground</Link>
        <Link to="/lab">Lab</Link>
        <Link to="/feedback">Feedback</Link>
      </div>

      <div className="faq-list">
        {FAQS.map((item) => (
          <details key={item.q} className="faq">
            <summary>{item.q}</summary>
            <p>{item.a}</p>
          </details>
        ))}
      </div>

      <section className="help-block">
        <h2>Language</h2>
        <pre className="console-body">{`fn main() { show("hi"); }
fn add(a: Int, b: Int) -> Int { a + b }
let mut x = 0;
while x < 3 { x = x + 1; }`}</pre>
      </section>

      <section className="help-block">
        <h2>CLI</h2>
        <pre className="console-body">{`foxa show examples/hello.foxa
foxa fn greet --params "name: String" --ret String --file main.foxa
foxa check main.foxa
foxa run main.foxa`}</pre>
      </section>
    </div>
  );
}
