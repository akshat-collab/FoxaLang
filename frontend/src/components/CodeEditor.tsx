import './CodeEditor.css';

type Props = {
  value: string;
  onChange: (v: string) => void;
  language?: string;
  minHeight?: number;
  readOnly?: boolean;
};

export function CodeEditor({ value, onChange, language = 'foxa', minHeight = 280, readOnly }: Props) {
  return (
    <div className="code-editor" style={{ minHeight }}>
      <div className="code-gutter" aria-hidden>
        {value.split('\n').map((_, i) => (
          <span key={i}>{i + 1}</span>
        ))}
      </div>
      <textarea
        className="code-area mono"
        value={value}
        readOnly={readOnly}
        spellCheck={false}
        aria-label={`${language} source`}
        onChange={(e) => onChange(e.target.value)}
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
  );
}
