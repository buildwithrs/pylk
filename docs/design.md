# PyLike Interpreter Design

- Does not support Indentation, use `{` and `}` for blocks.
- Does not support class base class(no parent class).
- Does not support `Try`, `Except`, `Finally`, `Raise`, or `Assert` statements.
- Does not support `async` or `await` keywords.
- Does not support `with` or `yield` statements.
- Does not support `IMAGINARY` literals.
- Does not support `:=` assign.
- Does not support `Ellipsis(...)` literal.
- Does not support `type_annotations` or `type_comments`.

## Tokens

The lexer uses Python-style token categories. The keyword tokens explicitly excluded by the design constraints above are not included in this list. Tokenizing a keyword does not require the parser to support its statement or expression yet.

### Names and literals

| Token | Python forms |
| --- | --- |
| `IDENTIFIER` | Names beginning with a letter or `_`, followed by letters, digits, or `_`. Unicode names may be added later. |
| `INTEGER` | Decimal, binary (`0b...`), octal (`0o...`), and hexadecimal (`0x...`) integers; digit separators (`_`) are allowed. |
| `FLOAT` | Decimal and exponent notation, such as `3.14`, `.5`, or `1e6`. |
| `STRING` | Single-quoted, double-quoted, and triple-quoted strings. Python string prefixes (`r`, `b`, `u`, and `f`) are reserved for later support. |
| `BOOLEAN` | `True` and `False`. |
| `NONE` | `None`, lexed as `Token::Kw(Keyword::None)`. |
| `ELLIPSIS` | `...`. |

### Keyword tokens

The lexer classifies these Python keywords as `KEYWORD` tokens:

```text
None
and  as  break  class  continue  def  del  elif  else
for  from  global  if  import  in  is  lambda  nonlocal
not  or  pass  return  while
```

`True` and `False` are represented by `BOOLEAN` tokens, and `None` is represented by `Token::Kw(Keyword::None)`. `case` and `match` are Python soft keywords. They remain `IDENTIFIER` tokens unless the parser is in a context where their syntax is recognized.

### Operators

```text
+    -    *    **   /    //   %    @
<<   >>   &    |    ^    ~
<    <=   >    >=   ==   !=
=    +=   -=   *=   /=   //=  %=   **=
@=   &=   |=   ^=   <<=  >>=
```

`!` and `?` are not standalone Python operators. `!=` is a single comparison operator, and Python conditional expressions use the `if ... else ...` keywords.

### Delimiters and special tokens

```text
(  )  [  ]  {  }
,  :  .  ;  ->
```

The lexer also recognizes end-of-input (`EOF`) and may report invalid input as `ERROR`. Whitespace and comments beginning with `#` are ignored. Newlines are ordinary whitespace in PyLike; `INDENT` and `DEDENT` tokens are therefore not emitted because blocks use `{` and `}`.
