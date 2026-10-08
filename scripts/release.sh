#!/usr/bin/env bash
set -euo pipefail

# Bump the shared ORust workspace version and optionally prepare the Git tag
# that starts the GitHub release workflow.
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

COMMIT=0
TAG=0
PUSH=0
CHECK=1

usage() {
    cat <<'USAGE'
Usage: scripts/release.sh <patch|minor|major|VERSION> [options]

Bumps the shared workspace version in Cargo.toml and internal crate links.

Options:
  --commit       Commit the version files
  --tag          Create the matching vVERSION Git tag
  --push         Push the commit and tag to origin
  --skip-checks  Do not run cargo fmt/check before preparing the release
  --help         Show this help

Examples:
  scripts/release.sh patch
  scripts/release.sh minor --commit --tag
  scripts/release.sh patch --commit --tag --push
USAGE
}

if [[ $# -eq 0 ]]; then
    usage
    exit 2
fi
if [[ "$1" == "--help" || "$1" == "-h" ]]; then
    usage
    exit 0
fi
REQUESTED_VERSION="$1"
shift

while [[ $# -gt 0 ]]; do
    case "$1" in
        --commit) COMMIT=1 ;;
        --tag) TAG=1 ;;
        --push) PUSH=1 ;;
        --skip-checks) CHECK=0 ;;
        --help|-h) usage; exit 0 ;;
        *) printf 'error: unknown option: %s\n' "$1" >&2; exit 2 ;;
    esac
    shift
done

if [[ "${PUSH}" -eq 1 ]]; then
    COMMIT=1
    TAG=1
fi
if [[ "${TAG}" -eq 1 && "${COMMIT}" -eq 0 ]]; then
    printf '%s\n' 'error: --tag requires --commit' >&2
    exit 2
fi

CURRENT_VERSION="$(sed -nE 's/^version = "([0-9]+\.[0-9]+\.[0-9]+)"$/\1/p' Cargo.toml | head -n 1)"
[[ "${CURRENT_VERSION}" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || {
    printf 'error: could not read the workspace version\n' >&2
    exit 1
}

case "${REQUESTED_VERSION}" in
    patch|minor|major)
        IFS=. read -r major minor patch <<<"${CURRENT_VERSION}"
        case "${REQUESTED_VERSION}" in
            patch) patch=$((patch + 1)) ;;
            minor) minor=$((minor + 1)); patch=0 ;;
            major) major=$((major + 1)); minor=0; patch=0 ;;
        esac
        NEW_VERSION="${major}.${minor}.${patch}"
        ;;
    *)
        NEW_VERSION="${REQUESTED_VERSION}"
        [[ "${NEW_VERSION}" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || {
            printf 'error: version must be patch, minor, major, or X.Y.Z\n' >&2
            exit 2
        }
        ;;
esac

[[ "${NEW_VERSION}" != "${CURRENT_VERSION}" ]] || {
    printf 'error: new version is the same as the current version (%s)\n' "${CURRENT_VERSION}" >&2
    exit 2
}

replace_versions() {
    local file="$1"
    local temporary="${file}.orust-release-tmp"
    sed -E "s/version = \"[0-9]+\\.[0-9]+\\.[0-9]+\"/version = \"${NEW_VERSION}\"/g" "${file}" > "${temporary}"
    mv "${temporary}" "${file}"
}

temporary="Cargo.toml.orust-release-tmp"
awk -v version="${NEW_VERSION}" '
    BEGIN { changed = 0 }
    !changed && $0 ~ /^version = "[^"]*"$/ {
        print "version = \"" version "\""
        changed = 1
        next
    }
    { print }
' Cargo.toml > "${temporary}"
mv "${temporary}" Cargo.toml

while IFS= read -r -d '' manifest; do
    replace_versions "${manifest}"
done < <(find crates -name Cargo.toml -print0)

if [[ "${CHECK}" -eq 1 ]]; then
    cargo fmt --all
    cargo check --workspace
fi

printf 'ORust version: %s -> %s\n' "${CURRENT_VERSION}" "${NEW_VERSION}"
printf 'Updated workspace and internal crate dependency versions.\n'

if [[ "${COMMIT}" -eq 1 ]]; then
    git add Cargo.toml Cargo.lock
    while IFS= read -r -d '' manifest; do
        git add "${manifest}"
    done < <(find crates -name Cargo.toml -print0)
    git commit -m "Release v${NEW_VERSION}"
fi

if [[ "${TAG}" -eq 1 ]]; then
    git tag -a "v${NEW_VERSION}" -m "ORust v${NEW_VERSION}"
fi

if [[ "${PUSH}" -eq 1 ]]; then
    git push origin HEAD
    git push origin "v${NEW_VERSION}"
fi

printf 'Release preparation complete for v%s.\n' "${NEW_VERSION}"
if [[ "${TAG}" -eq 0 ]]; then
    printf 'Next deployment step: git tag -a v%s -m "ORust v%s" && git push origin v%s\n' "${NEW_VERSION}" "${NEW_VERSION}" "${NEW_VERSION}"
fi
