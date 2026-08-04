import { BrowserRouter, Navigate, Route, Routes } from 'react-router-dom';
import { Layout } from './components/Layout';
import { Home } from './pages/Home';
import { Learn } from './pages/Learn';
import { Compiler } from './pages/Compiler';
import { Lab } from './pages/Lab';
import { Help } from './pages/Help';
import { Feedback } from './pages/Feedback';

export default function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route element={<Layout />}>
          <Route index element={<Home />} />
          <Route path="learn" element={<Learn />} />
          <Route path="compiler" element={<Compiler />} />
          <Route path="lab" element={<Lab />} />
          <Route path="help" element={<Help />} />
          <Route path="feedback" element={<Feedback />} />
          <Route path="*" element={<Navigate to="/" replace />} />
        </Route>
      </Routes>
    </BrowserRouter>
  );
}
