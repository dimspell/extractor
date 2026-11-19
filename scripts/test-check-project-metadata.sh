#!/usr/bin/env bash
# Self-test for scripts/check-project-metadata.sh. Copies the metadata inputs to
# a scratch tree, verifies the clean tree passes, then injects eight independent
# drifts and asserts each one is caught. No repository files are modified.
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

tmp="$(mktemp -d "${TMPDIR:-/tmp}/check-metadata-selftest.XXXXXX")"
trap 'rm -rf "$tmp"' EXIT

seed() {
  mkdir -p "$tmp/scripts" "$tmp/.github/workflows" "$tmp/src" "$tmp/hexedit" "$tmp/dispel-gui/src" "$tmp/docs"
  cp AGENTS.md Makefile Cargo.toml "$tmp/"
  cp scripts/ci-commands.sh "$tmp/scripts/"
  cp .github/build-matrix.json "$tmp/.github/"
  cp .github/workflows/*.yml "$tmp/.github/workflows/"
  cp -R src/. "$tmp/src/"
  cp -R hexedit/. "$tmp/hexedit/"
  cp -R dispel-gui/src/. "$tmp/dispel-gui/src/"
  cp -R docs/. "$tmp/docs/"
  # The checker scopes its stale-workflow check to git-tracked files. Make the
  # scratch tree a git repo and stage only the real workflow files; local-only
  # *.ignore.* copies are stripped so this stays deterministic regardless of
  # the developer's global gitignore.
  find "$tmp/.github/workflows" -name '*.ignore.*' -delete
  git -C "$tmp" init -q
  git -C "$tmp" add .github/workflows
}

checker() { bash "$repo_root/scripts/check-project-metadata.sh" "$tmp"; }

expect_ok() {
  if ! checker >"$tmp/out" 2>&1; then
    cat "$tmp/out" >&2
    printf 'selftest FAIL: %s should have passed\n' "$1" >&2
    exit 1
  fi
  printf 'selftest OK: %s passes\n' "$1"
}

expect_fail() {
  if checker >"$tmp/out" 2>&1; then
    printf 'selftest FAIL: %s was not detected\n' "$1" >&2
    exit 1
  fi
  if ! grep -q 'check-project-metadata: FAIL' "$tmp/out"; then
    cat "$tmp/out" >&2
    printf 'selftest FAIL: %s produced an unexpected error\n' "$1" >&2
    exit 1
  fi
  printf 'selftest OK: %s detected\n' "$1"
}

seed
expect_ok "clean tree"

# 1. Doc count drift.
python3 - "$tmp/AGENTS.md" <<'PY'
import pathlib, sys
p = pathlib.Path(sys.argv[1])
p.write_text(p.read_text().replace("6-crate Cargo workspace", "5-crate Cargo workspace"))
PY
expect_fail "stale AGENTS.md crate count"
cp AGENTS.md "$tmp/AGENTS.md"

# 2. .PHONY drift (a real recipe drops out of the phony list).
python3 - "$tmp/Makefile" <<'PY'
import pathlib, sys
p = pathlib.Path(sys.argv[1])
p.write_text(p.read_text().replace(
    "map-help map-tiles map-atlas $(MAP_RENDER_TARGETS) \\",
    "map-help map-tiles map-atlas \\",
))
PY
expect_fail "Makefile .PHONY / target parity drift"
cp Makefile "$tmp/Makefile"

# 3. CI command drift (inline clippy instead of the canonical command).
python3 - "$tmp/.github/workflows/pr.yml" <<'PY'
import pathlib, sys
p = pathlib.Path(sys.argv[1])
p.write_text(p.read_text().replace(
    "bash scripts/ci-commands.sh clippy",
    "cargo clippy --workspace -- -D warnings",
))
PY
expect_fail "CI clippy command drift"
cp .github/workflows/pr.yml "$tmp/.github/workflows/pr.yml"

# 4. Shared matrix drift (a build entry disappears).
python3 - "$tmp/.github/build-matrix.json" <<'PY'
import json, pathlib, sys
p = pathlib.Path(sys.argv[1])
data = json.loads(p.read_text())
data["include"] = data["include"][:-1]
p.write_text(json.dumps(data))
PY
expect_fail "build-matrix entry drift"
cp .github/build-matrix.json "$tmp/.github/build-matrix.json"

# 5. Toolchain action drift (back to the ad-hoc composite action).
python3 - "$tmp/.github/workflows/test.yml" <<'PY'
import pathlib, sys
p = pathlib.Path(sys.argv[1])
p.write_text(p.read_text().replace(
    "uses: dtolnay/rust-toolchain@stable",
    "uses: dimspell/dispel-extractor/.github/actions/setup-rust@master",
))
PY
expect_fail "CI toolchain action drift"
cp .github/workflows/test.yml "$tmp/.github/workflows/test.yml"

# 6. New recipe with no .PHONY entry.
printf '\norphan-target:\n\t@echo hi\n' >> "$tmp/Makefile"
expect_fail "orphan Makefile recipe"
cp Makefile "$tmp/Makefile"

# 7. Doc count drift in a different domain (per-format docs).
printf '# scratch\n' > "$tmp/docs/files/ZZ_scratch.md"
expect_fail "docs/files count drift"
rm -f "$tmp/docs/files/ZZ_scratch.md"

# 8. Stale workflow copies are scoped to tracked files: an untracked, ignored
#    local copy is tolerated, but a tracked copy with 'ignore' in its name is
#    a real drift that must be caught.
printf '# local scratch copy\n' > "$tmp/.github/workflows/local.ignore.yml"
expect_ok "untracked ignored workflow copy tolerated"
rm -f "$tmp/.github/workflows/local.ignore.yml"

printf '# stale tracked copy\n' > "$tmp/.github/workflows/stale.ignore.yml"
git -C "$tmp" add -f .github/workflows/stale.ignore.yml
expect_fail "tracked stale workflow copy"
git -C "$tmp" rm --cached -q .github/workflows/stale.ignore.yml
rm -f "$tmp/.github/workflows/stale.ignore.yml"

expect_ok "restored tree"

printf 'test-check-project-metadata: OK (8 injected drifts detected)\n'
