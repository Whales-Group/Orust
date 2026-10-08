# Pattern matching

`switch` patterns are emitted as Rust match arms, so rustc remains responsible
for exhaustiveness and unreachable-pattern diagnostics. ORust supports numeric
ranges, or-patterns, and guards:

```orust
switch (value) {
  case 1..=5: { print("small"); }
  case 6 | 7 when value > 0: { print("positive"); }
  default: { print("other"); }
}
```

Constructor patterns retain their bindings, while `_` remains a wildcard.
Nullable `null`/`var value` cases normalize to `None`/`Some(value)`. Range
bindings such as `n @ 1..=5` and list rest patterns such as
`[first, ..rest]` lower to Rust's `@` and slice-rest syntax.

Refutable declarations use Rust's `let-else` scope rules:

```orust
var Circle(radius) = shape else { return; }
print(radius);
```

The binding remains available after the declaration, and the `else` branch
must diverge just as it does in the generated Rust.
