# ORust naming bridge

ORust preserves source-level names while generating Rust declarations. File and
directory names become snake_case module names; Rust keywords receive the
`r#` raw-identifier prefix.

An exported declaration can opt into an exact Rust-facing name:

```orust
export @rustName("ExternalUser") class User {}
```

The override is applied to the generated declaration and its generated span
marker. The emitter also rewrites matching constructors, top-level function
calls, referenced types, imports, and re-exports through the same symbol map;
diagnostics continue to report the original `.or` spelling.

`private` members remain file/class-private, `internal` members become
`pub(crate)`, and ordinary exported members become `pub` only when their class
is exported.
