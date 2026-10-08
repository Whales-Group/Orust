# Async semantics

ORust Futures are lazy, like Rust futures. Calling an `async` function creates a
Future value; the function body does not run until that Future is awaited or
passed to `spawn`.

This keeps execution and ownership visible. `await work()` runs `work()` on the
current task. `spawn(work())` moves the Future into a Tokio task and therefore
requires the Future and captured values to satisfy Tokio's `Send + 'static`
constraints. ORust does not weaken those constraints or emulate a garbage
collector.

The CLI warns when it can prove that a Future returned by a direct async call is
created and then dropped without being awaited or spawned. The warning is a
lint, not a replacement for Rust's type checker.

Explicitly shared classes use `Rc<RefCell<...>>` when the program has no spawned
work. When `spawn` is present, shared classes use
`Arc<tokio::sync::Mutex<...>>`; async shared-class methods use
`.lock().await`, while synchronous access uses Tokio's `blocking_lock()`.

## Surface syntax

An asynchronous function is declared with `async`. Calling it creates a lazy
future; execution begins only when the future is awaited or spawned:

```orust
async Future<int> load() { return 42; }

async void main() {
  var answer = await load();
  print(answer);
}
```

An asynchronous `main` receives `#[tokio::main]` in generated Rust. `spawn`
creates a Tokio task and returns `Task<T>`; awaiting a task surfaces its join
failure as an ORust `Error`.

`Future.delayed`, `100.millis`, and `1.seconds` are runtime helpers. `Future.wait`
uses runtime futures utilities for collections, while `Future.any` selects the
first completed future. `await for` lowers to repeated `StreamExt::next()`
awaits.

## Errors and ownership

`throws` functions emit `Result<T, orust_runtime::Error>`. Awaiting a throwing
future inside a `throws` function gets a generated `?`; `try/catch` lowers to a
`match` over the result. Rust remains responsible for all type, borrow,
`Send`, and `'static` checks.

The compiler warns when it can prove a direct future-producing call is dropped
without `await` or `spawn`. This warning does not make futures eager and does
not replace compiler errors.
