import { Link } from 'react-router-dom';
import './Help.css';

const FAQS = [
  {
    q: 'How do I run Foxa code?',
    a: 'Open Compiler, write or upload a .foxa file with fn main(), then press Run or Show. Show mirrors the CLI command foxa show and prints compile status plus program output.',
  },
  {
    q: 'What is foxa show?',
    a: 'On the CLI: foxa show path/to/file.foxa compiles the file, runs main, and prints output. In the playground, use the Show button for the same report.',
  },
  {
    q: 'How do I create a function with Foxa?',
    a: 'Use Foxa fn syntax: fn name(params) -> Ret { ... }. CLI: foxa fn greet --params "name: String" --ret String. Playground: New fn button inserts a stub into the open .foxa file.',
  },
  {
    q: 'What does the ML Lab do?',
    a: 'Lab is a Colab-style notebook. Code cells run Foxa scripts. Train cells parse model/epochs/dataset settings and simulate a training loop with live loss and accuracy.',
  },
  {
    q: 'Can I upload my own files?',
    a: 'Yes. On Compiler and Lab, use the upload dropzone for .foxa files. The compiler keeps multiple file tabs. You can also download the buffer as .foxa.',
  },
  {
    q: 'Is this the native foxac compiler?',
    a: 'The playground uses an in-browser Foxa checker/interpreter. The native Rust toolchain (foxa show / run / check / fn) still lives in this repo for local builds.',
  },
  {
    q: 'Where should I start learning?',
    a: 'Go to Learn for short lessons with runnable examples, then move to Compiler and Lab for longer work.',
  },
];

export function Help() {
  return (
    <div className="help container">
      <header className="page-head anim-fade-up">
        <h1>Help</h1>
        <p>Quick answers for the Foxa playground — compiler, lab, and learning path.</p>
      </header>

      <div className="help-quick anim-fade-up anim-delay-1">
        <Link to="/learn">Learn Foxa →</Link>
        <Link to="/compiler">Online compiler →</Link>
        <Link to="/lab">ML Lab →</Link>
        <Link to="/feedback">Send feedback →</Link>
      </div>

      <div className="faq-list anim-fade-up anim-delay-2">
        {FAQS.map((item) => (
          <details key={item.q} className="faq">
            <summary>{item.q}</summary>
            <p>{item.a}</p>
          </details>
        ))}
      </div>

      <section className="help-cheatsheet anim-fade-up anim-delay-3">
        <h2>Language cheatsheet</h2>
        <pre className="mono">{`fn main() { show("hi"); }
fn add(a: Int, b: Int) -> Int { a + b }
let mut x = 0;
while x < 3 { x = x + 1; }
for n in [1, 2] { show(n); }
struct Point { x: Int, y: Int }
match Some(1) { Some(v) => show(v), None => show(0) }`}</pre>
        <h2 style={{ marginTop: '1.5rem' }}>CLI</h2>
        <pre className="mono">{`foxa show examples/hello.foxa
foxa fn greet --params "name: String" --ret String --file main.foxa
foxa run main.foxa
foxa check main.foxa`}</pre>
      </section>
    </div>
  );
}
