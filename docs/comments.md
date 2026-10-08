# Comments and documentation

ORust keeps comments as source metadata while parsing the executable syntax.
The original source spelling, byte span, and line range are retained on every
`Comment`, so tooling can make a lossless decision without reconstructing the
comment text.

## Comment forms

- `// ...` and `/* ... */` are ordinary comments.
- `/// ...` is an outer line documentation comment. `//// ...` is ordinary.
- `/** ... */` is an outer block documentation comment. The exact spellings
  `/**/` and `/***/` remain ordinary block comments.
- `//! ...` and `/*! ... */` are inner documentation comments for the current
  container.
- A `#!...` shebang is retained only when it is the first line (after an
  optional UTF-8 BOM).
- Block comments may nest. An unterminated nested block reports the span from
  the opening delimiter to end of input.

## Attachment

Documentation immediately before a declaration attaches to that declaration.
Same-line comments after a syntax span are trailing trivia. A blank line
detaches a comment from the following declaration. Comments inside argument,
parameter, and expression lists remain comments in the nearest source span;
they do not alter parsing or ownership semantics.

The syntax API exposes `Program::docs`, `Program::leading_trivia`,
`Program::trailing_trivia`, and the corresponding methods on `Spanned<T>`.
Outer documentation with no nearby declaration produces warning `OR0601`.

The source text is retained on `Program` for comment-aware emission. `orust
emit` preserves ordinary comments by default (use `--no-comments` to suppress
them); `build` keeps comments out of generated artifacts. Comment emission is
a tooling concern and does not change generated Rust semantics.
