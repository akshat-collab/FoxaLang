import { useState, type FormEvent } from 'react';
import './Feedback.css';

type FormState = {
  name: string;
  email: string;
  category: string;
  message: string;
};

const INITIAL: FormState = { name: '', email: '', category: 'general', message: '' };

export function Feedback() {
  const [form, setForm] = useState<FormState>(INITIAL);
  const [sent, setSent] = useState(false);
  const [error, setError] = useState('');

  const submit = (e: FormEvent) => {
    e.preventDefault();
    if (!form.message.trim() || form.message.trim().length < 8) {
      setError('Message needs at least 8 characters.');
      return;
    }
    setError('');
    const entry = { ...form, at: new Date().toISOString() };
    const prev = JSON.parse(localStorage.getItem('foxa-feedback') ?? '[]') as unknown[];
    localStorage.setItem('foxa-feedback', JSON.stringify([entry, ...prev].slice(0, 50)));
    setSent(true);
    setForm(INITIAL);
  };

  return (
    <div className="feedback">
      <header className="feedback-head">
        <h1>Feedback</h1>
        <p>Stored locally in this browser — no backend.</p>
      </header>

      {sent ? (
        <div className="feedback-thanks">
          <p className="ok-line mono">saved</p>
          <p>Thanks. You can send another note anytime.</p>
          <button type="button" className="btn btn-primary" onClick={() => setSent(false)}>
            Send another
          </button>
        </div>
      ) : (
        <form className="feedback-form" onSubmit={submit}>
          <div className="feedback-row">
            <div className="field">
              <label htmlFor="name">Name</label>
              <input id="name" value={form.name} onChange={(e) => setForm((f) => ({ ...f, name: e.target.value }))} />
            </div>
            <div className="field">
              <label htmlFor="email">Email</label>
              <input
                id="email"
                type="email"
                value={form.email}
                onChange={(e) => setForm((f) => ({ ...f, email: e.target.value }))}
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
              <option value="compiler">Playground</option>
              <option value="lab">ML Lab</option>
              <option value="learn">Learn</option>
              <option value="bug">Bug</option>
              <option value="idea">Idea</option>
            </select>
          </div>
          <div className="field">
            <label htmlFor="message">Message</label>
            <textarea
              id="message"
              value={form.message}
              onChange={(e) => setForm((f) => ({ ...f, message: e.target.value }))}
              required
            />
          </div>
          {error && <p className="feedback-error">{error}</p>}
          <button type="submit" className="btn btn-primary">
            Submit
          </button>
        </form>
      )}
    </div>
  );
}
