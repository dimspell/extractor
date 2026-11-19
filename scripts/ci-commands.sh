#!/usr/bin/env bash
# Single source of truth for the fmt / clippy / test commands shared by the
# Makefile and GitHub Actions. Inline these commands nowhere else:
# scripts/check-project-metadata.sh enforces that both callers use this script.
#
# Usage: bash scripts/ci-commands.sh <command>
#        bash scripts/ci-commands.sh --list   # machine-readable command list
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

COMMANDS=(
  fmt
  fmt-check
  clippy
  test
  check
  iced-test
  hexedit-test
  check-iced-config
  check-metadata
  check-metadata-test
)

if [[ "${1:-}" == "--list" ]]; then
  printf '%s\n' "${COMMANDS[@]}"
  exit 0
fi

cmd="${1:-}"
case "$cmd" in
  fmt)
    cargo fmt --all
    ;;
  fmt-check)
    cargo fmt --all --check
    ;;
  clippy)
    cargo clippy --workspace --all-features --all-targets -- -D warnings
    ;;
  test)
    # GUI simulations need Iced's CPU renderer when no GPU/window server is
    # available (CI, headless shells). Safe no-op for crates without that feature.
    ICED_TEST_BACKEND=software cargo test --workspace --all-features
    ;;
  check)
    cargo check --workspace
    ;;
  iced-test)
    ICED_TEST_BACKEND=software cargo test -p dispel-gui --features "iced_test"
    ;;
  hexedit-test)
    ICED_TEST_BACKEND=software cargo test -p hexedit --quiet
    ;;
  check-iced-config)
    bash scripts/check-iced-config.sh
    ;;
  check-metadata)
    bash scripts/check-project-metadata.sh
    ;;
  check-metadata-test)
    bash scripts/test-check-project-metadata.sh
    ;;
  *)
    printf 'ci-commands: unknown command %q\n' "$cmd" >&2
    printf 'available commands: %s\n' "${COMMANDS[*]}" >&2
    exit 2
    ;;
esac
