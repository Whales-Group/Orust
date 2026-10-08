# Decisions

## 2026-10-08

- Milestone 1 starts with a small hand-written lexer and recursive-descent parser. This keeps spans explicit and avoids committing to parser-combinator error-recovery details before the language surface is exercised.
- Operator overloading is opt-in through named `operator_*` methods. Codegen implements the matching Rust traits, so operator behavior remains ordinary Rust behavior.
- Named and optional parameters use generated `<Function>Args` structs. Every field is an `Option`; the callee applies defaults or reports a missing required argument, making omitted arguments explicit in generated Rust.
- Interface-typed owned values use `Box<dyn Trait>`, while borrowed interface values use `&dyn Trait` or `&mut dyn Trait`. Spawned programs add `Send + Sync` bounds rather than weakening Tokio requirements.
- Async interfaces use `async_trait`, and async `main` uses `#[tokio::main]`; these are emission conveniences, not alternate runtime semantics.
## Async Futures are lazy

An `async` call creates a lazy Future. It runs only when awaited or spawned.
This matches Rust and makes dropped work visible through the CLI warning.

## Shared state with spawned work

Explicitly shared classes use `Rc<RefCell<T>>` without spawned work and
`Arc<tokio::sync::Mutex<T>>` when `spawn` appears in the program. This keeps
single-threaded code lightweight while preserving Tokio's real `Send` and
`'static` requirements for spawned tasks.
# Phase 3 module decisions

- A configured project is discovered by walking upward for `orust.toml`; its
  `entry` value identifies the crate-root `.or` file.
- The first module implementation mirrors direct child directories and files
  into nested `pub mod` declarations. Imports become explicit `use` lists,
  never globs; aliases import the module path.
- `export` is represented on top-level declarations and lowers to Rust `pub`.
  Class members are public only as part of an exported class; unexported types
  remain file-private and are intentionally left to `rustc` for enforcement.
- `show` and `hide` are import filters, not visibility declarations. The
  generator keeps imports explicit and applies the filters before writing the
  Rust `use` list; aliases continue to represent a module namespace.
- Rust interop remains intentionally shallow until package generation is
  available: `.rs` files are not parsed by ORust, and Rust signatures are
  checked by `rustc` rather than duplicated in the frontend.
- `orust build --lib` generates a conventional Cargo library target. The
  `@rustName` bridge uses one emitter symbol map for declarations, references,
  imports, and re-exports while diagnostics retain ORust source names.
- `int` is represented as `i64`, matching Dart's 64-bit integer model. Collection
  lengths are explicitly cast to `i64`; Rust still performs the bounds check.
- Borrowed class fields share one inferred `'a` lifetime. This keeps the common
  case readable and leaves named lifetime grouping for a later phase.
- Initializing formals such as `Reader(this.text)` define the constructor
  parameter from the field declaration; a borrowed field therefore produces an
  `&'a` constructor parameter and Rust remains responsible for lifetime checks.

## rustplain P1

- `rustplain` is a separate set of workspace crates and has no dependency on
  ORust or any transpiler. It consumes rustc/Cargo JSON as its source of truth.
- `serde` and `serde_json` provide tolerant structured diagnostic parsing;
  `clap` provides the first CLI surface; `thiserror` provides parser errors.
- The first renderer is intentionally deterministic and plain-text. Cargo
  process streaming and richer terminal rendering belong to later milestones.

## ORust Phase 5: recursive types

- Recursive direct class edges are auto-boxed at the syntax-to-Rust boundary.
  Every direct recursive field/payload in the selected component is boxed,
  because a binary tree or variant such as `Add(Expr, Expr)` remains infinite
  if only one recursive occurrence is boxed. Collection, shared, and
  existing-box edges are already indirect and do not participate in cycle
  breaking.
- `for (var item in values)` borrows the collection and emits `for item in
  &values`; consuming iteration must be explicit so a loop cannot silently
  move a user-owned collection.
- Interface `extends` and associated `type` declarations map directly to Rust
  supertraits and associated types; no parallel ORust trait runtime is added.
- `typedef` remains a transparent Rust type alias, while `type Name(T);` emits
  a tuple newtype so Rust enforces explicit wrapping at API boundaries.
- Match guards, or-patterns, and ranges are emitted as Rust pattern syntax;
  rustc remains responsible for exhaustiveness and unreachable-pattern checks.
- `implements Iterator<T>` is lowered to Rust's standard `Iterator` trait with
  `type Item = T`; iterator protocol methods remain ordinary Rust methods.
- String `length` follows Rust `String::len` (UTF-8 bytes), while
  `charCount()` is provided for Unicode scalar counts; this makes the cost and
  semantics explicit rather than pretending all characters have one byte.
- `for-in` and collection adaptors borrow through `.iter()` by default. A
  consuming form remains explicit so ownership cannot disappear in generated
  code.
- Numeric helper methods preserve Rust's checked, wrapping, and saturating
  families instead of inventing runtime arithmetic; `int` remains `i64` while
  explicit fixed-width types are passed through.
- Refutable destructuring uses Rust `let-else` directly so binding scope and
  divergence remain Rust semantics rather than a compiler-simulated scope.
- Function type aliases use `Box<dyn Fn(...) -> ()>` as the first stable
  representation; this keeps aliases usable across ORust/Rust boundaries while
  avoiding an implicit ownership or calling-convention conversion.
