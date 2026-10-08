# Interfaces and traits

ORust interfaces map directly to Rust traits. Interface inheritance is written
with `extends` and becomes a Rust supertrait:

```orust
interface Animal { void speak(); }
interface Pet extends Animal { type Kind; Kind value(); }
```

Associated types use `type Name;` and are implemented by the generated Rust
implementation. A class implementation supplies the concrete type with
`type Item = String;` inside its class body. A plain interface type remains dynamically dispatched through
`Box<dyn Trait>`; using an interface as a generic bound keeps static dispatch.

The standard `Default` and `From<T>` names are also recognized in
`implements`. `Default` delegates to `new()`, while `From<T>` delegates to a
named `from` constructor and enables ordinary Rust `.into()` calls.

`Comparable`, `Hashable`, and `Clone` are built-in capability names. They map
to Rust derives (`PartialEq`/`Ord`, `Hash`, and `Clone`) rather than an invented
parallel trait runtime.

For `implements Comparable<T>`, define `int compareTo(T other)`. ORust emits
`Ord` and `PartialOrd`; the sign of `compareTo` supplies Rust's
`std::cmp::Ordering`.
