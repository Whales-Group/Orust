# Aliases and newtypes

Type aliases preserve interchangeability:

```orust
typedef UserId = int;
```

This emits `type UserId = i64`. Newtypes are distinct and require an explicit
constructor:

```orust
type UserId(int);
```

This emits a tuple struct with a public inner value. Rust therefore rejects
passing a plain `int` where `UserId` is expected, preserving the distinction at
API boundaries. In ORust source, `id.value` is rewritten to the generated
accessor (`id.value()` in Rust), and `new UserId(5)` remains the source-level
constructor spelling.

Tuple values and positional access use the familiar ORust spelling:

```orust
(int, String) pair = (1, "one");
print(pair.$1);
```

They lower to Rust tuples and `.0`/`.1` access.

Named record shapes are anonymous in source but canonical in generated Rust:

```orust
({int x, String label}) point = (x: 1, label: "one");
```

The emitter creates a deduplicated public struct in `orust_records`, named from
the ordered field names and types. Record fields are public so the source-level
`point.x` access remains direct; derives are selected only when the field types
support them.

Generic aliases are also transparent. For example,
`typedef Callback<T> = void Function(T);` emits a boxed Rust `Fn` trait object
so the alias remains usable at an ORust/Rust boundary.
