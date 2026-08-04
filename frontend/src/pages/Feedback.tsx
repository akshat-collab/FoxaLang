import { useState, type FormEvent } from 'react';
import { Send, CheckCircle2 } from 'lucide-react';
import './Feedback.css';

type FormState = {
  name: string;
  email: string;
  category: string;
  message: string;
};

const INITIAL: FormState = {
  name: '',
  email: '',
  category: 'general',
  message: '',
};

export function Feedback() {
  const [form, setForm] = useState<FormState>(INITIAL);
  const [sent, setSent] = useState(false);
  const [error, setError] = useState('');

  const submit = (e: FormEvent) => {
    e.preventDefault();
    if (!form.message.trim() || form.message.trim().length < 8) {
      setError('Please write at least a short message (8+ characters).');
      return;
    }
    setError('');
    const entry = {
      ...form,
      at: new Date().toISOString(),
    };
    const prev = JSON.parse(localStorage.getItem('foxa-feedback') ?? '[]') as unknown[];
    localStorage.setItem('foxa-feedback', JSON.stringify([entry, ...prev].slice(0, 50)));
    setSent(true);
    setForm(INITIAL);
  };

  return (
    <div className="feedback container">
      <header className="page-head anim-fade-up">
        <h1>Feedback</h1>
        <p>Tell us what works, what breaks, and what you want next in the Foxa playground.</p>
      </header>

      {sent ? (
        <div className="thanks anim-fade-up">
          <CheckCircle2 size={36} color="var(--mint)" />
          <h2>Thanks — feedback saved locally</h2>
          <p>Your note is stored in this browser so you can keep iterating without a backend.</p>
          <button type="button" className="btn btn-primary" onClick={() => setSent(false)}>
            Send another
          </button>
        </div>
      ) : (
        <form className="feedback-form anim-fade-up anim-delay-1" onSubmit={submit}>
          <div className="form-row">
            <div className="field">
              <label htmlFor="name">Name</label>
              <input
                id="name"
                value={form.name}
                onChange={(e) => setForm((f) => ({ ...f, name: e.target.value }))}
                placeholder="Optional"
              />
            </div>
            <div className="field">
              <label htmlFor="email">Email</label>
              <input
                id="email"
                type="email"
                value={form.email}
                onChange={(e) => setForm((f) => ({ ...f, email: e.target.value }))}
                placeholder="Optional"
              />
            </div>
          </div>
          <div className="field">
            <label htmlFor="category">Category</label>
            <select
              id="category"
              value={form.category}
              onChange={(e) => setForm((f) => ({ ...f, category: e.target.value }))}
            >
              <option value="general">General</option>
              <option value="compiler">Compiler / interpreter</option>
              <option value="lab">ML Lab</option>
              <option value="learn">Learn section</option>
              <option value="bug">Bug report</option>
              <option value="idea">Feature idea</option>
            </select>
          </div>
          <div className="field">
            <label htmlFor="message">Message</label>
            <textarea
              id="message"
              value={form.message}
              onChange={(e) => setForm((f) => ({ ...f, message: e.target.value }))}
              placeholder="What should we improve?"
              required
            />
          </div>
          {error && <p className="form-error">{error}</p>}
          <button type="submit" className="btn btn-primary">
            <Send size={16} /> Submit feedback
          </button>
        </form>
      )}
    </div>
  );
}
