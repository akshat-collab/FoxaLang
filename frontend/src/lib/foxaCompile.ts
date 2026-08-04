/** Lightweight Foxa file checks for the playground compiler. */

export type Diagnostic = {
  severity: 'error' | 'warning' | 'info';
  message: string;
  line?: number;
};

export type CompileReport = {
  ok: boolean;
  diagnostics: Diagnostic[];
  functions: string[];
  hasMain: boolean;
};

/** Structural compile/check of a `.foxa` buffer (playground). */
export function compileFoxaFile(filename: string, source: string): CompileReport {
  const diagnostics: Diagnostic[] = [];
  const functions: string[] = [];

  if (!filename.endsWith('.foxa')) {
    diagnostics.push({
      severity: 'warning',
      message: `expected a .foxa file, got \`${filename}\``,
    });
  }

  if (!source.trim()) {
    diagnostics.push({ severity: 'error', message: 'empty source file' });
    return { ok: false, diagnostics, functions, hasMain: false };
  }

  const fnRe = /\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(/g;
  let m: RegExpExecArray | null;
  while ((m = fnRe.exec(source))) {
    functions.push(m[1]);
  }

  const hasMain = functions.includes('main');
  if (!hasMain) {
    diagnostics.push({
      severity: 'error',
      message: 'missing `fn main()` — Foxa programs start at main',
    });
  }

  const open = (source.match(/\{/g) ?? []).length;
  const close = (source.match(/\}/g) ?? []).length;
  if (open !== close) {
    diagnostics.push({
      severity: 'error',
      message: `unbalanced braces ({ ${open} vs } ${close})`,
    });
  }

  const lines = source.split('\n');
  lines.forEach((line, i) => {
    if (/\bfunction\b/.test(line)) {
      diagnostics.push({
        severity: 'error',
        line: i + 1,
        message: 'use Foxa `fn`, not `function`',
      });
    }
    if (/\bdef\b/.test(line)) {
      diagnostics.push({
        severity: 'error',
        line: i + 1,
        message: 'use Foxa `fn`, not `def`',
      });
    }
  });

  if (functions.length === 0) {
    diagnostics.push({
      severity: 'error',
      message: 'no `fn` definitions found — create functions with `fn name(...) { ... }`',
    });
  }

  const ok = !diagnostics.some((d) => d.severity === 'error');
  if (ok) {
    diagnostics.push({
      severity: 'info',
      message: `ok — ${functions.length} function(s): ${functions.join(', ')}`,
    });
  }

  return { ok, diagnostics, functions, hasMain };
}

/** Scaffold a Foxa function source snippet. */
export function scaffoldFoxaFn(opts: {
  name: string;
  params?: string;
  ret?: string;
  body?: string;
}): string {
  const name = opts.name.trim() || 'unnamed';
  const params = (opts.params ?? '').trim();
  const ret = opts.ret?.trim();
  const retClause = ret ? ` -> ${ret}` : '';
  const body =
    opts.body?.trim() ||
    (ret === 'Int' || ret === 'Float'
      ? '0'
      : ret === 'Bool'
        ? 'false'
        : ret === 'String'
          ? '""'
          : ret
            ? '0'
            : 'show("todo");');
  const indented = body
    .split('\n')
    .map((l) => (l.trim() ? `    ${l}` : l))
    .join('\n');
  return `fn ${name}(${params})${retClause} {\n${indented}\n}\n`;
}

/** Format output the way `foxa show` does in the CLI. */
export function formatShowReport(
  filename: string,
  compile: CompileReport,
  output: string[],
  error?: string,
): string[] {
  const lines = [
    `=== foxa show: ${filename} ===`,
    compile.ok
      ? `compile: ok (${compile.functions.length} function(s))`
      : 'compile: failed',
  ];
  if (compile.functions.length) {
    lines.push(`functions: ${compile.functions.join(', ')}`);
  }
  lines.push('--- output ---');
  if (error) {
    lines.push(`error: ${error}`);
  } else if (output.length === 0) {
    lines.push('(no print/show output)');
  } else {
    lines.push(...output);
  }
  return lines;
}
