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

### Install a published release (recommended)

Published releases install prebuilt `orust` and `orust-lsp` binaries, so users
do not need Git or a Rust toolchain just to install ORust:

```sh
curl --proto '=https' --tlsv1.2 -sSf \
  https://raw.githubusercontent.com/Whales-Group/Orust/main/scripts/install-binary.sh \
  | bash
```

On Windows PowerShell:

```powershell
irm https://raw.githubusercontent.com/Whales-Group/Orust/main/scripts/install.ps1 | iex
```

The installer detects macOS or Linux, verifies the archive with the release
`SHA256SUMS` file, installs versioned files under `~/.orust`, and keeps the
active binaries in `~/.orust/bin`. Rust and Cargo are still required when an
ORust project is built.

Check the project toolchain with `orust doctor`.

`orust doctor` reports the ORust version, required Rust tools, and optional
`orust-lsp`/Rust Analyzer editor support. Missing editor tools are warnings;
Cargo and rustc are required for project builds.

For ORust development from a checkout, the original source-build installer is
still available as `scripts/install.sh`.

### Development install from source

The following installer is for contributors. It builds the workspace locally
and requires Rust, Cargo, and Git.

Install Rust first if it is not already installed:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

The installer asks which user-facing components to install:

- `orust` — the compiler and project CLI
- `orust-runtime` — published on crates.io and added automatically to generated projects
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

### Source install — interactive

This asks which components to install:

```sh
curl --proto '=https' --tlsv1.2 -sSf \
  https://raw.githubusercontent.com/Whales-Group/Orust/main/scripts/install.sh \
  | bash
```

### Install all — non-interactive

This installs the CLI, runtime, LSP, and Rust Analyzer when available:

```sh
curl --proto '=https' --tlsv1.2 -sSf \
  https://raw.githubusercontent.com/Whales-Group/Orust/main/scripts/install.sh \
  | bash -s -- --all --non-interactive
```

### Update — interactive

This checks the public repository and asks which installed components to
update:

```sh
curl --proto '=https' --tlsv1.2 -sSf \
  https://raw.githubusercontent.com/Whales-Group/Orust/main/scripts/install.sh \
  | bash -s -- --update
```

### Update — non-interactive

This updates the CLI, runtime, LSP, and rustplain helpers without prompts:

```sh
curl --proto '=https' --tlsv1.2 -sSf \
  https://raw.githubusercontent.com/Whales-Group/Orust/main/scripts/install.sh \
  | bash -s -- --update --non-interactive
```

If a dependency download is temporarily unavailable, the installer retries the
public workspace build three times and leaves existing installed binaries
untouched if the build cannot finish. Run the update command again later.
Retry behavior can be adjusted with `ORUST_RETRIES` and `ORUST_RETRY_DELAY`.

### Uninstall — interactive

This asks which installed components to remove:

```sh
curl --proto '=https' --tlsv1.2 -sSf \
  https://raw.githubusercontent.com/Whales-Group/Orust/main/scripts/install.sh \
  | bash -s -- --uninstall
```

### Uninstall all — non-interactive

This removes all installed ORust binaries without prompts:

```sh
curl --proto '=https' --tlsv1.2 -sSf \
  https://raw.githubusercontent.com/Whales-Group/Orust/main/scripts/install.sh \
  | bash -s -- --uninstall --all --non-interactive
```

### Install one component

```sh
# CLI and runtime
curl --proto '=https' --tlsv1.2 -sSf \
  https://raw.githubusercontent.com/Whales-Group/Orust/main/scripts/install.sh \
  | bash -s -- --cli

# LSP and runtime
curl --proto '=https' --tlsv1.2 -sSf \
  https://raw.githubusercontent.com/Whales-Group/Orust/main/scripts/install.sh \
  | bash -s -- --lsp

# Runtime only
curl --proto '=https' --tlsv1.2 -sSf \
  https://raw.githubusercontent.com/Whales-Group/Orust/main/scripts/install.sh \
  | bash -s -- --runtime
```

The runtime is a Cargo library shared by projects, so uninstalling the global
tools does not delete it from existing projects. Remove `orust-runtime` from a
project's `Cargo.toml` only when that project no longer uses generated ORust
code.

### Prepare a release

See the complete [release and maintenance guide](docs/releasing.md) for
versioning, CI, user upgrades, test commands, rollback, and the acceptance
checklist.

All workspace crates use the shared version in the root `Cargo.toml`. Use the
release script to bump that version and update internal crate links:

```sh
scripts/release.sh patch
```

To check the workspace, commit the version change, create the deployment tag,
and push both the commit and tag:

```sh
scripts/release.sh patch --commit --tag --push
```

The `vX.Y.Z` tag starts the GitHub release workflow, which builds the platform
archives. Use `minor`, `major`, or an explicit version such as `1.0.0` instead
of `patch` when appropriate.

For a normal interactive update, omit `--non-interactive` and choose the
components to update. Project dependencies are updated separately with:

```sh
orust fetch
orust update
```

When required crates are already in Cargo's local cache, use
`orust fetch --offline` or `orust update --offline` to avoid network access.

The source installer is intended for ORust contributors. End users should use
the prebuilt release installer at the top of this section.

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
`orust-runtime = "=0.1.9"`; Cargo fetches the published runtime automatically when
the project is first built. The runtime is a project dependency, not a global
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
orust fetch [--offline]          # download declared dependencies
orust update [--offline]         # update the Cargo lockfile
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
generated Rust. Use `orust add crate@version`, then `orust fetch` (or
`orust fetch --offline` when the Cargo cache is already populated).

## License

ORust is distributed under the [MIT License](LICENSE).
