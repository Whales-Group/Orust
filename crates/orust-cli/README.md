# `orust-cli`

The `orust` command-line tool compiles and runs ORust programs.

## Usage

```sh
orust new hello-orust
cd hello-orust
orust run
```

Useful commands include `check`, `build`, `run`, `emit`, `format`, `lint`,
`new`, and `explain`. A project may use `orust.toml` or a standard
`Cargo.toml`; generated Rust is written under `target/rust/`.

The CLI uses the ORust parser, emitter, diagnostics, and runtime crates. See
the [main ORust README](../../README.md) and [CLI documentation](../../docs/modules.md).
