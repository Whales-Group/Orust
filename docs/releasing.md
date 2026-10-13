# ORust releases

This is the short version. There are two normal actions:

1. release a new version;
2. install or inspect an existing version.

## Release a new version

Use a new version every time. Never move or reuse a tag that already exists.

Replace `0.1.12` below with the next version you want to release.

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

## If a release fails

Do not push the same tag again and do not force-move the tag.

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
