# Forked Iced configuration: single pin set and feature matrix

The GUI builds against a git fork of Iced, `dimspell/iced`. This document records the pins, the features each crate enables and the renderer that headless tests select. `scripts/check-iced-config.sh` (also `make check_iced_config`) enforces these rules in CI.

## Single pin set

All iced-family git dependencies must resolve to the same commit:

| Dependency | Source | Rev |
|---|---|---|
| `iced` (workspace dep) | `https://github.com/dimspell/iced` | `c0ffdd8e7352df9283496f3db334731c00ef6d0d` |
| `iced_test` (workspace dep) | `https://github.com/dimspell/iced` | `c0ffdd8e7352df9283496f3db334731c00ef6d0d` |
| `iced_renderer` (hexedit **dev**-dep) | `https://github.com/dimspell/iced` | `c0ffdd8e7352df9283496f3db334731c00ef6d0d` |
| `winit` (`[patch.crates-io]`) | `https://github.com/iced-rs/winit.git` | `05b8ff17a06562f0a10bb46e6eaacbe2a95cb5ed` |

hexedit's `iced_renderer` dev-dependency uses the same commit as the workspace. A different commit (`48fb8fe...`) gave `Cargo.lock` two copies of each iced crate. The copies had incompatible internal types. `Cargo.lock` now has one `iced_renderer` entry. Do not add a second iced rev. Enable any needed feature through the workspace pin.

## Feature matrix

Cargo features are additive and unify across the dependency graph. The table lists the features each workspace crate turns on directly. Every crate sets `default-features = false` on `iced`.

| Crate | `iced` features (direct) | Own features | Forces on `gui-widgets` |
|---|---|---|---|
| `gui-widgets` | `advanced`, `thread-pool`, `wgpu` | `default = [accessibility]`, `accessibility = [iced/accessibility]` | none |
| `hexedit` | `advanced`, `tokio`, `wgpu`, `canvas`, `lazy` | none | `accessibility` |
| `dispel-gui` | `tokio`, `wgpu`, `image`, `canvas`, `advanced`, `lazy`, `x11`, `wayland`, `accessibility` | `default = []`, `iced_test`, `debug` | `accessibility` |
| `dispel-extractor` (root) | none (no iced dependency) | none | none |

Notes:

- Accessibility is on by default, and both GUI consumers force it on. The shared `table_widget` then compiles with one shape across the workspace. A single crate cannot turn accessibility off. Feature unification turns it back on for all crates and changes the build. Only `table_widget` implements full accesskit nodes. The other custom widgets use iced's no-op defaults.
- `dispel-gui/debug = ["iced/hot"]` enables the fork's experimental hot reload. So `cargo test --workspace --all-features` compiles hot reload and `iced_test` together.
- Three paths make the software renderer available. The first is `dispel-gui/iced_test = ["dep:iced_test", "iced/tiny-skia"]`. The second is hexedit's `iced_renderer` dev-dep (`features = ["tiny-skia"]`). The third is the same `gui-widgets` dev-dep. One path is enough for feature unification. Each path stays so the crate's tests work on their own.
- `gui-widgets` also enables `iced_test/accessibility` as a dev-dependency. Its widget tests can inspect accessibility trees and dispatch accesskit actions. This matches the crate's default-on `accessibility` feature.

## Test backend

`iced_test` reads the `ICED_TEST_BACKEND` environment variable at runtime. The fork handles this in `test/src/simulator.rs` and `test/src/emulator.rs`. The value `software` selects the `tiny-skia` renderer. An unset value falls back to the `wgpu` renderer. That renderer needs a GPU and a window server. It fails in headless CI.

Rules:

- Every iced UI-simulation test target sets `ICED_TEST_BACKEND=software` in the Makefile (`iced_test`, `hexedit_test`). Use those targets, or export the variable yourself when you run `cargo test -p dispel-gui -p hexedit -p gui-widgets`.
- You must also compile `tiny-skia` (see the feature matrix above). The env variable alone is not enough.
- `cargo test --workspace --all-features` enables `iced_test` and hot reload together. The command compiles. On a machine with no GPU, export `ICED_TEST_BACKEND=software` before the UI-simulation tests.

## Validation

Run locally:

```bash
make check_iced_config        # or: bash scripts/check-iced-config.sh
```

The script checks for one iced rev in all manifests and in `Cargo.lock`. It checks the expected winit patch rev, the `ICED_TEST_BACKEND=software` settings in the Makefile, `tiny-skia` on both `iced_test` paths, and consistent accessibility configuration. It runs as a CI step in both `pr.yml` (the `verify` job) and `test.yml` (master push). Those steps do not replace the build, test or clippy steps.
