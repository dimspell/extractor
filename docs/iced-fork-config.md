# Forked-Iced Configuration (single pin set & feature matrix)

The GUI stack builds against a git fork of Iced, `dimspell/iced`. This document
is the authoritative description of how that fork is pinned, which features
each workspace crate enables, and how headless tests select their renderer.
`scripts/check-iced-config.sh` (also `make check_iced_config`) enforces the
invariants below and runs in CI.

## 1. Single pin set

All iced-family git dependencies must resolve to the **same** commit:

| Dependency | Source | Rev |
|---|---|---|
| `iced` (workspace dep) | `https://github.com/dimspell/iced` | `c0ffdd8e7352df9283496f3db334731c00ef6d0d` |
| `iced_test` (workspace dep) | `https://github.com/dimspell/iced` | `c0ffdd8e7352df9283496f3db334731c00ef6d0d` |
| `iced_renderer` (hexedit **dev**-dep) | `https://github.com/dimspell/iced` | `c0ffdd8e7352df9283496f3db334731c00ef6d0d` |
| `winit` (`[patch.crates-io]`) | `https://github.com/iced-rs/winit.git` | `05b8ff17a06562f0a10bb46e6eaacbe2a95cb5ed` |

Historically `hexedit`'s `iced_renderer` dev-dependency was pinned to a
different commit (`48fb8fe…`). That produced two parallel copies of every
iced crate in `Cargo.lock` with incompatible internal types. It is now aligned
to the workspace pin; `Cargo.lock` contains a single `iced_renderer` entry.
**Do not reintroduce a second iced rev** — enable the needed feature through
the workspace pin instead.

## 2. Feature matrix

Cargo features are additive and unify across the dependency graph. The table
lists the features each workspace crate turns on directly (everything uses
`default-features = false` on `iced`):

| Crate | `iced` features (direct) | Own features | Forces on `gui-widgets` |
|---|---|---|---|
| `gui-widgets` | `advanced`, `thread-pool`, `wgpu` | `default = [accessibility]`, `accessibility = [iced/accessibility]` | — |
| `hexedit` | `advanced`, `tokio`, `wgpu`, `canvas`, `lazy` | `default = [lua]` (`lua = [dep:mlua]`) | `accessibility` |
| `dispel-gui` | `tokio`, `wgpu`, `image`, `canvas`, `advanced`, `lazy`, `x11`, `wayland`, `accessibility` | `default = []`, `iced_test`, `debug` | `accessibility` |
| `dispel-extractor` (root) | — (no iced dependency) | — | — |

Notes:

- **Accessibility is default-on and force-enabled by both GUI consumers**, so
  the shared `table_widget` compiles with one shape workspace-wide. Turning it
  off in one crate only would not work: feature unification would turn it back
  on for everyone, producing unexpected build differences. Only `table_widget`
  implements full accesskit nodes today; the other custom widgets use iced's
  no-op defaults.
- `dispel-gui/debug = ["iced/hot"]` enables the fork's experimental hot
  reload. `cargo test --workspace --all-features` therefore compiles hot
  reload *and* `iced_test` together.
- `dispel-gui/iced_test = ["dep:iced_test", "iced/tiny-skia"]` and hexedit's
  `iced_renderer` dev-dep (`features = ["tiny-skia"]`) both make the software
  renderer available; either path is enough for feature unification, but both
  are kept so each crate's tests work standalone.

## 3. Test backend

`iced_test` reads the **`ICED_TEST_BACKEND`** environment variable at runtime
(iced fork `test/src/simulator.rs` and `test/src/emulator.rs`). The value
`software` selects the `tiny-skia` renderer; an unset value falls back to the
`wgpu` renderer, which needs a GPU/window server and fails in headless CI.

Consequences:

- Every iced UI-simulation test target sets `ICED_TEST_BACKEND=software` in the
  Makefile (`iced_test`, `hexedit_test`). Use those targets, or export the
  variable yourself when running `cargo test -p dispel-gui -p hexedit`.
- `tiny-skia` must also be compiled in (see the feature matrix above); the env
  variable alone is not enough.
- `cargo test --workspace --all-features` enables `iced_test` and hot reload
  together. It compiles, but GPU-less environments must still export
  `ICED_TEST_BACKEND=software` for the UI-simulation tests to run.

## 4. Validation

Run locally:

```bash
make check_iced_config        # or: bash scripts/check-iced-config.sh
```

The script checks: one iced rev in all manifests and in `Cargo.lock`; the
expected winit patch rev; the `ICED_TEST_BACKEND=software` settings in the
Makefile; `tiny-skia` compiled in on both iced_test paths; and consistent
accessibility configuration. It runs as a CI step in both `pr.yml` (verify job)
and `test.yml` (master push), in addition to the existing build/test/clippy
steps — it does not replace or weaken them.
