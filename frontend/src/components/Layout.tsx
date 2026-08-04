import { NavLink, Outlet } from 'react-router-dom';
import { Code2, BookOpen, FlaskConical, HelpCircle, MessageSquare, Home } from 'lucide-react';
import './Layout.css';

const links = [
  { to: '/', label: 'Home', icon: Home, end: true },
  { to: '/learn', label: 'Learn', icon: BookOpen },
  { to: '/compiler', label: 'Compiler', icon: Code2 },
  { to: '/lab', label: 'ML Lab', icon: FlaskConical },
  { to: '/help', label: 'Help', icon: HelpCircle },
  { to: '/feedback', label: 'Feedback', icon: MessageSquare },
];

export function Layout() {
  return (
    <div className="app-shell">
      <header className="topnav">
        <div className="topnav-inner">
          <NavLink to="/" className="brand" end>
            <span className="brand-mark" aria-hidden />
            <span className="brand-name">Foxa</span>
          </NavLink>
          <nav className="nav-links" aria-label="Primary">
            {links.map(({ to, label, icon: Icon, end }) => (
              <NavLink key={to} to={to} end={end} className={({ isActive }) => (isActive ? 'nav-link active' : 'nav-link')}>
                <Icon size={16} strokeWidth={2.2} />
                <span>{label}</span>
              </NavLink>
            ))}
          </nav>
        </div>
      </header>
      <main className="main">
        <Outlet />
      </main>
      <footer className="site-footer">
        <div className="container footer-inner">
          <span>Foxa · systems language playground</span>
          <span className="footer-muted">Compiler + ML Lab run in your browser</span>
        </div>
      </footer>
    </div>
  );
}
