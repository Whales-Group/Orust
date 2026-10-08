# ORust

ORust is an object-oriented language that compiles to ordinary Rust. It adds
classes, constructors, interfaces, named parameters, and a simpler application
syntax while keeping Rust ownership, borrowing, lifetimes, traits, and Cargo
as the authority.

Write `.or` files, run them with the ORust CLI, and inspect the generated Rust
when you need complete control.

## Features

- Classes, fields, constructors, methods, and interfaces
- Rust-compatible ownership with `lend`, `lend mut`, and `copy`
- Async functions, `await`, tasks, streams, channels, and Tokio integration
- Rust blocks, Rust imports, Rust types, and external Cargo crates
- Modules shared between `.or` and `.rs` files
- Generics, enums, records, patterns, iterators, options, and results
- Source-mapped diagnostics, formatting, linting, and generated Rust output
- VS Code language support through the ORust language server

## Installation

### One-command installer

Install Rust first if it is not already installed:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Then install the ORust CLI, runtime, and optional language server from the
public repository:

```sh
curl --proto '=https' --tlsv1.2 -sSf \
  https://raw.githubusercontent.com/Whales-Group/Orust/main/scripts/install.sh | bash
```

The installer asks which user-facing components to install:

- `orust` — the compiler and project CLI
- `orust-runtime` — prepared in Cargo's registry cache for generated projects
- `orust-lsp` — optional editor support

The parser, emitter, lowering, diagnostics, runtime, and rustplain support
crates are built under the hood. Users do not need to install those internal
crates separately. The installer detects your platform, builds the complete
workspace from the public repository, installs `orust`, `orust-lsp`, and the
`rustplain` diagnostic helpers, prepares the published runtime crate, and
checks for Rust Analyzer. It supports macOS and Linux directly, and Windows
through Git Bash or WSL.

Installation is isolated from any local Orust development checkout. The
installer clones the selected public repository into a temporary directory and
runs Cargo there, so local path dependencies and local workspace builds are
not used.

Verify the installation:

```sh
orust --version
orust-lsp --version
```

The installer supports unattended installation:

```sh
curl --proto '=https' --tlsv1.2 -sSf \
  https://raw.githubusercontent.com/Whales-Group/Orust/main/scripts/install.sh \
  | bash -s -- --all --non-interactive
```

To update the selected ORust tools later, run the same public script with
`--update`. It checks the repository, reports the workspace components being
rebuilt, updates the CLI/LSP and rustplain binaries, and refreshes the runtime
dependency:

```sh
curl --proto '=https' --tlsv1.2 -sSf \
  https://raw.githubusercontent.com/Whales-Group/Orust/main/scripts/install.sh \
  | bash -s -- --update --non-interactive
```

The same installer can remove the installed tools. Run it without
`--non-interactive` to choose which components to remove:

```sh
curl --proto '=https' --tlsv1.2 -sSf \
  https://raw.githubusercontent.com/Whales-Group/Orust/main/scripts/install.sh \
  | bash -s -- --uninstall
```

For unattended removal of all installed ORust binaries:

```sh
curl --proto '=https' --tlsv1.2 -sSf \
  https://raw.githubusercontent.com/Whales-Group/Orust/main/scripts/install.sh \
  | bash -s -- --uninstall --all --non-interactive
```

The runtime is a Cargo library shared by projects, so uninstalling the global
tools does not delete it from existing projects. Remove `orust-runtime` from a
project's `Cargo.toml` only when that project no longer uses generated ORust
code.

For a normal interactive update, omit `--non-interactive` and choose the
components to update. Project dependencies are updated separately with:

```sh
orust fetch
orust update
```

When `orust-cli` and `orust-lsp` are available on crates.io, they can also be
installed with Cargo:

```sh
cargo install orust-cli
cargo install orust-lsp
```

## Quick start

Create a project:

```sh
orust new hello-orust
cd hello-orust
orust run
```

Create a file named `src/main.or`:

```orust
class Counter {
  int value = 0;

  void increment() {
    value++;
  }
}

void main() {
  var counter = new Counter();
  counter.increment();
  print(counter.value);
}
```

Run it with:

```sh
orust run
```

ORust generates Rust under `target/rust/`. Cargo compiles that Rust and runs
the resulting binary. New projects contain a standard `Cargo.toml` with
`orust-runtime = "0.1"`; Cargo fetches the runtime automatically when the
project is first built. The runtime is a project dependency, not a global
command.

## Language examples

### Ownership and borrowing

Use `lend` when a function should borrow a value and `copy` when an owned clone
is intended:

```orust
String shout(lend String value) {
  return value.toUpperCase();
}

void main() {
  var message = "hello";
  print(shout(lend message));
  print(message);
}
```

### Async code

```orust
async Future<String> load_message() {
  await Future.delayed(100.millis);
  return "ready";
}

async void main() {
  var message = await load_message();
  print(message);
}
```

ORust preserves Tokio and Rust's `Send` and `'static` requirements. See
[async semantics](docs/async.md).

### Records and collections

```orust
type User(String name, int age);

void main() {
  var users = [
    new User("Ada", 36),
    new User("Grace", 37),
  ];

  for (var user in users) {
    print(user.name);
  }
}
```

More examples are in [`examples/`](examples).

## Rust interoperability

ORust and Rust can be used in the same project. Declare dependencies in the
project's standard `Cargo.toml`; Cargo resolves them normally.

```toml
[dependencies]
serde_json = "1"
```

Use a Rust import and a Rust block when an API needs exact Rust syntax:

```orust
rust use serde_json::json;

void main() {
  rust {
    let value = json!({ "language": "ORust" });
    println!("{}", value["language"]);
  }
}
```

Handwritten `.rs` files can live beside `.or` files. ORust preserves the Rust
module structure and Cargo remains responsible for compiling both languages.
See [modules](docs/modules.md) and [Rust/ORust interoperability](docs/rust-orust-interop-tags.md).

## CLI commands

```sh
orust new my-project             # create an application
orust new my-library --lib       # create a library
orust run                        # generate, compile, and run
orust build                      # generate and compile
orust check                      # parse and validate
orust lint                       # report source lints
orust format                     # format an ORust file
orust format --dry-run           # preview formatting changes
orust emit                       # print generated Rust
orust add serde_json@1            # add a Cargo dependency
orust fetch                       # download declared dependencies
orust update                      # update the Cargo lockfile
orust explain OR0005             # explain a diagnostic code
```

The CLI accepts a file, project directory, or `Cargo.toml` as the input
context. From a project root, `orust run` uses `src/main.or` (or `src/main.rs`)
and the generated Cargo project. Existing `orust.toml` projects remain
readable for migration, but new projects use Cargo's standard manifest.

## VS Code

Install the ORust VS Code extension from its separate extension repository or
release package. Install `orust-lsp` with the installer above and ensure
Cargo's binary directory is on `PATH`.

The extension provides syntax highlighting, diagnostics, hover information,
completion, formatting, source navigation, run commands, and debug commands.
Rust sections continue to use the normal Rust Analyzer extension.

## Documentation

- [Runtime architecture](docs/runtime.md)
- [Async and Tokio semantics](docs/async.md)
- [Modules and Cargo dependencies](docs/modules.md)
- [Rust/ORust interoperability](docs/rust-orust-interop-tags.md)
- [Ownership and cleanup](docs/drop.md)
- [Diagnostics and error codes](docs/diagnostics.md)
- [Traits and polymorphism](docs/traits.md)
- [Records](docs/records.md), [strings](docs/strings.md), and [numbers](docs/numbers.md)
- [Patterns](docs/patterns.md), [iterators](docs/iterators.md), and [slices](docs/slices.md)

## Troubleshooting

### `orust: command not found`

Add Cargo's binary directory to your shell `PATH`:

```sh
export PATH="$HOME/.cargo/bin:$PATH"
```

Restart the terminal after changing your shell profile.

### Rust Analyzer is unavailable

Install the component with:

```sh
rustup component add rust-analyzer
```

The ORust extension can still parse and run `.or` files without Rust Analyzer,
but embedded Rust completion and diagnostics will be limited.

### Cargo dependency errors

Run the command from the project directory and check `Cargo.toml`. Third-party
crates must be declared there before they can be used by Rust blocks or
generated Rust. Use `orust add crate@version`, then `orust fetch`.

## License

ORust is distributed under the [MIT License](LICENSE).
