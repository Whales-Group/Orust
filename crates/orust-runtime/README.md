# `orust-runtime`

`orust-runtime` is the Rust runtime library used by programs generated from
[ORust](https://github.com/Whales-Group/Orust). ORust is an object-oriented front
end for Rust: the compiler emits ordinary Rust, while this crate supplies
shared helpers for async execution, tasks, channels, diagnostics, collections,
I/O, environment access, and math.

The runtime does not replace Rust's ownership, borrowing, type, privacy,
`Send`, or `'static` checks. It is a thin, versioned bridge over Tokio,
`futures`, and `async-trait`.

## Installation

Most ORust users install the `orust` CLI and do not install this crate
manually. Running `orust new project-name` creates a Cargo project whose
manifest already includes `orust-runtime`; `orust run` builds it through
Cargo.

For a handwritten Rust project that uses runtime APIs directly, add:

```toml
[dependencies]
orust-runtime = "0.1"
```

Example:

```rust
use orust_runtime::{seconds, sleep};

#[tokio::main]
async fn main() {
    sleep(seconds(1)).await;
}
```

The crate currently enables Tokio's `macros`, `rt-multi-thread`, `time`, and
`sync` features, and depends on `futures` and `async-trait`.

## Generated Rust mapping

| ORust feature | Runtime/Rust representation |
| --- | --- |
| `Future<T>` | Rust future values |
| `spawn(task())` | `orust_runtime::spawn(task())` |
| `Channel<T>` | `orust_runtime::Channel<T>` |
| `Stream<T>` | `orust_runtime::Stream<T>` |
| `await` | Rust `.await` |
| `throws` | `Result<T, orust_runtime::Error>` |
| list indexing | `checked_index` before indexing |
| optional display | `display_option` |
| `1.seconds` / `100.millis` | `seconds(1)` / `millis(100)` |
| standard helpers | `std_math`, `std_env`, and `std_io` |

See the [runtime architecture guide](https://github.com/Whales-Group/Orust/blob/main/docs/runtime.md),
[async guide](https://github.com/Whales-Group/Orust/blob/main/docs/async.md),
and [modules guide](https://github.com/Whales-Group/Orust/blob/main/docs/modules.md).

## Tasks and async execution

`spawn` starts a Tokio task and returns `Task<T>`. Awaiting the task exposes a
join failure as `Error`:

```rust
use orust_runtime::{spawn, Error, Task};

async fn compute() -> i64 { 40 + 2 }

async fn run() -> Result<i64, Error> {
    let task: Task<i64> = spawn(compute());
    task.await_task().await
}
```

Spawning requires the future and captured values to satisfy `Send + 'static`.
The runtime deliberately preserves this Tokio requirement. Calling an async
function creates a lazy future; execution starts only when it is awaited or
spawned. See the [async semantics documentation](https://github.com/Whales-Group/Orust/blob/main/docs/async.md).

Time helpers are available to generated and handwritten Rust:

```rust
use orust_runtime::{millis, seconds};

let short = millis(250);
let long = seconds(2);
```

## Channels

`channel(capacity)` creates a bounded Tokio multi-producer channel. Cloning a
`Channel<T>` clones its sender and shares the receiver behind an async mutex:

```rust
use orust_runtime::{channel, spawn, Error};

async fn example() -> Result<i64, Error> {
    let queue = channel::<i64>(8);
    let producer = queue.clone();
    let task = spawn(async move { producer.send(42).await });

    let value = queue.recv().await.unwrap_or_default();
    task.await_task().await??;
    Ok(value)
}
```

`send` returns `Error` when the receiver is dropped. `recv` returns `None` when
the channel is closed and empty.

## Errors and panic diagnostics

`Error` stores a user-facing message and converts from errors implementing
`std::error::Error + Send + Sync + 'static`:

```rust
use orust_runtime::Error;

fn adapt(error: std::io::Error) -> Error { error.into() }
```

Generated ORust entry points install the panic hook automatically. Rust
applications can install it explicitly:

```rust
fn main() {
    orust_runtime::install_panic_hook();
}
```

Known failures map to stable ORust codes:

- `OR0008` — arithmetic overflow
- `OR0010` — list index out of range
- `OR0011` — conflicting borrow
- `OR0012` — invalid string character boundary
- `OR0013` — division by zero
- `OR0014` — missing optional value was unwrapped
- `OR0099` — unknown runtime panic

Set `ORUST_BACKTRACE=1` to include the Rust panic location. See the
[diagnostics guide](https://github.com/Whales-Group/Orust/blob/main/docs/diagnostics.md)
and [error-code registry](https://github.com/Whales-Group/Orust/blob/main/docs/error-codes.md).

## Safe indexing and options

Generated collection access uses `checked_index`:

```rust
let index = orust_runtime::checked_index(2, values.len());
let value = &values[index];
```

Negative indexes and indexes outside the collection produce an ORust runtime
diagnostic. `display_option` renders `Some(value)` through `Display` and
renders `None` as `null`:

```rust
assert_eq!(orust_runtime::display_option(&Some(7)), "7");
assert_eq!(orust_runtime::display_option::<i32>(&None), "null");
```

## Standard helper modules

### `std_math`

Provides `pi`, `e`, and common `f64` operations: `sqrt`, `pow`, `sin`, `cos`,
`tan`, `log`, `exp`, `min`, `max`, `abs`, `floor`, and `ceil`.

### `std_env`

Provides `args`, `get`, `set`, and `cwd`. These follow Rust's process-wide
environment semantics.

### `std_io`

`std_io::File` provides string-oriented existence, read, write, and append
helpers. `std_io::Path` provides portable join, parent, and file-name helpers.
I/O failures become `Error` values:

```rust
use orust_runtime::std_io::File;

fn read(path: String) -> Result<String, orust_runtime::Error> {
    File::read_string(path)
}
```

Use `std::fs` directly in a Rust block for advanced filesystem operations.

## Rust interoperability

The runtime is ordinary Rust and can be used beside ORust-generated code:

```orust
rust use orust_runtime::{seconds, sleep};

async void main() {
  rust { sleep(seconds(1)).await; }
}
```

External crates such as `serde_json` belong in the project's `Cargo.toml` or
`orust.toml`; Cargo resolves those dependencies. Use explicit Rust imports,
`@rustImport`, `@rustType`, or passthrough blocks at the boundary. See the
[Rust/ORust interoperability guide](https://github.com/Whales-Group/Orust/blob/main/docs/rust-orust-interop-tags.md).

## Versioning and packaging

The runtime version follows the ORust release line. Generated code should use
the same major/minor runtime line as the CLI that emitted it. Patch releases
should preserve generated-code compatibility; breaking runtime API changes
require a coordinated compiler release.

Generated projects use the crate name `orust_runtime` in emitted Rust. Cargo
handles downloading and compiling the matching runtime version automatically.

## Documentation map

- [Runtime architecture](https://github.com/Whales-Group/Orust/blob/main/docs/runtime.md)
- [Async and Tokio semantics](https://github.com/Whales-Group/Orust/blob/main/docs/async.md)
- [Ownership and cleanup](https://github.com/Whales-Group/Orust/blob/main/docs/drop.md)
- [Modules and Cargo dependencies](https://github.com/Whales-Group/Orust/blob/main/docs/modules.md)
- [Rust/ORust interoperability](https://github.com/Whales-Group/Orust/blob/main/docs/rust-orust-interop-tags.md)
- [Diagnostics and error codes](https://github.com/Whales-Group/Orust/blob/main/docs/diagnostics.md)
