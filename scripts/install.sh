#!/usr/bin/env bash
set -eu

# Public ORust installer. It is safe to run from curl | bash because prompts
# use /dev/tty when the script itself is being read from standard input.
ORUST_REPOSITORY="${ORUST_REPOSITORY:-https://github.com/Whales-Group/Orust.git}"
ORUST_REF="${ORUST_REF:-main}"
CARGO_COMMAND="${CARGO_COMMAND:-cargo}"
RUSTUP_COMMAND="${RUSTUP_COMMAND:-rustup}"
INSTALL_CLI=1
INSTALL_LSP=1
INSTALL_RUNTIME=1
INSTALL_RUST_ANALYZER=0
CREATE_PROJECT=""
NON_INTERACTIVE=0
UPDATE_ONLY=0
UNINSTALL_ONLY=0
WORKSPACE_DIR=""
CARGO_BIN_DIR="${CARGO_HOME:-${HOME}/.cargo}/bin"

say() { printf '%s\n' "[orust] $*"; }
warn() { printf '%s\n' "[orust] warning: $*" >&2; }
fail() { printf '%s\n' "[orust] error: $*" >&2; exit 1; }

usage() {
    cat <<'USAGE'
Usage: install.sh [options]

Installs the ORust user tools from the public GitHub repository.

Options:
  --all                  Install CLI, runtime, LSP, and Rust Analyzer when available
  --cli                  Install only the orust CLI and runtime
  --lsp                  Install only orust-lsp and runtime
  --runtime              Pre-fetch the published orust-runtime crate
  --rust-analyzer        Try to install Rust Analyzer through rustup when missing
  --update               Check the repository and update selected installed tools
  --uninstall            Remove selected ORust tools from this machine
  --project NAME         Create a starter project after installation
  --non-interactive      Do not prompt; install selected/default components
  --help                 Show this help

Environment:
  ORUST_REF              Git branch or tag; default: main
  ORUST_REPOSITORY       Git repository URL

The runtime is a library, not an executable. It is made available through
Cargo's registry cache and is added automatically to new ORust projects.
USAGE
}

while (($# > 0)); do
    case "$1" in
        --all)
            INSTALL_CLI=1
            INSTALL_LSP=1
            INSTALL_RUNTIME=1
            INSTALL_RUST_ANALYZER=1
            ;;
        --cli)
            INSTALL_CLI=1
            INSTALL_LSP=0
            INSTALL_RUNTIME=1
            ;;
        --lsp)
            INSTALL_CLI=0
            INSTALL_LSP=1
            INSTALL_RUNTIME=1
            ;;
        --runtime)
            INSTALL_CLI=0
            INSTALL_LSP=0
            INSTALL_RUNTIME=1
            ;;
        --rust-analyzer)
            INSTALL_RUST_ANALYZER=1
            ;;
        --project)
            (($# >= 2)) || fail "--project requires a directory name"
            CREATE_PROJECT="$2"
            shift
            ;;
        --non-interactive)
            NON_INTERACTIVE=1
            ;;
        --update)
            UPDATE_ONLY=1
            ;;
        --uninstall)
            UNINSTALL_ONLY=1
            ;;
        --help|-h)
            usage
            exit 0
            ;;
        *)
            fail "unknown option: $1 (use --help for usage)"
            ;;
    esac
    shift
done

case "$(uname -s 2>/dev/null || printf unknown)" in
    Darwin) PLATFORM="macOS" ;;
    Linux) PLATFORM="Linux" ;;
    MINGW*|MSYS*|CYGWIN*) PLATFORM="Windows (Bash)" ;;
    *) PLATFORM="$(uname -s 2>/dev/null || printf unknown)" ;;
esac
ARCH="$(uname -m 2>/dev/null || printf unknown)"
say "Detected ${PLATFORM} (${ARCH})"

PROMPT_FD=0
if [[ "${NON_INTERACTIVE}" -eq 0 && -r /dev/tty ]]; then
    exec 3</dev/tty
    PROMPT_FD=3
fi

ask_yes_no() {
    local question="$1"
    local default="${2:-y}"
    local answer=""
    if [[ "${PROMPT_FD}" -eq 0 ]]; then
        if [[ "${default}" == y ]]; then
            return 0
        fi
        return 1
    fi
    if [[ "${default}" == y ]]; then
        read -r -u "${PROMPT_FD}" -p "[orust] ${question} [Y/n] " answer || true
    else
        read -r -u "${PROMPT_FD}" -p "[orust] ${question} [y/N] " answer || true
    fi
    answer="${answer:-${default}}"
    [[ "${answer}" == y || "${answer}" == Y || "${answer}" == yes || "${answer}" == YES ]]
}

if [[ "${PROMPT_FD}" -ne 0 ]]; then
    action="Install"
    if [[ "${UNINSTALL_ONLY}" -eq 1 ]]; then
        say "Choose what to uninstall:"
        action="Remove"
    else
        say "Choose what to install:"
    fi
    ask_yes_no "${action} the orust CLI?" y && INSTALL_CLI=1 || INSTALL_CLI=0
    ask_yes_no "${action} orust-lsp?" y && INSTALL_LSP=1 || INSTALL_LSP=0
    ask_yes_no "${action} the orust-runtime component?" y && INSTALL_RUNTIME=1 || INSTALL_RUNTIME=0
    if [[ -z "${CREATE_PROJECT}" ]] && ask_yes_no "Create a starter ORust project after installation?" n; then
        read -r -u "${PROMPT_FD}" -p "[orust] Project directory [hello-orust] " CREATE_PROJECT || true
        CREATE_PROJECT="${CREATE_PROJECT:-hello-orust}"
    fi
fi

remove_binary() {
    local name="$1"
    local path="${CARGO_BIN_DIR}/${name}"
    if [[ -f "${path}" ]]; then
        rm -f "${path}"
        say "Removed ${path}"
    elif [[ -f "${path}.exe" ]]; then
        rm -f "${path}.exe"
        say "Removed ${path}.exe"
    else
        say "${name} is not installed"
    fi
}

if [[ "${UNINSTALL_ONLY}" -eq 1 ]]; then
    if [[ "${INSTALL_CLI}" -eq 1 ]]; then
        say "Removing the ORust CLI and its rustplain diagnostic helpers"
        remove_binary orust
        remove_binary rustplain
        remove_binary cargo-plain
    fi
    if [[ "${INSTALL_LSP}" -eq 1 ]]; then
        say "Removing the ORust language server"
        remove_binary orust-lsp
    fi
    if [[ "${INSTALL_RUNTIME}" -eq 1 ]]; then
        say "orust-runtime is a shared Cargo library, not a global executable"
        say "It remains available to existing projects; remove it from a project Cargo.toml if no longer needed"
    fi
    say "Uninstallation complete."
    exit 0
fi

install_rust() {
    say "Rust/Cargo is required to build ORust. Installing rustup..."
    command -v curl >/dev/null 2>&1 || fail "curl is required to install Rust automatically"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    export PATH="${HOME}/.cargo/bin:${PATH}"
}

if ! command -v "${CARGO_COMMAND}" >/dev/null 2>&1; then
    if [[ "${PROMPT_FD}" -ne 0 ]]; then
        ask_yes_no "Rust/Cargo is missing. Install Rust now?" y || fail "Cargo is required"
        install_rust
    else
        install_rust
    fi
fi

command -v "${CARGO_COMMAND}" >/dev/null 2>&1 || fail "Cargo is still unavailable"

prepare_workspace() {
    command -v git >/dev/null 2>&1 || fail "git is required to install ORust from the public repository"
    WORKSPACE_DIR="$(mktemp -d "${TMPDIR:-/tmp}/orust-source.XXXXXX")"
    say "Downloading ORust source from ${ORUST_REPOSITORY} (${ORUST_REF})"
    git clone --depth 1 --branch "${ORUST_REF}" "${ORUST_REPOSITORY}" "${WORKSPACE_DIR}" >/dev/null
    say "Building the ORust workspace"
    say "This compiles the parser, emitter, lowering, diagnostics, runtime, and rustplain support used by the installed tools."
    local packages=(
        orust-cli
        orust-lsp
        orust-runtime
        orust-syntax
        orust-lower
        orust-emit
        orust-diag
        rustplain-cargo
        rustplain-cli
        rustplain-diag
        rustplain-explain
        rustplain-render
    )
    local package_args=()
    local package
    for package in "${packages[@]}"; do
        package_args+=("-p" "${package}")
    done
    (
        cd "${WORKSPACE_DIR}"
        "${CARGO_COMMAND}" build --release --locked \
            --manifest-path Cargo.toml "${package_args[@]}"
    )
}

install_workspace_binary() {
    local name="$1"
    local source="${WORKSPACE_DIR}/target/release/${name}"
    local destination="${CARGO_BIN_DIR}/${name}"
    if [[ ! -f "${source}" && -f "${source}.exe" ]]; then
        source="${source}.exe"
        destination="${destination}.exe"
    fi
    [[ -f "${source}" ]] || fail "the workspace did not produce ${name}"
    mkdir -p "${CARGO_BIN_DIR}"
    cp "${source}" "${destination}"
    chmod +x "${destination}" 2>/dev/null || true
    say "Installed ${name} to ${destination}"
}

if [[ "${INSTALL_CLI}" -eq 1 || "${INSTALL_LSP}" -eq 1 ]]; then
    prepare_workspace
    if [[ "${INSTALL_CLI}" -eq 1 ]]; then
        if [[ "${UPDATE_ONLY}" -eq 1 ]]; then
            say "Updated orust-cli and its compiler libraries"
        else
            say "Installing orust-cli and its compiler libraries"
        fi
        install_workspace_binary orust
        say "Installing rustplain support for readable Rust diagnostics"
        install_workspace_binary rustplain
        install_workspace_binary cargo-plain
    fi
    if [[ "${INSTALL_LSP}" -eq 1 ]]; then
        if [[ "${UPDATE_ONLY}" -eq 1 ]]; then
            say "Updated orust-lsp and its language-service libraries"
        else
            say "Installing orust-lsp and its language-service libraries"
        fi
        install_workspace_binary orust-lsp
    fi
fi

prepare_runtime() {
    local temp_dir
    temp_dir="$(mktemp -d "${TMPDIR:-/tmp}/orust-runtime.XXXXXX")"
    trap 'rm -rf "${temp_dir}"' EXIT
    mkdir -p "${temp_dir}/src"
    printf '%s\n' \
        '[package]' \
        'name = "orust-runtime-bootstrap"' \
        'version = "0.0.0"' \
        'edition = "2021"' \
        '' \
        '[dependencies]' \
        'orust-runtime = "0.1"' \
        > "${temp_dir}/Cargo.toml"
    printf '%s\n' 'fn main() {}' > "${temp_dir}/src/main.rs"
    if [[ "${UPDATE_ONLY}" -eq 1 ]]; then
        say "Checking for updates to orust-runtime"
        "${CARGO_COMMAND}" update --manifest-path "${temp_dir}/Cargo.toml"
    else
        say "Preparing orust-runtime for generated projects"
    fi
    (
        cd "${temp_dir}"
        "${CARGO_COMMAND}" fetch --manifest-path Cargo.toml
    )
    rm -rf "${temp_dir}"
    trap - EXIT
    say "orust-runtime is ready; new projects will include it automatically"
}

if [[ "${INSTALL_RUNTIME}" -eq 1 ]]; then
    prepare_runtime
fi

if command -v rust-analyzer >/dev/null 2>&1; then
    say "Rust Analyzer is available: $(rust-analyzer --version 2>/dev/null || command -v rust-analyzer)"
elif [[ "${INSTALL_RUST_ANALYZER}" -eq 1 ]]; then
    if command -v "${RUSTUP_COMMAND}" >/dev/null 2>&1; then
        say "Installing Rust Analyzer component for ${PLATFORM} (${ARCH})"
        if ! "${RUSTUP_COMMAND}" component add rust-analyzer; then
            warn "This Rust toolchain does not provide rust-analyzer as a rustup component."
        fi
    else
        warn "rustup is unavailable; install Rust Analyzer from the official editor/tool distribution."
    fi
fi

if ! command -v rust-analyzer >/dev/null 2>&1; then
    warn "Rust Analyzer was not found. Rust code embedded in ORust will not have Rust Analyzer features."
    warn "Install it with rustup component add rust-analyzer, or install the Rust Analyzer editor extension."
fi

if [[ -n "${CREATE_PROJECT}" ]]; then
    command -v orust >/dev/null 2>&1 || fail "orust was not installed, so the project cannot be created"
    say "Creating starter project: ${CREATE_PROJECT}"
    orust new "${CREATE_PROJECT}"
    say "Project created. Run: cd ${CREATE_PROJECT} && orust run"
fi

if [[ -n "${WORKSPACE_DIR}" ]]; then
    rm -rf "${WORKSPACE_DIR}"
fi

say "Installation complete."
say "Run: orust --version"
say "Run: orust new my-project"
say "Cargo binaries are normally installed in: ${HOME}/.cargo/bin"
