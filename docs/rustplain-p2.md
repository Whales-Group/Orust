# rustplain P2: Cargo wrapper

The `rustplain` binary can now wrap Cargo's JSON diagnostic stream for the
standard Cargo workflows:

```text
rustplain check [cargo arguments]
rustplain build [cargo arguments]
rustplain test [cargo arguments]
rustplain run [cargo arguments]
rustplain clippy [cargo arguments]
```

It invokes Cargo with `--message-format=json`, renders compiler diagnostics in
plain language, passes artifact/build-finished/unknown JSON events through, and
returns Cargo's exit code. Cargo's stderr is kept unchanged so warnings and
tool output that are not part of the JSON stream remain visible.

Examples:

```bash
rustplain check
rustplain clippy --workspace --all-targets
rustplain test --package my_app -- --nocapture
rustplain run --package my_app -- --help
```

For a local checkout before installing the binary, use:

```bash
cargo run --offline -p rustplain-cli --bin rustplain -- check --workspace
```

The Cargo subcommand form is provided by the `cargo-plain` binary. After
installing it, Cargo discovers it as `cargo plain`:

```bash
cargo install --path crates/rustplain-cli --bin cargo-plain
cargo plain check
cargo plain clippy --workspace --all-targets
```

`--raw` is available on wrapper commands when the original Cargo JSON stream
is needed:

```bash
rustplain --raw check --workspace
```

The current wrapper is intentionally streaming and deterministic. The richer
two-phase handling for `run` and `test`—separating build diagnostics from the
program's own stdout/stderr—is a later P2 refinement.
