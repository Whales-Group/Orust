#!/usr/bin/env bash
set -euo pipefail

# Install a published ORust release without requiring Git or a local Rust build.
ORUST_REPOSITORY="${ORUST_REPOSITORY:-Jesse-Dan/Orust}"
ORUST_VERSION="${ORUST_VERSION:-latest}"
ORUST_HOME="${ORUST_HOME:-${HOME}/.orust}"
ORUST_BIN="${ORUST_HOME}/bin"

fail() { printf '[orust] error: %s\n' "$*" >&2; exit 1; }
say() { printf '[orust] %s\n' "$*"; }

command -v curl >/dev/null 2>&1 || fail "curl is required (macOS: it is preinstalled; Linux: install the curl package)"
command -v tar >/dev/null 2>&1 || fail "tar is required (install the tar package)"
if command -v sha256sum >/dev/null 2>&1; then
    SHA256="sha256sum"
elif command -v shasum >/dev/null 2>&1; then
    SHA256="shasum -a 256"
else
    fail "sha256sum or shasum is required to verify the download"
fi

case "$(uname -s)" in
    Darwin)
        case "$(uname -m)" in
            arm64|aarch64) ASSET="macos-aarch64"; ARCHIVE_SUFFIX="tar.gz" ;;
            x86_64) ASSET="macos-x86_64"; ARCHIVE_SUFFIX="tar.gz" ;;
            *) fail "unsupported macOS architecture: $(uname -m)" ;;
        esac
        ;;
    Linux)
        [[ "$(uname -m)" == "x86_64" ]] || fail "unsupported Linux architecture: $(uname -m)"
        ASSET="linux-x86_64"
        ARCHIVE_SUFFIX="tar.gz"
        ;;
    *) fail "unsupported operating system; use install.ps1 on Windows" ;;
esac

if [[ "${ORUST_VERSION}" == latest ]]; then
    ORUST_VERSION="$(curl -fsSL "https://api.github.com/repos/${ORUST_REPOSITORY}/releases/latest" \
        | sed -nE 's/.*"tag_name"[[:space:]]*:[[:space:]]*"([^"]+)".*/\1/p' \
        | head -n 1)"
    [[ -n "${ORUST_VERSION}" ]] || fail "could not determine the latest release"
fi
ORUST_VERSION="${ORUST_VERSION#v}"
TAG="v${ORUST_VERSION}"
ARCHIVE="orust-${ORUST_VERSION}-${ASSET}.${ARCHIVE_SUFFIX}"
BASE_URL="https://github.com/${ORUST_REPOSITORY}/releases/download/${TAG}"

TEMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/orust-install.XXXXXX")"
trap 'rm -rf "${TEMP_DIR}"' EXIT
say "Downloading ORust ${ORUST_VERSION} for ${ASSET}"
curl -fsSL "${BASE_URL}/${ARCHIVE}" -o "${TEMP_DIR}/${ARCHIVE}"
curl -fsSL "${BASE_URL}/SHA256SUMS" -o "${TEMP_DIR}/SHA256SUMS"

EXPECTED="$(awk -v name="${ARCHIVE}" '$NF == name { print $1; exit }' "${TEMP_DIR}/SHA256SUMS")"
[[ -n "${EXPECTED}" ]] || fail "SHA256SUMS does not contain ${ARCHIVE}"
ACTUAL="$(${SHA256} "${TEMP_DIR}/${ARCHIVE}" | awk '{print $1}')"
[[ "${ACTUAL}" == "${EXPECTED}" ]] || fail "checksum verification failed for ${ARCHIVE}"

VERSION_DIR="${ORUST_HOME}/versions/${ORUST_VERSION}"
rm -rf "${VERSION_DIR}"
mkdir -p "${VERSION_DIR}" "${ORUST_BIN}"
tar -xzf "${TEMP_DIR}/${ARCHIVE}" -C "${VERSION_DIR}"
[[ -x "${VERSION_DIR}/orust" ]] || fail "release archive does not contain an executable orust"
[[ -x "${VERSION_DIR}/orust-lsp" ]] || fail "release archive does not contain an executable orust-lsp"
ln -sfn "${VERSION_DIR}/orust" "${ORUST_BIN}/orust"
ln -sfn "${VERSION_DIR}/orust-lsp" "${ORUST_BIN}/orust-lsp"

PROFILE=""
for candidate in "${ZDOTDIR:-${HOME}}/.zshrc" "${HOME}/.bashrc" "${HOME}/.profile"; do
    if [[ -f "${candidate}" ]] || [[ "${candidate}" == "${ZDOTDIR:-${HOME}}/.zshrc" && "${SHELL:-}" == */zsh ]]; then
        PROFILE="${candidate}"
        break
    fi
done
if [[ -n "${PROFILE}" ]]; then
    touch "${PROFILE}"
    if ! grep -qF '# >>> orust >>>' "${PROFILE}"; then
        {
            printf '\n# >>> orust >>>\n'
            printf 'export PATH="%s:$PATH"\n' "${ORUST_BIN}"
            printf '# <<< orust <<<\n'
        } >> "${PROFILE}"
    fi
fi

say "Installed orust and orust-lsp to ${ORUST_BIN}"
if ! command -v cargo >/dev/null 2>&1; then
    say "Rust/Cargo is not installed. ORust needs it to build projects."
    say "Install it with: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
fi
say "Restart your shell or run: export PATH=\"${ORUST_BIN}:\$PATH\""
say "Run: orust --version"
