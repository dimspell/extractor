# AGENTS.md — Dispel Game File Extractor

**A game modding toolkit for Dispel RPG.** Reads/writes game files in native binary, INI, and DB formats — including save games. All edits are persisted directly — no intermediate database. Includes a full desktop GUI for editing game data with undo/redo, mod packaging, and a hex editor.

---

## Project Overview

This workspace is a **5-crate Cargo workspace** (Rust edition 2024) versioned `0.9.3`:

| Crate | Type | Role |
|---|---|---|
| `dispel-extractor` (root) | lib + bin | `dispel-core` library + CLI binary. Game logic only. |
| `dispel-gui` | bin only | Iced desktop GUI (git fork of iced). No `lib.rs` — pure binary crate. |
| `dispel-macros` | proc-macro | 7 derive macros used by `dispel-core`. |
| `gui-widgets` | lib | Reusable custom Iced widgets (ContextMenu, modal, popover, toast, ParagraphCache…). |
| `hexedit` | lib + bin | Full hex editor with Lua scripting — used by `dispel-gui` as a tab type. Single binary: `hexedit`. |

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
│   ├── references/           # ~35 game data file parsers incl. save_file/, save_ifo/, layout.rs
│   ├── map/                  # .map parser + isometric renderer + TMX export (tmx.rs)
│   ├── modding/              # Mod authoring pipeline (+ patchers/ per-format record patchers)
│   ├── commands/             # CLI command implementations
│   └── queries/              # 90 SQL files (45 schema DDL + 45 INSERTs)
├── dispel-macros/            # proc-macro crate
│   └── src/                  # 7 derive macros
├── gui-widgets/              # Iced widgets library
│   └── src/components/       # ContextMenu, modal, popover, tab_bar, table_widget, text_area, toast, collapsible_tree, ParagraphCache
├── hexedit/                  # Hex editor library + standalone bin
│   └── src/                  # app/config/state/message/lua_engine + domain/ + ui/ + tests/
├── dispel-gui/               # Iced desktop GUI
│   └── src/
│       ├── app.rs            # App (transient UI state; embeds AppState)
│       ├── state.rs          # AppState (persistent model) + RecordingSession
│       ├── workspace.rs      # Tabs, EditorType enum (40 named variants + Unknown)
│       ├── editor_registry.rs # All ~46 editor states
│       ├── platform.rs       # OS-specific ops
│       ├── style.rs + style/ # Iced style functions ("Medieval" theme), split per component
│       ├── editors/          # ~38 editor subdirectories
│       ├── components/       # Reusable UI components
│       ├── message/          # Message enum hierarchy + macros
│       ├── update/           # Message handlers (domain logic)
│       ├── view/             # Pure render functions
│       ├── subscriptions.rs  # Keyboard/timer subscription wiring
│       ├── dispatch_table.rs # Editor message dispatch table
│       ├── binary_layout.rs  # Binary layout helpers
│       ├── indexation/       # File/asset caching + full-text search
│       ├── tests/            # Dedicated GUI test suite (27 test files)
│       └── recording_tests.rs # Mod recording integration tests (38 tests)
├── examples/                 # e.g. check_save_roundtrip.rs
├── docs/                     # Overview .md files + docs/files/ (36 per-format .md in subdirs)
├── fixtures/                 # Local-only, git-ignored test files (user-supplied; never committed)
├── scripts/release.sh        # Automated release script
├── Makefile                  # Dev/extract targets (~120 targets)
└── .github/workflows/        # CI: test.yml (build only), pr.yml (multi-OS PR matrix), release.yml (5 platforms)
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
| `.ifo` / save file | Save game | Binary | Player saves (`Save.ifo`, parsed by `save_ifo` / `save_file`) |

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
- See `docs/files/CROSS_REFERENCES.md` for the full cross-reference table and relationship diagrams.

### Game Data Structs (all implement `Extractor` + `Localizable`)

**Binary DB records** (in `src/references/`, derive-based):
`WeaponItem`, `HealItem`, `EditItem`, `EventItem`, `MiscItem`, `Monster`, `MagicSpell`, `Store`, `ChData`, `PartyIniNpc`

**Binary ref records:**
`MonsterRef` (Mondun*.ref), `NPC` (Npccat*.ref, 672 bytes), `ExtraRef` (Extdun*.ref)

**INI/CSV/Text records (derive-based):**
`Map`, `MapIni`, `Event`, `Extra`, `MonsterIni`, `NpcIni`, `WaveIni`, `Message`, `EventNpcRef`, `PartyRef`

**Manually implemented parsers:**
`PartyLevelNpc` (+ `PartyLevelRecord`), `DrawItem`, `Quest`, `EventScript`, `DialogueScript`, `DialogueParagraph`, **`SaveFile`**, **`SaveIfo`**

---

## Binary Format Essentials

- **All integers**: little-endian
- **Colors**: RGB 565 (16-bit). Helpers: `sprite::Color::from_rgb565()` / `to_rgb565()`
- **Map tiles**: seeked from EOF (event/GTL/BTL layers)
- **Sprites**: start at byte 268; validation: `ints[11] * ints[12] == ints[13]`
- **Tilesets**: contiguous 32×32 tiles, RGB 565 (2 bytes/pixel)
- **Map rendering**: isometric projection, 62×32 display size; Y-sorted draw order for walls/objects. **Warning:** full map ≈ 300MB; GUI uses viewport + LRU cache (≤50MB via `lru` 0.12).
- **TMX export**: `map::tmx::export_tmx()` converts `.map` + tilesets to Tiled TMX XML (isometric, CSV tile data)
- **SNF audio**: custom PCM header; prepend RIFF WAVE header for conversion (`SnfFile::to_wav_bytes()`)
- **DB record sizes** (all fixed): WeaponItem 284B, Monster 160B, Magic 88B, HealItem 252B, EditItem 268B, EventItem 240B, MiscItem 256B, ChData 84B, Store 948B (variable product list), PartyIniNpc 28B, PartyLevelNpc 5760B (8×20 grid), NPC 672B, MonsterRef 56B, ExtraRef 184B. Save records (`SaveFile`, `SaveIfo`) are variable-layout — see `docs/files/`.
- **Static layout metadata**: `references/layout.rs` defines `FieldDef` / `FixedRecordLayout` and the `RecordLayout` derive trait — generated byte-offset metadata for fixed-size records.

See `docs/file_formats.md` and per-file `docs/files/*.md` for detailed specs.

---

## dispel-core Architecture

### Key traits

| Trait | File | Purpose |
|---|---|---|
| `Extractor` | `references/extractor.rs` | `parse(reader, len) -> Vec<Self>`, `to_writer()`, `read_file`, `save_file`. Implemented by all game data structs (derive or manual). |
| `Localizable` | `localization.rs` | Extract/apply translatable text fields. Auto-derived via `#[derive(Localizable)]`. |
| `RecordLayout` | `references/layout.rs` | Static `FixedRecordLayout` metadata (field names/offsets/sizes) generated from an `Extractor` definition. |
| `RecordPatcher` | `modding/patcher.rs` | Applies mod patches to individual records. Two derive variants (binary + text). |
| `Command` | `commands/mod.rs` | CLI command trait: `fn execute(&self) -> Result<(), Box<dyn Error>>` |

### Modding pipeline (`src/modding/`)

Core modules: `apply`, `bsdiff`, `change`, `changelog`, `conflicts`, `manifest`, `package`, `patcher`, `registry`, `resolution`, `value`, `vanilla`, `workspace` (+ `error.rs`). Per-format record patchers live in `modding/patchers/`: `derived`, `dialogue`, `draw_item`, `event_script`, `misc_item`, `party_level`, `quest`, `save_ifo`, `store`.

### Error types
- `ModdingError` (thiserror, in `modding/error.rs`): `Io`, `Zip`, `Json`, `Malformed(String)`, `MissingEntry(String)`, `UnsupportedManifestVersion(u32)`
- Other modules use `std::io::Result` / `rusqlite::Result` directly (plus thiserror errors in `dispel-gui`'s indexation/file-tree modules).

### Encoding helpers
- `read_null_terminated_windows_1250(bytes)` — in `references/extractor.rs`
- `TextEncoding` enum (`Windows1250`, `EucKr`, `Utf8`) + `encoding_rs_for()`, `encoded_len()` in `localization.rs`
- `WINDOWS_1250` / `EUC_KR` used directly from `encoding_rs` crate

### SQLite schema (`src/queries/`)
90 SQL files: 45 `create_table_*.sql` (schema DDL) + 45 parameterized `insert_*.sql`. Run by `database.rs` to drop and recreate the schema. **No foreign keys** — relationships are enforced in code, not SQL constraints.

### Tests
**~70 `#[cfg(test)]` blocks across ~65 files** in the root crate. Parsers have hardcoded-byte-slice unit tests; save-file round-trip checking lives in `examples/check_save_roundtrip.rs`.

---

## dispel-macros — Proc-Macro Crate

7 derive macros using `syn 2` + `quote 1`:

| Macro | Generates | Use for |
|---|---|---|
| `#[derive(Extractor)]` | `Extractor` impl (binary) | Binary `.db` / `.ref` structs |
| `#[derive(TextExtractor)]` | `Extractor` impl (text) | INI / CSV / `.scr` structs |
| `#[derive(BinaryRecord)]` | Binary record decoding helper | Fixed-size binary records |
| `#[derive(RecordLayout)]` | Static `FixedRecordLayout` const | Byte-layout introspection of binary records |
| `#[derive(RecordPatcher)]` | `RecordPatcher` impl (binary) + unit struct | Modding for binary formats |
| `#[derive(TextRecordPatcher)]` | `RecordPatcher` impl (text) + unit struct | Modding for text formats |
| `#[derive(Localizable)]` | `Localizable` impl | Translation extraction |

**Attributes:** `#[extractor(id, string, primitive, enum_from_*, padding, array, skip, ...)]` on binary; `#[extractor(field=N, parse_null, enum_from_i32)]` + struct-level `encoding, delimiter, comment_char` on text; `#[patcher(filename=...)]` or `#[patcher(extension=..., stem_prefix=...)]`; `#[translatable(encoding=..., max_bytes=N)]`.

**Used by 26+ structs in `src/references/`.** No direct unit tests — verified indirectly via dependent crate's tests.

---

## gui-widgets — Reusable Iced Widgets

| Widget | Location | Purpose |
|---|---|---|
| `ContextMenu<M>` | `components/context_menu/` | Right-click overlay; uses native AppKit/Win32 menus where possible (mac/Windows) |
| `modal()` + `Modal<M,T,R>` | `components/modal.rs` | Backdrop overlay with escape/click-outside dismiss, 7 unit tests |
| `ParagraphCache` | `components/paragraph_cache.rs` | LRU-cached Iced `Paragraph` (16K entries, ~16MB), 12 unit tests |
| Popover | `components/popover/` | Anchored floating panels (e.g. map layer visibility) |
| Toast | `components/toast/` | Transient notifications (replacing status-bar messaging in some editors) |
| Tab bar / Table widget | `components/tab_bar/`, `components/table_widget/` | Reusable tab strip; accessible table (full accesskit node support) |
| Others | `collapsible_tree/`, `text_area` | Collapsible tree view, enhanced text input |
| `style::*` | `style.rs` | Stylesheet functions (dark RPG leather theme) |

Also: `sweeten/` (builder-style helpers) and `lucide.rs` (icon set).

**Consumers:** `dispel-gui` (file tree, tab bar, spreadsheet, editor modals) and `hexedit` (hex matrix, inspector, modals).

---

## hexedit — Hex Editor Crate

Standalone library + single binary (`hexedit`, `src/main.rs`). Rust edition 2024. Iced (fork) with optional `lua` feature (default on, Lua 5.4 vendored via `mlua`).

Structure:
- Top level: `app.rs`, `config.rs`, `state.rs`, `message.rs`, `lua_engine.rs`, `main.rs`
- `domain/` (~16 files): editing, goto, pattern, provider, search, selection, vanilla_diff, …
- `ui/`: coloring, inspector, update, view/, theme
- `tests/`: 17 dedicated test files (**24 files with `#[cfg(test)]` total**)

**Used by `dispel-gui`:** stored as `HashMap<usize, HexEditorState>` in `EditorRegistry`, rendered in tabs, with save wired to mod packager via `OnSaveFn`.

---

## dispel-gui Architecture

### Tech Stack
- **UI**: Iced from git fork `dimspell/iced` (rev `c0ffdd8`; GPU via wgpu, Elm/MVU, advanced + lazy features)
- **Async**: Tokio multi-thread runtime + `iced::Task::perform`
- **Core**: `dispel_core` (sibling crate)
- **Search**: nucleo-matcher 0.3 (fuzzy full-text)
- **SQLite**: rusqlite 0.39 (bundled, for `DbViewer` only)
- **Audio**: rodio 0.22 (SNF playback)
- **File dialogs**: rfd 0.16
- **Window**: 1100×800, custom "Medieval" theme (dark leather/gold)
- **macOS native**: objc2 + objc2-app-kit + objc2-foundation (context menus, file manager reveal)
- **Accessibility**: forked iced `accessibility` feature with vendored accesskit; enabled on `dispel-gui` only. The workspace `[patch.crates-io]` pins `winit` to an iced-rs fork for version alignment. Only `table_widget` implements full accesskit nodes (`accessibility()` + `accessibility_action()`). Custom widgets (context menu, modal, hex matrix) use the default no-op methods.
- **Tests**: `iced_test` (same git fork, optional feature), `proptest`, `syn`

### Flow
`user action → Message → update/ handler (mutates App + AppState) → view/ (pure render)`

### App vs AppState Separation

`App` embeds `AppState` directly (`app.state`). 

| Lives on `App` (transient UI) | Lives on `AppState` (persistent model) |
|---|---|
| `file_tree` (UI tree) | `editors: EditorRegistry` (all ~46 editor states) |
| `window_id`, `app_mode` | `workspace: Workspace` (tabs, game path, recent) |
| `search_index` (full-text) | `status_msg`, `shared_game_path` |
| `command_palette`, `error_dialog` | `is_running`, `recording: Option<RecordingSession>` |
| `history_panel_visible`, `sidebar_visible` | `pane_state` (sidebar/main/history) |
| `empty_edit_history`, `global_search` | `lookups: HashMap<String, Vec<(String, String)>>` |
| `start_page_input`, `is_indexing` | `file_index_cache_manager`, `recent_files` (last 10) |

**Rule of thumb:** Transient UI state → `App`. Persistent game/model state → `AppState`.

### Message Routing

Top-level `Message` enum (`message/mod.rs`) has 6 variants:

| Variant | Routes to | Contents |
|---|---|---|
| `Workspace(InternalWorkspaceMessage)` | `update/workspace.rs` | Tab bar, sidebar, command palette, global search, pane grid, tool tabs |
| `Editor(EditorMessage)` | `update/editor/mod.rs` | ~46 per-editor messages |
| `FileTree(FileTreeMessage)` | `update/file_tree.rs` | File tree actions |
| `Viewer(ViewerMessage)` | (DbViewer handler) | SQLite DB viewer queries, pagination, CSV export |
| `System(SystemMessage)` | `update/system.rs` | Undo, Redo, Save, index, errors, file scan |
| `StartPage(StartPageMessage)` | `update/startpage.rs` | Game path selection |

**`MessageExt` trait** (`message/ext.rs`, generated by `define_message_ext!` macro) provides shorthand constructors like `Message::weapon(WeaponEditorMessage::LoadCatalog)`.

### Editor System — ~46 editor types

The `EditorType` enum (`workspace.rs`) has 40 named variants + `Unknown` (with `#[serde(other)]`). Variants are suffixed `Editor` (e.g. `WeaponEditor`, `SaveIfoEditor`); new since v0.7: `SaveFileViewer`, `SaveIfoEditor`, `HexEditor`, `DialogueTextEditor`.

**Standard editors (20)** — macro-generated via `define_standard_editor!`, invoked in each editor's own `editors/<name>/mod.rs`:
`weapon`, `monster`, `heal_item`, `misc_item`, `edit_item`, `event_item`, `party_ref`, `all_map_ini`, `event_ini`, `draw_item`, `chdata`, `event_npc_ref`, `map_ini`, `npc_ini`, `message_scr`, `extra_ini`, `magic`, `quest_scr`, `monster_ini`, `party_ini`

**Custom boxed editors** — custom state in `Box<...>` (all listed in `EditorRegistry`):
`save_ifo_editor`, `event_scr`, `wave_ini`, `store`, `party_level_db`, `db_viewer` (viewer), `localization_manager`, …

**Tabbed editors (5)** — `TabbedEditor<T>` (HashMap<tab_id, MultiFileEditorState<T>>):
`monster_ref`, `dialogue_script`, `dialogue_paragraph`, `extra_ref`, `npc_ref`

**Single-instance special editors:** viewer (DbViewer), mod_packager_editor (ModPackagerState)

**Per-tab HashMap editors (6)** — one state per open tab:
`sprite_viewers`, `map_editors`, `tileset_editors`, `snf_editors`, `hex_editors`, `save_file_viewers`

### `define_standard_editor!` macro

Generates from a single declaration (now placed in each editor's `editors/<name>/mod.rs`):
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

**Key files:** `components/standard/{macros,state,update,message}.rs`.

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
    pub tabs: Vec<WorkspaceTab>,          // Dynamic tab list (#[serde(skip)])
    pub active_tab: Option<usize>,        // #[serde(skip)]
    pub next_id: usize,                   // Monotonic tab ID counter (#[serde(skip)])
    pub game_path: Option<PathBuf>,
    pub recent_files: Vec<PathBuf>,
    pub last_reindexed_at: Option<u64>,
    pub recent_game_paths: Vec<PathBuf>,  // Max 5
}
```
`WorkspaceTab { id, label, path, editor_type, modified, pinned }`. Methods: `open()`, `open_with_editor_type()`, `open_tool()`, `close()`, `mark_modified()`, `clear_all_tabs()`, `active()`, `active_index()`, `clear_modified()`, `save()/load()` (JSON to `~/.config/dispel-gui/workspace.json`), `validate_timestamp()`, `validate_last_reindexed()`, `debug_info()`. `EditorType::from_path()` auto-detects editor from file extension.

### Pane Grid Layout
3-pane layout (sidebar | main content | history panel) using `pane_grid::State<PaneContent>`. `PaneContent` enum: `Sidebar | MainContent | HistoryPanel`. `PaneState` tracks focus and maximized.

### Subscriptions (`subscriptions.rs`)
- Keyboard handling via a single `keyboard::listen()` filter (shortcuts live in the filter logic, not a literal key list)
- Separate global-search subscription; Event Script Editor ESC-shortcut subscription
- Sprite animation tick (16ms)
- SNF playback poll (250ms)
- Event script indexing poll (100ms)
- Spreadsheet navigation (arrow keys)

### View Functions

| Function | Location | Renders |
|---|---|---|
| `App::view()` | `view/mod.rs` | Top-level dispatch: start page or editor view |
| `App::view_start_page()` | `view/start_page.rs` | Game path selection card with recent paths |
| `App::view_editor()` | `view/mod.rs` | Full pane grid (sidebar + tab bar + editor + history) |
| `App::view_sidebar()` | `view/mod.rs` | File tree + tools section |
| `App::view_recent_files()` | `view/mod.rs` | Recent files list (10 most recent) |
| `view_history_panel()` | `view/history_panel.rs` | Undo/redo stack display |
| Per-editor `view()` | `editors/*/view.rs` | Editor-specific content (~46 editors) |

### Mod Recording

`RecordingSession` is defined in `state.rs`. Every `FieldChanged` handler calls `observe_field_change()` in `editors/mod_packager/recording.rs`. Records are debounced via `PendingEdit` / `RecordingKey` per field edit, then flushed to the mod changelog on save. 38 integration tests in `recording_tests.rs` cover macro-generated, custom-wrapper, fully-custom, and tab-based editor types.

### Indexation & Search

`indexation/` module:
- `file_index_cache.rs` — `FileIndexCache` + `FileIndexCacheManager` (persistent bincode on disk)
- `indexation_service.rs` — `IndexationService` background file scanning
- `search_index.rs` — `SearchIndex` nucleo-matcher full-text search with persistence
- `tests.rs` — Indexation tests

### Adding a New Editor

1. Implement `Extractor` (derive via `dispel-macros`) in `dispel-core/src/references/`
2. If single-file spreadsheet: invoke `define_standard_editor!` in `dispel-gui/src/editors/<name>/mod.rs`
3. Otherwise: create `editors/my_editor/{mod,state,message,update,view}.rs`
4. Add field to `EditorRegistry` (not `AppState` — editors are consolidated)
5. Add `EditorMessage` variant in `message/editor/mod.rs` (use `define_message_ext!` macro for shorthand)
6. Add handler in `update/editor/` and route via `dispatch_table.rs` / `update/editor/mod.rs`
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
| `StandardEditor<T>` | GenericEditorState + SpreadsheetState — used by the 20 standard editors |
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
- **Iced (fork)**: `container(...).center_x(width)` is a convenience setter for `container(...).width(width).align_x(alignment::Horizontal::Center)`.

---

## Documentation

| Path | Contents |
|---|---|
| `docs/overview.md` | High-level project intro |
| `docs/file_formats.md` | Binary format specs for SNF, map, tileset, sprite |
| `docs/rendering.md` | Isometric map rendering: coordinate transforms, layer ordering, painter's algorithm, sprite transparency |
| `docs/database_and_references.md` | SQLite schema overview |
| `docs/iced-overlay-system.md` | Overlay system notes for the forked iced |
| `docs/files/*.md` | **36 per-format docs** organized in subdirectories (`CharacterInGame/`, `MonsterInGame/`, `NpcInGame/`, `Ref/`, `Map/`, `ExtraInGame/`, `MagicInGame/`) + `CROSS_REFERENCES.md` |

Always read the relevant `docs/files/*.md` before modifying a parser.

## Build System & CI

### Makefile targets (~120 total)
- Basic dev: `fmt`, `check`, `build`, `cargo_test`, `clippy`, `run`, `help`
- `iced_test` — `cargo test -p dispel-gui --features "iced_test app::tests"`
- `hexedit`, `hexedit_test` — build/test the hex editor crate
- Extraction/render: `sound`, `sprite-*`, `map-render` (+ ~15 `map-render-*` variants), `map-atlas-gtl/btl`, `map-to-db` / `map-from-db`
- `extract-*` (~24 targets) — extract individual game files to JSON
- `database-*` — SQLite import (maps, databases, refs, rest, dialog-texts)
- `template*`, `patch-*`, `list-json`, `sprite-info`, `dialogs`, `mod-pack-*` — misc pipelines

Run `make help` for the authoritative list.

### GitHub Actions
- `.github/workflows/test.yml` — CI on push to `master`: `cargo build` on ubuntu-latest (tests NOT run in CI)
- `.github/workflows/pr.yml` — multi-OS PR build matrix
- `.github/workflows/release.yml` — Release on tag `v*.*.*`: 5 builds (CLI Win/Linux, GUI Win/Linux/macOS ARM), archives with SHA256, uploads to GitHub release (draft)

### Release script
`scripts/release.sh` automates: validate semver → `cargo fmt --all --check` + `cargo check --workspace` → bump version in all workspace `Cargo.toml` (discovered via `cargo metadata`) → regen `Cargo.lock` → commit + tag `v{major}.{minor}.{patch}` → print push instructions. (Master-branch and clean-tree checks are currently commented out.)

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

- **dispel-core**: unit test every new parser with hardcoded byte slices (~70 `#[cfg(test)]` blocks)
- **dispel-gui**: write iced_test integration tests that simulate real user flows (open tabs, edit data, undo/redo, save). Test state transitions and behaviors, not visuals. Dedicated suite in `dispel-gui/src/tests/` (27 files) plus co-located `#[cfg(test)]` blocks.
- **Round-trip checks**: `examples/check_save_roundtrip.rs` (parse → write → compare)
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

Subcommands defined in `src/cli.rs`: extract, patch, validate, list, schema, template, map, database, sprite, sound, dialog, mod-pack.

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
- **Editor count drifts**: Verify editor count against `EditorRegistry` and `EditorType` if unsure

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

## iced GUI patterns

This project uses iced from a git fork: `https://github.com/dimspell/iced` (pinned rev `c0ffdd8`). It is a nightly version (v0.15-dev) with modifications to handle accessibility features (like a VoiceOver screen reader). A local clone may exist at `../iced` for development.

### Custom patterns

Custom patterns are in the `gui-widgets` crate - `./gui-widgets/src`.

### `min` and `max` methods for `Length`

There are `min` and `max` methods to the `Length` type, allowing to define minimum and maximum constraints for any widget:

```rs
row![
    container(sidebar).width(Fill.min(150).max(200)),
    container(content).width(FillPortion(3).min(300)),
]
```

### Notification on `scale_factor` change

`window::Event::Resized` is notified when `scale_factor` changes.

---

*Last updated: 2026-08-22*  
**DISPEL®** is a registered trademark. This project is **not affiliated with, endorsed by, or sponsored by** the trademark owner.
