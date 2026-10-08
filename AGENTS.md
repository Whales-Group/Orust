# AGENTS.md: ORust (.or)

Instructions for any coding agent working in this repo. Read this at the start of every session.

## What this project is

ORust is a transpiler plus tooling that lets developers from OOP languages (Dart, Java, C#, Swift) write Rust through familiar vocabulary.

- ORust source (`.or`) compiles to plain Rust.
- `rustc` does the real type checking and borrow checking.
- Our main value-add is translating `rustc` errors back into friendly, OOP-flavored messages that point at `.or` source lines.

**Core principle:** `class` is a friendlier spelling of `struct` + `impl`. It looks like a class but behaves like Rust. Ownership stays real. We rename it; we never hide it by default.

## Hard rules (never violate)

1. **Never reimplement the borrow checker.** `rustc` is the source of truth.
2. **No inheritance.** There is no `extends`. Reuse comes from `implements` (traits) and composition.
3. **No hidden `Rc<RefCell<>>`.** Sharing only happens when a class is explicitly marked `shared`.
4. **No garbage collection** and no custom runtime.
5. **Error messages show `.or` source, never generated Rust.** Fall back to a cleaned-up `rustc` message only when no template exists.
6. **Every AST node carries a span.** The error translator depends on it.

## Tech choices

- Language: Rust, Cargo workspace.
- Lexer: `logos`. Parser: `chumsky` (or hand-written recursive descent if error recovery is easier).
- Codegen: emit Rust text via a small pretty-printer (or `quote` + `prettyplease`). Output goes to a generated cargo project under `target/orust/`.
- Diagnostics: run `cargo check --message-format=json`, parse it, and map spans back to `.or` using a span table recorded during codegen.
- Tests: `insta` or a simple snapshot harness.

### Workspace layout

| Crate | Purpose |
|---|---|
| `orust-syntax` | lexer, parser, AST |
| `orust-lower` | inference of `mut` and lifetimes, desugaring |
| `orust-emit` | Rust codegen and span table |
| `orust-diag` | rustc error translator |
| `orust-cli` | the `orust` binary |

### CLI

- `orust check <file.or>`
- `orust build <file.or>`
- `orust run <file.or>`
- `orust emit <file.or>` (prints the generated Rust)

Binary name: `orust`. File extension: `.or`.

## Language surface (v0.1)

| ORust | Emitted Rust |
|---|---|
| `class Foo { int x = 0; void inc() { x++; } }` | `struct Foo { x: i32 }` + `impl Foo { fn new() -> Self; fn inc(&mut self) }` |
| `new Foo()` | `Foo::new()` |
| `this` | `self` |
| `interface Speak { String speak(); }` | `trait Speak { fn speak(&self) -> String; }` |
| `class Dog implements Speak` | `impl Speak for Dog` |
| `var x = ...;` | `let x = ...;` (add `mut` automatically if mutated) |
| `lend T` / `lend a` | `&T` / `&a` |
| `lend mut T` / `lend mut a` | `&mut T` / `&mut a` |
| `copy a` | `a.clone()` |
| `T?` | `Option<T>` |
| `int`, `double`, `bool`, `String` | `i64`, `f64`, `bool`, `String` |
| `[1, 2, 3]`, `List<T>` | `vec![..]`, `Vec<T>` |
| `print(x)` | `println!("{}", x)` |
| `shared class Foo` | `Rc<RefCell<FooData>>` with auto `.borrow()` / `.borrow_mut()` / `.clone()` (milestone 6) |
| `rust { ... }` | raw Rust passthrough block (milestone 7) |
| `async`, `await`, `spawn` | Tokio futures and tasks with real `Send + 'static` checks |
| `interface I` / `implements I` | Rust traits and trait implementations |
| `operator_eq`, `operator_lt`, `operator_index` | `PartialEq`, `PartialOrd`, and `Index` implementations |
| `f(x: value)` / `T x = default` | generated `XArgs` parameter objects for named/optional calls |

## Inference rules

- `mut` is inferred from usage: assignment, `lend mut`, or calling a method that needs `&mut self`.
- Method receivers: infer `&self` vs `&mut self` vs `self` from the method body.
- Function parameters: plain `T` means move, `lend T` means `&T`, `lend mut T` means `&mut T`.
- Lifetimes: rely on Rust elision. If a function returns a `lend` and has multiple `lend` inputs, emit one shared lifetime `'a` on all of them. If that is ambiguous, error and ask for `lend(a) T f(...)` syntax.
- Async functions are lazy: calling one creates a future; execution begins only at `await` or `spawn`.
- `spawn` keeps Tokio's `Send + 'static` requirements visible. ORust does not infer away ownership constraints or add garbage collection.
- Operator overloading is explicit: `operator_add`, `operator_eq`, `operator_lt`, `operator_gt`, and `operator_index` are lowered to Rust traits.
- A top-level function with a default parameter receives a generated `<Function>Args` struct. Named and positional calls populate `Option` fields; defaults are resolved inside the function.
- Interface objects are boxed when owned. Spawned programs add `Send + Sync` bounds; async interfaces use `async_trait`.

## Error translator

Parse `rustc` JSON diagnostics, map spans back to `.or`, and rewrite by error code. Each message has: one plain sentence on what happened, the relevant `.or` lines, and concrete fixes in ORust vocabulary (`copy`, `lend`, `lend mut`, `shared class`).

| Code | Friendly headline |
|---|---|
| E0382 | "`a` no longer has its Counter" (fixes: `copy a`, `lend a`, make the class `shared`) |
| E0499 | "two things are trying to edit `a` at once. Only one editor at a time." |
| E0502 | "you changed `items` while `first` was still looking at it" |
| E0106 | rare, since we infer lifetimes; if it fires, ask for `lend(a)` syntax |
| E0505 | "tried to give away `a` while something was still looking at it" |

Target output shape:

```
error: `a` no longer has its Counter
 --> main.or:3:9
  |
2 |   var b = a;      // a handed its Counter to b here
3 |   print(a.get()); // but a is used again here
  |
help: give b its own Counter:    var b = copy a;
help: let b look without owning: var b = lend a;
help: have both see the same one: make Counter a `shared class`
```

## Phase 3 locked decisions

- Files are private by default. `export` controls cross-file visibility;
  `internal` lowers to `pub(crate)` and `private` remains non-public.
- Imports and re-exports use explicit Rust paths. `show`, `hide`, and `as` do
  not create a runtime namespace or bypass Rust privacy.
- `orust build --lib` emits a conventional Cargo library target. Hand-written
  `.rs` files are copied unchanged and checked by `rustc`.
- Module names use snake_case with raw identifiers for Rust keywords. An
  explicit `@rustName("...")` may override an emitted declaration name.

## Milestones (do in order, commit after each, keep tests green)

1. **Skeleton + hello world:** workspace, lexer, parser for `class`, fields, methods, `var`, `print`, `if`/`while`/`for`, literals, basic expressions. `orust emit` and `orust run` work for a single-class program.
2. **Borrowing:** `lend`, `lend mut`, `copy`, `new`, `this`, inferred `mut` and `&self`/`&mut self`. The example program below runs.
3. **Span table + diagnostics plumbing:** record `.or` to `.rs` span mappings in codegen. Run `cargo check` JSON and map errors back to `.or` locations.
4. **Error translator:** E0382, E0499, E0502 templates with golden tests.
5. **Interfaces and Option:** `interface`, `implements`, `T?`, `?.`, `??`, null checks.
6. **`shared class`:** Rc/RefCell emission with auto borrow insertion, plus runtime panic message rewriting.
7. **Lifetime inference + `rust { }` passthrough.**

### Milestone 2 acceptance program

```
class Counter {
  int count = 0;
  void inc() { count++; }
  int get() { return count; }
}

void show(lend Counter c) {
  print(c.get());
}

void main() {
  var a = new Counter();
  a.inc();
  show(lend a);
  show(lend a);
}
```

Expected emitted Rust (shape, not exact formatting):

```rust
struct Counter { count: i32 }
impl Counter {
    fn new() -> Self { Counter { count: 0 } }
    fn inc(&mut self) { self.count += 1; }
    fn get(&self) -> i32 { self.count }
}
fn show(c: &Counter) { println!("{}", c.get()); }
fn main() {
    let mut a = Counter::new();
    a.inc();
    show(&a);
    show(&a);
}
```

## Testing requirements

- Golden tests live in `tests/cases/`: `*.or` input, `*.rs` expected emitted Rust, `*.err` expected friendly error (when applicable).
- Every example in this file becomes a test case.
- End-to-end tests compile and run sample programs and assert on stdout.
- Parser tests cover syntax errors with helpful messages and correct spans.
- Run `cargo fmt`, `cargo clippy`, and `cargo test` before every commit.

## Working rules

- Don't stop to ask questions. When something is ambiguous, pick the simplest option that fits the core principle and record it in `docs/decisions.md`.
- Finish the current milestone's tests before starting the next one.
- Prefer small, reviewable commits with clear messages.
- Keep the AST and span plumbing clean from day one.
- Maintain `README.md` (language tour) and `docs/error-translation.md` (how templates work).
- If you drift, re-read this file and the milestone list, then continue.

## rustplain project rules

The `rustplain-*` crates are a standalone diagnostic explainer for any Rust
project. Rustc remains the source of truth: never reimplement the borrow
checker, type checker, trait solver, or name resolution. Prefer structured JSON
fields over message matching, never fabricate explanations or edits, and label
confidence. Keep `--raw` available, preserve underlying exit codes, remain
offline and deterministic, and never panic on malformed input. Show the user's
source rather than macro or standard-library internals. Keep explanation text
in templates/data as the rule engine grows; unknown diagnostics must fall back
to cleaned-up original rustc output.

## Phase 5 locked decisions

- Direct recursive value cycles are auto-boxed deterministically. Every direct
  recursive field/payload in the selected component is boxed so binary trees
  and variants with multiple recursive operands remain finite.
- Range slices use checked `usize` bounds and remain borrowed Rust views.
- `for (var item in values)` borrows the collection; consuming iteration must
  be explicit.
- `interface extends` and associated `type` declarations lower to Rust
  supertraits and associated types; class implementations provide concrete
  associated types with `type Item = T;`.
- `typedef` is transparent; `type Name(T);` is a distinct tuple newtype.
- `String.length` is UTF-8 byte length; `charCount()` counts Unicode scalar
  values. Collection adaptors are lazy Rust iterators and `toList`/`toSet`
  are explicit collection terminals.
- Refutable destructuring declarations lower to Rust `let-else`; bindings stay
  in scope after the declaration and the `else` branch must diverge.
- Generic transparent aliases, including `void Function(T)` aliases, lower to
  ordinary Rust aliases and boxed `Fn` trait objects.
