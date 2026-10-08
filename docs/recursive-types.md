# Recursive types

ORust lets classes refer to themselves directly:

```orust
class Node {
  int value;
  Node? next;
  Node(this.value, this.next);
}
```

Rust cannot store a recursive struct by value because its size would be
infinite. ORust detects direct recursive value cycles and inserts a `Box` at a
deterministic edges. Optional edges are preferred when choosing the cycle
shape; every direct recursive field/payload in the selected component is boxed
so structs with two child fields and variants with multiple recursive payloads
remain finite. `List`, `Map`, `Set`, shared
values, and existing boxes are already indirect and do not need another
allocation.

The generated shape is equivalent to:

```rust
struct Node {
    value: i64,
    next: Option<Box<Node>>,
}
```

The source language and constructor calls remain unchanged; ownership and
drop behavior are still Rust's behavior.

The same rule applies to self-referential enum payloads. For example,
`Add(Expr, Expr)` becomes a finite Rust variant with both direct payloads
stored behind `Box<Expr>`; otherwise one unboxed payload would still leave an
infinite-size variant.

Enum construction is rewritten at the same boundary: `new Expr.Add(left,
right)` remains unboxed in ORust and receives `Box::new` only for generated
recursive payload edges.
