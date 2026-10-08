#!/usr/bin/env bash
set -eu

# Public ORust installer. It is safe to run from curl | bash because prompts
# use /dev/tty when the script itself is being read from standard input.
ORUST_REPOSITORY="${ORUST_REPOSITORY:-https://github.com/Jesse-Dan/Orust.git}"
ORUST_REF="${ORUST_REF:-main}"
CARGO_COMMAND="${CARGO_COMMAND:-cargo}"
RUSTUP_COMMAND="${RUSTUP_COMMAND:-rustup}"
INSTALL_CLI=1
INSTALL_LSP=1
INSTALL_RUST_ANALYZER=0
CREATE_PROJECT=""
NON_INTERACTIVE=0

say() { printf '%s\n' "[orust] $*"; }
warn() { printf '%s\n' "[orust] warning: $*" >&2; }
fail() { printf '%s\n' "[orust] error: $*" >&2; exit 1; }

usage() {
    cat <<'USAGE'
Usage: install.sh [options]

Installs ORust CLI tools from the public GitHub repository.

Options:
  --all                  Install CLI, LSP, Rust Analyzer, and Rust toolchain
  --cli                  Install only the orust CLI
  --lsp                  Install only orust-lsp
  --rust-analyzer        Install Rust Analyzer through rustup when missing
  --project NAME         Create a starter project after installation
  --non-interactive      Do not prompt; install selected/default components
  --help                 Show this help

Environment:
  ORUST_REF              Git branch or tag; default: main
  ORUST_REPOSITORY       Git repository URL
USAGE
}

while (($# > 0)); do
    case "$1" in
        --all)
            INSTALL_CLI=1
            INSTALL_LSP=1
            INSTALL_RUST_ANALYZER=1
            ;;
        --cli)
            INSTALL_CLI=1
            INSTALL_LSP=0
            ;;
        --lsp)
            INSTALL_CLI=0
            INSTALL_LSP=1
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
    say "Choose what to install:"
    ask_yes_no "Install the orust CLI?" y && INSTALL_CLI=1 || INSTALL_CLI=0
    ask_yes_no "Install orust-lsp?" y && INSTALL_LSP=1 || INSTALL_LSP=0
    if command -v rust-analyzer >/dev/null 2>&1; then
        say "Rust Analyzer is already installed: $(command -v rust-analyzer)"
    elif ask_yes_no "Install Rust Analyzer for ${PLATFORM}?" y; then
        INSTALL_RUST_ANALYZER=1
    fi
    if [[ -z "${CREATE_PROJECT}" ]] && ask_yes_no "Create a starter ORust project after installation?" n; then
        read -r -u "${PROMPT_FD}" -p "[orust] Project directory [hello-orust] " CREATE_PROJECT || true
        CREATE_PROJECT="${CREATE_PROJECT:-hello-orust}"
    fi
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

if [[ "${INSTALL_CLI}" -eq 1 ]]; then
    say "Installing orust-cli from ${ORUST_REPOSITORY} (${ORUST_REF})"
    "${CARGO_COMMAND}" install --git "${ORUST_REPOSITORY}" --branch "${ORUST_REF}" --locked --force orust-cli
fi

if [[ "${INSTALL_LSP}" -eq 1 ]]; then
    say "Installing orust-lsp from ${ORUST_REPOSITORY} (${ORUST_REF})"
    "${CARGO_COMMAND}" install --git "${ORUST_REPOSITORY}" --branch "${ORUST_REF}" --locked --force orust-lsp
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

say "Installation complete."
say "Run: orust --version"
say "Run: orust new my-project"
say "Cargo binaries are normally installed in: ${HOME}/.cargo/bin"
