import { Link } from 'react-router-dom';
import { ArrowRight, Code2, FlaskConical, BookOpen } from 'lucide-react';
import './Home.css';

export function Home() {
  return (
    <div className="home">
      <section className="hero">
        <div className="hero-atmosphere" aria-hidden />
        <div className="container hero-content">
          <h1 className="hero-brand anim-fade-up">Foxa</h1>
          <p className="hero-tag anim-fade-up anim-delay-1">
            Write, run, and train — a Colab-style playground for the Foxa language.
          </p>
          <div className="hero-cta anim-fade-up anim-delay-2">
            <Link to="/compiler" className="btn btn-primary">
              Open compiler <ArrowRight size={18} />
            </Link>
            <Link to="/lab" className="btn btn-ghost">
              ML Lab
            </Link>
          </div>
        </div>
      </section>

      <section className="home-paths container">
        <Link to="/learn" className="path-tile anim-fade-up">
          <BookOpen size={22} />
          <h2>Learn Foxa</h2>
          <p>Guided lessons from hello world to structs and ML scripts.</p>
        </Link>
        <Link to="/compiler" className="path-tile anim-fade-up anim-delay-1">
          <Code2 size={22} />
          <h2>Online compiler</h2>
          <p>Edit .foxa files, upload sources, and run in the browser.</p>
        </Link>
        <Link to="/lab" className="path-tile anim-fade-up anim-delay-2">
          <FlaskConical size={22} />
          <h2>Model training</h2>
          <p>Notebook cells for scripts and simulated model training.</p>
        </Link>
      </section>
    </div>
  );
}
