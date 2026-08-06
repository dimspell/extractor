# hexedit — A standalone, embeddable hex editor widget for Iced

**hexedit** is a full-featured hex editor library built on [Iced 0.15](https://iced.rs/).
It ships as a reusable Rust library (`hexedit` crate) and one standalone binary
(`hexedit`).

Designed for the [Dispel RPG modding toolkit](https://github.com/piotr/dispel-extractor),
it is embeddable inside any Iced application and powers the hex-editing tab in `dispel-gui`.

---

## Table of Contents

1. [Features](#features)
2. [Architecture Overview](#architecture-overview)
3. [Domain Layer (Pure Data Model)](#domain-layer-pure-data-model)
4. [UI Layer (Iced-specific)](#ui-layer-iced-specific)
5. [HexEditorState (Aggregate State)](#hexeditorstate-aggregate-state)
6. [HexEditorConfig (Host Injection)](#hexeditorconfig-host-injection)
7. [HexEditorMessage (Enum Reference)](#hexeditormessage-enum-reference)
8. [HexMatrix Custom Widget](#hexmatrix-custom-widget)
9. [Coloring System](#coloring-system)
10. [Inspector Decoder System](#inspector-decoder-system)
11. [Pattern System](#pattern-system)
12. [Writing / Inline Editing](#writing--inline-editing)
13. [Search & Navigation](#search--navigation)
14. [Write Modes & Text Encoding](#write-modes--text-encoding)
15. [Byte Statistics & Entropy Panel](#byte-statistics--entropy-panel)
16. [Lua Scripting Engine](#lua-scripting-engine)
17. [Pane Grid Layout](#pane-grid-layout)
18. [Minimap](#minimap)
19. [Theme System](#theme-system)
20. [Test Structure](#test-structure)
21. [Usage (library)](#usage-library)
22. [Usage (standalone binaries)](#usage-standalone-binaries)
23. [Build](#build)
24. [Feature flags](#feature-flags)
25. [Keybindings](#keybindings)
26. [Dependencies](#dependencies)
27. [LLM Guidance — Conventions & Invariants](#llm-guidance--conventions--invariants)

---

## Features

### Core editing
- **Virtualized hex matrix** — Address gutter, hex bytes (grouped 8+8), ASCII column, and
  annotation column. Only rows in the viewport are rendered (virtualized scrolling).
- **Inline overwrite editing** — Double-click or `F2` to begin editing a byte; type two hex
  digits to auto-commit and advance to the next byte. `Tab`/`Enter` to commit and advance,
  `Esc` to cancel.
- **Selection** — Click, drag, or Shift+arrow to select byte ranges. Arrow keys, `Home`/`End`,
  `Page Up`/`Page Down`, `Ctrl+Home`/`Ctrl+End` for keyboard navigation.
- **Copy / Paste** — `Ctrl+C` copies the selected range as hex text (`DE AD BE EF`);
  `Ctrl+V` pastes hex text from the clipboard back into the buffer.
- **Write bytes** — Programmatic `WriteBytes` message for inspector edits and scripted
  modifications.

### Search & navigation
- **Search** (`Ctrl+F`) — Hex byte-sequence search (`DE AD BE EF` or `DEADBEEF`) and
  ASCII substring search. Next/prev match navigation with match highlighting in the matrix.
- **Goto** (`Ctrl+G`) — Navigate to a specific address. Accepts hex (`0xFF`, `FF`),
  decimal (`255`), and relative offsets (`+10`, `-5`).
- **Bytes-per-row toggle** — Switch between 8, 16, and 32 bytes per row.

### Data inspector
- **Built-in decoders** — `u8`, `i8`, `u16`, `i16`, `u32`, `i32`, `u64`, `i64`, `f32`, `f64`,
  ASCII char, UTF-8 char, RGB565 pixel, null-terminated C string, and raw hex dump.
- **Editable values** — All numeric decoders support inline editing via a modal: type a
  decimal or `0x`-prefixed hex value and it's written back in the correct endianness.
- **Lua scripting** (`feature = "lua"`) — Write custom Lua decoder scripts that appear
  as additional inspector entries. Scripts are sandboxed (`os`, `io`, `loadfile`, `require`,
  etc. removed) unless `HEXEDIT_LUA_UNSAFE=1` is set.

### Pattern highlighting
- **Named byte ranges** — Select a range, right-click → "Create Pattern" (or `Ctrl+E`).
  Patterns are rendered with a distinct background/foreground colour in the matrix.
- **Repeated (zebra) patterns** — Create multi-instance repeating patterns with automatic
  annotation labels (e.g. `Monster[0]`, `Monster[1]`...).
- **Pattern groups** — Organize related patterns into collapsible groups with rename, colour
  cycling, and bulk delete.
- **Annotations** — Text annotations per pattern shown in the annotation column to the right
  of the ASCII area. Annotations of patterns under the cursor are highlighted.
- **Import / Export** — Patterns and groups can be exported to and imported from a
  JSON file.

### Visual colouring
- 5 colour schemes via the settings modal:
  - **Monochrome** — Single colour for all bytes (the classic hex editor look).
  - **Nybble** — 18 colour groups by high nibble (one per nybble + special `0x00`/`0xFF`).
  - **Categories** — 6 semantic groups (NULL, whitespace, printable, DEL, control, non-ASCII).
  - **Rainbow** — Continuous hue gradient across the full `0x00–0xFF` range.
  - **Heatmap** — Cold-to-hot heatmap (blue → cyan → green → yellow → red).
- **Dim nulls** — Optionally render `0x00` bytes in a dim colour regardless of the active scheme.
- **Dirty vs vanilla-diff colouring** — Bytes dirtied this session use one colour; bytes that
  differ cumulatively from the on-disk snapshot use another.

### Layout & panels
- **Pane grid** — Halloy-style movable, splittable, resizable panels. The default layout is a
  vertical split: hex matrix on the left, data inspector on the right. The pattern list opens
  as an additional split pane.
- **Toolbar** — Save button, Goto, Patterns toggle, text export, settings, bytes-per-row toggle,
  and status message.
- **Minimap** — Overview strip showing file structure at a glance, with cursor position marker
  and dirty/diff pixel indicators.

### Save & integration
- **Host-configurable save** — The host app provides an `OnSaveFn` callback, a save label,
  and a `can_save` flag. The editor surfaces these in the toolbar.
- **Dirty tracking** — `BufferProvider` tracks which addresses have been modified since load.
- **Text export** — Export the file as a formatted hex dump text file (configurable: address
  gutter, decimal/hex addresses, ASCII column).

### File support
- Library: `hexedit::HexEditorState::load_from_path(&path)` loads any file into memory.
- Binaries: Open files via command-line argument (`hexedit path/to/file.bin`) or file dialog.

---

## Architecture Overview

```
hexedit/
├── src/
│   ├── domain/          # Pure data model (no Iced types, no widget dependencies)
│   │   ├── provider.rs       # HexProvider trait + BufferProvider (in-memory editing buffer)
│   │   ├── selection.rs      # Selection (anchor, cursor), NavDir enum, nav_target()
│   │   ├── pattern.rs        # Pattern, RepeatedPatternGroup, RepeatPatternDialog, PatternExport
│   │   ├── search.rs         # SearchState, SearchMode, hex/ASCII search, SearchMatchProvider
│   │   ├── goto.rs           # GotoState — parse hex/decimal/relative address expressions
│   │   ├── editing.rs        # EditState (per-cell hex draft), InspectorEditState
│   │   ├── panel.rs          # HexPanel, HexPanelContent (Matrix/Inspector/PatternList/Statistics)
│   │   ├── layout.rs         # BinaryLayout trait, FieldSpan, LayoutRegistry (extensible overlays)
│   │   ├── vanilla_diff.rs   # compute_diff() — BTreeSet of addresses changed vs snapshot
│   │   ├── export_config.rs  # ExportConfig for hex dump text export
│   │   ├── fill_dialog.rs    # FillDialog — repeat a byte pattern across a selection
│   │   ├── extend_dialog.rs  # ExtendDialog — insert bytes at the cursor (count + fill pattern)
│   │   ├── write_mode.rs     # WriteMode enum, EncodingEntry, encode_text(), is_text_mode()
│   │   ├── byte_stats.rs     # ByteStatistics, RowEntropyCache, compute_statistics()
│   │   └── pattern_layout.rs # Git-log-style branch connectors for grouped patterns in pattern list
│   │
│   ├── ui/               # Iced-specific (widget, view, update, coloring, theme)
│   │   ├── update.rs         # Pure fn update(state, config, msg) -> Task<Message>
│   │   ├── inspector.rs      # ENTRIES lazy static — 15 built-in decode/encode pairs
│   │   ├── coloring.rs       # CellColorProvider trait + fold_color() provider chain
│   │   ├── theme/            # HexEditorTheme struct + DARK_THEME + LIGHT_THEME consts
│   │   └── view/             # 16 view submodules
│   │       ├── mod.rs            # Root view() — dispatches error/toolbar/pane-grid/modal stack
│   │       ├── panel.rs          # Pane title bars & content dispatch per HexPanelContent
│   │       ├── toolbar.rs        # Save, goto, patterns toggle, export, settings, bytes/row
│   │       ├── footer.rs         # Status bar (cursor address, selection size, write mode)
│   │       ├── matrix/           # Custom HexMatrix widget (virtualized, event handling)
│   │       ├── patterns.rs       # Pattern list panel (accordion groups, rename, colour cycle)
│   │       ├── inspector.rs      # Inspector panel view (decoded values at cursor)
│   │       ├── statistics.rs     # Byte statistics & entropy panel
│   │       ├── minimap.rs        # Overview strip at the bottom
│   │       ├── search_overlay.rs # Search bar overlay
│   │       ├── goto_modal.rs     # Goto-address modal
│   │       ├── inspector_modal.rs # Inspector inline-edit modal
│   │       ├── settings_modal.rs # Settings modal (theme, colour scheme, dim nulls)
│   │       ├── export_modal.rs   # Text export config modal
│   │       ├── fill_modal.rs     # Fill selection modal
│   │       ├── extend_modal.rs   # Extend file modal
│   │       ├── repeat_modal.rs   # Repeated pattern dialog
│   │       └── encoding_modal.rs # Custom encoding settings modal
│   │
│   ├── config.rs        # HexEditorConfig (7 fields + OnSaveFn callback type)
│   ├── message.rs       # HexEditorMessage enum (~48 variant categories, ~200+ total)
│   ├── state.rs         # HexEditorState (aggregate state, ~30 fields)
│   ├── lua_engine.rs    # Lua 5.4 scripting engine (feature-gated, sandboxed)
│   ├── main.rs          # Standalone bin: hexedit (file dialog + file path arg)
│   ├── lib.rs           # Public API re-exports
│   └── tests/           # iced_test integration tests (12 files)
├── examples/            # Lua decoder script examples
│   ├── minimal_decoder.lua
│   ├── decode_dispel_types.lua
│   └── decode_item_type.lua
└── Cargo.toml
```

**Design principle:** `domain/` is pure Rust — no Iced types, no widget dependencies.
`ui/` contains all Iced-specific code. This lets the data model be tested and reused
without a running UI.

---

## Domain Layer (Pure Data Model)

All files in `src/domain/` are free of Iced widget types. They can be unit-tested
without a running UI.

### `provider.rs` — Byte source abstraction
- **`HexProvider` trait** (`read()`, `write()`, `len()`, `is_writable()`): Abstraction over byte sources so vanilla snapshots and the live editing buffer can coexist.
- **`BufferProvider`**: In-memory `Vec<u8>` with a `BTreeSet<u64>` of dirty addresses. Tracks which bytes have been modified since load. Key methods: `from_bytes()`, `dirty()`, `dirty_count()`, `clear_dirty()`, `as_slice()`.
- **Rule:** `write()` only marks an address dirty if the new byte differs from the old one.

### `selection.rs` — Cursor & selection model
- **`Selection`**: `{ anchor: u64, cursor: u64 }`. The selected range is `min(anchor,cursor)..=max(anchor,cursor)`. Methods: `single()`, `range()`, `start()`, `end()`, `len()`, `is_single()`, `contains()`, `select()`, `extend()`.
- **`NavDir` enum**: Left, Right, Up, Down, LineStart, LineEnd, PageUp, PageDown, DocumentStart, DocumentEnd.
- **`nav_target(cursor, dir, bytes_per_row, page_rows, max_addr) -> u64`**: Pure function computing the new cursor after navigation. All edge cases (saturating at 0, clamping to max_addr) are unit-tested.

### `pattern.rs` — Pattern highlighting
- **`Pattern`**: `{ id, start, end, color_idx, group_id, annotation }`. Serializable via serde.
- **`RepeatedPatternGroup`**: `{ id, label, color_idx }`. Groups related patterns together.
- **`RepeatPatternDialog`**: Transient modal state for creating repeated patterns from a selection.
- **`PatternExport`**: Versioned JSON envelope (`{ version, groups, patterns }`) for import/export.
- **`pattern_bg(idx)` / `pattern_fg(idx)`**: Return palette colours (16-colour cyclic palette from `DARK_THEME`).

### `search.rs` — Search engine
- **`SearchMode`**: `Hex` or `Ascii`.
- **`SearchState`**: `{ visible, query, mode, results: Vec<u64>, query_len, current_match, match_set: BTreeSet<u64> }`. The `match_set` is a precomputed O(log n) lookup table for the renderer.
- **`parse_hex_query(s)`**: Parse `"DE AD BE EF"` or `"DEADBEEF"` into `Vec<u8>`. Requires even number of hex digits.
- **`SearchMatchProvider`**: Implements `CellColorProvider` for search match highlighting in the matrix.

### `goto.rs` — Goto address parsing
- **`GotoState`**: `{ draft, error }`.
- **`parse(cursor, max_addr)`**: Accepts hex (`0xFF`, `FF`), decimal (`255`), relative (`+10`, `-5`). Clamps to `[0, max_addr]`. Pure function, fully unit-tested.

### `editing.rs` — Inline edit state
- **`EditState`**: `{ addr, draft: String }`. Draft holds 0–2 uppercase hex characters. `push_char()`, `pop_char()`, `is_complete()`, `staged_byte()` (single-digit draft treated as low nibble, e.g., `"A"` → `0x0A`).
- **`InspectorEditState`**: `{ entry_idx, addr, draft, error }`. Modal state for the inspector inline-edit flow.

### `write_mode.rs` — Text encoding modes
- **`WriteMode`**: `Hex`, `Ascii`, `Utf8`, `Windows1250`, `EucKr`, `Custom(usize)`.
- **`EncodingEntry`**: `{ label, encoding_name }` for user-added encodings (persisted by host).
- **`encode_text(text, mode, custom) -> Vec<u8>`**: Encode a single string character into bytes. Used by the inline text-typing path.
- **`is_text_mode(mode) -> bool`**: `true` for any mode except `Hex`.
- **`COMMON_ENCODINGS`**: Static list of 30+ `(label, encoding_rs_name)` pairs available for custom encoding entries.
- **`remap_write_mode()`**: Re-indexes `WriteMode::Custom(i)` when items are removed from the custom list.

### `panel.rs` — Pane grid types
- **`HexPanelContent`**: `Matrix`, `Inspector`, `PatternList`, `Statistics`.
- **`HexPanel`**: Wrapper around `HexPanelContent`.
- **`default_pane_grid() -> pane_grid::State<HexPanel>`**: Creates the default vertical split (75/25 matrix/inspector).

### `layout.rs` — Structure overlay registry (future)
- **`BinaryLayout` trait**: `layout(bytes) -> Vec<FieldSpan>`.
- **`FieldSpan`**: `{ range, name, ty }`.
- **`LayoutRegistry`**: Extension-keyed lookup table. Empty in v1 — reserved for future auto-derived structure overlays from `dispel_core`'s `#[extractor]` attributes.

### `byte_stats.rs` — Byte statistics & entropy
- **`ByteStatistics`**: Counts, frequencies, min/max, Shannon entropy, structure heuristics.
- **`RowEntropyCache`**: Per-row entropy values for the gutter colour band.
- **`compute_statistics(bytes) -> ByteStatistics`**: Full analysis (async, via `Task::perform`).
- **`compute_row_entropies(bytes, bpr) -> RowEntropyCache`**: Per-row entropy values.
- **`entropy_to_color()`**: Maps entropy value to a colour for the gutter band.
- **`StructureHeuristic`**: Detects uniform runs, high/low entropy patterns, mixed content.

### `vanilla_diff.rs` — Change tracking
- **`compute_diff(vanilla, current) -> BTreeSet<u64>`**: Linear scan comparing two byte slices, returning addresses where they differ.

### `fill_dialog.rs` — Fill selection
- **`FillDialog`**: `{ draft, error }`. Parses a hex byte pattern string to repeat across the selected range.

### `pattern_layout.rs` — Branch connectors
- Types for rendering Git-log-style branch connectors in the pattern list panel for grouped patterns.

---

## UI Layer (Iced-specific)

### `update.rs` — Message handler
```rust
pub fn update(
    state: &mut HexEditorState,
    config: &HexEditorConfig,
    message: HexEditorMessage,
) -> Task<HexEditorMessage>
```
Pure update function (~1300 lines). Dispatches all message variants. Returns `Task::none()` for synchronous mutations, or an async `Task` for file I/O, clipboard, and analysis operations.

Key handler sections:
- Pane grid operations (click, resize, drag, split, close)
- Cursor movement & selection (SelectAt, ExtendTo, Nav)
- Inline editing (BeginEdit, EditTypeChar, EditBackspace, EditCancel, EditCommit)
- Inspector (copy, begin edit, commit edit)
- Save (delegates to config's `OnSaveFn`)
- Search (open, execute, toggle mode, next/prev, close)
- Goto dialog flow
- Pattern CRUD (create, remove, clear, right-click)
- Repeated pattern dialog
- Pattern list & group operations
- Pattern import/export (async file dialogs)
- Settings (theme, colour scheme, dim nulls, entropy band, minimap)
- Write mode & encoding settings
- Copy/Paste (clipboard)
- Fill selection
- Extend file (insert bytes at the cursor)
- Text export (async file dialog + write)
- Byte statistics analysis (async)
- Navigation centering via `pending_center_on`

### `inspector.rs` — Decoder registry
```rust
pub struct InspectorEntry {
    pub name: String,
    pub min_size: usize,
    pub decode: DecodeFn,    // Box<dyn Fn(&[u8]) -> String + Send + Sync>
    pub encode: Option<EncodeFn>,  // Option<Box<dyn Fn(&str) -> Result<Vec<u8>, String> + Send + Sync>>
    pub category: String,
    pub description: String,
}

pub static ENTRIES: Lazy<Vec<InspectorEntry>>;
```
15 built-in entries: u8, i8, u16, i16, u32, i32, u64, i64, f32, f64, ascii, utf8, rgb565, cstr, hex.
Numeric entries are editable (have `encode`). The host can inject additional entries via `config.extra_entries`.

### `coloring.rs` — Cell colouring system
Layered provider chain pattern. Each provider implements `CellColorProvider`:
```rust
pub trait CellColorProvider {
    fn color(&self, addr: u64, byte: u8) -> (Option<Color>, Option<Color>);
}

pub fn fold_color(providers, addr, byte) -> (Option<Color>, Option<Color>);
```
Later providers in the chain override earlier ones when they return `Some`.

Built-in providers (in application order):
1. **`SearchMatchProvider`** — highlights search matches and current match
2. **`DiffVsVanillaProvider`** — highlights bytes differing from vanilla snapshot
3. **`DirtyProvider`** — highlights bytes modified this session
4. **`PatternBgProvider`** — applies pattern background colours
5. **`AnnotationProvider`** — applies annotation foreground for active patterns
6. **`SchemeProvider`** — applies the active `ColorScheme` (Monochrome/Nybble/Categories/Rainbow/Heatmap)
7. **`DimNullsProvider`** — optionally dims `0x00` bytes
8. **`SelectionProvider`** — highlights selection range and cursor cell

### `theme/` — Theme system
- **`HexEditorTheme`**: Flat struct with ~90+ colour fields covering matrix, header, selection, edit/dirty/diff, search matches, annotations, scrollbar, modals, minimap, pattern panel, statistics panel, byte-colouring schemes, pattern palettes, and Iced application palette.
- **`DARK_THEME`**: Near-black background with warm amber/brown tones (original).
- **`LIGHT_THEME`**: Warm parchment background for accessibility (WCAG AA).
- Both are `const` values compiled at compile time using `const fn hex()`.
- `ThemeVariant` enum: `Dark` / `Light` (with `Default` on `Dark`).
- Theme includes HSL saturation/lightness params for gradient schemes (0.70 for dark, 0.35 for light).

---

## HexEditorState (Aggregate State)

```rust
pub struct HexEditorState {
    pub path: PathBuf,
    pub name: String,
    pub panes: pane_grid::State<HexPanel>,
    pub pane_focus: pane_grid::Pane,
    pub provider: BufferProvider,
    pub bytes_per_row: u8,                    // 8, 16, or 32
    pub selection: Selection,
    pub edit_mode: Option<EditState>,
    pub inspector_edit: Option<InspectorEditState>,
    pub vanilla: Option<Vec<u8>>,            // on-disk snapshot for diff
    pub vanilla_diff: BTreeSet<u64>,         // cached diff result
    pub patterns: Vec<Pattern>,
    pub pattern_by_addr: BTreeMap<u64, (usize, u8)>,  // addr -> (pattern_id, color_idx)
    pub show_pattern_list: bool,
    pub next_pattern_id: usize,
    pub groups: Vec<RepeatedPatternGroup>,
    pub next_group_id: usize,
    pub collapsed_groups: BTreeSet<usize>,    // collapsed repeated-group accordion sections
    pub row_annotations: BTreeMap<u64, Vec<(usize, String)>>,  // row_start -> [(pat_id, text)]
    pub active_patterns: BTreeSet<usize>,     // pattern ids under cursor
    pub renaming_group: Option<usize>,
    pub renaming_group_draft: String,
    pub context_menu_addr: Option<u64>,
    pub goto: Option<GotoState>,
    pub export_config: Option<ExportConfig>,
    pub fill_dialog: Option<FillDialog>,
    pub search: SearchState,
    pub show_decimal: bool,                   // addr format toggle
    pub status_msg: String,
    pub error: Option<String>,
    pub repeat_pattern: Option<RepeatPatternDialog>,
    pub cache: ParagraphCache,                // shared across frames
    pub color_scheme: ColorScheme,
    pub dim_nulls: bool,
    pub settings_open: bool,
    pub lua_engine: LuaScriptEngine,
    pub write_mode: WriteMode,
    pub custom_encodings: Vec<EncodingEntry>,
    pub encoding_settings_open: bool,
    pub encoding_settings_selection: Option<usize>,
    pub show_stats: bool,
    pub file_stats: Option<ByteStatistics>,
    pub selection_stats: Option<ByteStatistics>,
    pub row_entropies: Option<RowEntropyCache>,
    pub show_entropy_band: bool,
    pub show_minimap: bool,
    pub pending_center_on: Cell<Option<u64>>,  // one-frame viewport center request
    pub theme: &'static HexEditorTheme,
    pub theme_variant: ThemeVariant,
}
```

Key methods:
- `load_from_path(path)` — constructs state from file (reads bytes, computes initial entropies, initializes Lua engine, sets default pane layout)
- `max_addr()` — `provider.len().saturating_sub(1)`
- `recompute_vanilla_diff()` — linear scan, call after every write
- `add_pattern(start, end) -> usize` — creates pattern, rebuilds lookups
- `remove_pattern(id)` — removes pattern + orphan groups
- `clear_patterns()` — wipes all patterns and groups
- `rebuild_pattern_lookup()` — rebuilds `pattern_by_addr` BTreeMap
- `recompute_row_annotations()` — rebuilds annotation map + refreshes active patterns
- `refresh_active_patterns()` — recomputes which patterns contain the cursor
- `invalidate_stats()` — clears cached byte statistics when content changes
- `load_lua_scripts(dir)` — loads all `.lua` files from a directory

---

## HexEditorConfig (Host Injection)

```rust
pub type OnSaveFn = Arc<dyn Fn(&HexEditorState) -> Task<HexEditorMessage> + Send + Sync>;

pub struct HexEditorConfig {
    pub pane_gap: u16,                                    // default: 4
    pub on_save: Option<OnSaveFn>,                        // None hides save button
    pub save_label: String,                               // e.g. "Save into my-mod"
    pub can_save: bool,                                   // prerequisites met?
    pub save_hint: String,                                // why save is disabled
    pub extra_entries: Vec<InspectorEntry>,                // host-specific decoders
    pub custom_encodings: Vec<EncodingEntry>,              // persisted by host
    pub on_write_mode_changed: Option<Arc<dyn Fn(WriteMode) -> Task<HexEditorMessage> + Send + Sync>>,
}
```

Helper methods: `save_label()`, `has_save()`, `can_save_now(state)` (checks `can_save && on_save.is_some() && dirty_count > 0`).

---

## HexEditorMessage (Enum Reference)

~48 variant categories, ~200+ total variants. All variants are `#[derive(Debug, Clone)]` and produce `iced::Task<HexEditorMessage>`.

### Pane grid
| Variant | Purpose |
|---------|---------|
| `PaneClicked(pane)` | Set keyboard focus to pane |
| `PaneResized(ResizeEvent)` | Divider dragged |
| `PaneDragged(DragEvent)` | Pane reorder/dock |
| `SplitPane(Axis)` | Split focused pane (max 8 panes) |
| `ClosePane` | Close focused pane |

### Cursor & selection
| Variant | Purpose |
|---------|---------|
| `SetBytesPerRow(u8)` | Toggle 8/16/32 |
| `SelectAt(u64)` | Single click — set anchor=cursor=addr |
| `ExtendTo(u64)` | Shift-click/drag — move cursor only |
| `Nav { dir, extend }` | Keyboard navigation |

### Inline editing
| Variant | Purpose |
|---------|---------|
| `BeginEdit(u64)` | Enter edit mode at addr |
| `EditTypeChar(char)` | Append hex digit or encode text char |
| `EditBackspace` | Remove last digit / move cursor left (text mode) |
| `EditCancel` | Cancel edit (Esc) |
| `EditCommit { advance }` | Commit byte, optionally advance |
| `DeleteByteAtCursor` | Write 0x00 at cursor (text mode only) |
| `WriteBytes { addr, bytes }` | Programmatic write (inspector/injection) |

### Inspector
| Variant | Purpose |
|---------|---------|
| `CopyInspectorValue(usize)` | Copy decoded value to clipboard |
| `BeginInspectorEdit(usize)` | Open edit modal for entry |
| `SetInspectorDraft(String)` | Update modal draft |
| `CloseInspectorEdit` | Dismiss modal |
| `CommitInspectorEdit` | Encode and write |

### Save
| Variant | Purpose |
|---------|---------|
| `SaveIntoRecording` | Trigger save (delegates to OnSaveFn) |
| `SavedIntoRecording(Result<...>)` | Async save result |
| `ClearStatus` | Clear status message |

### Pattern highlighting
| Variant | Purpose |
|---------|---------|
| `CreatePattern` | From selection range |
| `RemovePatternAt(u64)` | By address |
| `RemovePatternAtContextMenu` | Via context menu |
| `ClearAllPatterns` | Wipe |
| `RightClickAt(u64)` | For context menu targeting |

### Repeated pattern dialog
| Variant | Purpose |
|---------|---------|
| `BeginRepeatedPattern` | Open dialog from selection |
| `SetRepeatedPatternDraft(String)` | Repeat count |
| `SetRepeatedPatternLabel(String)` | Group label |
| `CommitRepeatedPattern` | Create zebra patterns |
| `CloseRepeatedPattern` | Dismiss |

### Goto
| Variant | Purpose |
|---------|---------|
| `OpenGotoDialog` | Opens modal, focuses input |
| `SetGotoDraft(String)` | Update text |
| `CommitGoto` | Parse and navigate |
| `CloseGotoDialog` | Dismiss |

### Search
| Variant | Purpose |
|---------|---------|
| `OpenSearch` | Show overlay |
| `Search(String)` | Execute search with query |
| `ToggleSearchMode` | Hex ↔ ASCII |
| `SearchNext` / `SearchPrev` | Navigate matches |
| `CloseSearch` | Hide overlay |

### Pattern list & groups
| Variant | Purpose |
|---------|---------|
| `TogglePatternList` / `ToggleInspector` | Show/hide panels |
| `NavigateToPattern(usize)` | Jump to pattern start |
| `RemovePattern(usize)` | By id |
| `TogglePatternGroup(usize)` | Collapse/expand |
| `RemovePatternGroup(usize)` | Bulk delete |
| `BeginRenameGroup` / `CommitRenameGroup` / `CancelRenameGroup` | Inline rename |
| `CycleGroupColor` / `CyclePatternColor` | Cycle palette index |
| `SetPatternAnnotation` / `ClearPatternAnnotation` | Annotation CRUD |
| `ExportPatterns` / `ImportPatterns` | JSON file I/O |
| `PatternsExported` / `PatternsImported` | Async results |

### Address format
| Variant | Purpose |
|---------|---------|
| `ToggleAddrFormat` | Hex ↔ decimal |
| `SetAddrFormat(bool)` | Explicit |

### Settings modal
| Variant | Purpose |
|---------|---------|
| `OpenSettings` / `CloseSettings` | Modal visibility |
| `SetTheme(ThemeVariant)` | Dark / Light |
| `SetColorScheme(ColorScheme)` | Monochrome / Nybble / Categories / Rainbow / Heatmap |
| `SetDimNulls(bool)` | Dim 0x00 |
| `SetShowEntropyBand(bool)` | Gutter colour band |
| `SetShowMinimapEnabled(bool)` | Minimap toggle |
| `ResetSettings` | All defaults |

### Write mode & encoding
| Variant | Purpose |
|---------|---------|
| `SetWriteMode(WriteMode)` | Switch mode |
| `OpenEncodingSettings` / `CloseEncodingSettings` | Encoding settings modal |
| `AddCustomEncoding(usize)` | From common list |
| `RemoveCustomEncoding(usize)` | Remove entry |
| `SetCustomEncodings(Vec<EncodingEntry>)` | Bulk replace (deserialize) |

### Copy / Paste
| Variant | Purpose |
|---------|---------|
| `CopySelection` | Copy as hex text |
| `Paste` | Read clipboard (async) |
| `PasteContent(String)` | Parse and write |

### Fill selection
| Variant | Purpose |
|---------|---------|
| `BeginFill` | Open dialog |
| `SetFillDraft(String)` | Update pattern |
| `CommitFill` | Write repeated pattern |
| `CloseFill` | Dismiss |

### Extend file
| Variant | Purpose |
|---------|---------|
| `BeginExtend` | Open dialog (context menu at cursor) |
| `SetExtendCount(String)` | Update byte count |
| `SetExtendPattern(String)` | Update fill pattern |
| `CommitExtend` | Insert bytes at cursor |
| `CloseExtend` | Dismiss |

### Byte statistics
| Variant | Purpose |
|---------|---------|
| `ToggleStats` | Show/hide panel |
| `AnalyzeFile` | Full-file async analysis |
| `AnalyzeSelection` | Selection-only analysis |
| `FileAndRowEntropiesComputed(ByteStatistics, RowEntropyCache)` | Async result |
| `SelectionAnalyzed(ByteStatistics)` | Async result |

### Text export
| Variant | Purpose |
|---------|---------|
| `OpenExportConfig` / `CloseExportConfig` | Modal visibility |
| `SetExportShowAddress(bool)` | Address gutter toggle |
| `SetExportAddressDecimal(bool)` | Address format |
| `SetExportShowAscii(bool)` | ASCII column toggle |
| `CommitExport` | File dialog + write |
| `TextExportCompleted(Result<...>)` | Async result |

---

## HexMatrix Custom Widget

Located in `src/ui/view/matrix/` (5 files):

### `mod.rs`
Public API: `HexMatrix` widget struct, `hex_matrix()` constructor function. Wraps `layout`, `state`, `draw`, and `event` modules.

### `layout.rs`
Computes the visual layout: column widths (address gutter, hex bytes grouped 8+8, ASCII column, annotation column), row height, total content height for scrolling. Returns sizing metrics consumed by the draw and event handlers.

### `state.rs`
Internal mutable state cached between frames: scroll offset, selection/cursor visual positions, hot/cold state for click handling.

### `draw.rs` (~984 lines)
The rendering core. Paints via Iced's `Renderer` (wgpu backend):
- Address gutter with hex or decimal labels
- Hex byte columns with 8+8 grouping and group separator lines
- ASCII column with printable characters (0x20–0x7E), dots for non-printable
- Annotation column with pattern text
- Colour resolution via the provider chain (CellColorProvider)
- Selection highlight, cursor cell
- Edit-in-progress highlight
- Scrollbar with search match dots and cursor dot

### `event.rs` (~605 lines)
Mouse and keyboard event handling:
- Click/drag selection
- Double-click → `BeginEdit`
- Keyboard navigation (arrows, Home/End, PageUp/Down, Ctrl+Home/End)
- Hex digit typing → auto-starts edit
- Shift+arrows → extend selection
- Scroll wheel
- All event handling defers to `HexEditorMessage` for state mutation

**Virtualization:** Only rows in the viewport are drawn. The widget computes which rows are visible based on scroll offset and viewport height, then iterates byte addresses in that range.

---

## Coloring System

The hex matrix resolves cell colours through a layered provider chain. The application order in `draw.rs` is:

1. **Search match** — `SearchMatchProvider` highlights all bytes in search results (green tones)
2. **Diff vs vanilla** — `DiffVsVanillaProvider` shows cumulative changes from on-disk snapshot
3. **Dirty bytes** — `DirtyProvider` shows bytes modified this session
4. **Pattern background** — `PatternBgProvider` applies pattern background colours
5. **Pattern annotation** — `AnnotationProvider` adjusts foreground for active pattern annotations
6. **Colour scheme** — `SchemeProvider` applies the base font colour (Monochrome/Nybble/Categories/Rainbow/Heatmap)
7. **Dim nulls** — `DimNullsProvider` optionally dims `0x00` bytes
8. **Selection** — `SelectionProvider` paints selection range and cursor cell

Each provider returns `(Option<fg>, Option<bg>)`. `fold_color()` merges them: later providers override earlier ones. This means search highlights take priority over dirty/diff, which take priority over patterns, which take priority over the base scheme.

**`default_byte_colors(scheme, byte, dim_nulls)`** is the convenience function used by both the matrix widget and the settings-modal palette preview to ensure consistency.

---

## Inspector Decoder System

15 built-in `InspectorEntry` values in `ENTRIES` lazy static:

| Name | Size | Editable | Category |
|------|------|----------|----------|
| u8 | 1 | Yes | Integer |
| i8 | 1 | Yes | Integer |
| u16 | 2 | Yes | Integer |
| i16 | 2 | Yes | Integer |
| u32 | 4 | Yes | Integer |
| i32 | 4 | Yes | Integer |
| u64 | 8 | Yes | Integer |
| i64 | 8 | Yes | Integer |
| f32 | 4 | Yes | Float |
| f64 | 8 | Yes | Float |
| ascii | 1 | No | Text |
| utf8 | 1 | No | Text |
| rgb565 | 2 | No | Color |
| cstr | 1 | No | Text |
| hex | 1 | No | Binary |

Each entry has a `decode` function that converts bytes to a display string, and optionally an `encode` function that parses user input and produces bytes.

The host can add entries via `config.extra_entries`. The update handler selects between built-in and extra entries based on index: indices below `ENTRIES.len()` are built-in; higher indices index into `config.extra_entries`.

---

## Pattern System

### Data model
- **`Pattern`**: Serializable struct with id, byte range, colour index (0–15, cyclic palette), optional group id, optional annotation string.
- **`RepeatedPatternGroup`**: Named group with shared colour. All child patterns inherit the group's colour.

### Lookup structures
- **`pattern_by_addr: BTreeMap<u64, (usize, u8)>`**: Maps every byte address to the pattern id and colour index that covers it. Rebuilt on every pattern mutation.
- **`row_annotations: BTreeMap<u64, Vec<(usize, String)>>`**: Maps row start addresses to annotation segments for the annotation column. Rebuilt on every pattern mutation.
- **`active_patterns: BTreeSet<usize>`**: Pattern ids whose span includes the cursor. Refreshed on cursor movement.

### Creation flow
1. User selects a byte range
2. Right-click → "Create Pattern" or `Ctrl+E`
3. `add_pattern(start, end)` creates `Pattern { id, start, end, color_idx }`, rebuilds lookups

### Repeated pattern flow
1. User selects a block
2. Right-click → "Add Repeated Pattern"
3. Modal: enter repeat count and group label
4. `CommitRepeatedPattern` creates alternating-colour `Pattern` entries, auto-fills annotations as `"GroupName[0]"`, `"GroupName[1]"`, etc.
5. Group rename updates child annotations matching `"{old_label}[{digits}]"` pattern

### Import/Export
Patterns export as JSON via `PatternExport` struct (versioned, currently v1). Async file dialog flow.

---

## Writing / Inline Editing

### Hex mode (default)
1. User presses a hex digit → auto-creates `EditState{ addr: cursor, draft: "" }` and appends the digit
2. Second digit appended → auto-commits: `provider.write(addr, byte)`, advances cursor +1, creates new `EditState` at next address
3. `Tab`/`Enter` → `EditCommit { advance: true }`: commit current draft (or nothing if empty), advance cursor
4. `Esc` → `EditCancel`: discard draft

### Text mode (ASCII, UTF-8, Windows-1250, EUC-KR, Custom)
1. User types any printable character → immediately encoded via `encode_text()`, written to buffer, cursor advanced by encoded byte count
2. No draft — each character is written atomically
3. `Backspace` moves cursor left by one byte (no delete)
4. `Delete` key writes 0x00 at cursor and advances

### Inspector edit
1. User clicks "Edit" on an inspector entry → `BeginInspectorEdit(idx)` opens modal with pre-filled current value
2. User edits draft → `SetInspectorDraft(String)`
3. User confirms → `CommitInspectorEdit`: calls entry's `encode(&draft)`, writes bytes, closes modal

---

## Search & Navigation

### Search
- Two modes: Hex (`"DE AD BE EF"` or `"DEADBEEF"`) and ASCII (`"hello"`)
- Results are `Vec<u64>` of match start addresses, pre-computed on every query change
- `match_set: BTreeSet<u64>` for O(log n) lookup during rendering
- Current match highlighted differently from other matches
- Next/prev wrap around
- Search overlay appears below toolbar, rendered in `search_overlay.rs`

### Goto
- Modal dialog activated by `Ctrl+G`
- Input parsing: `0xFF` (hex prefix), `FF` (hex-detected if contains a-f), `255` (decimal), `+10` (relative forward), `-5` (relative backward)
- On commit: selection jumps to target address, viewport centers on it via `pending_center_on`
- Backed by `GotoState.parse(cursor, max_addr)`, fully unit-tested

---

## Write Modes & Text Encoding

The `WriteMode` enum controls how keyboard input is interpreted:

| Mode | Input → Bytes | Use case |
|------|--------------|----------|
| `Hex` | Each digit pair → 1 byte | Classic hex editing |
| `Ascii` | Each char → 1 byte (0x00–0x7F) | Editing ASCII text |
| `Utf8` | Each char → 1–4 bytes | Unicode text |
| `Windows1250` | Each char → 1 byte via encoding_rs | Central European text |
| `EucKr` | Each char → 1–2 bytes via encoding_rs | Korean text |
| `Custom(idx)` | Via `encoding_rs` for user-added encoding | Any encoding_rs-supported encoding |

Users can add custom encodings from a list of 30+ `COMMON_ENCODINGS` via the encoding settings modal.

**Serde:** `WriteMode` serializes as string label. `Custom` stores its label, not index — index is resolved at deserialization through the custom encoding list.

---

## Byte Statistics & Entropy Panel

Toggleable panel showing:
- **Byte distribution histogram** (bar chart of byte value frequencies)
- **Shannon entropy** (file-level and per-row)
- **Structure heuristics** (uniform runs, high/low entropy regions, mixed content)
- **Entropy colour band** in the address gutter (visual entropy fingerprint)

All analysis runs async via `Task::perform`. Selection-only analysis available. Statistics are cached and invalidated on content change or row-width change.

---

## Lua Scripting Engine

Feature-gated behind `lua` (default on). Uses `mlua 0.11` with vendored Lua 5.4.

**`LuaScriptEngine`**: Manages a sandboxed Lua environment. Exposes `register_decoder(name, decode_fn)` that Lua scripts call to add inspector entries.

**Sandboxing:** `os`, `io`, `loadfile`, `require`, `dofilesystem` tables are removed from the global environment. Additionally `ffi` (luajit) and `package` are removed. Environment variable `HEXEDIT_LUA_UNSAFE=1` disables sandboxing.

**Script API:**
```lua
-- Register a decoder that appears in the inspector panel
register_decoder("my_type", function(bytes)
    -- bytes: string of raw bytes (up to the requested length)
    -- return: string to display in the inspector
    return string.format("0x%02X", string.byte(bytes, 1))
end)
```

**Entry lifecycle:**
1. `load_script(&path)` — loads and executes script (calls `register_decoder()`)
2. `entries()` — returns all registered decoders as `Vec<InspectorEntry>`
3. Decoders are re-evaluated every frame (the script's function is called with the current cursor bytes)

---

## Pane Grid Layout

Hex editor uses the **Halloy pattern** for pane management: `pane_grid::State<HexPanel>` with movable, splittable, resizable panels.

**Default layout:** Vertical split — 75% hex matrix (left), 25% inspector (right).

**Panel types** (`HexPanelContent`):
- `Matrix` — The main hex byte display (always present)
- `Inspector` — Decoded value inspector (shown by default)
- `PatternList` — Pattern management panel (toggled via toolbar)
- `Statistics` — Byte statistics & entropy panel (toggled via toolbar)

**Operations:**
- Panels can be split (`SplitPane`) up to 8 total
- Panels can be reordered via drag (`PaneDragged`)
- Panels can be closed (`ClosePane`, minimum 1 panel)
- Focused panel receives keyboard input
- Divider resize with configurable gap (`config.pane_gap`)

---

## Minimap

Overview strip at the bottom of the matrix panel shows:
- **File structure** — each row of the minimap represents many file rows (pixel-per-row mapping)
- **Cursor marker** — shows current position in the file
- **Dirty/diff pixels** — shows which regions have been modified
- Cached as `RefCell<Option<MinimapCache>>` to avoid recomputing every frame

---

## Theme System

All colours consolidated into `HexEditorTheme` — a flat struct with ~90+ fields. Two built-in compile-time constants:

- **`DARK_THEME`**: Leather/amber (original). Near-black background `#14110f`, warm text `#d4cabd`, amber accents.
- **`LIGHT_THEME`**: Warm parchment. Background `#F5EDE0`, dark text `#3A3228`, blue selection.

Both themes meet WCAG AA contrast ratio (4.5:1) for normal-size text across all byte colouring schemes (with the intentional exception of `dim_nulls` for `0x00`).

The theme is stored as `&'static HexEditorTheme` on state (a reference to the static constant), so switching themes is just changing the pointer. The `ThemeVariant` enum tracks which is active for the settings UI.

---

## Test Structure

Located in `src/tests/`, using `iced_test::Simulator` for end-to-end view→update→view pipeline tests:

| File | Tests |
|------|-------|
| `mod.rs` | Helpers: `make_state()`, `default_config()`, `send()`. Error state, status message, ParagraphCache integration |
| `editing.rs` | Inline hex editing flow |
| `navigation.rs` | Keyboard navigation, selection extension |
| `search.rs` | Search execution, match navigation, clear |
| `header.rs` | Header display (name, size, bytes/row) |
| `footer.rs` | Footer display (cursor, selection, mode) |
| `goto.rs` | Goto dialog flow |
| `inspector.rs` | Inspector values at cursor |
| `patterns.rs` | Pattern creation, removal |
| `pattern_group.rs` | Group operations (rename, colour cycle, collapsible) |
| `saving.rs` | Save button, dirty state |
| `settings.rs` | Settings modal (theme, scheme, dim nulls) |
| `toolbar.rs` | Toolbar buttons visibility |
| `pane_grid.rs` | Pane split, close, focus |
| `lua_tests.rs` | (feature-gated) Lua script loading, decoder lifecycle |

---

## Usage (library)

```rust
use hexedit::{HexEditorState, HexEditorConfig, HexEditorMessage};
use hexedit::{update, view};

fn main() -> iced::Result {
    iced::application(MyApp::new, MyApp::update, MyApp::view)
        .run()
}

struct MyApp {
    editor: HexEditorState,
    config: HexEditorConfig,
}

// In your app's update:
fn update(&mut self, msg: HexEditorMessage) -> iced::Task<HexEditorMessage> {
    update(&mut self.editor, &self.config, msg)
}

// In your app's view:
fn view(&self) -> iced::Element<'_, HexEditorMessage> {
    view(&self.editor, &self.config)
}
```

The host app provides an `OnSaveFn` callback in `HexEditorConfig` if persistence
should be wired to mod recording or direct file saves.

---

## Usage (standalone binaries)

```bash
# hexedit — Opens with a file dialog (no path given)
cargo run -p hexedit

# Opens a specific file from the command line
cargo run -p hexedit -- path/to/file.bin

# Load Lua scripts from a directory
cargo run -p hexedit -- path/to/file.bin --script-dir ./scripts/
```

---

## Build

```bash
cargo build -p hexedit                          # Default (with Lua)
cargo build -p hexedit --no-default-features     # Without Lua scripting
cargo test -p hexedit                            # Unit + integration tests
cargo test -p hexedit --test '' -- --ignored     # All tests (including slow ones)
```

---

## Feature flags

| Feature | Default | Description |
|---------|---------|-------------|
| `lua`   | on      | Lua scripting engine (vendored Lua 5.4 via `mlua`) |

---

## Keybindings

| Key | Action |
|-----|--------|
| `←` `→` `↑` `↓` | Navigate one byte / one row |
| `Shift` + arrows | Extend selection |
| `Home` / `End` | Jump to start / end of line |
| `Ctrl+Home` / `Ctrl+End` | Jump to document start / end |
| `Page Up` / `Page Down` | Scroll one screenful |
| `F2` / Double-click | Begin editing byte at cursor |
| `Tab` / `Enter` | Commit edit and advance |
| `Esc` | Cancel edit |
| `Ctrl+C` | Copy selection as hex text |
| `Ctrl+V` | Paste hex text from clipboard |
| `Ctrl+F` | Open search overlay |
| `Ctrl+G` | Open goto dialog |
| `Ctrl+E` | Create pattern from selection |
| Hex digit (`0-9 A-F`) | Auto-start editing at cursor |

---

## Dependencies

- **Iced 0.15** — UI framework (wgpu, advanced, canvas, lazy features)
- **gui-widgets** — Reusable Iced widgets (context menu, modal, paragraph cache)
- **mlua 0.11** (optional) — Lua 5.4 scripting via vendored build
- **lru 0.12** — LRU cache for inspector / internal use
- **tokio 1** — Async file I/O
- **rfd 0.17** — File dialogs (standalone binaries)
- **encoding_rs** — Text encoding for write modes and custom encodings
- **serde** + **serde_json** — Pattern import/export serialization
- **once_cell** — Lazy static for inspector entries

---

## LLM Guidance — Conventions & Invariants

### When modifying this code, preserve these invariants:

1. **Domain/UI separation:** Never import Iced widget types into `src/domain/`. Domain types should be pure data structures with no widget dependencies. UI-specific code always goes in `src/ui/`.

2. **State mutation discipline:** All state mutation happens in the `update()` function. The `view()` function is pure — it never mutates state (except reading `Cell<Option<u64>>` via `Cell::take()` for one-frame events).

3. **Provider chain order:** The `fold_color()` provider chain in the matrix rendering has a specific layered order (search → diff → dirty → pattern → scheme → dim → selection). Adding a new provider means inserting it at the correct position. The order determines priority.

4. **Pattern lookup consistency:** After any pattern mutation (create, remove, update colour, update annotation), you MUST call `rebuild_pattern_lookup()` + `recompute_row_annotations()` to keep address-based lookups and annotation rendering in sync.

5. **Vanilla diff freshness:** Call `recompute_vanilla_diff()` after every `provider.write()` call. It's a linear scan so it's cheap, but forgetting it will cause stale diff highlighting.

6. **Statistics invalidation:** Call `invalidate_stats()` whenever file content changes or `bytes_per_row` changes. Statistics are cached and computed async.

7. **Error string handling:** Never use `"."` or empty strings for error cases. The host uses `status_msg` for user-facing messages and `error` for load-time failures. Both should be informative.

8. **Message return discipline:** The `update()` function returns `Task<HexEditorMessage>`. Synchronous mutations return `Task::none()`. Async operations (file I/O, clipboard, analysis) return a `Task::perform(...)`. Never return `Task::none()` from an update that should produce follow-up messages.

9. **Pane count limit:** Maximum 8 panes (`state.panes.len() < 8` check before split). Enforced in the `SplitPane` handler.

10. **Bytes-per-row validation:** Only 8, 16, and 32 are valid. The `SetBytesPerRow` handler enforces this.

11. **ParagraphCache:** The `state.cache` is shared across frames. It's cheaply cloned into the widget each frame. Always use it for shaped text that survives between frames.

12. **Test pattern:** Integration tests use `iced_test::Simulator`: construct state, call `view()`, assert widget tree with `ui.find()`, send messages via `update()`, re-check view. See `tests/` for examples.

13. **Adding a new decoder:**
    - Add a `dec_*` function and optional `enc_*` function in `ui/inspector.rs`
    - Add an `entry()` call in the `ENTRIES` lazy static
    - Add a `call_decode` test in the `#[cfg(test)]` block
    - The host can also add entries via `config.extra_entries` (no code change needed)

14. **When adding a new message variant:**
    - Add the variant to `HexEditorMessage` in `message.rs`
    - Add a handler arm in the `match` in `update.rs`
    - Wire the update handler to any async returns or state mutations
    - If the message changes the view, write an iced_test for it

---

## Project context

hexedit is part of the [Dispel RPG modding toolkit](https://github.com/piotr/dispel-extractor),
a 5-crate Rust workspace. It powers the hex editor tab inside `dispel-gui` and can also
be used standalone for general-purpose binary editing.

**DISPEL®** is a registered trademark. This project is **not affiliated with, endorsed by,
or sponsored by** the trademark owner.

---

## License

Like the rest of the Dispel Extractor workspace, this crate is distributed under
the terms of the project license.
