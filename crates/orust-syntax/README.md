# `orust-syntax`

Parser and syntax model for the ORust language.

This library tokenizes and parses `.or` source, retains source spans and
comments, and exposes the `Program` model used by the CLI, language server,
diagnostics, lowering, and Rust emitter crates.

It is primarily an ORust toolchain dependency. Application users normally use
the `orust` CLI instead of calling this crate directly.

See [comments](../../docs/comments.md), [diagnostics](../../docs/diagnostics.md),
and the [main README](../../README.md).
