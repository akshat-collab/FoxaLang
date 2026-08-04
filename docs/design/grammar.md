# Foxa Grammar Sketch (v0.1)

This is an evolving sketch, not a frozen specification.

```
Module      := Item*
Item        := Vis? FnItem
Vis         := 'pub'

FnItem      := 'fn' Ident '(' ParamList? ')' RetTy? Block
ParamList   := Param (',' Param)* ','?
Param       := Ident ':' Type
RetTy       := '->' Type

Type        := Path
Path        := Ident ('::' Ident)*

Block       := '{' Stmt* Expr? '}'

Stmt        := LetStmt | ReturnStmt | ExprStmt | ';'
LetStmt     := 'let' 'mut'? Ident (':' Type)? ('=' Expr)? ';'
ReturnStmt  := 'return' Expr? ';'
ExprStmt    := Expr ';'

Expr        := Assign
Assign      := Or ('=' Assign)?
Or          := And ('||' And)*
And          := Cmp ('&&' Cmp)*
Cmp         := Add (('=='| '!=' | '<' | '<=' | '>' | '>=') Add)*
Add         := Mul (('+' | '-') Mul)*
Mul         := Unary (('*' | '/' | '%') Unary)*
Unary       := ('-' | '!' | '&' | '*') Unary | Postfix
Postfix    := Primary ('(' ArgList? ')')*
Primary     := Lit | Path | '(' Expr ')' | Block | IfExpr
IfExpr      := 'if' Expr Block ('else' (IfExpr | Block))?
ArgList     := Expr (',' Expr)* ','?

Lit         := INT | FLOAT | STRING | CHAR | 'true' | 'false'
```

## Lexer tokens

Keywords: `fn let mut const struct enum impl trait if else while for loop match
return break continue true false pub use mod as in where type self Self async
await unsafe`

Operators and punctuation as implemented in `foxa-lexer`.

## Comments

- Line: `// ...`
- Block: `/* ... */` (non-nesting in v0.1)
