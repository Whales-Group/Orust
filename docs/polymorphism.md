# Polymorphism

ORust interfaces emit Rust traits. A class declares conformance with
`implements`; there is no inheritance or implicit downcast.

```orust
interface Speak { String speak(); }
class Dog implements Speak {
  String speak() { return "woof"; }
}
```

Owned interface values use `Box<dyn Trait>`. Concrete values assigned to an
interface slot are boxed automatically, and `List<Speak>` becomes a vector of
boxed trait objects. Borrowed interface parameters remain real Rust borrows:
`lend Speak` emits `&dyn Speak`, while `lend mut Speak` emits `&mut dyn Speak`.

When a program uses `spawn`, interface objects receive `Send + Sync` bounds so
Tokio's requirements stay visible. Async interfaces use `async_trait` because
Rust traits cannot contain ordinary async object methods on their own. Default
interface methods are emitted as trait method bodies.

## Generic polymorphism

Generic classes and functions emit Rust generics and explicit trait bounds.
`extends`-style syntax is treated as a trait bound, never as class
inheritance. Data-carrying enums and `switch` lower to Rust enums and `match`.
The generated Rust compiler remains the authority for dyn compatibility,
missing methods, and unsatisfied trait bounds; ORust only translates those
diagnostics into friendlier vocabulary.

## Operators and calls

Operator methods such as `operator_eq`, `operator_lt`, and `operator_index`
generate the corresponding Rust trait implementations. Named arguments and
defaults use a generated parameter object, so `greet(name: "Jesse")`
constructs `greetArgs`; omitted optional fields are `None` and are resolved in
the function body. Required omitted fields fail explicitly rather than being
silently invented.
