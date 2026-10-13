# ORust releases

This is the short version. There are three normal flows:

1. release a new version;
2. push a fix to `main` without releasing;
3. publish a patch release after a previous release.

## Flow 1: release a new version

Use this when users should receive a new published ORust version.

Replace `0.1.12` below with the next version you want to release. The tag will
be `v0.1.12`.

### 1. Make and test your changes

```sh
git status
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test --workspace --doc
bash scripts/check-book.sh
cargo test --manifest-path conformance/Cargo.toml
cargo test --manifest-path fuzz/Cargo.toml
```

### 2. Update the version

```sh
scripts/release.sh 0.1.12
```

Review what changed:

```sh
git diff
cargo pkgid -p orust-cli
cargo pkgid -p orust-runtime
```

Both packages must report `0.1.12`.

### 3. Commit and push `main`

```sh
git add Cargo.toml Cargo.lock crates README.md docs scripts .github/workflows/ci.yml
git commit -m "Prepare release v0.1.12"
git push origin main
```

Do not use `git add .` if you have personal files such as `.idea/` or local
experiments in the repository.

### 4. Publish the runtime

The runtime must be published before users generate projects with this CLI.

```sh
cargo publish -p orust-runtime --dry-run
cargo publish -p orust-runtime
```

If Cargo says that the version already exists, do not publish it again. Use a
new patch version instead.

### 5. Create and push the tag

```sh
git tag -a v0.1.12 -m "ORust v0.1.12"
git push origin v0.1.12
```

Pushing the tag starts GitHub Actions. Do not manually run the workflow for a
release; a manual run usually uses `main` instead of the release tag.

### 6. Wait for GitHub Actions

The workflow must pass:

- `verify`
- all release-binary jobs
- checksums
- GitHub Release publication

Open the repository's **Actions** tab. When it finishes, open
**Releases → ORust v0.1.12 → Assets**.

The release should contain archives for macOS ARM, macOS Intel, Linux, and
Windows, plus `SHA256SUMS`.

## Flow 2: push a fix without releasing

Use this when the fix is still development work and users do not need a new
download yet.

Do not bump the version, publish the runtime, or create a tag:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
git add -u
git commit -m "Fix <short description>"
git push origin main
```

This updates `main` only. Existing tags and released binaries do not change.
Users installing `v0.1.12` will still receive the old `v0.1.12` code.

## Flow 3: publish a patch after a release

If users need the fix, you must create a new patch version. You cannot send a
new patch through an existing tag such as `v0.1.12`, because Git tags and
published crates are immutable release identifiers.

For a fix after `v0.1.12`, use `v0.1.13`:

```sh
# Fix the code first, then run the tests from Flow 1.
scripts/release.sh 0.1.13
git add -u
git commit -m "Prepare release v0.1.13"
git push origin main

cargo publish -p orust-runtime --dry-run
cargo publish -p orust-runtime

git tag -a v0.1.13 -m "ORust v0.1.13"
git push origin v0.1.13
```

The new tag starts a new GitHub Actions release workflow.

## If a release fails

Do not push the same tag again and do not force-move the tag. Use Flow 2 to
push the fix to `main`, then Flow 3 to publish the next patch version.

1. Read the failed job's last red step.
2. Fix the problem on `main`.
3. Bump to a new patch version.
4. Repeat the release steps with the new version.

Example: if `v0.1.12` fails, fix `main` and release `v0.1.13`.

Already-built artifacts from a failed workflow are not a completed release.
Only use archives attached to a successful GitHub Release.

## Use an existing release or tag

There are three different operations. Choose the one you need.

Remember the spelling:

- Git tag: `v0.1.11`
- Installer version: `0.1.11` (without `v`)

The installer changes the CLI on your machine. Running `orust new` or
`orust check` then creates or regenerates a project with that CLI's matching
runtime.

### A. Install an old ORust version

This changes the `orust` and `orust-lsp` tools on your computer. It does not
change any project.

On macOS or Linux:

```sh
ORUST_VERSION=0.1.11 curl --proto '=https' --tlsv1.2 -sSf \
  https://raw.githubusercontent.com/Whales-Group/Orust/main/scripts/install-binary.sh | bash
```

On Windows PowerShell:

```powershell
$env:ORUST_VERSION = '0.1.11'
irm https://raw.githubusercontent.com/Whales-Group/Orust/main/scripts/install.ps1 | iex
```

Check the installed tool:

```sh
orust --version
```

It should print `orust 0.1.11`.

### B. Create a new project with an old version

Install the old ORust version first using section A. Then create the project:

```sh
orust new my-old-project
cd my-old-project
orust check
```

Because the old CLI creates the project, its `Cargo.toml` points to the
matching old `orust-runtime` version.

### C. Change an existing project to an old runtime

First install the old CLI using section A. Then, from the existing project,
run the old CLI again:

```sh
cd my-existing-project
orust check
```

The old CLI regenerates `target/rust/Cargo.toml` with the matching old
`orust-runtime` version and Cargo refreshes the generated lockfile.

Do not edit `target/rust/Cargo.toml` as a permanent fix; the CLI regenerates
that file. The installed CLI version is what selects the generated runtime
version.

`orust update` only runs Cargo's normal dependency update. It does not change
the ORust version. Use the old installer first when you need an old ORust
version.

### Install the latest release on macOS or Linux

```sh
curl --proto '=https' --tlsv1.2 -sSf \
  https://raw.githubusercontent.com/Whales-Group/Orust/main/scripts/install-binary.sh | bash
```

Then open a new shell and check it:

```sh
orust --version
orust-lsp --version
orust doctor
```

### If you downloaded an archive manually

The archive is for testing or manual installation. It is not opened by
double-clicking. From Terminal, run:

```sh
cd ~/Downloads/orust-0.1.12-macos-aarch64
chmod +x ./orust ./orust-lsp
ORUST_BIN="$PWD/orust"
"${ORUST_BIN}" --version
"${ORUST_BIN}" new "$HOME/hello-orust"
(cd "$HOME/hello-orust" && "${ORUST_BIN}" check)
```

Replace the folder name with the archive you downloaded. Use
`macos-aarch64` for Apple Silicon and `macos-x86_64` for Intel Macs.

For a normal installation, use the one-command installer above. It installs
both binaries and configures the PATH automatically.

### Install on Windows

Latest version:

```powershell
irm https://raw.githubusercontent.com/Whales-Group/Orust/main/scripts/install.ps1 | iex
```

Specific version:

```powershell
$env:ORUST_VERSION = '0.1.11'
irm https://raw.githubusercontent.com/Whales-Group/Orust/main/scripts/install.ps1 | iex
```

### Inspect or build the source at an old Git tag

```sh
git fetch --tags origin
git switch --detach v0.1.11
cargo test --workspace
```

This changes the source checkout, not the installed `orust` command. To use
the old source to build and install the tools, run:

```sh
bash scripts/install.sh --non-interactive
```

Return to normal development afterward:

```sh
git switch main
git pull origin main
```

## Version rules

- Rust crates in the workspace share one version.
- `orust-cli` and `orust-runtime` must have the same version.
- Every release tag is `vX.Y.Z`.
- The tag version must match the workspace version without the `v`.
- A published crates.io version cannot be replaced.
- An existing Git tag should not be moved.
- The VS Code extension is released separately from the Rust crates.

## Quick checks after installation

```sh
orust --version
orust doctor
orust new my-project
cd my-project
orust check
```
