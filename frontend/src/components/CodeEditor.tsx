import { useMemo } from 'react';
import './CodeEditor.css';

type Props = {
  value: string;
  onChange: (v: string) => void;
  language?: string;
  minHeight?: number | string;
  readOnly?: boolean;
  className?: string;
};

const KEYWORDS =
  /\b(fn|let|mut|pub|struct|enum|impl|trait|if|else|while|for|loop|match|return|break|continue|true|false|use|mod|as|in|where|type|self|Self|async|await|unsafe|const)\b/g;
const TYPES = /\b(Int|Float|Bool|String|Char|Unit|Option|Result|Vec|Some|None|Ok|Err)\b/g;
const BUILTINS = /\b(print|show|assert|len|abs|min|max|sqrt|floor|ceil)\b/g;
const STRINGS = /"(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*'/g;
const COMMENTS = /\/\/.*$|\/\*[\s\S]*?\*\//gm;
const NUMBERS = /\b\d+(?:\.\d+)?\b/g;

function escapeHtml(s: string) {
  return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
}

function highlightFoxa(src: string): string {
  const placeholders: string[] = [];
  const hold = (html: string) => {
    placeholders.push(html);
    return `\u0000${placeholders.length - 1}\u0000`;
  };

  let out = src;
  out = out.replace(COMMENTS, (m) => hold(`<span class="tok-comment">${escapeHtml(m)}</span>`));
  out = out.replace(STRINGS, (m) => hold(`<span class="tok-string">${escapeHtml(m)}</span>`));
  out = escapeHtml(out);
  out = out.replace(KEYWORDS, '<span class="tok-kw">$1</span>');
  out = out.replace(TYPES, '<span class="tok-type">$1</span>');
  out = out.replace(BUILTINS, '<span class="tok-fn">$1</span>');
  out = out.replace(NUMBERS, '<span class="tok-num">$&</span>');
  out = out.replace(/\u0000(\d+)\u0000/g, (_, i) => placeholders[Number(i)]);
  return out || ' ';
}

export function CodeEditor({
  value,
  onChange,
  language = 'foxa',
  minHeight = 240,
  readOnly,
  className = '',
}: Props) {
  const lines = useMemo(() => Math.max(value.split('\n').length, 1), [value]);
  const html = useMemo(() => highlightFoxa(value), [value]);

  return (
    <div
      className={`code-editor ${className}`.trim()}
      style={{ minHeight: typeof minHeight === 'number' ? `${minHeight}px` : minHeight }}
    >
      <div className="code-gutter" aria-hidden>
        {Array.from({ length: lines }, (_, i) => (
          <span key={i}>{i + 1}</span>
        ))}
      </div>
      <div className="code-stack">
        <pre className="code-highlight" aria-hidden dangerouslySetInnerHTML={{ __html: html + '\n' }} />
        <textarea
          className="code-area"
          value={value}
          readOnly={readOnly}
          spellCheck={false}
          aria-label={`${language} source`}
          onChange={(e) => onChange(e.target.value)}
          onScroll={(e) => {
            const pre = e.currentTarget.previousElementSibling as HTMLElement | null;
            if (pre) {
              pre.scrollTop = e.currentTarget.scrollTop;
              pre.scrollLeft = e.currentTarget.scrollLeft;
            }
          }}
          onKeyDown={(e) => {
            if (e.key === 'Tab') {
              e.preventDefault();
              const el = e.currentTarget;
              const start = el.selectionStart;
              const end = el.selectionEnd;
              const next = value.slice(0, start) + '    ' + value.slice(end);
              onChange(next);
              requestAnimationFrame(() => {
                el.selectionStart = el.selectionEnd = start + 4;
              });
            }
          }}
        />
      </div>
    </div>
  );
}
