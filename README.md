# PyLK — Mini Python Interpreter

A toy Python-flavored language interpreter written in Rust. PyLK keeps
Python's surface syntax (keywords, operators, literals) but replaces
indentation-based blocks with `{` and `}` so the parser can be a
straightforward recursive-descent implementation.

> **Status: parser complete. Evaluator is next.**

The lexer and parser are fully implemented and unit-tested. The
project currently produces a `Program` AST from source code but does
not yet execute it. The next milestone is a tree-walking evaluator
that walks the AST and returns a value.

## Repository layout

```text
pylk/
├── LICENSE
├── README.md                      # this file
├── docs/
│   ├── design.md                  # language design notes
│   ├── grammar.ebnf               # full EBNF grammar
│   ├── lexer.md                   # lexer notes
│   └── parser_bugs_01.md          # known parser gaps (7 open)
└── interpreter/
    ├── Cargo.toml                 # `name = "interpreter"`, edition 2024
    ├── examples/                  # (reserved for .pyk sample programs)
    └── src/
        ├── main.rs                # binary entry point (stub)
        ├── lib.rs                 # module declarations
        ├── ast.rs                 # AST node types
        ├── errors.rs              # LexerError / ParserError enums
        ├── lexer.rs               # source → Vec<Token>
        └── parser.rs              # Vec<Token> → Program (AST)
```

The interpreter is a single Cargo crate with six source files. There
is no runtime, no standard library, and no I/O — the current goal is
to finish the front end and then build a tree-walking evaluator on
top of the existing AST.

## Crate map (`interpreter/src/lib.rs`)

```rust
pub mod ast;
pub mod errors;
pub mod lexer;
pub mod parser;
```

Each module has a single responsibility:

| Module     | Responsibility                                          | Size       |
| ---------- | ------------------------------------------------------- | ---------- |
| `ast`      | Define every AST node, statement, expression, and op    | ~315 lines |
| `errors`   | `LexerError` and `ParserError` via `thiserror`           | ~50 lines  |
| `lexer`    | `Lexer::new(src).lex() -> Result<Vec<Token>>`           | ~1,280 lines |
| `parser`   | `Parser::new(tokens).parse() -> Result<Program>`        | ~3,400 lines (incl. tests) |

## What works today

The parser handles a large subset of the grammar in
`docs/grammar.ebnf`. The following pass under `cargo test parser`
(124 of 131 tests green):

### Statements

- `import a.b.c [as alias]`
- `from a.b.c import name, name as alias, *`
- `def name(params) { body }` — params are plain identifiers, no
  defaults or type annotations
- `class Name { members }` — no inheritance
- `if expr { } [elif expr { }]* [else { }]`
- `while expr { body }`
- `for ident in expr { body }`
- `return [expr]`
- `break`, `continue`, `pass`
- `target = expr` and every compound assignment (`+= -= *= /= //=
  %= **= &= |= ^= <<= >>=`)
- Tuple-destructure assignment: `(a, b) = (1, 2)`
- Expression statements (e.g. `f(x);`)

### Expressions

- Literals: integers, floats, strings, booleans, `None`
- Identifiers
- Arithmetic: `+ - * / // % **` (right-associative power)
- Bitwise: `| ^ &` and shift `<< >>`
- Comparison: `== != < <= > >=`
- Logical: `and`, `or`, `not` (with Python-style precedence)
- Ternary: `a if cond else b`
- Lambda: `lambda params: expr`
- Unary: `+ - ~`
- Grouping: `(expr)`
- Lists `[a, b, c]`, tuples `(a, b, c)` / `(a,)` / `()`, dicts
  `{k: v, ...}` (empty `{}` parses as empty dict, not block)
- Slicing: `a[i]`, `a[i:j]`, `a[i:j:k]`, `a[:]`, etc.
- Postfix chaining: `a.b`, `a.b()`, `a[0]`, `a[i:j]`, `f(1, 2, 3)`
- Nested expressions, left-associative binary ops, proper
  precedence/associativity for the whole operator tower

## What is intentionally **not** supported

Per `docs/design.md`, the following Python features are out of scope
for the language:

### Syntax

- **No indentation-based blocks** — `{` and `}` are mandatory
- **No class inheritance** (no parent class)
- **No `try` / `except` / `finally` / `raise` / `assert`**
- **No `async` / `await`**
- **No `with` / `yield`** (no generators / context managers)
- **No `:=` walrus operator**
- **No `...` Ellipsis literal**
- **No type annotations or type comments** (the parser does not read
  `:` after a parameter for a type, nor `->` return annotations)
- **No default parameter values** in `def f(x = 1)` — `parse_function`
  always sets `default: None` and does not consume a `=`
- **No `match` / `case`**
- **No `del`** (the keyword is lexed but not parsed)
- **No `global` / `nonlocal`** (keywords lexed, not parsed)
- **No string prefixes** `r"..."` / `b"..."` / `f"..."`
- **No integer separators** (`1_000_000` is rejected)
- **No binary / octal / hex integer literals** in source (`0b`,
  `0o`, `0x` are lexed but the lexer does not parse them yet)
- **No triple-quoted strings**

### Semantics (parser-level gaps)

These are the features the parser does not yet implement, tracked in
[`docs/parser_bugs_01.md`](docs/parser_bugs_01.md):

- **Set literals** `{1, 2, 3}` — `Expr::SetLiteral` exists in
  `ast.rs` but `parse_dict_literal` has no set branch
- **Membership / identity comparisons** `in`, `not in`, `is`,
  `is not` — the AST has `CompareOp::{In, NotIn, Is, IsNot}` but
  `parse_comparison` only matches `== != < <= > >=`

### Things the parser accepts that the evaluator will need to handle

These are not "unsupported" — the parser produces a valid AST — but
they have no runtime meaning yet:

- Integer / float / string / boolean / `None` literals (need runtime
  value representation)
- Arithmetic / bitwise / comparison / logical operators (need
  evaluation semantics)
- Slicing (needs sequence representation)
- Function and class definitions (need an environment / scoping
  model)
- Control flow (`if` / `while` / `for` / `break` / `continue` /
  `return`) — needs a control-flow evaluator
- Imports — currently parsed but no module system

## Running the tests

```sh
cd interpreter
cargo test
```

The parser test suite has 131 tests, of which 124 pass and 7 fail
with the documented gaps above. The 7 failures are expected and
listed in `docs/parser_bugs_01.md`.

To see the parser's debug trace while a test runs, use
`--nocapture`:

```sh
cargo test parser::tests::test_expr_add -- --nocapture
```

Each `parse_*` function in the expression layer prints a start log
and a result log so you can watch the AST build up from `primary` up
through `ternary`.

## Design notes

- `docs/design.md` — the high-level language design, including the
  no-indentation rule and the list of deliberately unsupported
  features
- `docs/grammar.ebnf` — the full EBNF grammar; the parser follows it
  with a few noted exceptions (chained comparison, set literals)
- `docs/lexer.md` — notes on the lexer
- `docs/parser_bugs_01.md` — the running list of parser-level gaps
  (7 open, all unimplemented features rather than regressions)

## Roadmap

| Phase            | Status         | Notes                                       |
| ---------------- | -------------- | ------------------------------------------- |
| Lexer            | ✅ Complete     | All tokens in `design.md` are lexed         |
| AST              | ✅ Complete     | All statement and expression variants       |
| Parser           | ✅ Complete     | 124/131 tests green; 7 known gaps           |
| **Evaluator**    | 🚧 **Next**     | Tree-walking interpreter over the AST       |
| **ByteCode VM**    | 🚧 **Next**     | Bytecode interpreter over the AST          |
| Standard library | ⏳ Not started  | Built-ins (`print`, `len`, etc.)            |
| REPL             | ⏳ Not started  | Once the evaluator can run a `Program`     |

The next milestone is the **evaluator**: a module that consumes the
`Program` AST and executes it in a tree-walking style with a
suitably simple value model (likely an `enum Value { Int(i64), Float(f64), Str(String), Bool(bool), None, Func(...), ... }`)
and an environment for variable bindings. Once the evaluator can run
the existing test inputs end-to-end, the project graduates from a
"front end only" parser to a working Mini Python interpreter.
