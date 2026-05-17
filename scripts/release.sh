#!/usr/bin/env bash
#
# release.sh — Bump version, commit, tag, and push for dispel-extractor
#
# Usage: ./scripts/release.sh <new_version>
# Example: ./scripts/release.sh 0.6.4
#
# Must be run on the master branch with a clean working tree.

set -euo pipefail

# --- Validation ---

if [[ $# -ne 1 ]]; then
  echo "Usage: $0 <new_version>"
  echo "Example: $0 0.6.4"
  exit 1
fi

NEW_VERSION="$1"

# Validate semver format (major.minor.patch)
if [[ ! "$NEW_VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "Error: Version must be in semver format (e.g., 0.6.4)"
  exit 1
fi

# Check we're on master
BRANCH=$(git branch --show-current)
if [[ "$BRANCH" != "master" ]]; then
  echo "Error: Must be on master branch (currently on '$BRANCH')"
  exit 1
fi

# Check working tree is clean
if [[ -n $(git status --porcelain) ]]; then
  echo "Error: Working tree is not clean. Commit or stash changes first."
  exit 1
fi

# --- Pre-release checks ---

echo "Checking code formatting..."
if ! cargo fmt --all --check; then
    echo "Code is not formatted. Run 'cargo fmt --all' to fix."
    exit 1
fi

echo "Running tests..."
if ! cargo test --workspace --all-features; then
    echo "Tests failed. Fix before releasing."
    exit 1
fi

# --- Bump versions ---

echo "Bumping version to v${NEW_VERSION}..."

# Discover workspace packages dynamically via cargo metadata
MANIFESTS=$(cargo metadata --format-version 1 --no-deps | jq -r '.packages[].manifest_path')

if [[ -z "$MANIFESTS" ]]; then
    echo "Error: No packages found in workspace"
    exit 1
fi

CHANGED_FILES=()

while IFS= read -r manifest; do
    echo "* Updating $(basename "$(dirname "$manifest")")/Cargo.toml..."
    sed -i '' '/^\[package\]/,/^\[/ s/^version = "[^"]*"/version = "'"$NEW_VERSION"'"/' "$manifest"
    CHANGED_FILES+=("$manifest")
done <<< "$MANIFESTS"

# Regenerate Cargo.lock
echo "Regenerating Cargo.lock..."
cargo check --workspace --quiet

# --- Commit ---

COMMIT_MSG="🚀 Version v${NEW_VERSION}"
echo "Creating commit: ${COMMIT_MSG}"

git add "${CHANGED_FILES[@]}" Cargo.lock
git commit -m "$COMMIT_MSG"

# --- Tag ---

TAG="v${NEW_VERSION}"
echo "Creating tag: ${TAG}"
git tag -a "$TAG" -m "$COMMIT_MSG"

# --- Summary ---

echo ""
echo "Done! Local changes:"
echo "  Commit: $(git log -1 --format='%H')"
echo "  Tag:    ${TAG}"
echo ""
echo "To push to remote, run:"
echo "  git push origin master --tags"
