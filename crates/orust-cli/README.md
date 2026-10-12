# `orust-cli`

The `orust` command-line tool compiles and runs ORust programs.

## Usage

```sh
orust new hello-orust
cd hello-orust
orust run [--offline]
```

Useful commands include `check`, `build`, `run`, `emit`, `format`, `lint`,
`new`, `add`, `fetch`, `update`, and `explain`. New projects use a standard
`Cargo.toml` with the published `orust-runtime` dependency; generated Rust is
written under `target/rust/`. Existing `orust.toml` projects remain readable.

From a project root:

```sh
orust add serde_json@1
orust fetch [--offline]
orust run
orust update [--offline]
```

The CLI uses the ORust parser, emitter, diagnostics, and runtime crates. See
the [main ORust README](../../README.md) and [CLI documentation](../../docs/modules.md).
