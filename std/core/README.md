# Foxa Standard Library — Core

## Builtins (always available)

| Name | Kind | Notes |
|------|------|-------|
| `Option` / `None` / `Some(T)` | enum | Pattern-matchable |
| `Result` / `Ok(T)` / `Err(E)` | enum | Pattern-matchable |
| `Vec[T]` | type | Construct with `[a, b, c]`; iterate with `for` |
| `String` | type | String literals `"..."`; `+` concatenates |
| `Int`, `Float`, `Bool`, `Char`, `Unit` | primitives | |
| `print(x)` | fn | Writes a line to stdout |
| `assert(b)` | fn | Panics if false |

## Example

```foxa
fn main() {
    let xs = [1, 2, 3];
    for x in xs {
        match Some(x) {
            Some(n) => print(n),
            None => print(0),
        }
    }

    let msg = "Hello, " + "Foxa";
    print(msg);
}
```

Full module implementations (methods like `Vec.push`, `String.len`) land as
the MIR/codegen and package loader mature. The interpreter already supports
the constructors and operations above.
