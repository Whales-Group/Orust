# ORust Development Phases

Use this checklist to track progress. Complete and verify each phase before moving to the next one.

## Phase 1 — Skeleton and hello world

- [x] Create the Cargo workspace and planned crates.
- [x] Add a span-carrying AST foundation.
- [x] Add the initial hand-written lexer and recursive-descent parser.
- [x] Parse classes, fields, methods, functions, variables, `print`, `return`, literals, and basic expressions.
- [x] Add the `orust emit <file.or>` CLI command.
- [x] Add the first golden input/output case.
- [x] Add README, decisions, and diagnostics documentation.
- [x] Add complete `if`, `while`, and `for` parsing and emission.
- [x] Add parser error tests with source spans.
- [x] Make emitted milestone-1 programs compile and run through `orust run`.

## Phase 2 — Borrowing and ownership vocabulary

- [x] Parse `lend` and `lend mut` types and expressions.
- [x] Parse `copy`, `new`, and `this`.
- [x] Infer local `mut` from assignments and mutable borrows.
- [x] Infer method receivers: `&self`, `&mut self`, or `self`.
- [x] Emit the Counter acceptance program correctly.
- [x] Add end-to-end tests that compile and run the acceptance program.

## Phase 3 — Span table and diagnostics plumbing

- [x] Record `.or` to generated `.rs` span mappings during emission.
- [x] Generate a temporary Cargo project under `target/orust/`.
- [x] Run `cargo check --message-format=json` on generated Rust.
- [x] Parse compiler diagnostics and map generated spans back to `.or`.
- [x] Add diagnostics plumbing tests.

## Phase 4 — Friendly error translator

- [x] Add the E0382 use-after-move template.
- [x] Add the E0499 simultaneous mutable-borrow template.
- [x] Add the E0502 mutable-borrow-while-borrowed template.
- [x] Add the E0106 missing-lifetime fallback guidance.
- [x] Add the E0505 move-while-borrowed template.
- [x] Show `.or` source lines in every translated diagnostic.
- [x] Add golden `.err` tests for each template.

## Phase 5 — Interfaces and Option

- [x] Parse and emit `interface` as Rust traits.
- [x] Parse and emit `implements` as trait implementations.
- [x] Parse and emit `T?` as `Option<T>`.
- [x] Add `?.` optional chaining.
- [x] Add `??` fallback expressions.
- [x] Add null checks and tests.

## Phase 6 — Explicit shared classes

- [x] Parse `shared class`.
- [x] Emit `Rc<RefCell<...>>` only for explicitly shared classes.
- [x] Insert `.borrow()`, `.borrow_mut()`, and `.clone()` where required.
- [x] Rewrite runtime borrow panic messages into ORust-friendly diagnostics.
- [x] Add shared ownership end-to-end tests.

## Phase 7 — Lifetimes and Rust passthrough

- [x] Infer shared lifetime `'a` for compatible `lend` inputs and returns.
- [x] Detect ambiguous lifetime cases.
- [x] Add the `lend(a) T f(...)` escape hatch.
- [x] Parse and emit `rust { ... }` passthrough blocks.
- [x] Add lifetime and passthrough tests.

## Every-phase checklist

- [ ] Keep AST nodes span-aware. *(Expression nodes still need individual spans.)*
- [x] Keep Rust ownership semantics visible and real.
- [x] Do not reimplement the borrow checker.
- [x] Update `README.md` and relevant docs.
- [x] Add or update golden and end-to-end tests.
- [x] Run `cargo fmt`.
- [x] Run `cargo clippy`.
- [x] Run `cargo test`.

## VS Code extension

- [x] Initialize `vscode-orust/` as a standalone extension project.
- [x] Register the `.or` file extension and ORust language id.
- [x] Add basic language configuration for comments, brackets, and quotes.
- [x] Add TextMate syntax highlighting for the current ORust vocabulary.
- [x] Add a TypeScript activation scaffold and status command.
- [ ] Install npm dependencies and compile the extension.
- [ ] Add diagnostics integration with `orust check`.
- [ ] Add hover, completion, and go-to-definition support.
- [ ] Package and publish the extension.
