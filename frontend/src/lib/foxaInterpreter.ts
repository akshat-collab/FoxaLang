/** Lightweight Foxa interpreter for the browser playground. */

export type RunResult = {
  ok: boolean;
  output: string[];
  error?: string;
  elapsedMs: number;
};

type Value =
  | number
  | string
  | boolean
  | null
  | Value[]
  | { [key: string]: Value };

const BUILTINS = new Set(['print', 'len', 'abs', 'min', 'max', 'sqrt', 'floor', 'ceil']);

function stripComments(src: string): string {
  return src
    .replace(/\/\*[\s\S]*?\*\//g, '')
    .replace(/\/\/.*$/gm, '');
}

function tokenize(src: string): string[] {
  const tokens: string[] = [];
  const re =
    /\s+|("(?:\\.|[^"\\])*")|('(?:\\.|[^'\\])*')|(\d+\.?\d*)|([A-Za-z_][A-Za-z0-9_]*)|(==|!=|<=|>=|=>|&&|\|\||->)|([+\-*/%=<>!(){}[\],.;:])/g;
  let m: RegExpExecArray | null;
  while ((m = re.exec(src))) {
    if (m[0].trim() === '') continue;
    tokens.push(m[0]);
  }
  return tokens;
}

class Env {
  private vars = new Map<string, Value>();
  private mut = new Set<string>();
  private parent?: Env;

  constructor(parent?: Env) {
    this.parent = parent;
  }

  define(name: string, value: Value, isMut = false) {
    this.vars.set(name, value);
    if (isMut) this.mut.add(name);
  }

  get(name: string): Value {
    if (this.vars.has(name)) return this.vars.get(name)!;
    if (this.parent) return this.parent.get(name);
    throw new Error(`undefined variable \`${name}\``);
  }

  set(name: string, value: Value) {
    if (this.vars.has(name)) {
      this.vars.set(name, value);
      return;
    }
    if (this.parent) {
      this.parent.set(name, value);
      return;
    }
    throw new Error(`cannot assign undeclared \`${name}\``);
  }

  hasLocal(name: string) {
    return this.vars.has(name);
  }
}

class Parser {
  private i = 0;
  tokens: string[];

  constructor(tokens: string[]) {
    this.tokens = tokens;
  }

  peek(n = 0) {
    return this.tokens[this.i + n];
  }

  eat(expected?: string) {
    const t = this.tokens[this.i++];
    if (expected && t !== expected) throw new Error(`expected \`${expected}\`, got \`${t ?? 'EOF'}\``);
    return t;
  }

  match(...opts: string[]) {
    if (opts.includes(this.peek())) {
      this.i++;
      return true;
    }
    return false;
  }

  atEnd() {
    return this.i >= this.tokens.length;
  }

  get index() {
    return this.i;
  }

  set index(v: number) {
    this.i = v;
  }
}

type FnDef = { params: string[]; bodyStart: number; bodyEnd: number };

function findBlockEnd(tokens: string[], openIdx: number): number {
  let depth = 0;
  for (let i = openIdx; i < tokens.length; i++) {
    if (tokens[i] === '{') depth++;
    if (tokens[i] === '}') {
      depth--;
      if (depth === 0) return i;
    }
  }
  throw new Error('unclosed `{`');
}

function parseFns(tokens: string[]): Map<string, FnDef> {
  const fns = new Map<string, FnDef>();
  let i = 0;
  while (i < tokens.length) {
    if (tokens[i] === 'fn' || (tokens[i] === 'pub' && tokens[i + 1] === 'fn')) {
      if (tokens[i] === 'pub') i++;
      i++; // fn
      const name = tokens[i++];
      if (tokens[i++] !== '(') throw new Error(`expected ( after fn ${name}`);
      const params: string[] = [];
      while (tokens[i] !== ')') {
        if (tokens[i] === ',') {
          i++;
          continue;
        }
        params.push(tokens[i++]);
        if (tokens[i] === ':') {
          i++; // skip type
          while (tokens[i] && ![')', ',', '{'].includes(tokens[i]) && tokens[i] !== '-') i++;
          if (tokens[i] === '-' && tokens[i + 1] === '>') {
            /* handled outside */
          }
        }
      }
      i++; // )
      if (tokens[i] === '-' && tokens[i + 1] === '>') {
        i += 2;
        while (tokens[i] && tokens[i] !== '{') i++;
      } else if (tokens[i] === '->') {
        i++;
        while (tokens[i] && tokens[i] !== '{') i++;
      }
      if (tokens[i] !== '{') throw new Error(`expected { for fn ${name}`);
      const bodyStart = i + 1;
      const bodyEnd = findBlockEnd(tokens, i);
      fns.set(name, { params, bodyStart, bodyEnd });
      i = bodyEnd + 1;
      continue;
    }
    // skip struct / enum definitions
    if (tokens[i] === 'struct' || tokens[i] === 'enum') {
      while (i < tokens.length && tokens[i] !== '{') i++;
      if (tokens[i] === '{') i = findBlockEnd(tokens, i) + 1;
      continue;
    }
    i++;
  }
  return fns;
}

export function runFoxa(source: string): RunResult {
  const started = performance.now();
  const output: string[] = [];
  try {
    const cleaned = stripComments(source);
    const tokens = tokenize(cleaned);
    if (tokens.length === 0) {
      return { ok: true, output: ['(empty program)'], elapsedMs: performance.now() - started };
    }

    const fns = parseFns(tokens);
    const global = new Env();

    const callBuiltin = (name: string, args: Value[]): Value => {
      switch (name) {
        case 'print':
          output.push(args.map(stringify).join(' '));
          return null;
        case 'len': {
          const v = args[0];
          if (typeof v === 'string' || Array.isArray(v)) return v.length;
          throw new Error('len expects string or array');
        }
        case 'abs':
          return Math.abs(Number(args[0]));
        case 'min':
          return Math.min(...args.map(Number));
        case 'max':
          return Math.max(...args.map(Number));
        case 'sqrt':
          return Math.sqrt(Number(args[0]));
        case 'floor':
          return Math.floor(Number(args[0]));
        case 'ceil':
          return Math.ceil(Number(args[0]));
        default:
          throw new Error(`unknown builtin ${name}`);
      }
    };

    const evalExpr = (p: Parser, env: Env): Value => {
      const parseOr = (): Value => {
        let left = parseAnd();
        while (p.match('||')) {
          const right = parseAnd();
          left = Boolean(left) || Boolean(right);
        }
        return left;
      };
      const parseAnd = (): Value => {
        let left = parseCmp();
        while (p.match('&&')) {
          const right = parseCmp();
          left = Boolean(left) && Boolean(right);
        }
        return left;
      };
      const parseCmp = (): Value => {
        let left = parseAdd();
        while (['==', '!=', '<', '<=', '>', '>='].includes(p.peek())) {
          const op = p.eat()!;
          const right = parseAdd();
          switch (op) {
            case '==':
              left = left === right;
              break;
            case '!=':
              left = left !== right;
              break;
            case '<':
              left = Number(left) < Number(right);
              break;
            case '<=':
              left = Number(left) <= Number(right);
              break;
            case '>':
              left = Number(left) > Number(right);
              break;
            case '>=':
              left = Number(left) >= Number(right);
              break;
          }
        }
        return left;
      };
      const parseAdd = (): Value => {
        let left = parseMul();
        while (p.peek() === '+' || p.peek() === '-') {
          const op = p.eat()!;
          const right = parseMul();
          if (op === '+' && (typeof left === 'string' || typeof right === 'string')) {
            left = String(left) + String(right);
          } else if (op === '+') left = Number(left) + Number(right);
          else left = Number(left) - Number(right);
        }
        return left;
      };
      const parseMul = (): Value => {
        let left = parseUnary();
        while (p.peek() === '*' || p.peek() === '/' || p.peek() === '%') {
          const op = p.eat()!;
          const right = parseUnary();
          if (op === '*') left = Number(left) * Number(right);
          else if (op === '/') left = Number(left) / Number(right);
          else left = Number(left) % Number(right);
        }
        return left;
      };
      const parseUnary = (): Value => {
        if (p.match('-')) return -Number(parseUnary());
        if (p.match('!')) return !parseUnary();
        return parsePostfix();
      };
      const parsePostfix = (): Value => {
        let val = parsePrimary();
        while (true) {
          if (p.match('(')) {
            const args: Value[] = [];
            if (p.peek() !== ')') {
              args.push(evalExpr(p, env));
              while (p.match(',')) args.push(evalExpr(p, env));
            }
            p.eat(')');
            if (typeof val === 'string' && BUILTINS.has(val)) {
              val = callBuiltin(val, args);
            } else if (typeof val === 'string' && fns.has(val)) {
              val = callFn(val, args, env);
            } else {
              throw new Error(`not callable: ${stringify(val)}`);
            }
          } else if (p.match('.')) {
            const field = p.eat()!;
            if (val && typeof val === 'object' && !Array.isArray(val)) {
              val = (val as Record<string, Value>)[field] ?? null;
            } else {
              throw new Error(`cannot access field on ${stringify(val)}`);
            }
          } else if (p.match('[')) {
            const idx = evalExpr(p, env);
            p.eat(']');
            if (Array.isArray(val)) val = val[Number(idx)] ?? null;
            else if (typeof val === 'string') val = val[Number(idx)] ?? null;
            else throw new Error('index on non-indexable');
          } else break;
        }
        return val;
      };
      const parsePrimary = (): Value => {
        const t = p.peek();
        if (!t) throw new Error('unexpected end of expression');
        if (t === 'true') {
          p.eat();
          return true;
        }
        if (t === 'false') {
          p.eat();
          return false;
        }
        if (t === 'None' || t === 'null') {
          p.eat();
          return null;
        }
        if (/^\d/.test(t)) {
          p.eat();
          return Number(t);
        }
        if (t.startsWith('"') || t.startsWith("'")) {
          p.eat();
          return JSON.parse(t.startsWith("'") ? `"${t.slice(1, -1)}"` : t);
        }
        if (t === '[') {
          p.eat();
          const arr: Value[] = [];
          if (p.peek() !== ']') {
            arr.push(evalExpr(p, env));
            while (p.match(',')) arr.push(evalExpr(p, env));
          }
          p.eat(']');
          return arr;
        }
        if (t === '(') {
          p.eat();
          const v = evalExpr(p, env);
          p.eat(')');
          return v;
        }
        if (t === 'Some') {
          p.eat();
          p.eat('(');
          const v = evalExpr(p, env);
          p.eat(')');
          return v;
        }
        // struct literal: Ident { field: expr, ... }
        if (/^[A-Za-z_]/.test(t) && p.peek(1) === '{') {
          p.eat();
          p.eat('{');
          const obj: Record<string, Value> = {};
          while (p.peek() !== '}') {
            const field = p.eat()!;
            p.eat(':');
            obj[field] = evalExpr(p, env);
            p.match(',');
          }
          p.eat('}');
          return obj;
        }
        if (/^[A-Za-z_]/.test(t)) {
          const name = p.eat()!;
          if (BUILTINS.has(name) || fns.has(name)) return name; // deferred call
          return env.get(name);
        }
        throw new Error(`unexpected token \`${t}\``);
      };

      return parseOr();
    };

    const execBlock = (slice: string[], env: Env): Value => {
      const p = new Parser(slice);
      let last: Value = null;
      while (!p.atEnd()) {
        const t = p.peek();
        if (t === ';') {
          p.eat();
          continue;
        }
        if (t === 'let') {
          p.eat();
          const isMut = p.match('mut');
          const name = p.eat()!;
          if (p.peek() === ':') {
            p.eat();
            while (p.peek() && p.peek() !== '=' && p.peek() !== ';') p.eat();
          }
          let val: Value = null;
          if (p.match('=')) val = evalExpr(p, env);
          p.match(';');
          env.define(name, val, isMut);
          last = val;
          continue;
        }
        if (t === 'return') {
          p.eat();
          if (p.peek() === ';' || p.atEnd()) return null;
          const v = evalExpr(p, env);
          p.match(';');
          return v;
        }
        if (t === 'while') {
          p.eat();
          const condTokens: string[] = [];
          // condition until {
          let depth = 0;
          while (!(p.peek() === '{' && depth === 0)) {
            const tok = p.eat();
            if (!tok) throw new Error('while: expected {');
            if (tok === '(') depth++;
            if (tok === ')') depth--;
            condTokens.push(tok);
          }
          p.eat('{');
          const bodyStart = 0;
          const bodyTokens: string[] = [];
          let bd = 1;
          while (bd > 0) {
            const tok = p.eat();
            if (!tok) throw new Error('while: unclosed body');
            if (tok === '{') bd++;
            if (tok === '}') {
              bd--;
              if (bd === 0) break;
            }
            bodyTokens.push(tok);
          }
          let guard = 0;
          while (truthy(evalExpr(new Parser(condTokens), env))) {
            if (++guard > 100000) throw new Error('while: iteration limit exceeded');
            execBlock(bodyTokens, new Env(env));
          }
          void bodyStart;
          last = null;
          continue;
        }
        if (t === 'for') {
          p.eat();
          const name = p.eat()!;
          p.eat('in');
          const iter = evalExpr(p, env);
          p.eat('{');
          const bodyTokens: string[] = [];
          let bd = 1;
          while (bd > 0) {
            const tok = p.eat();
            if (!tok) throw new Error('for: unclosed body');
            if (tok === '{') bd++;
            if (tok === '}') {
              bd--;
              if (bd === 0) break;
            }
            bodyTokens.push(tok);
          }
          const list = Array.isArray(iter) ? iter : typeof iter === 'string' ? [...iter] : null;
          if (!list) throw new Error('for: expected array or string');
          for (const item of list) {
            const local = new Env(env);
            local.define(name, item, true);
            execBlock(bodyTokens, local);
          }
          last = null;
          continue;
        }
        if (t === 'if') {
          p.eat();
          const condTokens: string[] = [];
          let depth = 0;
          while (!(p.peek() === '{' && depth === 0)) {
            const tok = p.eat();
            if (!tok) throw new Error('if: expected {');
            if (tok === '(') depth++;
            if (tok === ')') depth--;
            condTokens.push(tok);
          }
          p.eat('{');
          const thenBody: string[] = [];
          let bd = 1;
          while (bd > 0) {
            const tok = p.eat();
            if (!tok) throw new Error('if: unclosed');
            if (tok === '{') bd++;
            if (tok === '}') {
              bd--;
              if (bd === 0) break;
            }
            thenBody.push(tok);
          }
          let elseBody: string[] | null = null;
          if (p.match('else')) {
            if (p.peek() === 'if') {
              // else if — re-feed
              const rest = p.tokens.slice(p.index);
              elseBody = ['if', ...rest];
              // consume rest of tokens in this block by executing via recursive if
              // simpler: parse else-if as nested
              const nested = execBlock(['if', ...rest], env);
              // mark parser done
              while (!p.atEnd()) p.eat();
              last = truthy(evalExpr(new Parser(condTokens), env))
                ? execBlock(thenBody, new Env(env))
                : nested;
              continue;
            }
            p.eat('{');
            elseBody = [];
            let ed = 1;
            while (ed > 0) {
              const tok = p.eat();
              if (!tok) throw new Error('else: unclosed');
              if (tok === '{') ed++;
              if (tok === '}') {
                ed--;
                if (ed === 0) break;
              }
              elseBody.push(tok);
            }
          }
          if (truthy(evalExpr(new Parser(condTokens), env))) {
            last = execBlock(thenBody, new Env(env));
          } else if (elseBody) {
            last = execBlock(elseBody, new Env(env));
          } else last = null;
          continue;
        }
        if (t === 'match') {
          p.eat();
          const subject = evalExpr(p, env);
          p.eat('{');
          let matched = false;
          let result: Value = null;
          while (p.peek() !== '}') {
            // arm: Pattern => expr ,
            const pat = p.eat()!;
            if (pat === 'Some') {
              p.eat('(');
              const bind = p.eat()!;
              p.eat(')');
              p.eat('=');
              p.eat('>'); // => may be = >
              // handle => as single token or =
              let armExpr: string[] = [];
              if (p.peek() === '>') p.eat();
              // collect until comma or }
              while (p.peek() && p.peek() !== ',' && p.peek() !== '}') {
                armExpr.push(p.eat()!);
              }
              p.match(',');
              if (!matched && subject !== null) {
                const local = new Env(env);
                local.define(bind, subject);
                result = evalExpr(new Parser(armExpr), local);
                matched = true;
              }
              continue;
            }
            if (pat === 'None') {
              // => expr
              if (p.peek() === '=' || p.peek() === '=>') {
                if (p.peek() === '=>') p.eat();
                else {
                  p.eat();
                  if (p.peek() === '>') p.eat();
                }
              }
              const armExpr: string[] = [];
              while (p.peek() && p.peek() !== ',' && p.peek() !== '}') armExpr.push(p.eat()!);
              p.match(',');
              if (!matched && subject === null) {
                result = evalExpr(new Parser(armExpr), env);
                matched = true;
              }
              continue;
            }
            // Ident(x) or literal
            if (p.peek() === '(') {
              p.eat();
              const bind = p.eat()!;
              p.eat(')');
              if (p.peek() === '=>' || p.peek() === '=') {
                if (p.peek() === '=>') p.eat();
                else {
                  p.eat();
                  if (p.peek() === '>') p.eat();
                }
              }
              const armExpr: string[] = [];
              while (p.peek() && p.peek() !== ',' && p.peek() !== '}') armExpr.push(p.eat()!);
              p.match(',');
              if (!matched) {
                const local = new Env(env);
                local.define(bind, subject);
                result = evalExpr(new Parser(armExpr), local);
                matched = true;
              }
              continue;
            }
            // skip unknown arm
            while (p.peek() && p.peek() !== ',' && p.peek() !== '}') p.eat();
            p.match(',');
          }
          p.eat('}');
          last = result;
          continue;
        }

        // assignment or expression statement
        // look ahead for Ident =
        if (/^[A-Za-z_]/.test(t) && p.peek(1) === '=' && p.peek(2) !== '=') {
          const name = p.eat()!;
          p.eat('=');
          const val = evalExpr(p, env);
          p.match(';');
          env.set(name, val);
          last = val;
          continue;
        }

        last = evalExpr(p, env);
        p.match(';');
      }
      return last;
    };

    const callFn = (name: string, args: Value[], parent: Env): Value => {
      const def = fns.get(name);
      if (!def) throw new Error(`unknown function ${name}`);
      const local = new Env(parent);
      def.params.forEach((param, idx) => local.define(param, args[idx] ?? null, true));
      const body = tokens.slice(def.bodyStart, def.bodyEnd);
      return execBlock(body, local);
    };

    if (!fns.has('main')) {
      // treat whole file as script body (skip fn/struct defs already handled)
      throw new Error('missing `fn main()` — Foxa programs start at main');
    }

    callFn('main', [], global);
    return {
      ok: true,
      output: output.length ? output : ['(ran successfully, no output)'],
      elapsedMs: performance.now() - started,
    };
  } catch (err) {
    return {
      ok: false,
      output,
      error: err instanceof Error ? err.message : String(err),
      elapsedMs: performance.now() - started,
    };
  }
}

function truthy(v: Value): boolean {
  if (v === null || v === false || v === 0 || v === '') return false;
  return true;
}

function stringify(v: Value): string {
  if (v === null) return 'None';
  if (typeof v === 'string') return v;
  if (typeof v === 'object') return JSON.stringify(v);
  return String(v);
}
