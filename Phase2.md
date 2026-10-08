# ORust Phase 2 — Dart-style async and polymorphism

Read `AGENTS.md` before working. Continue milestone numbering from the original plan at milestone 8. Keep Rust ownership and borrow checking real; do not hide `Rc`, `Arc`, `Mutex`, `Send`, or `'static` constraints.

## Part A — Futures on Tokio

### Milestone 8 — Basic async

- [x] Add `async` function parsing and emit `async fn`.
- [x] Add `Future<T>` parsing and emit the underlying Rust output type.
- [x] Add `await` expressions and emit `.await`.
- [x] Emit `#[tokio::main]` automatically for an async `main`.
- [x] Add `Future.delayed` and duration helpers such as `100.millis` and `1.seconds`.
- [x] Add the `orust-runtime` crate with Tokio and Futures re-exports.
- [x] Add the milestone-8 acceptance program without `spawn`.
- [x] Run the acceptance program and verify deterministic stdout.

### Milestone 9 — Concurrency and async errors

- [x] Add `Future.wait` using `join_all` for lists and `tokio::join!` for fixed arity.
- [x] Add `Future.any` using `select!` or `select_all`.
- [x] Add `spawn` and `Task<T>`.
- [x] Emit task awaiting and surface join errors as ORust `Error`.
- [x] Add `throws` functions returning `Result<T, Error>`.
- [x] Insert `?` when awaiting throwing futures inside `throws` functions.
- [x] Add `try/catch` around async operations.
- [x] Warn when a Future is created and dropped without being awaited or spawned.

### Milestone 10 — Async diagnostics and shared state

- [x] Translate E0373 into clear `spawn` ownership guidance.
- [x] Translate async E0277 (`Send` not satisfied).
- [x] Translate E0597 and E0521 borrowed-value lifetime failures.
- [x] Translate E0728 (`await` outside async).
- [x] Make shared classes context-sensitive: `Rc<RefCell>` without `spawn`, `Arc<tokio::sync::Mutex>` with `spawn`.
- [x] Insert `.lock().await` for async shared-class access.
- [x] Document the lazy-future semantic decision in `docs/async.md`.

## Part B — Polymorphism

### Milestone 11 — Dynamic dispatch

- [x] Emit interface-typed owned values as `Box<dyn Trait>`.
- [x] Emit borrowed interface values as `&dyn Trait` or `&mut dyn Trait`.
- [x] Emit `List<Interface>` as `Vec<Box<dyn Trait>>`.
- [x] Auto-insert `Box::new` for concrete values assigned to interface slots.
- [x] Add automatic `async_trait` support for async interfaces.
- [x] Add `Send + Sync` supertraits when `spawn` is used.
- [x] Add interface default methods.
- [x] Translate E0038 dyn-compatibility failures.

### Milestone 12 — Generics and enums

- [x] Add generic classes and functions.
- [x] Add `extends`-style trait bounds without inheritance.
- [x] Infer simple bounds from method usage where safe.
- [x] Translate E0277 trait-bound failures for generic values.
- [x] Translate E0599 missing methods on generic values.
- [x] Add data-carrying enums.
- [x] Add `switch` pattern matching and emit Rust `match`.
- [x] Add `if (x case Pattern(...))` and emit `if let`.
- [x] Translate E0004 missing enum cases.
- [x] *(Stretch)* Add `Stream<T>` and `await for`.
- [x] *(Stretch)* Add `Channel<T>` backed by Tokio mpsc.

### Milestone 13 — Language quality of life

- [x] Add operator overloading for arithmetic, comparison, equality, and indexing.
- [x] Add extension traits for `extension on Type`.
- [x] Add `@data` derives and plain-class `Debug` derives.
- [x] Map `toString()` to Rust `Display`.
- [x] Add expression and block closures.
- [x] Infer `move` for closures passed to `spawn`.
- [x] Add ORust function types and emit `Fn`/`FnMut`/`FnOnce` forms.
- [x] Add named and optional parameters through generated parameter structs.
- [x] Add collection helpers for lists, maps, and sets.

### Milestone 14 — Polish

- [x] Update `docs/async.md`.
- [x] Add `docs/polymorphism.md`.
- [x] Update the README language tour.
- [x] Update `AGENTS.md` with newly locked-in design decisions.
- [x] Record important choices in `docs/decisions.md`.

## Testing checklist

- [x] Add `.or` → `.rs` golden tests for every syntax/emission table row.
- [x] Add `.err` fixtures for every listed Rust diagnostic.
- [x] Add runtime tests asserting deterministic stdout and ordering.
- [x] Use Tokio virtual time or short deterministic delays for concurrency tests.
- [x] Keep every milestone green before starting the next one.

## Every-phase checklist

- [ ] Keep every AST node span-aware.
- [ ] Keep every AST node span-aware.
- [x] Keep Rust ownership, borrowing, `Send`, and `'static` semantics visible.
- [x] Delegate type and borrow checking to `rustc`.
- [x] Update docs and decisions with each locked-in design choice.
- [x] Run `cargo fmt`.
- [x] Run `cargo clippy`.
- [x] Run `cargo test`.

## Starting point

- [ ] Begin with milestone 8: basic async, Tokio runtime support, and the acceptance program without `spawn`.
