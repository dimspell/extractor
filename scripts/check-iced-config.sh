#!/usr/bin/env bash
# Verify the forked-iced configuration is coherent across the workspace:
#   * one git rev of the dimspell/iced fork everywhere (manifests + lockfile)
#   * the expected winit patch rev
#   * the software test backend is compiled in and requested by test targets
#   * the shared accessibility feature is enabled consistently
#
# See docs/iced-fork-config.md for the full feature matrix and rationale.
# Doc counts, Makefile target parity and CI command parity are covered by the
# companion scripts/check-project-metadata.sh.
# Run locally with `make check_iced_config` (or `bash scripts/check-iced-config.sh`).
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

fail() { printf 'check-iced-config: FAIL: %s\n' "$*" >&2; exit 1; }

# Keep these in sync with the root Cargo.toml.
ICED_REV="c0ffdd8e7352df9283496f3db334731c00ef6d0d"
WINIT_REV="05b8ff17a06562f0a10bb46e6eaacbe2a95cb5ed"

# 1. Every dimspell/iced git dependency in every manifest uses the one pin.
manifest_revs="$(
  find . -name Cargo.toml -not -path './target/*' -print0 \
    | xargs -0 grep -hE 'dimspell/iced' \
    | grep -oE 'rev[[:space:]]*=[[:space:]]*"[0-9a-f]{40}"' \
    | grep -oE '[0-9a-f]{40}' \
    | sort -u
)"
[ "$(printf '%s\n' "$manifest_revs" | grep -c .)" -eq 1 ] \
  || fail "expected exactly one iced fork rev in manifests, got: ${manifest_revs:-<none>}"
[ "$manifest_revs" = "$ICED_REV" ] \
  || fail "manifest iced rev '$manifest_revs' != expected '$ICED_REV'"

# 2. The lockfile must not carry a second iced fork rev (that would build a
#    duplicate, incompatible crate graph — the bug this check guards against).
lock_revs="$(grep -oE 'dimspell/iced\?rev=[0-9a-f]{40}' Cargo.lock | sed 's/.*rev=//' | sort -u)"
[ "$lock_revs" = "$ICED_REV" ] \
  || fail "Cargo.lock iced revs ('$lock_revs') are not the single pin '$ICED_REV'"

# 3. The winit patch rev pinned for version alignment with the iced fork.
grep -Eq "iced-rs/winit\.git.*rev = \"$WINIT_REV\"" Cargo.toml \
  || fail "winit patch rev in Cargo.toml does not match '$WINIT_REV'"

# 4. GUI/hexedit test commands must request the headless software backend.
#    Commands are defined once in scripts/ci-commands.sh and called by both the
#    Makefile and CI (enforced by scripts/check-project-metadata.sh). Read each
#    dispatch arm from that script instead of repeating its command text here.
assert_software_test_backend() {
  local arm="$1" body
  body="$(sed -n "/^  ${arm})/,/^    ;;/p" scripts/ci-commands.sh)"
  [ -n "$body" ] \
    || fail "ci-commands.sh has no $arm dispatch arm"
  printf '%s\n' "$body" | grep -q 'cargo test' \
    || fail "ci-commands.sh $arm arm must run cargo test"
  printf '%s\n' "$body" | grep -qF 'ICED_TEST_BACKEND=software' \
    || fail "ci-commands.sh $arm must set ICED_TEST_BACKEND=software"
}
assert_software_test_backend test
assert_software_test_backend iced-test
assert_software_test_backend hexedit-test

# 5. The software backend needs tiny-skia compiled in on both iced_test paths.
grep -q 'iced/tiny-skia' dispel-gui/Cargo.toml \
  || fail "dispel-gui 'iced_test' feature must enable iced/tiny-skia"
grep -q 'features = \["tiny-skia"\]' hexedit/Cargo.toml \
  || fail "hexedit iced_renderer dev-dep must enable tiny-skia"

# 6. Accessibility is default-on in gui-widgets and force-enabled by both hosts.
grep -q '^default = \["accessibility"\]' gui-widgets/Cargo.toml \
  || fail "gui-widgets must keep accessibility as a default feature"
for manifest in dispel-gui/Cargo.toml hexedit/Cargo.toml; do
  grep -q 'gui-widgets = { path = "../gui-widgets", features = \["accessibility"\] }' "$manifest" \
    || fail "$manifest must force-enable gui-widgets/accessibility"
done

printf 'check-iced-config: OK (single iced pin %s, software test backend)\n' "$ICED_REV"
