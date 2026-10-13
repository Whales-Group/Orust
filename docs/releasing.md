# ORust release and maintenance guide

This is the operational guide for releasing ORust, testing changes, and
upgrading an installation. The intended release flow is:

```text
edit -> format/check/test -> bump version -> tag -> GitHub Actions
     -> platform archives + checksums -> user installer -> smoke test
```

## What is versioned

The Rust workspace has one shared version in the root `Cargo.toml`. It is used
by the CLI, language server, runtime, parser, emitter, diagnostics, and the
other Rust crates. The CLI and runtime must have the same release version:

```sh
cargo pkgid -p orust-cli
cargo pkgid -p orust-runtime
```

The VS Code extension is a separate package in `vscode-orust/package.json`.
Changing the Rust compiler version does not automatically publish a new VS Code
extension, and changing the extension version does not change the compiler.

## Before releasing

Start from a clean branch and inspect the current version and remotes:

```sh
git status --short
git remote -v
cargo pkgid -p orust-cli
```

Run the complete local verification suite. Use `--offline` only when all
dependencies are already cached; CI runs the same checks with network access.

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test --workspace --doc
bash scripts/check-book.sh
cargo test --manifest-path conformance/Cargo.toml
cargo test --manifest-path fuzz/Cargo.toml
```

For a faster inner loop, use the affected package first:

```sh
cargo test -p orust-syntax
cargo test -p orust-emit
cargo test -p orust-diag
cargo test -p orust-cli --bin orust
cargo test -p orust-cli --test modules
cargo test -p orust-cli --test phase6_docs
cargo test -p orust-cli --test runtime
cargo test -p orust-lsp
```

If the VS Code extension changed, run its checks from its own directory:

```sh
cd vscode-orust
npm install
npm run check
npm run compile
cd ..
```

## Bump and publish a Rust release

The release helper updates the shared workspace version and internal crate
links. It runs formatting and `cargo check` unless told not to:

```sh
# Preview and review the changes first.
scripts/release.sh patch
git diff

# Or use minor, major, or an exact version such as 1.0.0.
scripts/release.sh minor
```

After reviewing the version diff and rerunning the release checks, commit and
tag the release. The tag must be exactly `vX.Y.Z`:

```sh
git add Cargo.toml Cargo.lock crates
git commit -m "Release v0.1.9"
git tag -a v0.1.9 -m "ORust v0.1.9"
git push origin HEAD
git push origin v0.1.9
```

The helper can perform those final actions, but use it only after reviewing
the generated diff:

```sh
scripts/release.sh patch --commit --tag --push
```

Do not reuse a tag. If the tag or release is wrong, fix the source and publish
a new patch version rather than moving a tag that users may already have
downloaded.

## What GitHub Actions does

Pushing `vX.Y.Z` starts `.github/workflows/ci.yml`:

1. `verify` runs formatting, Clippy, workspace tests, doctests, book checks,
   and conformance tests.
2. `release-binaries` builds `orust` and `orust-lsp` for macOS Apple Silicon,
   macOS Intel, Linux x86_64, and Windows x86_64.
3. Each archive contains the two binaries, the runtime source, a
   `runtime-version.txt` file, `README.md`, and `LICENSE`.
4. The workflow smoke-tests `orust --version`, `orust new`, and `orust check`.
5. `release-checksums` creates `SHA256SUMS`.
6. `github-release` publishes the archives and checksum file as a GitHub
   Release with generated notes.

Watch the workflow before announcing the release:

```sh
gh run list --workflow ci.yml --limit 5
gh run watch <run-id>
```

If the workflow fails, fix the branch and create a new version. Do not tell
users to install an archive from a failed or incomplete release.

## How users install or upgrade

### macOS and Linux

The binary installer detects the platform, downloads the latest release,
verifies `SHA256SUMS`, and installs both `orust` and `orust-lsp`:

```sh
curl --proto '=https' --tlsv1.2 -sSf \
  https://raw.githubusercontent.com/Whales-Group/Orust/main/scripts/install-binary.sh | bash
```

Restart the shell or reload the PATH, then verify:

```sh
orust --version
orust-lsp --version
orust doctor
```

To install a specific release instead of the latest one:

```sh
ORUST_VERSION=0.1.9 curl --proto '=https' --tlsv1.2 -sSf \
  https://raw.githubusercontent.com/Whales-Group/Orust/main/scripts/install-binary.sh | bash
```

The active binaries are in `~/.orust/bin`; versioned files are stored in
`~/.orust/versions/<version>`. Existing ORust projects keep their runtime
dependency in `Cargo.toml`, so Cargo uses the version declared by that project.
Run `orust update` when intentionally moving a project's dependency lockfile
to a newer compatible runtime.

### Windows PowerShell

```powershell
irm https://raw.githubusercontent.com/Whales-Group/Orust/main/scripts/install.ps1 | iex
orust --version
orust doctor
```

For a pinned release:

```powershell
$env:ORUST_VERSION = '0.1.9'
irm https://raw.githubusercontent.com/Whales-Group/Orust/main/scripts/install.ps1 | iex
```

Open a new PowerShell window after installation so the updated user PATH is
loaded.

### Contributor/source installation

Users should use the binary installer. Contributors who are changing ORust
itself can build from source:

```sh
git clone https://github.com/Whales-Group/Orust.git
cd Orust
bash scripts/install.sh
```

This path requires Rust and Cargo. It is useful for testing an unreleased
change, but it is not the recommended end-user installation path.

## Test matrix by component

| Area | Fast test | Full/extra verification |
| --- | --- | --- |
| Syntax/parser | `cargo test -p orust-syntax` | `cargo bench -p orust-syntax` when parser performance changes |
| Lowering | `cargo test -p orust-lower` | workspace tests for cross-crate behavior |
| Rust emitter | `cargo test -p orust-emit` | golden, recursive, quality, and span tests |
| Diagnostics | `cargo test -p orust-diag` | diagnostic fixtures and JSON compiler-span tests |
| CLI | `cargo test -p orust-cli --bin orust` | modules, docs, runtime, and conformance tests |
| Runtime | `cargo test -p orust-runtime` | CLI runtime acceptance tests |
| LSP | `cargo test -p orust-lsp` | launch `orust-lsp` and exercise an `.or` file in VS Code |
| VS Code | `npm run check && npm run compile` in `vscode-orust` | press `F5` and test diagnostics, hover, formatting, run, and debug |
| Documentation | `bash scripts/check-book.sh` | `cargo test --workspace --doc` |
| Release artifacts | workflow smoke test | download archive and verify checksum on each supported OS |

For a generated project smoke test after building the CLI:

```sh
tmp_dir="$(mktemp -d)"
orust new "$tmp_dir/example"
(cd "$tmp_dir/example" && orust check)
```

If the CLI is installed globally, use `orust new` and `orust check` instead.

## Release acceptance checklist

Before announcing a release, confirm:

- [ ] `Cargo.toml` and `Cargo.lock` contain the intended version.
- [ ] `orust-cli` and `orust-runtime` report the same version.
- [ ] The VS Code extension version was updated separately if needed.
- [ ] Format, Clippy, tests, doctests, book, conformance, and fuzz smoke pass.
- [ ] The tag is `vX.Y.Z` and matches the workspace version exactly.
- [ ] All GitHub Actions release jobs passed.
- [ ] `SHA256SUMS` is attached to the GitHub Release.
- [ ] macOS, Linux, and Windows archives contain both executables.
- [ ] A clean-machine install succeeds on at least one Unix system and
      Windows when available.
- [ ] `orust --version`, `orust doctor`, `orust new`, and `orust check` work.
- [ ] The release notes mention breaking syntax, runtime, or CLI changes.

## Troubleshooting and rollback

`orust doctor` is the first diagnostic command. It checks the ORust version,
Cargo, rustc, and optional tooling. If Cargo is missing, install Rust through
[rustup](https://rustup.rs) and reopen the shell.

If an upgrade is bad on macOS/Linux, select an older archive explicitly:

```sh
ORUST_VERSION=0.1.2 curl --proto '=https' --tlsv1.2 -sSf \
  https://raw.githubusercontent.com/Whales-Group/Orust/main/scripts/install-binary.sh | bash
```

Then verify `orust --version`. On Windows, set `ORUST_VERSION` to the known
good release and rerun `install.ps1`.

For a project that fails after a runtime change, inspect the generated project
under `target/rust`, run `orust emit <file.or>`, and use `orust check --offline`
to distinguish a network problem from a compiler or runtime problem.
