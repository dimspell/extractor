#!/usr/bin/env bash
# Verify cross-cutting project metadata cannot drift silently:
#   * AGENTS.md counts match the actual tree (crates, SQL, tests, docs, editors, targets)
#   * Makefile .PHONY mirrors the real recipe targets
#   * fmt/clippy/test commands are defined once (scripts/ci-commands.sh) and used
#     by both the Makefile and every workflow
#   * pr.yml / release.yml share one build matrix (.github/build-matrix.json)
#   * all workflows use the canonical toolchain action and no stale workflow file
#
# Usage: bash scripts/check-project-metadata.sh [repo-root]   (default: this repo)
# Self-test: bash scripts/test-check-project-metadata.sh
set -euo pipefail

repo_root="${1:-}"
if [[ -z "$repo_root" ]]; then
  repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fi

python3 - "$repo_root" <<'PY'
import json
import pathlib
import re
import subprocess
import sys

root = pathlib.Path(sys.argv[1])
fails = []


def check(cond, message):
    if not cond:
        fails.append(message)


def read(rel):
    path = root / rel
    if not path.is_file():
        fails.append(f"missing required file: {rel}")
        return ""
    return path.read_text(encoding="utf-8")


def walk(rel_dir, pattern):
    d = root / rel_dir
    return sorted(d.rglob(pattern)) if d.is_dir() else []


agents = read("AGENTS.md")
makefile = read("Makefile")
cargo_toml = read("Cargo.toml")
matrix_raw = read(".github/build-matrix.json")
commands_sh = read("scripts/ci-commands.sh")
workflows = {
    name: read(f".github/workflows/{name}.yml") for name in ("test", "pr", "release")
}

# ---------------------------------------------------------------------------
# 1. AGENTS.md counts vs. the actual tree
# ---------------------------------------------------------------------------
workspace_members = re.search(r"\[workspace\]\s*members\s*=\s*\[(.*?)\]", cargo_toml, re.S)
crate_count = len(re.findall(r'"[^"]+"', workspace_members.group(1))) if workspace_members else 0

sql_files = walk("src/queries", "*.sql")
ddl_files = [p for p in sql_files if p.name.startswith("create_table_")]
insert_files = [p for p in sql_files if p.name.startswith("insert_")]
check(len(sql_files) == len(ddl_files) + len(insert_files), "src/queries has unclassified .sql files")

hexedit_test_rs = [
    p for p in walk("hexedit", "*.rs") if "#[cfg(test)]" in p.read_text(encoding="utf-8")
]
root_test_rs = [
    p for p in walk("src", "*.rs") if "#[cfg(test)]" in p.read_text(encoding="utf-8")
]
root_test_blocks = sum(p.read_text(encoding="utf-8").count("#[cfg(test)]") for p in root_test_rs)
gui_test_files = walk("dispel-gui/src/tests", "*.rs")
docs_all = walk("docs/files", "*.md")
docs_per_format = [p for p in docs_all if p.name != "CROSS_REFERENCES.md"]

catalog_src = read("dispel-gui/src/editor_catalog/mod.rs")
catalog_pos = catalog_src.find("define_editor_catalog! {")
editor_variants = (
    re.findall(r"^\s{4}([A-Z]\w*)\s*\{", catalog_src[catalog_pos:], re.M)
    if catalog_pos >= 0
    else []
)
named_variants = [v for v in editor_variants if v != "Unknown"]

registry_src = read("dispel-gui/src/editor_registry.rs")
registry_match = re.search(r"pub struct EditorRegistry \{(.*?)\n\}", registry_src, re.S)
registry_fields = (
    re.findall(r"^\s+pub \w+:", registry_match.group(1), re.M) if registry_match else []
)

editor_dirs = [
    p for p in (root / "dispel-gui/src/editors").iterdir() if p.is_dir()
] if (root / "dispel-gui/src/editors").is_dir() else []
reference_rs = walk("src/references", "*.rs")

# The metadata self-test's advertised drift count must track its real drift
# sites so AGENTS.md cannot claim a stale number.
selftest_src = read("scripts/test-check-project-metadata.sh")
selftest_drift_count = len(re.findall(r"^expect_fail\s+", selftest_src, re.M))
drift_words = {
    0: "zero", 1: "one", 2: "two", 3: "three", 4: "four", 5: "five",
    6: "six", 7: "seven", 8: "eight", 9: "nine", 10: "ten",
}
check(
    selftest_drift_count in drift_words,
    f"metadata selftest drift count {selftest_drift_count} has no word mapping; extend the mapping and AGENTS.md",
)
selftest_drift_word = drift_words.get(selftest_drift_count, "")

doc_expectations = [
    (f"{crate_count}-crate Cargo workspace", "workspace crate count"),
    (f"{len(sql_files)} SQL files ({len(ddl_files)} schema DDL + {len(insert_files)} INSERTs)", "SQL file counts"),
    (f"{root_test_blocks} `#[cfg(test)]` blocks across {len(root_test_rs)} `.rs` files", "root crate cfg(test) counts"),
    (f"{len(hexedit_test_rs)} `.rs` files with `#[cfg(test)]`", "hexedit cfg(test) file count"),
    (f"{len(gui_test_files)} test files", "dispel-gui test file count"),
    (f"{len(docs_per_format)} per-format docs", "per-format doc count"),
    (f"{len(docs_all)} files total", "docs/files total count"),
    (f"{len(named_variants)} named variants + `Unknown`", "editor variant count"),
    (f"EDITOR_COUNT = {len(editor_variants)}", "EDITOR_COUNT"),
    (f"{len(registry_fields)} state fields", "EditorRegistry field count"),
    (f"{len(editor_dirs)} editor subdirectories", "editor directory count"),
    (f"{len(reference_rs)} source files", "references source file count"),
    (f"injects {selftest_drift_word} independent drifts", "metadata selftest drift count"),
]
for needle, label in doc_expectations:
    check(needle in agents, f"AGENTS.md {label} is stale (expected {needle!r})")

stale_doc_strings = [
    "5-crate Cargo workspace",
    "90 SQL files",
    "45 schema DDL",
    "~120 targets",
    "24 files with `#[cfg(test)]`",
    "40 named variants",
    "~35 game data file parsers",
    "~15 `map-render-*",
    "~38 editor subdirectories",
    "~70 `#[cfg(test)]` blocks",
    "tests NOT run in CI",
]
for stale in stale_doc_strings:
    check(stale not in agents, f"AGENTS.md still contains stale claim: {stale!r}")
check(
    not any(line.startswith("├── tests/") for line in agents.splitlines()),
    "AGENTS.md must not list an empty root tests/ directory",
)
check("dispel-encodings" in agents, "AGENTS.md must document the dispel-encodings crate")

# ---------------------------------------------------------------------------
# 2. Makefile: single map-render recipe, .PHONY parity, no sibling-repo path
# ---------------------------------------------------------------------------
# Join backslash continuations for non-recipe lines only.
logical_lines = []
pending = ""
for line in makefile.splitlines():
    if line.startswith("\t"):
        continue
    if pending:
        pending += " " + line.strip()
        if not line.rstrip().endswith("\\"):
            logical_lines.append(pending)
            pending = ""
        continue
    if line.rstrip().endswith("\\"):
        pending = line.rstrip()[:-1]
    else:
        logical_lines.append(line)
if pending:
    logical_lines.append(pending)

make_vars = {}
var_def = re.compile(r"^([A-Za-z_][A-Za-z0-9_]*)\s*(?::=|\?=|=)\s*(.*)$")
phony_raw = ""
for line in logical_lines:
    stripped = line.strip()
    if not stripped or stripped.startswith("#"):
        continue
    if line.startswith(".PHONY:"):
        phony_raw += " " + line[len(".PHONY:"):]
        continue
    match = var_def.match(line)
    if match:
        make_vars[match.group(1)] = match.group(2).strip()
        continue

var_ref = re.compile(r"\$\(([A-Za-z_][A-Za-z0-9_]*)\)")


def expand_make(text, depth=0):
    if depth > 4:
        return text
    return var_ref.sub(lambda m: expand_make(make_vars.get(m.group(1), ""), depth + 1), text)


token_ok = re.compile(r"[A-Za-z0-9_][A-Za-z0-9_.+/-]*\Z")
real_targets = set()
for line in logical_lines:
    if line.startswith((".PHONY:", "#")) or ":" not in line:
        continue
    if var_def.match(line):  # variable assignment, not a rule
        continue
    lhs = line.split(":", 1)[0].strip()
    if not lhs:
        continue
    for token in expand_make(lhs).split():
        if token_ok.fullmatch(token):
            real_targets.add(token)

phony_targets = {
    token
    for token in expand_make(phony_raw).replace("\\", " ").split()
    if token_ok.fullmatch(token)
}

missing_phony = sorted(real_targets - phony_targets)
extra_phony = sorted(phony_targets - real_targets)
check(not missing_phony, f"Makefile targets missing from .PHONY: {missing_phony}")
check(not extra_phony, f"Makefile .PHONY entries with no recipe: {extra_phony}")

target_count = len(real_targets)
check(f"{target_count} targets" in agents, f"AGENTS.md Makefile target count is stale (expected {target_count})")
map_render_targets = sorted(t for t in real_targets if t.startswith("map-render"))
check(len(map_render_targets) == 14, f"expected 14 map-render targets, found {map_render_targets}")
check(
    makefile.count("cargo run -- map render $(MAP_RENDER_ARGS) $(MAP_RENDER_EXTRA)") == 1,
    "map-render flavors must share exactly one parameterized recipe",
)
check("../dispel-godot" not in makefile, "Makefile still hardcodes a sibling-repo light template path")
check("MAP_RENDER_TARGETS" in make_vars, "MAP_RENDER_TARGETS list is missing")

# ---------------------------------------------------------------------------
# 3. One canonical command set, used by Makefile and all workflows
# ---------------------------------------------------------------------------
# The eleven canonical command names live here once; every check below derives
# its command set from this tuple. ci-commands.sh's COMMANDS array is still read
# separately so a missing dispatch entry remains detectable.
CANONICAL_COMMANDS = (
    "fmt", "fmt-check", "clippy", "test", "build", "check",
    "iced-test", "hexedit-test", "check-iced-config", "check-metadata", "check-metadata-test",
)
required_commands = set(CANONICAL_COMMANDS)

command_block = re.search(r"COMMANDS=\(\s*(.*?)\s*\)", commands_sh, re.S)
commands = re.findall(r"^\s*([a-z][a-z-]*)\s*$", command_block.group(1), re.M) if command_block else []
check(required_commands <= set(commands), f"ci-commands.sh missing commands: {sorted(required_commands - set(commands))}")
for command in commands:
    check(
        re.search(r"^\s*" + re.escape(command) + r"\)\s*$", commands_sh, re.M) is not None,
        f"ci-commands.sh has no dispatch arm for {command!r}",
    )

# The Makefile drives every canonical command except fmt-check, which is a
# CI-only verification step.
makefile_commands = tuple(c for c in CANONICAL_COMMANDS if c != "fmt-check")
for command in makefile_commands:
    check(f"scripts/ci-commands.sh {command}" in makefile, f"Makefile does not use canonical command {command!r}")

# The shared verify job (test.yml + pr.yml) runs this six-command subset.
VERIFY_COMMANDS = frozenset({
    "fmt-check", "check-iced-config", "check-metadata", "check-metadata-test", "clippy", "test",
})
verify_commands = tuple(c for c in CANONICAL_COMMANDS if c in VERIFY_COMMANDS)
for name in ("test", "pr"):
    for command in verify_commands:
        needle = f"bash scripts/ci-commands.sh {command}"
        check(needle in workflows[name], f"{name}.yml does not call {needle!r}")

# Shell scripts must not inline a canonical command either; ci-commands.sh is
# the definition site and is therefore exempt from this scan.
script_sources = [
    (str(p.relative_to(root)), p.read_text(encoding="utf-8"))
    for p in walk("scripts", "*.sh")
    if p.name != "ci-commands.sh"
]
for label, content in [("Makefile", makefile)] + script_sources + [
    (f"{name}.yml", wf) for name, wf in workflows.items()
]:
    for line in content.splitlines():
        # Matrix release builds intentionally pass --release/--target; only the
        # plain workspace build is the canonical command that must be routed out.
        # Match both a bare command in a `run: |` block and `run: cargo build`.
        inlined_build = re.match(
            r"^\s*(?:run:\s*)?cargo build(?:\s+--workspace)?\s*$", line
        )
        if inlined_build or re.match(r"^\s*cargo (fmt|clippy|test|check)\b", line):
            fails.append(f"{label} inlines a canonical command instead of ci-commands.sh: {line.strip()!r}")

# ---------------------------------------------------------------------------
# 4. CI: canonical toolchain, real tests, one shared build matrix
# ---------------------------------------------------------------------------
for name, wf in workflows.items():
    check("dtolnay/rust-toolchain@stable" in wf, f"{name}.yml must use dtolnay/rust-toolchain@stable")
    check(
        "dimspell/dispel-extractor/.github/actions/setup-rust" not in wf,
        f"{name}.yml still uses the ad-hoc composite setup-rust action",
    )
    check(not re.search(r"^\s+include:\s*$", wf, re.M), f"{name}.yml still duplicates the build matrix inline")

test_wf = workflows["test"]
check(
    "bash scripts/ci-commands.sh build" in test_wf,
    "test.yml must keep its build gate",
)
check(
    "aarch64-unknown-linux-musl" not in test_wf and "x86_64-pc-windows-msvc" not in test_wf,
    "test.yml still installs cross targets it never builds",
)

try:
    matrix = json.loads(matrix_raw)
    builds = matrix.get("include", [])
except (ValueError, AttributeError) as exc:
    builds = []
    fails.append(f".github/build-matrix.json is invalid: {exc}")

expected_builds = {"cli-windows", "gui-windows", "gui-linux", "cli-linux", "gui-macos-arm"}
check({b.get("build") for b in builds} == expected_builds, "canonical matrix build names drifted")
required_fields = {"build", "app", "os", "rust", "target"}
for build in builds:
    check(required_fields <= set(build), f"matrix entry {build.get('build')!r} missing fields: {sorted(required_fields - set(build))}")
    check(build.get("rust") == "stable", f"matrix entry {build.get('build')!r} must pin rust=stable")

check("needs: [verify, build-matrix]" in workflows["pr"], "pr.yml build job must consume build-matrix output")
check('needs: ["create-release", "build-matrix"]' in workflows["release"], "release.yml build job must consume build-matrix output")
for name in ("pr", "release"):
    check(".github/build-matrix.json" in workflows[name], f"{name}.yml does not read the canonical matrix")
    check(
        "fromJSON(needs.build-matrix.outputs.matrix)" in workflows[name],
        f"{name}.yml does not expand the canonical matrix via fromJSON",
    )

# Only flag workflow files that git would actually commit. Untracked,
# locally-ignored copies (e.g. a developer's *.ignore.* scratch copies) belong
# to the user and must not be reported. Outside a git work tree (e.g. the
# validator self-test scratch tree), fall back to considering every file.
workflow_dir = root / ".github/workflows"
try:
    tracked_out = subprocess.run(
        ["git", "-C", str(root), "ls-files", "-z", "--", ".github/workflows"],
        capture_output=True,
        text=True,
        check=True,
    ).stdout
    tracked_names = {pathlib.Path(p).name for p in tracked_out.split("\0") if p}
except (OSError, subprocess.CalledProcessError):
    tracked_names = None  # not a git work tree
if tracked_names is None:
    stale_workflows = sorted(
        p.name for p in workflow_dir.iterdir() if "ignore" in p.name.lower()
    ) if workflow_dir.is_dir() else []
else:
    stale_workflows = sorted(n for n in tracked_names if "ignore" in n.lower())
check(not stale_workflows, f"stale ignored workflow copies present: {stale_workflows}")

# ---------------------------------------------------------------------------
if fails:
    print("check-project-metadata: FAIL", file=sys.stderr)
    for failure in fails:
        print(f"  - {failure}", file=sys.stderr)
    sys.exit(1)

print(
    f"check-project-metadata: OK ({crate_count} crates, {len(sql_files)} SQL files, "
    f"{target_count} Makefile targets, {len(builds)} build-matrix entries, "
    f"{len(commands)} canonical commands)"
)
PY
