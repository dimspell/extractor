# AGENTS.md — Dispel Game File Extractor

**A game modding toolkit for Dispel RPG.** Reads/writes game files in native binary, INI, and DB formats. All edits are persisted directly — no intermediate database. Includes a full desktop GUI for editing game data with undo/redo, mod packaging, and a hex editor.

---

## Project Overview

This workspace is a **5-crate Cargo workspace** (Rust 2021, edition 2021) versioned `0.7.1`:

| Crate | Type | Role |
|---|---|---|
| `dispel-extractor` (root) | lib + bin | `dispel-core` library + CLI binary. Game logic only. |
| `dispel-gui` | bin only | Iced 0.14 desktop GUI. No `lib.rs` — pure binary crate. |
| `dispel-macros` | proc-macro | 5 derive macros used by `dispel-core`. |
| `gui-widgets` | lib | Reusable custom Iced widgets (ContextMenu, modal, ParagraphCache). |
| `hexedit` | lib + bin | Full hex editor with Lua scripting — used by `dispel-gui` as a tab type. |

**Separation of concerns:**
- `dispel-core` + `dispel-macros`: game logic only. Zero GUI/presentation code.
- `dispel-gui` + `gui-widgets` + `hexedit`: UI only. Zero game logic (they consume `dispel-core`).
- `dispel-extractor` (root `main.rs`): thin CLI wrapper around the library.

---

## Workspace Structure

```
dispel-extractor/
├── src/                      # dispel-core library
│   ├── lib.rs                # Public re-exports
│   ├── main.rs               # CLI binary entry
│   ├── cli.rs                # clap subcommand definitions
│   ├── database.rs           # SQLite schema bootstrap
│   ├── localization.rs       # Text export/import, Localizable trait
│   ├── sprite.rs             # .spr parser
│   ├── snf.rs                # .snf audio parser
│   ├── references/           # 30 game data file parsers
│   ├── map/                  # .map parser + isometric renderer
│   ├── modding/              # Mod authoring pipeline
│   ├── commands/             # CLI command implementations
│   └── queries/              # 70 SQL files (schema DDL + INSERTs)
├── dispel-macros/            # proc-macro crate
│   └── src/                  # 5 derive macros
├── gui-widgets/              # Iced widgets library
│   └── src/components/       # ContextMenu, modal, ParagraphCache
├── hexedit/                  # Hex editor library + standalone bin
│   └── src/                  # 20+ modules incl. Lua scripting
├── dispel-gui/               # Iced 0.14 desktop GUI
│   └── src/
│       ├── app.rs            # App (transient UI state)
│       ├── state.rs          # AppState (persistent model)
│       ├── workspace.rs      # Tabs, EditorType enum (52 variants)
│       ├── editor_registry.rs # All 35+ editor states
│       ├── platform.rs       # OS-specific ops
│       ├── auto_save.rs      # DraftManager (persists to ~/.config)
│       ├── style.rs          # Iced style functions (~1430 lines, "Medieval" theme)
│       ├── editors/          # 38 editor subdirectories
│       ├── components/       # Reusable UI components
│       ├── message/          # Message enum hierarchy + macros
│       ├── update/           # Message handlers (domain logic)
│       ├── view/             # Pure render functions
│       ├── workspace/        # Workspace tests
│       ├── indexation/       # File/asset caching + full-text search
│       └── recording_tests.rs # Mod recording integration tests
├── tests/                    # Integration tests (round_trip.rs, etc.)
├── docs/                     # 4 overview .md + docs/files/ (32 per-format .md)
├── fixtures/                 # Test game files (Dispel/, save data, etc.)
├── scripts/release.sh        # Automated release script
├── Makefile                  # Dev/extract targets
└── .github/workflows/        # CI: test.yml (build only), release.yml (5 platforms)
```

---

## Game Data Model

### File Formats

| Ext | Category | Encoding | Purpose |
|-----|----------|----------|---------|
| `.db` | Database | Binary + text | Items, monsters, magic, stats |
| `.ini` | Config | EUC-KR or 1250 | Maps, NPCs, monsters, visuals |
| `.ref` | Placement | 1250 | Entity instances on maps |
| `.dlg` / `.pgp` | Dialogue | EUC-KR / 1250 | Scripts + text |
| `.scr` | Script | 1250 | Quests, messages |
| `.map` | Geometry | Binary | Tiles, sprites, events |
| `.gtl` / `.btl` | Tilesets | RGB 565 | Ground/roof tiles (32×32) |
| `.spr` | Sprites | RGB 565 | Character animations |
| `.snf` | Audio | PCM | Sound effects |

**Encoding is critical:** `Event.ini`, `Npc.ini`, `Monster.db` (desc) → **EUC-KR**. `Monster.ini`, `Store.db`, `WeaponItem.db` → **WINDOWS-1250**. Per-file encoding documented in `docs/files/CROSS_REFERENCES.md`.

### Item Type Enum
| Value | Type | Database |
|-------|------|----------|
| 1 | Weapon | WeaponItem.db |
| 2 | Heal | HealItem.db |
| 3 | Edit | EditItem.db |
| 4 | Event | EventItem.db |
| 5 | Misc | MiscItem.db |

### File Dependencies
- **AllMap.ini** → Map.ini (per-map config) → Map.map (geometry, sprites, events)
- **Monster.ini** (visual) + **Monster.db** (combat stats)
- **Npc.ini** (visual) + **DlgMapFiles.dlg** (dialogue) + **PgpMapFiles.pgp** (text)
- See `docs/files/CROSS_REFERENCES.md` for the full 321-row cross-reference table and 9 relationship diagrams.

### Game Data Structs (all implement `Extractor` + `Localizable`)

**Binary DB records** (in `src/references/`):
`WeaponItem`, `HealItem`, `EditItem`, `EventItem`, `MiscItem`, `Monster`, `MagicSpell`, `Store`, `ChData`, `PartyIniNpc`, `PartyLevelNpc` (+ `PartyLevelRecord`)

**Binary ref records:**
`MonsterRef` (Mondun*.ref), `NPC` (Npccat*.ref, 672 bytes), `ExtraRef` (Extdun*.ref)

**INI/CSV/Text records:**
`Map`, `MapIni`, `Event`, `Extra`, `MonsterIni`, `NpcIni`, `WaveIni`, `Message`, `Quest`, `EventNpcRef`, `PartyRef`, `DrawItem`, `EventScript`, `DialogueScript`, `DialogueParagraph`

---

## Binary Format Essentials

- **All integers**: little-endian
- **Colors**: RGB 565 (16-bit). Helpers: `sprite::Color::from_rgb565()` / `to_rgb565()`
- **Map tiles**: seeked from EOF (event/GTL/BTL layers)
- **Sprites**: start at byte 268; validation: `ints[11] * ints[12] == ints[13]`
- **Tilesets**: contiguous 32×32 tiles, RGB 565 (2 bytes/pixel)
- **Map rendering**: isometric projection, 62×32 display size. **Warning:** full map ≈ 300MB; GUI uses viewport + LRU cache (≤50MB via `lru` 0.12).
- **SNF audio**: custom PCM header; prepend RIFF WAVE header for conversion (`SnfFile::to_wav_bytes()`)
- **DB record sizes** (all fixed): WeaponItem 284B, Monster 160B, Magic 88B, HealItem 252B, EditItem 268B, EventItem 240B, MiscItem 256B, ChData 84B, Store 948B (variable product list), PartyIniNpc 28B, PartyLevelNpc 5760B (8×20 grid), NPC 672B, MonsterRef 56B, ExtraRef 184B.

See `docs/file_formats.md` and per-file `docs/files/*.md` for detailed specs.

---

## dispel-core Architecture

### Top-level modules (declared in `src/lib.rs`)

```rust
pub mod database;        // SQLite schema bootstrap
pub mod localization;    // TextEncoding, Localizable trait, CSV/PO export
pub mod map;             // .map parser + isometric renderer
pub mod modding;         // Mod authoring pipeline
pub mod references;      // 30 game data file parsers
pub mod snf;             // .snf audio parser
pub mod sprite;          // .spr sprite parser
```

### Key traits

| Trait | File | Purpose |
|---|---|---|
| `Extractor` | `references/extractor.rs` | `parse(reader, len) -> Vec<Self>`, `to_writer()`, `read_file`, `save_file`. **All 28+ game data structs implement this.** |
| `Localizable` | `localization.rs` | Extract/apply translatable text fields. Auto-derived via `#[derive(Localizable)]`. |
| `RecordPatcher` | `modding/patcher.rs` | Applies mod patches to individual records. Two derive variants (binary + text). |
| `Command` | `commands/mod.rs` | CLI command trait: `fn execute(&self) -> Result<(), Box<dyn Error>>` |

### Modding pipeline (`src/modding/`)

14 files: `apply.rs` (`apply_all`, `revert_to_vanilla`, `ApplyReport`), `bsdiff.rs` (binary delta), `change.rs` (`ChangeOp`, `ChangeAction`, `BlobKind`), `changelog.rs` (`ChangeLog`, 50-entry cap), `conflicts.rs` (detection), `manifest.rs` (`ModManifest`, `MANIFEST_VERSION`), `package.rs` (ZIP read/write), `patcher.rs` (`RecordPatcher`), `patchers/*.rs` (per-type patcher impls), `registry.rs` (`PatcherRegistry`), `resolution.rs` (`FieldKey`, `ResolutionMap`), `value.rs` (`Value` enum), `vanilla.rs` (`VanillaStore`), `workspace.rs` (`InstalledMod`, `Workspace`).

### Error types
- `ModdingError` (thiserror, in `modding/error.rs`): `Io`, `Zip`, `Json`, `Malformed(String)`, `MissingEntry(String)`, `UnsupportedManifestVersion(u32)`
- Other modules use `std::io::Result` / `rusqlite::Result` directly.

### Encoding helpers
- `read_null_terminated_windows_1250(bytes)` — in `references/extractor.rs`
- `TextEncoding` enum (`Windows1250`, `EucKr`, `Utf8`) + `encoding_rs_for()`, `encoded_len()` in `localization.rs`
- `WINDOWS_1250` / `EUC_KR` used directly from `encoding_rs` crate

### SQLite schema (`src/queries/`)
70 SQL files: 37 `create_table_*.sql` (schema DDL) + 33 parameterized `insert_*.sql`. Run by `database.rs` to drop and recreate the schema.

### Tests
**55+ `#[cfg(test)]` blocks** across the codebase. Every parser has `parse_single_*` / `parse_two_*` / `serialize_round_trip` tests. Plus integration tests in workspace `tests/` (round_trip, integration_weapon_item).

---

## dispel-macros — Proc-Macro Crate

5 derive macros using `syn 2` + `quote 1`:

| Macro | Generates | Use for |
|---|---|---|
| `#[derive(Extractor)]` | `Extractor` impl (binary) | Binary `.db` / `.ref` structs |
| `#[derive(TextExtractor)]` | `Extractor` impl (text) | INI / CSV / `.scr` structs |
| `#[derive(RecordPatcher)]` | `RecordPatcher` impl (binary) + unit struct | Modding for binary formats |
| `#[derive(TextRecordPatcher)]` | `RecordPatcher` impl (text) + unit struct | Modding for text formats |
| `#[derive(Localizable)]` | `Localizable` impl | Translation extraction |

**Attributes:** `#[extractor(id, string, primitive, enum_from_*, padding, array, skip, ...)]` on binary; `#[extractor(field=N, parse_null, enum_from_i32)]` + struct-level `encoding, delimiter, comment_char` on text; `#[patcher(filename=...)]` or `#[patcher(extension=..., stem_prefix=...)]`; `#[translatable(encoding=..., max_bytes=N)]`.

**Used by 26+ structs in `src/references/`.** No direct unit tests — verified indirectly via round-trip tests.

---

## gui-widgets — Reusable Iced Widgets

| Widget | Location | Purpose |
|---|---|---|
| `ContextMenu<M>` | `components/context_menu/` | Right-click overlay; uses native AppKit/Win32 menus where possible (mac/Windows) |
| `modal()` + `Modal<M,T,R>` | `components/modal.rs` | Backdrop overlay with escape/click-outside dismiss, 3 unit tests |
| `ParagraphCache` | `components/paragraph_cache.rs` | LRU-cached Iced `Paragraph` (16K entries, ~16MB), 5 unit tests |
| `style::*` | `style.rs` | 6 stylesheet functions: `context_menu`, `menu_item`, `menu_separator`, `menu_disabled_item/text` (dark RPG leather theme) |

**Consumers:** `dispel-gui` (file tree, tab bar, spreadsheet, editor modals) and `hexedit` (hex matrix, inspector, modals).

---

## hexedit — Hex Editor Crate

Standalone library + 2 binaries (`hexedit-bin`, `bin/hexedit`). 20+ source files. Iced 0.14 with optional `lua` feature (default on, Lua 5.4 vendored via `mlua`).

| Module | Purpose |
|---|---|
| `state.rs` | `HexEditorState` — selection, editing, vanilla_diff, patterns, goto, search, lua engine, paragraph cache |
| `config.rs` | `HexEditorConfig` — `OnSaveFn`, `save_label`, `can_save`, `save_hint`, `extra_entries` |
| `message.rs` | 40+ `HexEditorMessage` variants (navigation, inline edit, inspector, save, patterns, goto, search) |
| `update.rs` / `view/mod.rs` | Pure `update(state, config, msg) -> Task<HM>` + `view(state, config) -> Element` |
| `provider.rs` | `HexProvider` trait + `BufferProvider` — byte source abstraction with dirty tracking |
| `selection.rs` | `Selection { anchor, cursor }`, `NavDir`, `nav_target()` navigation pure function |
| `editing.rs` | `EditState` (hex digit editing), `InspectorEditState` |
| `inspector.rs` | Built-in decoders (u8/u16/u32/i32/float/hex/ascii) + `InspectorEntry` |
| `pattern.rs` | `Pattern { id, start, end, color_idx }` — byte range highlights |
| `search.rs` | `SearchState` + `SearchMode` — hex/ASCII search with match iteration |
| `goto.rs` | `GotoState` — hex/decimal/offset parse |
| `coloring.rs` | `CellColorProvider` trait — custom byte-coloring strategy |
| `vanilla_diff.rs` | `compute_diff()` — BTreeSet of addresses differing from original |
| `lua_engine.rs` | `LuaScriptEngine` — custom inspector decoders via Lua scripting |
| `view/{matrix,inspector,inspector_modal,goto_modal,search_overlay,patterns,footer}.rs` | View components |

**Heavily tested:** 11 modules have unit tests (provider, selection, editing, goto, inspector, layout, coloring, vanilla_diff, search, view/matrix, view/footer).

**Used by `dispel-gui`:** stored as `HashMap<usize, HexEditorState>` in `EditorRegistry`, rendered in tabs, with save wired to mod packager via `OnSaveFn`.

---

## dispel-gui Architecture

### Tech Stack
- **UI**: Iced 0.14 (GPU via wgpu, Elm/MVU, advanced + lazy features)
- **Async**: Tokio multi-thread runtime + `iced::Task::perform`
- **Core**: `dispel_core` (sibling crate)
- **Search**: nucleo-matcher 0.3 (fuzzy full-text)
- **SQLite**: rusqlite 0.39 (bundled, for `DbViewer` only)
- **Audio**: rodio 0.21 (SNF playback)
- **File dialogs**: rfd 0.17.2
- **Window**: 1100×800, custom "Medieval" theme (dark leather/gold)
- **macOS native**: objc2 + objc2-app-kit + objc2-foundation (context menus, file manager reveal)
- **Tests**: `iced_test` 0.14 (optional feature), `proptest`, `syn`

### Flow
`user action → Message → update/ handler (mutates App + AppState) → view/ (pure render)`

### App vs AppState Separation

| Lives on `App` (transient UI) | Lives on `AppState` (persistent model) |
|---|---|
| `file_tree` (UI tree) | `editors: EditorRegistry` (all 35+ editor states) |
| `window_id`, `app_mode` | `workspace: Workspace` (tabs, game path, recent) |
| `search_index` (full-text) | `status_msg`, `shared_game_path` |
| `draft_manager` | `is_running`, `recording: Option<RecordingSession>` |
| `command_palette`, `error_dialog` | `pane_state` (sidebar/main/history) |
| `is_indexing` | `lookups: HashMap<String, Vec<(String, String)>>` |
| | `global_search`, `file_index_cache_manager` |
| | `recent_files` (last 10) |

**Rule of thumb:** Transient UI state → `App`. Persistent game/model state → `AppState`.

### Message Routing

Top-level `Message` enum (`message/mod.rs:21`) has 6 variants:

| Variant | Routes to | Contents |
|---|---|---|
| `Workspace(InternalWorkspaceMessage)` | `update/workspace.rs` | Tab bar, sidebar, command palette, global search, pane grid, tool tabs |
| `Editor(EditorMessage)` | `update/editor/mod.rs` | 35+ per-editor messages |
| `FileTree(FileTreeMessage)` | `update/file_tree.rs` | File tree actions |
| `Viewer(ViewerMessage)` | (DbViewer handler) | SQLite DB viewer queries, pagination, CSV export |
| `System(SystemMessage)` | `update/system.rs` | Undo, Redo, Save, index, drafts, errors, file scan |
| `StartPage(StartPageMessage)` | `update/startpage.rs` | Game path selection |

**`MessageExt` trait** (`message/ext.rs`, generated by `define_message_ext!` macro) provides shorthand constructors like `Message::weapon(WeaponEditorMessage::LoadCatalog)`.

### Editor System — 38 editor types

The `EditorType` enum (`workspace.rs:9`) has 52 named variants + `Unknown` (with `#[serde(other)]`).

**Standard editors (8)** — macro-generated via `define_standard_editor!`:
`weapon`, `monster`, `heal_item`, `misc_item`, `edit_item`, `event_item`, `party_level_db_level`, `party_ref`

**Custom boxed editors (17)** — custom state in `Box<...>` (all listed in `EditorRegistry`):
`monster_ini`, `npc_ini`, `magic`, `store`, `party_ini`, `all_map_ini`, `draw_item`, `event_ini`, `event_npc_ref`, `extra_ini`, `map_ini`, `message_scr`, `quest_scr`, `event_scr`, `wave_ini`, `chdata`, `party_level_db`

**Tabbed editors (5)** — `TabbedEditor<T>` (HashMap<tab_id, MultiFileEditorState<T>>):
`monster_ref`, `dialogue_script`, `dialogue_paragraph`, `extra_ref`, `npc_ref`

**Single-instance special editors (4):**
`viewer` (DbViewer), `chest_editor` (filtered ExtraRef), `mod_packager_editor` (ModPackagerState), `localization_manager`

**Per-tab HashMap editors (5)** — one state per open tab:
`sprite_viewers`, `map_editors`, `tileset_editors`, `snf_editors`, `hex_editors`

### `define_standard_editor!` macro

Generates from a single declaration:
```rust
define_standard_editor! {
    name: weapon,
    name_pascal: Weapon,
    record: dispel_core::WeaponItem,
    field: weapon_editor,
    file: "CharacterInGame/weaponItem.db",
}
```
Produces: `pub type WeaponEditorState = StandardEditor<WeaponItem>`, `pub type WeaponEditorMessage = StandardEditorMessage<WeaponItem>`, `pub fn handle(msg, app) -> Task<Message>`, `pub fn view(app) -> Element<Message>`.

**Key files:** `components/standard/macros.rs`, `components/standard/state.rs`, `components/standard/update.rs`, `components/standard/message.rs`.

### `EditorRegistry` (`editor_registry.rs`)

Aggregates all editor state fields. Provides consolidated lifecycle:

| Method | Role |
|---|---|
| `remove_tab(tab_id)` | Cleans up per-tab editors (map, tileset, hex, SNF, sprite, tabbed) on tab close |
| `close_all_tabs()` | Resets only HashMap-based editors (preserves single-instance Box editors) |
| `clear_all()` | Resets **every** editor to default (workspace change) |
| `undo_active(et, tab_id, lookups)` | Delegates to `EditHistory::undo` for active editor (uses `undo_redo_dispatch!` macro) |
| `redo_active(et, tab_id, lookups)` | Delegates to `EditHistory::redo` |
| `refresh_spreadsheet(et, tab_id, lookups)` | Recomputes spreadsheet caches after undo/redo |
| `get_active_edit_history(et, tab_id)` | Returns `Option<&EditHistory>` for the active editor |
| `stop_snf_playback()` | Stops any playing SNF audio |

**Principle:** `AppState` exposes only pass-through methods — all real work lives in `EditorRegistry`.

### Workspace & Tab Management (`workspace.rs`)

```rust
pub struct Workspace {
    pub tabs: Vec<WorkspaceTab>,          // Dynamic tab list
    pub active_tab: Option<usize>,
    pub next_id: usize,                   // Monotonic tab ID counter
    pub game_path: Option<PathBuf>,
    pub recent_files: Vec<PathBuf>,
    pub last_reindexed_at: Option<u64>,
    pub recent_game_paths: Vec<PathBuf>,  // Max 5
}
```
`WorkspaceTab { id, label, path, editor_type, modified, pinned }`. Methods: `open()`, `open_with_editor_type()`, `open_tool()`, `close()`, `mark_modified()`, `clear_all_tabs()`, `save()/load()` (JSON to `~/.config/dispel-gui/workspace.json`), `validate_timestamp()`, `debug_info()`. `EditorType::from_path()` auto-detects editor from file extension.

### Pane Grid Layout
3-pane layout (sidebar | main content | history panel) using `pane_grid::State<PaneContent>`. `PaneContent` enum: `Sidebar | MainContent | HistoryPanel`. `PaneState` tracks focus and maximized.

### Subscriptions
- Keyboard listeners: Ctrl+Z/Y/S/H/P/F/W, Shift+X
- Sprite animation tick (16ms)
- SNF playback poll (250ms)
- Event script indexing poll (100ms)
- Spreadsheet navigation (arrow keys)

### View Functions

| Function | Location | Renders |
|---|---|---|
| `App::view()` | `view/mod.rs:30` | Top-level dispatch: start page or editor view |
| `App::view_start_page()` | `view/start_page.rs:57` | Game path selection card with recent paths |
| `App::view_editor()` | `view/mod.rs:37` | Full pane grid (sidebar + tab bar + editor + history) |
| `App::view_sidebar()` | `view/mod.rs:354` | File tree + tools section |
| `App::view_recent_files()` | `view/mod.rs:425` | Recent files list (10 most recent) |
| `view_history_panel()` | `view/history_panel.rs:6` | Undo/redo stack display |
| Per-editor `view()` | `editors/*/view.rs` | Editor-specific content (38 editors) |

### Mod Recording

`RecordingSession` lives on `AppState`. Every `FieldChanged` handler calls `observe_field_change()` in `editors/mod_packager/recording/`. Records are debounced via `PendingEdit` / `RecordingKey` per field edit, then flushed to the mod changelog on save. 12 integration tests in `recording_tests.rs` cover macro-generated, custom-wrapper, fully-custom, and tab-based editor types.

### Auto-Save Drafts

`DraftManager` (`auto_save.rs`) persists in-progress edits to `~/.config/dispel-gui/drafts.json`. Detects conflicts on app start, allows user to apply or discard. `SystemMessage::Draft*` variants drive the flow.

### Indexation & Search

`indexation/` module:
- `file_index_cache.rs` — `FileIndexCache` + `FileIndexCacheManager` (persistent bincode on disk)
- `indexation_service.rs` — `IndexationService` background file scanning
- `search_index.rs` — `SearchIndex` nucleo-matcher full-text search with persistence
- `tests.rs` — Indexation tests

### Tests

**Writing effective tests:** The goal is finding real bugs, not just confirming compilation. Prefer integration tests (iced_test) that simulate real user flows — they catch behavioral regressions, state corruption, and edge cases that unit tests miss.

| Location | Coverage |
|---|---|
| `workspace/tests.rs` | 40+ iced_test integration tests: open/close tabs, `EditorType::from_path()` for every extension, serialization, timestamps, `clear_editor_states()` |
| `update/tests.rs` + `update/system.rs` inline | System message handlers: `ClearWorkspace`, `Undo`/`Redo` on weapon editor, edge cases |
| `recording_tests.rs` | 12 iced_test integration tests: `observe_field_change`, weapon, wave_ini, store, npc_ref, chest (known gap) |
| `components/generic_editor/mod.rs` inline | Undo/redo |
| `message/ext.rs` inline | Message constructor tests |
| `indexation/tests.rs` | Indexation |
| `components/field_coverage.rs` | Verifies all fields have coverage |
| Workspace `tests/round_trip.rs` + `tests/round_trip/*.rs` (31 files) | Read → parse → write → byte-for-byte verify |
| `tests/integration_weapon_item.rs` | End-to-end load → edit → save |

### Adding a New Editor

1. Implement `Extractor` (derive via `dispel-macros`) in `dispel-core/src/references/`
2. If single-file spreadsheet: use `define_standard_editor!` macro in `dispel-gui/src/components/standard/macros.rs` declaration
3. Otherwise: create `editors/my_editor/{mod,state,message,update,view}.rs`
4. Add field to `EditorRegistry` (not `AppState` — editors are consolidated)
5. Add `EditorMessage` variant in `message/editor/mod.rs` (use `define_message_ext!` macro for shorthand)
6. Add handler in `update/editor/` and route via `update/editor/mod.rs`
7. Wire `EditorType::from_path()` for file extension detection (in `workspace.rs`)
8. Wire `view/mod.rs` dispatch (the `view_editor` match)
9. Wire `EditorRegistry` lifecycle methods (`remove_tab`, `undo_active`, etc.)
10. Delete old editor code — never keep both

### Naming Glossary

| Term | Meaning |
|------|---------|
| `catalog` / `catan` | The in-memory `Vec<R>` of all loaded records — both spellings appear (legacy typo) |
| `filtered` | Subset of catalog matching current search query |
| `edit_buffers` | Per-field `String` buffers for in-progress edits (not committed to catalog) |
| `GenericEditorState<R>` | Core reusable editor: catalog, selection, edit_history |
| `StandardEditor<T>` | GenericEditorState + SpreadsheetState — used by 8 standard editors |
| `TabbedEditor<T>` | HashMap<tab_id, MultiFileEditorState<T>> + HashMap<tab_id, SpreadsheetState> |
| `MultiFileEditorState<R>` | Like GenericEditorState but with active-file tracking for multi-file formats |

### Conventions

- **Async**: Use `iced::Task` + Tokio. Never block the UI thread.
- **State mutation**: `update/` only. Views are pure.
- **Views**: defined as `impl App` blocks in `view/*.rs`, not `app.rs`.
- **Messages**: use `define_message_ext!` macro for shorthand constructors.
- **Scan vs Browse**: "Scan" = load from `shared_game_path`; "Browse" = file picker.
- **SQLite**: only `DbViewer`. Other editors read/write game files directly.
- **Enums over booleans**: e.g. `LoadingState<T> { Idle, Loading, Loaded(T), Failed(String) }`
- **Editor state lives on `EditorRegistry`, not `AppState`**.

### Iced 0.14 Lessons

| Problem | Fix |
|---------|-----|
| Views in both `app.rs` and `view/mod.rs` | Keep only in `view/*.rs` |
| `center_x()` compile error | Use `container(...).align_x(Horizontal::Center)` |
| Deprecated patterns | Check Iced 0.14 docs |

---

## Documentation

| Path | Contents |
|---|---|
| `docs/overview.md` | High-level project intro (a bit dated — old module layout) |
| `docs/file_formats.md` | Binary format specs for SNF, map, tileset, sprite |
| `docs/rendering.md` | Isometric map rendering: coordinate transforms, layer ordering, painter's algorithm, sprite transparency |
| `docs/database_and_references.md` | SQLite schema overview |
| `docs/files/*.md` | **32 per-format docs** (one per game file type) + `CROSS_REFERENCES.md` (321-row cross-reference table, 9 dependency diagrams) |

Always read the relevant `docs/files/*.md` before modifying a parser.

## Build System & CI

### Makefile targets
- `fmt`, `cargo_test`, `clippy`, `run`, `help` — basic dev
- `iced_test` — `cargo test -p dispel-gui --features "iced_test app::tests"` (equivalent: `rtk cargo test -p dispel-gui --features "iced_test app::tests"`)
- `sound`, `sprite-sprite`, `sprite-animation`, `map-render`, `map-atlas-gtl/btl` — extraction/render tests
- `extract-*` (18 targets) — extract individual game files to JSON
- `database-import` — SQLite import

### GitHub Actions
- `.github/workflows/test.yml` — CI on push to `master`: `cargo build` on ubuntu-latest (tests NOT run in CI)
- `.github/workflows/release.yml` — Release on tag `v*.*.*`: 5 builds (CLI Win/Linux, GUI Win/Linux/macOS ARM), archives with SHA256, uploads to GitHub release (draft)

### Release script
`scripts/release.sh` automates: validate semver, master branch, clean tree → `cargo fmt --all --check` + `cargo test --workspace --all-features` (script uses bare commands internally) → bump version in all workspace `Cargo.toml` → regen `Cargo.lock` → commit + tag `v{major}.{minor}.{patch}` → print push instructions.

---

## Development Best Practices

### Error Handling
- `dispel-core`: `thiserror` for enumerable error types (see `ModdingError`)
- GUI/CLI: `anyhow` for contextual bubbling
- Never `.unwrap()` on file I/O — show errors in `status_msg`
- **Never swallow I/O or serialization errors** with `let _ =` or `.unwrap_or_default()` — log them via `eprintln!` or propagate with `?`

### State & Async
- Use **enums over booleans**: `LoadingState<T> { Idle, Loading, Loaded(T), Failed(String) }`
- Always use `Task::perform` for async work — never block

### Code Quality
- Clippy: zero warnings (`rtk cargo clippy --workspace --all-features --test -- -D warnings`)
- Compiler warnings: use `rtk cargo test --workspace --all-features --no-run` to compile all crates (including tests) and surface any compiler warnings without waiting for tests to execute — useful as a fast pre-commit check
- Format: `cargo fmt --all` before commit
- Validate all binary bounds before indexing

### Testing
**Writing effective tests:** The goal is finding real bugs, not just confirming compilation. Prefer integration tests (iced_test) that simulate real user flows — they catch behavioral regressions, state corruption, and edge cases that unit tests miss.

- **dispel-core**: unit test every new parser with hardcoded byte slices
- **dispel-gui**: write iced_test integration tests that simulate real user flows (open tabs, edit data, undo/redo, save). Test state transitions and behaviors, not visuals.
- **Round-trip tests** (`tests/round_trip.rs`): read fixture → parse → write → verify byte-for-byte match
- **Integration tests** (`tests/integration_weapon_item.rs`): end-to-end load → edit → save workflow
- **Test naming**: `test_${scenario}_${condition}` e.g. `test_undo_weapon_editor_empty_history`
- **Test helpers**: tests in `$module/tests.rs` (co-located), separate `recording_tests.rs` for recording
- **Property tests**: use `proptest` for invariants
- Run before every commit: `rtk cargo test --workspace --all-features`

### Tools
- `cargo check --message-format=short`: fast compile errors
- **Always use `rtk` prefix** for cargo/git operations (see [rtk section](#rtk-rust-token-killer) above for the full list)
- `ripgrep` / `rg`: fast code search
- `fd`: faster than `find`
- `rust-analyzer`: essential LSP

---

## CLI Reference

```bash
# Extract INI/DB/reference files to JSON
cargo run -- extract -i "AllMap.ini"
cargo run -- extract -i "Monster.db"

# Sprites
cargo run -- sprite "file.spr" output_name

# Maps
cargo run -- map tiles "file.gtl" --output dir/
cargo run -- map render --map file.map --btl file.btl --gtl file.gtl --output out.png

# Audio
cargo run -- sound "file.snf" output.wav

# SQLite (optional)
cargo run -- database import "path/to/Dispel/" db.sqlite

# Modding (see src/commands/modding or src/modding/ for full surface)
cargo run -- mod-pack ...
```

Subcommands defined in `src/cli.rs`: extract, patch, validate, list, schema, sprite, sound, dialog, map, database, mod-pack, test.

---

## Legal Compliance

**✅ Permitted:**
- Analyzing file formats, documenting specs
- Creating modding/interoperability tools
- Using "Dispel" for identification only

**❌ Prohibited:**
- Extracting/distributing copyrighted content
- Bypassing copy protection
- Commercial exploitation
- Using DISPEL® trademark beyond identification

**When mentioning Dispel:** Use **DISPEL®** with ® symbol on first mention. Include disclaimer: "not affiliated with, endorsed by, or sponsored by the trademark owner."

---

## Common Pitfalls

- **Circular imports**: GUI ↔ core must never share presentation code
- **Blocking UI**: All file I/O is async via Task
- **Unsafe parsing**: Validate all bounds before indexing
- **Hardcoded paths**: Use `dirs` crate for config/cache (`~/.config/dispel-gui/`)
- **Text encoding**: Check encoding table before reading — wrong codec = corruption
- **Map memory**: Never load full rendered map (~300MB); use viewport + LRU cache
- **Macros have no unit tests**: `dispel-macros` is verified only via dependent crate's tests
- **Editor count changes**: AGENTS.md previously said 27 editors — actual is 39 (8 standard + 17 custom boxed + 5 tabbed + 4 single-instance + 5 per-tab HashMap)

---

## Quick Commands

```bash
cargo build --workspace                              # Build all
cargo test --workspace --all-features --quiet        # Test all
cargo test --workspace --all-features --no-run       # Compile all crates including tests (catches compiler warnings fast)
cargo clippy --workspace -- -D warnings              # Lint
cargo fmt --all                                      # Format
cargo check -p dispel-gui --message-format=short     # Fast GUI errors
cargo run -p dispel-gui                              # Launch GUI
make iced_test                                       # Iced UI simulation tests
```

### rtk (Rust Token Killer)

**Always use `rtk` instead of the bare command** when running these operations. It compresses output by 60-90%, saving significant context window for real work.

Note: the rtk hook only applies to Bash tool calls — built-in tools like `Read`, `Grep`, and `Glob` bypass it. For token-efficient file access, prefer shell commands through `rtk read`, `rtk grep`, `rtk ls` instead of the built-in tools.

```bash
rtk cargo test -v [cargo test args...]              # Tests with compact output (failures only, ~-90%)
rtk cargo build                                     # Build output filtered (~-80%)
rtk cargo clippy                                    # Clippy warnings grouped (~-80%)
rtk git status                                      # Compact status
rtk git diff                                        # Condensed diff
rtk git log -n 10                                   # One-line commits
rtk ls .                                            # Token-optimized directory tree
rtk grep "pattern" .                                # Grouped search results
rtk read file.rs                                    # Smart file reading
```

---

*Last updated: 2026-06-04*  
**DISPEL®** is a registered trademark. This project is **not affiliated with, endorsed by, or sponsored by** the trademark owner.
