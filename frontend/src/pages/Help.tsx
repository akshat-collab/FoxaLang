import { Link } from 'react-router-dom';
import './Help.css';

const FAQS = [
  {
    q: 'How do I run Foxa code?',
    a: 'Open Compiler, write or upload a .foxa file with fn main(), then press Run. Output appears in the console panel.',
  },
  {
    q: 'What does the ML Lab do?',
    a: 'Lab is a Colab-style notebook. Code cells run Foxa scripts. Train cells parse model/epochs/dataset settings and simulate a training loop with live loss and accuracy.',
  },
  {
    q: 'Can I upload my own files?',
    a: 'Yes. On Compiler and Lab, use the upload dropzone for .foxa, .txt, or .md files. You can also download your editor buffer as a .foxa file.',
  },
  {
    q: 'Is this the native foxac compiler?',
    a: 'The playground uses an in-browser Foxa interpreter for instant feedback. The native Rust toolchain (foxac) still lives in this repo for local builds.',
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
        <pre className="mono">{`fn main() { print("hi"); }
let mut x = 0;
while x < 3 { x = x + 1; }
for n in [1, 2] { print(n); }
struct Point { x: Int, y: Int }
match Some(1) { Some(v) => print(v), None => print(0) }`}</pre>
      </section>
    </div>
  );
}
