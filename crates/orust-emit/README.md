# `orust-emit`

Rust source emitter for ORust programs.

The emitter converts the ORust syntax model into ordinary Rust source,
including structs, impl blocks, traits, async functions, modules, imports,
runtime calls, comments, and source spans.

Generated output is intended to be compiled by Cargo and `rustc`; the emitter
does not replace Rust's type checker or borrow checker.

See [Rust/ORust interoperability](../../docs/rust-orust-interop-tags.md) and
the [main README](../../README.md).
