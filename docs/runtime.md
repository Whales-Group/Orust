# Runtime architecture

`orust-runtime` is the compatibility layer between generated Rust and the
ORust language surface. It is intentionally small: generated code remains
recognizable Rust, while repeated lowering patterns and friendly runtime
diagnostics live in one versioned crate.

## Responsibility boundaries

| Responsibility | Owner |
| --- | --- |
| Parse ORust | `orust-syntax` |
| Emit Rust | `orust-emit` |
| Compile generated Rust | Cargo and `rustc` |
| Async scheduling | Tokio through `orust-runtime` |
| Tasks and channels | `orust-runtime` |
| Friendly panic codes | `orust-runtime` |
| Ownership and lifetimes | Rust |

The runtime never hides ownership or changes Rust's type system. This boundary
is what allows an ORust program to call Rust crates and a Rust crate to consume
generated ORust libraries. See [modules](modules.md) and
[Rust/ORust interoperability](rust-orust-interop-tags.md).

## Generated projects

Users install the `orust` executable and prepare the runtime through the public
installer:

```sh
curl --proto '=https' --tlsv1.2 -sSf \
  https://raw.githubusercontent.com/Whales-Group/Orust/main/scripts/install.sh | bash
orust new my-project
cd my-project
orust run
```

The generated `Cargo.toml` includes the published `orust-runtime` crate, so
Cargo downloads the runtime during the first build. A direct Rust user can use
the same crates.io dependency:

```toml
[dependencies]
orust-runtime = "0.1.2"
```

## Public API areas

- `Task<T>` and `spawn` bridge ORust task syntax to Tokio tasks.
- `Channel<T>` and `channel` provide bounded async message passing.
- `Stream<T>` and `StreamExt` expose futures streams.
- `Error` is the generated fallible-operation error type.
- `sleep`, `seconds`, and `millis` provide time helpers.
- `checked_index` and `display_option` centralize safe generated operations.
- `std_math`, `std_env`, and `std_io` implement standard helpers.
- `tokio`, `futures`, and `async_trait` are re-exported for Rust adapters.

The [crate README](../crates/orust-runtime/README.md) contains runnable Rust
examples and the [async guide](async.md) explains language-level behavior.

## Error behavior

Generated entry points install `install_panic_hook`. The hook converts known
panic messages to stable ORust codes and optionally prints a Rust source
location when `ORUST_BACKTRACE=1` is set. It is diagnostic presentation only;
it does not catch panics or change Rust's unwind behavior.

The stable registry is maintained in [error-codes.md](error-codes.md). New
runtime errors should add a code, a user-facing explanation, and an example
before being exposed by the compiler.
