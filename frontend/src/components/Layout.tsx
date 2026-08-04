import { NavLink, Outlet, useLocation } from 'react-router-dom';
import './Layout.css';

const links = [
  { to: '/compiler', label: 'Playground' },
  { to: '/lab', label: 'Lab' },
  { to: '/learn', label: 'Learn' },
  { to: '/help', label: 'Help' },
  { to: '/feedback', label: 'Feedback' },
];

export function Layout() {
  const { pathname } = useLocation();
  const toolMode = pathname === '/compiler' || pathname === '/lab';

  return (
    <div className="app-shell">
      <header className="appbar">
        <NavLink to="/" className="brand" end>
          <span className="brand-mark" aria-hidden>
            Fx
          </span>
          <span className="brand-name">Foxa</span>
        </NavLink>
        <nav className="appbar-nav" aria-label="Primary">
          {links.map(({ to, label }) => (
            <NavLink
              key={to}
              to={to}
              className={({ isActive }) => (isActive ? 'appbar-link active' : 'appbar-link')}
            >
              {label}
            </NavLink>
          ))}
        </nav>
        <div className="appbar-meta mono" aria-hidden>
          v0.1 · browser
        </div>
      </header>
      <main className={toolMode ? 'main' : 'main main-padded'}>{<Outlet />}</main>
    </div>
  );
}
