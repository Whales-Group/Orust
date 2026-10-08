# `rustplain-cli`

The `rustplain` command explains Rust compiler and Cargo errors in plain
language.

## Usage

```sh
rustplain check
rustplain build
rustplain test
rustplain clippy --workspace --all-targets
```

Use `--raw` to preserve the original Cargo JSON stream. The command keeps
compiler locations and adds explanations without changing Cargo's result.
