# Parser Bugs (Round 1)

Inputs the parser currently fails on, found while adding unit tests
to `interpreter/src/parser.rs`. Each entry lists the test name, the
input, the expected behavior, the actual behavior observed, and a
brief reason.

Fixes are out of scope for this round — this document is a tracking
list so the gaps can be prioritized separately.

---

## 1. Set literals are not implemented

`Expr::SetLiteral(Vec<Expr>)` is defined in `ast.rs` and the grammar
(`docs/grammar.ebnf:293-297`) describes the `{x, y, z}` form, but
`parse_dict_literal` (parser.rs:987-1020) and
`try_parse_dict_literal_at_stmt` (parser.rs:1030-1075) only produce
`Expr::DictLiteral`. There is no set-literal branch.

### Test: `test_expr_set_literal`
- **Input:** `{1, 2, 3};`
- **Expected:** `Expr::SetLiteral(vec![Literal(Int(1)), Literal(Int(2)), Literal(Int(3))])`
- **Actual:** `ParserError::UnsupportToken(Token::Comma)` — the parser
  falls through to `parse_block` and chokes on the dangling `,`
  after the first element.
- **Reason:** No set-literal branch in `parse_dict_literal` /
  `try_parse_dict_literal_at_stmt`.

### Test: `test_expr_set_literal_single`
- **Input:** `{1};`
- **Expected:** `Expr::SetLiteral(vec![Literal(Int(1))])`
- **Actual:** The parse succeeds but as
  `Stmt::Block(Block([ExprStmt(Literal(Int(1)))]))` — `{1}` is a
  block containing `1`, not a set.
- **Reason:** Same as above — the dict-or-set disambiguation
  produces a block when the first inner expression is not followed
  by `:`.

### Test: `test_expr_set_literal_mixed`
- **Input:** `{1, "x", True};`
- **Expected:** `Expr::SetLiteral(vec![...])`
- **Actual:** `ParserError::UnsupportToken(Token::Comma)`.
- **Reason:** Same as `test_expr_set_literal`.

---

## 2. Comparison operators `in`, `not in`, `is`, `is not` are not implemented

`CompareOp::In`, `CompareOp::NotIn`, `CompareOp::Is`, and
`CompareOp::IsNot` are defined in `ast.rs` (lines 220-223), but
`parse_comparison` (parser.rs:583-616) only matches
`Eq, Ne, Le, Lt, Ge, Gt` and returns those `CompareOp` variants. The
`in` / `is` keyword tokens are never consumed inside
`parse_comparison`, so the surrounding `parse_assignment_stmt` falls
through and the outer parser then chokes on the leftover keyword.

### Test: `test_expr_in`
- **Input:** `1 in [1, 2];`
- **Expected:** `Expr::Compare { op: CompareOp::In, left: Literal(Int(1)), right: ListLiteral([1, 2]) }`
- **Actual:** `ParserError::UnsupportToken(Token::Kw(Keyword::In))`.
  `parse_comparison` returns `Literal(Int(1))` unchanged, then
  `parse_assignment_stmt` sees `in` and falls through to
  `Stmt::ExprStmt(Literal(Int(1)))`. The outer loop tries to parse
  the next statement starting at `in` and errors.
- **Reason:** `parse_comparison` does not consume `in` / `is` /
  `not in` / `is not` keyword sequences.

### Test: `test_expr_not_in`
- **Input:** `1 not in [1, 2];`
- **Expected:** `Expr::Compare { op: CompareOp::NotIn, ... }`
- **Actual:** `ParserError::UnsupportToken(Token::Kw(Keyword::In))`.
- **Reason:** Same as above. The `not` keyword is consumed by
  `parse_not_expr` first, but the subsequent `in` still has no
  consumer in `parse_comparison`.

### Test: `test_expr_is`
- **Input:** `x is None;`
- **Expected:** `Expr::Compare { op: CompareOp::Is, ... }`
- **Actual:** `ParserError::UnsupportToken(Token::Kw(Keyword::Is))`.
- **Reason:** Same as above.

### Test: `test_expr_is_not`
- **Input:** `x is not None;`
- **Expected:** `Expr::Compare { op: CompareOp::IsNot, ... }`
- **Actual:** `ParserError::UnsupportToken(Token::Kw(Keyword::Is))`.
- **Reason:** Same as above.
