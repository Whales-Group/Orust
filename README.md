# ORust

ORust is an OOP-friendly front end for Rust. It translates `.or` source into ordinary Rust while leaving ownership and borrow checking to `rustc`.

The current prototype supports parsing and emitting classes, real Rust
borrowing, async Tokio programs, interfaces, generics, enums, shared classes,
closures, operators, named parameters, collection helpers, lifetime hints, and
Rust passthrough blocks.

## Language tour

Classes are Rust structs plus `impl` blocks. `int` is a 64-bit `i64` value;
ownership is never hidden:

```orust
class Counter {
  int count = 0;
  void inc() { count++; }
}

void main() {
  var counter = new Counter();
  counter.inc();
  print(counter.count);
}
```

Use `lend` for a shared borrow, `lend mut` for an exclusive mutable borrow, and
`copy` when a value should be cloned. Mark a class `shared` only when shared
ownership is intended; ORust then emits `Rc<RefCell<...>>`, or
`Arc<tokio::sync::Mutex<...>>` when spawned work requires thread-safe sharing.

Interfaces emit traits, `implements` emits trait implementations, and owned
interface values use `Box<dyn Trait>`. `T?` emits `Option<T>`, with `?.` and
`??` available for optional values. `async`, `await`, `spawn`, `Future<T>`,
`Stream<T>`, and `Channel<T>` map to Tokio/runtime primitives while preserving
`Send` and `'static` constraints.

Phase 5 adds Rust-shaped depth without replacing Rust semantics:

```orust
class Node {
  int value;
  Node? next;
  Node(this.value, this.next);
}

void inspect(List<String> values) {
  for (var value in values) {
    print(value.toUpperCase());
  }
  var middle = values[1..];
}
```

Direct recursive fields are boxed automatically at the generated Rust
boundary, slices use checked bounds, and `for-in` borrows by default. Interfaces
support `extends` and associated types; `typedef` is transparent, while
`type EmailId(String);` is a distinct newtype. See the Phase 5 feature guides:
[recursive types](docs/recursive-types.md), [slices](docs/slices.md),
[patterns](docs/patterns.md), [iterators](docs/iterators.md),
[traits](docs/traits.md), [strings](docs/strings.md), and
[records](docs/records.md), and [numbers](docs/numbers.md).

Operator methods emit Rust operator traits. Named and optional parameters use a
generated parameter struct, so `greet(name: "Jesse")` is explicit in generated
Rust and omitted defaults remain visible.

For the complete syntax and ownership rules, see [AGENTS.md](AGENTS.md),
[async semantics](docs/async.md), and [polymorphism](docs/polymorphism.md).
The next planned module work is tracked in [Phase3.md](Phase3.md).

Projects may filter local imports with `show` and `hide`:

```orust
import 'models/user.or' show User, Admin hide Secret;
import 'utils/strings.or' as strings;
```

Imports are emitted as explicit Rust `use` statements. Rust remains the
authority for privacy and type errors. Projects may use either `orust.toml`
or a standard `Cargo.toml`; dependency entries are forwarded to the generated
Cargo project.

For example, a Cargo manifest can declare `serde_json`, then an embedded Rust
block can use `serde_json::json!` normally.

```sh
cargo run -p orust-cli -- check tests/cases/hello.or
cargo run -p orust-cli -- emit tests/cases/hello.or
cargo run -p orust-cli -- build tests/cases/hello.or
cargo run -p orust-cli -- build --lib tests/cases/hello.or
cargo run -p orust-cli -- run tests/cases/hello.or
cargo run -p orust-cli -- new my-orust-project
cargo run -p orust-cli -- new my-orust-library --lib
cargo run -p orust-cli -- explain OR0005
```

The root [examples](examples) directory contains three small programs. Once the CLI is installed, a project can be scaffolded with `orust new my-orust-project`.

The language tour and implementation rules are documented in [AGENTS.md](AGENTS.md). Progress is tracked in [Phases.md](Phases.md). Generated Cargo projects and Rust JSON diagnostics are written under `target/orust/`.
