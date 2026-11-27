# hexedit: a standalone, embeddable hex editor widget for Iced

hexedit is a hex editor library built on [Iced 0.15](https://iced.rs/). It ships as a reusable Rust library, the `hexedit` crate, and one binary, `hexedit`. The [Dispel RPG modding toolkit](https://github.com/piotr/dispel-extractor) uses the library for the hex editing tab in `dispel-gui`. You can also embed the library in any Iced application.

## Table of Contents

1. [Features](#features)
2. [Architecture overview](#architecture-overview)
3. [Domain layer (pure data model)](#domain-layer-pure-data-model)
4. [UI layer (Iced-specific)](#ui-layer-iced-specific)
5. [HexEditorState (aggregate state)](#hexeditorstate-aggregate-state)
6. [HexEditorConfig (host injection)](#hexeditorconfig-host-injection)
7. [HexEditorMessage (enum reference)](#hexeditormessage-enum-reference)
8. [HexMatrix custom widget](#hexmatrix-custom-widget)
9. [Cell colours](#the-coloring-system)
10. [Inspector decoder system](#inspector-decoder-system)
11. [Pattern system](#pattern-system)
12. [Inline editing](#inline-editing-and-writing)
13. [Search and navigation](#search-and-navigation)
14. [Write modes and text encoding](#write-modes-and-text-encoding)
15. [Byte statistics and entropy panel](#byte-statistics-and-entropy-panel)
16. [Pane grid layout](#pane-grid-layout)
17. [Minimap](#minimap)
18. [Theme system](#theme-system)
19. [Test structure](#test-structure)
20. [Usage (library)](#usage-library)
21. [Usage (standalone binaries)](#usage-standalone-binaries)
22. [Build](#build)
23. [Feature flags](#feature-flags)
24. [Keybindings](#keybindings)
25. [Dependencies](#dependencies)
26. [LLM guidance: conventions and invariants](#llm-guidance-conventions-and-invariants)

## Features

### Core editing

- **Virtualized hex matrix:** The address gutter, hex bytes grouped 8+8, the ASCII column and the annotation column appear on screen. The widget draws only rows in the viewport.
- **Inline overwrite editing:** Double-click or press `F2` to edit a byte. Type two hex digits to commit the byte and move to the next one. Press `Tab` or `Enter` to commit and advance. Press `Esc` to cancel.
- **Selection:** Click, drag or press Shift with an arrow key to select a byte range. Arrow keys, `Home`, `End`, `Page Up`, `Page Down`, `Ctrl+Home` and `Ctrl+End` move the cursor.
- **Copy and paste:** `Ctrl+C` copies the selected range as hex text (`DE AD BE EF`). `Ctrl+V` pastes hex text from the clipboard into the buffer.
- **Write bytes:** The `WriteBytes` message supports inspector edits and other host changes.

### Search and navigation

- **Search (`Ctrl+F`):** The tool finds hex byte sequences (`DE AD BE EF` or `DEADBEEF`) and ASCII substrings. It highlights matches in the matrix. It moves to the next or previous match.
- **Goto (`Ctrl+G`):** The tool moves to an address. The input accepts hex (`0xFF`, `FF`), decimal (`255`) and relative offsets (`+10`, `-5`).
- **Bytes-per-row toggle:** The settings modal sets any row width from 1 to 64 bytes. Preset buttons cover 8, 16 and 32. A custom numeric input accepts other widths.

### Data inspector

- **Built-in decoders:** `u8`, `i8`, `u16`, `i16`, `u32`, `i32`, `u64`, `i64`, `f32`, `f64`, ASCII char, UTF-8 char, RGB565 pixel, null-terminated C string and raw hex dump.
- **Editable values:** Each numeric decoder has an inline edit modal. Type a decimal value or a `0x` hex value. The tool writes the bytes in the matching endianness.

### Pattern highlighting

- **Named byte ranges:** Select a range, right-click and choose `Create Pattern`. You can also press `Ctrl+E`. Patterns use a distinct background and foreground colour in the matrix.
- **Repeated (zebra) patterns:** Create a repeating pattern with several instances. The tool adds annotation labels such as `Monster[0]` and `Monster[1]`.
- **Pattern groups:** Group related patterns. Groups support rename, colour cycling and bulk delete.
- **Annotations:** Each pattern can carry text in the annotation column to the right of the ASCII area. The tool highlights annotations for patterns under the cursor.
- **Import and export:** The tool exports patterns and groups to a JSON file and imports them from a JSON file.

### Visual colouring

The settings modal offers five colour schemes:

- **Monochrome:** One colour for all bytes.
- **Nybble:** 18 colour groups by high nibble, one per nybble plus special `0x00` and `0xFF`.
- **Categories:** Six semantic groups, NULL, whitespace, printable, DEL, control and non-ASCII.
- **Rainbow:** A continuous hue gradient across the full `0x00` to `0xFF` range.
- **Heatmap:** Cold to hot, blue, cyan, green, yellow and red.

- **Dim nulls:** The tool can draw `0x00` bytes in a dim colour under any scheme.
- **Dirty and vanilla diff colouring:** Bytes changed in this session use one colour. Bytes that differ from the on-disk snapshot use another.

### Layout and panels

- **Pane grid:** Movable, splittable, resizable panels follow the Halloy style. The default layout splits the hex matrix on the left from the inspector on the right. The pattern list opens as another split pane.
- **Toolbar:** The toolbar holds Save, Goto, Patterns toggle, text export, settings, bytes-per-row toggle and the status message.
- **Minimap:** An overview strip shows file structure, the cursor position marker and dirty or diff pixels.

### Save and integration

- **Host-configurable save:** The host app supplies a `SavePort` callback, a save label and a `can_save` flag. The editor shows them in the toolbar.
- **Dirty tracking:** `BufferProvider` tracks addresses that changed since load.
- **Text export:** The tool exports a formatted hex dump text file. The address gutter, decimal or hex addresses and the ASCII column are configurable.

### File support

- **Library:** `hexedit::HexEditorState::load_from_path(&path)` loads any file into memory.
- **Binaries:** Open a file with a command-line argument (`hexedit path/to/file.bin`) or a file dialog.

## Architecture overview

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
│   ├── config.rs        # HexEditorConfig (7 fields + SavePort port type)
│   ├── domain/save.rs   # SaveRequest / SaveOutcome domain-level save event
│   ├── message.rs       # HexEditorMessage enum (~48 variant categories, ~200+ total)
│   ├── state.rs         # HexEditorState (aggregate state, ~30 fields)
│   ├── main.rs          # Standalone bin: hexedit (file dialog + file path arg)
│   ├── lib.rs           # Public API re-exports
│   └── tests/           # iced_test integration tests (12 files)
└── Cargo.toml
```

The `domain/` directory holds pure Rust data types. It imports no Iced widget types. The `ui/` directory holds all Iced code. This split lets you test and reuse the data model without a UI.

## Domain layer (pure data model)

Every file in `src/domain/` is free of Iced widget types. Unit tests cover them without a running UI.

### `provider.rs`: Byte source abstraction

- **`HexProvider` trait** (`read()`, `write()`, `len()`, `is_writable()`): A common interface for byte sources. It lets a vanilla snapshot and the live editing buffer coexist.
- **`BufferProvider`**: An in-memory `Vec<u8>` with a `BTreeSet<u64>` of dirty addresses. It tracks bytes that changed since load. Key methods: `from_bytes()`, `dirty()`, `dirty_count()`, `clear_dirty()`, `as_slice()`.
- **Rule:** `write()` marks an address dirty only when the new byte differs from the old one.

### `selection.rs`: Cursor and selection model

- **`Selection`**: `{ anchor: u64, cursor: u64 }`. The selected range is `min(anchor,cursor)..=max(anchor,cursor)`. Methods: `single()`, `range()`, `start()`, `end()`, `len()`, `is_single()`, `contains()`, `select()`, `extend()`.
- **`NavDir` enum**: Left, Right, Up, Down, LineStart, LineEnd, PageUp, PageDown, DocumentStart, DocumentEnd.
- **`nav_target(cursor, dir, bytes_per_row, page_rows, max_addr) -> u64`**: A pure function that computes the new cursor. Unit tests cover the edge cases. These include saturation at 0 and clamping to `max_addr`.

### `pattern.rs`: Pattern highlighting

- **`Pattern`**: `{ id, start, end, color_idx, group_id, annotation }`. Serde serializes it.
- **`RepeatedPatternGroup`**: `{ id, label, color_idx }`. It groups related patterns.
- **`RepeatPatternDialog`**: Transient modal state for repeated patterns created from a selection.
- **`PatternExport`**: A versioned JSON envelope (`{ version, groups, patterns }`) for import and export.
- **`pattern_bg(idx)` / `pattern_fg(idx)`**: Palette colours from the 16-colour cyclic palette in `DARK_THEME`.

### `search.rs`: Search engine

- **`SearchMode`**: `Hex` or `Ascii`.
- **`SearchState`**: `{ visible, query, mode, results: Vec<u64>, query_len, current_match, match_set: BTreeSet<u64> }`. The `match_set` holds a precomputed O(log n) lookup table for the renderer.
- **`parse_hex_query(s)`**: Parses `"DE AD BE EF"` or `"DEADBEEF"` into `Vec<u8>`. The parser needs an even digit count.
- **`SearchMatchProvider`**: Implements `CellColorProvider` for search matches in the matrix.

### `goto.rs`: Goto address parsing

- **`GotoState`**: `{ draft, error }`.
- **`parse(cursor, max_addr)`**: Accepts hex (`0xFF`, `FF`), decimal (`255`) and relative values (`+10`, `-5`). It clamps the result to `[0, max_addr]`. Unit tests cover the function.

### `editing.rs`: Inline edit state

- **`EditState`**: `{ addr, draft: String }`. The draft holds 0 to 2 uppercase hex characters. Methods: `push_char()`, `pop_char()`, `is_complete()`, `staged_byte()`. A single-digit draft becomes the low nibble, so `"A"` gives `0x0A`.
- **`InspectorEditState`**: `{ entry_idx, addr, draft, error }`. Modal state for inspector inline edits.

### `write_mode.rs`: Text encoding modes

- **`WriteMode`**: `Hex`, `Ascii`, `Utf8`, `Windows1250`, `EucKr`, `Custom(usize)`.
- **`EncodingEntry`**: `{ label, encoding_name }` for user-added encodings. The host persists them.
- **`encode_text(text, mode, custom) -> Vec<u8>`**: Converts one string character to bytes. The inline text typing path calls it.
- **`is_text_mode(mode) -> bool`**: Returns `true` for every mode except `Hex`.
- **`COMMON_ENCODINGS`**: A static list of 30+ `(label, encoding_rs_name)` pairs for custom encoding entries.
- **`remap_write_mode()`**: Re-indexes `WriteMode::Custom(i)` when items leave the custom list.

### `panel.rs`: Pane grid types

- **`HexPanelContent`**: `Matrix`, `Inspector`, `PatternList`, `Statistics`.
- **`HexPanel`**: A wrapper around `HexPanelContent`.
- **`default_pane_grid() -> pane_grid::State<HexPanel>`**: Creates the default vertical split, 75/25 matrix to inspector.

### `layout.rs`: Structure overlay registry (future)

- **`BinaryLayout` trait**: `layout(bytes) -> Vec<FieldSpan>`.
- **`FieldSpan`**: `{ range, name, ty }`.
- **`LayoutRegistry`**: An extension-keyed lookup table. It is empty in v1. A future version may derive structure overlays from `dispel_core` `#[extractor]` attributes.

### `byte_stats.rs`: Byte statistics and entropy

- **`ByteStatistics`**: Counts, frequencies, min/max, Shannon entropy and structure heuristics.
- **`RowEntropyCache`**: Per-row entropy values for the gutter colour band.
- **`compute_statistics(bytes) -> ByteStatistics`**: Full analysis. It runs asynchronously through `Task::perform`.
- **`compute_row_entropies(bytes, bpr) -> RowEntropyCache`**: Per-row entropy values.
- **`entropy_to_color()`**: Maps an entropy value to a colour for the gutter band.
- **`StructureHeuristic`**: Detects uniform runs, high and low entropy patterns and mixed content.

### `vanilla_diff.rs`: Change tracking

- **`compute_diff(vanilla, current) -> BTreeSet<u64>`**: Compares two byte slices in a linear scan. It returns the addresses that differ.

### `fill_dialog.rs`: Fill selection

- **`FillDialog`**: `{ draft, error }`. It parses a hex byte pattern string to repeat across the selected range.

### `pattern_layout.rs`: Branch connectors

- Types that draw Git-log-style branch connectors in the pattern list panel for grouped patterns.

## UI layer (Iced-specific)

### `update.rs`: Message handler

```rust
pub fn update(
    state: &mut HexEditorState,
    config: &HexEditorConfig,
    message: HexEditorMessage,
) -> Task<HexEditorMessage>
```

The `update()` function is pure and about 1300 lines long. It dispatches every message variant. Synchronous changes return `Task::none()`. File I/O, clipboard and analysis return an async `Task`.

Handler sections:

- Pane grid operations: click, resize, drag, split and close
- Cursor movement and selection: `SelectAt`, `ExtendTo` and `Nav`
- Inline editing: `BeginEdit`, `EditTypeChar`, `EditBackspace`, `EditCancel` and `EditCommit`
- Inspector: copy, begin edit and commit edit
- Save through the config `SavePort`
- Search: open, execute, toggle mode, next, previous and close
- Goto dialog flow
- Pattern create, remove, clear and right-click actions
- Repeated pattern dialog
- Pattern list and group operations
- Pattern import and export through async file dialogs
- Settings: theme, colour scheme, dim nulls, entropy band and minimap
- Write mode and encoding settings
- Copy and paste through the clipboard
- Fill selection
- Extend file: insert bytes at the cursor
- Text export through an async file dialog and write
- Byte statistics analysis through async work
- Navigation centering through `pending_center_on`

### `inspector.rs`: Decoder registry

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

The 15 built-in entries are u8, i8, u16, i16, u32, i32, u64 and i64. The tool also ships f32, f64, ascii, utf8, rgb565, cstr and hex. Numeric entries have `encode`, so you can edit them. The host can add entries through `config.extra_entries`.

### `coloring.rs`: Cell colouring system

The providers form a layered chain. Each one implements `CellColorProvider`:

```rust
pub trait CellColorProvider {
    fn color(&self, addr: u64, byte: u8) -> (Option<Color>, Option<Color>);
}

pub fn fold_color(providers, addr, byte) -> (Option<Color>, Option<Color>);
```

Later providers in the chain override earlier ones when they return `Some`.

Built-in providers in application order:

1. **`SearchMatchProvider`**: highlights search matches and the current match
2. **`DiffVsVanillaProvider`**: highlights bytes that differ from the vanilla snapshot
3. **`DirtyProvider`**: highlights bytes changed in this session
4. **`PatternBgProvider`**: applies pattern background colours
5. **`AnnotationProvider`**: applies annotation foreground for active patterns
6. **`SchemeProvider`**: applies the active `ColorScheme` (Monochrome, Nybble, Categories, Rainbow, Heatmap)
7. **`DimNullsProvider`**: dims `0x00` bytes when enabled
8. **`SelectionProvider`**: highlights the selection range and cursor cell

### `theme/`: Theme system

- **`HexEditorTheme`**: A flat struct with 90+ colour fields. It covers the matrix, header, selection, edit, dirty and diff states. It also covers search matches, annotations, scrollbar, modals and minimap. Other groups are the pattern panel, statistics panel, byte-colouring schemes, pattern palettes and the Iced application palette.
- **`DARK_THEME`**: Near-black background with warm amber and brown tones.
- **`LIGHT_THEME`**: Warm parchment background for accessibility, WCAG AA.
- Both are `const` values. They compile at build time through `const fn hex()`.
- The `ThemeVariant` enum has `Dark` and `Light`, with `Default` on `Dark`.
- The theme holds HSL saturation and lightness parameters for gradient schemes: 0.70 for dark and 0.35 for light.

## HexEditorState (aggregate state)

```rust
pub struct HexEditorState {
    pub path: PathBuf,
    pub name: String,
    pub panes: pane_grid::State<HexPanel>,
    pub pane_focus: pane_grid::Pane,
    pub provider: BufferProvider,
    pub bytes_per_row: u8,                    // 1..=64 (custom via settings modal)
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

- `load_from_path(path)`: builds state from a file. It reads bytes, computes initial entropies and sets the default pane layout.
- `max_addr()`: `provider.len().saturating_sub(1)`
- `recompute_vanilla_diff()`: linear scan. Call it after every write.
- `add_pattern(start, end) -> usize`: creates a pattern and rebuilds lookups
- `remove_pattern(id)`: removes a pattern and orphan groups
- `clear_patterns()`: removes all patterns and groups
- `rebuild_pattern_lookup()`: rebuilds the `pattern_by_addr` BTreeMap
- `recompute_row_annotations()`: rebuilds the annotation map and refreshes active patterns
- `refresh_active_patterns()`: recomputes which patterns contain the cursor
- `invalidate_stats()`: clears cached byte statistics when content changes

## HexEditorConfig (host injection)

```rust
// domain/save.rs — plain data, no iced message types cross this boundary.
pub struct SaveRequest {
    pub path: PathBuf,   // file being edited
    pub bytes: Vec<u8>,  // full current buffer
}

pub enum SaveOutcome {
    Saved(String),  // human-readable success message
    Failed(String), // error description
}

// config.rs — the host-facing ports. The host receives a SaveRequest and
// returns a task producing a SaveOutcome; the editor maps that onto its own
// internal `SavedIntoRecording(SaveOutcome)` message. No hexedit message type
// crosses either boundary.
pub type SavePort = Arc<dyn Fn(SaveRequest) -> Task<SaveOutcome> + Send + Sync>;
pub type WriteModeCallback = Arc<dyn Fn(WriteMode, &[EncodingEntry]) + Send + Sync>; // domain value only

pub struct HexEditorConfig {
    pub pane_gap: u16,                        // default: 4
    pub on_save: Option<SavePort>,            // None hides save button
    pub save_label: String,                   // e.g. "Save into my-mod"
    pub can_save: bool,                       // prerequisites met?
    pub save_hint: String,                    // why save is disabled
    pub extra_entries: Vec<InspectorEntry>,   // host-specific decoders
    pub custom_encodings: Vec<EncodingEntry>, // persisted by host
    pub on_write_mode_changed: Option<WriteModeCallback>,
}
```

Build the config with `HexEditorConfig::new()` and its chainable setters: `pane_gap`, `on_save`, `save_label`, `can_save`, `save_hint`, `extra_entries`, `custom_encodings` and `on_write_mode_changed`. Both embedders use this builder. The standalone binary also uses the shared `standalone_config(...)` helper to keep its defaults aligned with the GUI.

Helper methods: `save_label()`, `has_save()` and `can_save_now(state)`. The last one checks `can_save && on_save.is_some() && dirty_count > 0`.

## HexEditorMessage (enum reference)

There are ~48 variant categories and ~200+ total variants. Every variant uses `#[derive(Debug, Clone)]` and produces `iced::Task<HexEditorMessage>`.

### Pane grid

| Variant | Purpose |
|---------|---------|
| `PaneClicked(pane)` | Set keyboard focus to pane |
| `PaneResized(ResizeEvent)` | Divider dragged |
| `PaneDragged(DragEvent)` | Pane reorder/dock |
| `SplitPane(Axis)` | Split focused pane (max 8 panes) |
| `ClosePane` | Close focused pane |

### Cursor and selection

| Variant | Purpose |
|---------|---------|
| `SetBytesPerRow(u8)` | Set row width (any value 1–64) |
| `BytesPerRowInputChanged(String)` | Draft text of the custom bytes-per-row input |
| `BytesPerRowInputInvalid` | Custom BPR submitted with an out-of-range/non-numeric value |
| `SelectAt(u64)` | Single click: set anchor=cursor=addr |
| `ExtendTo(u64)` | Shift-click/drag: move cursor only |
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
| `SaveIntoRecording` | Trigger save (delegates to `SavePort`) |
| `SavedIntoRecording(SaveOutcome)` | Async save result (domain-level event) |
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

### Pattern list and groups

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

### Write mode and encoding

| Variant | Purpose |
|---------|---------|
| `SetWriteMode(WriteMode)` | Switch mode |
| `OpenEncodingSettings` / `CloseEncodingSettings` | Encoding settings modal |
| `AddCustomEncoding(usize)` | From common list |
| `RemoveCustomEncoding(usize)` | Remove entry |
| `SetCustomEncodings(Vec<EncodingEntry>)` | Bulk replace (deserialize) |

### Copy and paste

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

## HexMatrix custom widget

The widget lives in `src/ui/view/matrix/` (5 files).

### `mod.rs`

Public API: the `HexMatrix` widget struct and the `hex_matrix()` constructor. It wraps the `layout`, `state`, `draw` and `event` modules.

### `layout.rs`

It computes column widths for the address gutter, hex bytes grouped 8+8, ASCII column and annotation column. It also computes row height and total content height for scrolling. It returns sizing metrics for the draw and event handlers.

### `state.rs`

Internal mutable state cached between frames: scroll offset, selection and cursor visual positions, and hot or cold state for click handling.

### `draw.rs` (~984 lines)

The rendering core paints through Iced's `Renderer` on the wgpu backend:

- Address gutter with hex or decimal labels
- Hex byte columns grouped 8+8 with group separator lines
- ASCII column with printable characters (0x20 to 0x7E) and dots for other values
- Annotation column with pattern text
- Colour resolution through the `CellColorProvider` chain
- Selection highlight and cursor cell
- Edit-in-progress highlight
- Scrollbar with search match dots and a cursor dot

### `event.rs` (~605 lines)

Mouse and keyboard event handling:

- Click and drag selection
- Double-click to begin an edit
- Keyboard navigation with arrows, `Home`, `End`, `PageUp`, `PageDown`, `Ctrl+Home` and `Ctrl+End`
- Hex digit typing to start an edit
- Shift with arrows to extend the selection
- Scroll wheel
- Every event defers state mutation to `HexEditorMessage`

**Virtualization:** The widget draws only rows in the viewport. It computes visible rows from the scroll offset and viewport height. It then iterates byte addresses in that range.

## The coloring system

The hex matrix resolves cell colours through a layered provider chain. The application order in `draw.rs` is:

1. **Search match:** `SearchMatchProvider` highlights all bytes in search results with green tones
2. **Diff vs vanilla:** `DiffVsVanillaProvider` shows cumulative changes from the on-disk snapshot
3. **Dirty bytes:** `DirtyProvider` shows bytes changed in this session
4. **Pattern background:** `PatternBgProvider` applies pattern background colours
5. **Pattern annotation:** `AnnotationProvider` adjusts foreground for active pattern annotations
6. **Colour scheme:** `SchemeProvider` applies the base font colour (Monochrome, Nybble, Categories, Rainbow, Heatmap)
7. **Dim nulls:** `DimNullsProvider` dims `0x00` bytes when enabled
8. **Selection:** `SelectionProvider` paints the selection range and cursor cell

Each provider returns `(Option<fg>, Option<bg>)`. `fold_color()` merges the results. Later providers override earlier ones. Search highlights take priority over dirty and diff colours. Those take priority over patterns. Patterns take priority over the base scheme.

`default_byte_colors(scheme, byte, dim_nulls)` is the shared convenience function. The matrix widget and the settings modal palette preview both call it.

## Inspector decoder system

15 built-in `InspectorEntry` values live in the `ENTRIES` lazy static:

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

Each entry has a `decode` function that converts bytes to a display string. Entries with an `encode` function also parse user input into bytes.

The host can add entries through `config.extra_entries`. The update handler picks between built-in and extra entries by index. Indices below `ENTRIES.len()` refer to built-in entries. Higher indices refer to `config.extra_entries`.

## Pattern system

### Data model

- **`Pattern`**: A serializable struct with id, byte range, colour index (0 to 15, cyclic palette), optional group id and optional annotation string.
- **`RepeatedPatternGroup`**: A named group with a shared colour. Child patterns inherit the group colour.

### Lookup structures

- **`pattern_by_addr: BTreeMap<u64, (usize, u8)>`**: Maps every byte address to its pattern id and colour index. A pattern mutation triggers a rebuild.
- **`row_annotations: BTreeMap<u64, Vec<(usize, String)>>`**: Maps row start addresses to annotation segments for the annotation column. A pattern mutation triggers a rebuild.
- **`active_patterns: BTreeSet<usize>`**: Pattern ids whose span includes the cursor. Cursor movement refreshes this set.

### Creation flow

1. The user selects a byte range.
2. The user right-clicks and chooses `Create Pattern`, or presses `Ctrl+E`.
3. `add_pattern(start, end)` creates `Pattern { id, start, end, color_idx }` and rebuilds lookups.

### Repeated pattern flow

1. The user selects a block.
2. The user right-clicks and chooses `Add Repeated Pattern`.
3. The modal asks for a repeat count and a group label.
4. `CommitRepeatedPattern` creates alternating-colour `Pattern` entries. It fills annotations such as `"GroupName[0]"` and `"GroupName[1]"`.
5. A group rename updates child annotations that match `"{old_label}[{digits}]"`.

### Import and export

Patterns export as JSON through the `PatternExport` struct, currently v1. The flow uses an async file dialog.

## Inline editing and writing

### Hex mode (default)

1. A hex digit starts `EditState{ addr: cursor, draft: "" }` and appends the digit.
2. A second digit commits the byte through `provider.write(addr, byte)`. The cursor advances by one. A new `EditState` starts at the next address.
3. `Tab` or `Enter` sends `EditCommit { advance: true }`. It commits a nonempty draft and advances the cursor.
4. `Esc` sends `EditCancel` and discards the draft.

### Text mode (ASCII, UTF-8, Windows-1250, EUC-KR, Custom)

1. A printable character goes through `encode_text()`. The tool writes the bytes and advances the cursor by the encoded byte count.
2. There is no draft. The tool writes each character atomically.
3. `Backspace` moves the cursor left by one byte. It does not delete.
4. `Delete` writes 0x00 at the cursor and advances.

### Inspector edit

1. The user clicks `Edit` on an inspector entry. `BeginInspectorEdit(idx)` opens a modal with the current value.
2. `SetInspectorDraft(String)` updates the draft.
3. `CommitInspectorEdit` calls the entry's `encode(&draft)`, writes bytes and closes the modal.

## Search and navigation

### Search

- Two modes: Hex (`"DE AD BE EF"` or `"DEADBEEF"`) and ASCII (`"hello"`)
- Results are `Vec<u64>` of match start addresses. The tool precomputes them on every query change.
- `match_set: BTreeSet<u64>` supports O(log n) lookup during rendering
- The current match uses a different highlight
- Next and previous wrap around
- The search overlay sits below the toolbar and lives in `search_overlay.rs`

### Goto

- The modal dialog opens with `Ctrl+G`.
- Input parsing: `0xFF` has a hex prefix. The tool treats `FF` as hex when it contains a-f. `255` is decimal. `+10` moves forward. `-5` moves backward.
- On commit, the selection jumps to the target address. The viewport centers on it through `pending_center_on`.
- The flow uses `GotoState.parse(cursor, max_addr)`, which has full unit tests.

## Write modes and text encoding

The `WriteMode` enum controls how keyboard input becomes bytes:

| Mode | Input → Bytes | Use case |
|------|--------------|----------|
| `Hex` | Each digit pair → 1 byte | Classic hex editing |
| `Ascii` | Each char → 1 byte (0x00–0x7F) | Editing ASCII text |
| `Utf8` | Each char → 1–4 bytes | Unicode text |
| `Windows1250` | Each char → 1 byte via encoding_rs | Central European text |
| `EucKr` | Each char → 1–2 bytes via encoding_rs | Korean text |
| `Custom(idx)` | Via `encoding_rs` for user-added encoding | Any encoding_rs-supported encoding |

Users can add custom encodings from a list of 30+ `COMMON_ENCODINGS` through the encoding settings modal.

`WriteMode` serializes as a string label through Serde. `Custom` stores its label rather than its index. Deserialization resolves the index through the custom encoding list.

## Byte statistics and entropy panel

A toggleable panel shows:

- **Byte distribution histogram:** a bar chart of byte value frequencies
- **Shannon entropy:** file level and per row
- **Structure heuristics:** uniform runs, high and low entropy regions, mixed content
- **Entropy colour band** in the address gutter, a visual entropy fingerprint

All analysis runs asynchronously through `Task::perform`. The tool also analyzes the selection only. It caches statistics and invalidates them when content or row width changes.

## Pane grid layout

The editor uses the Halloy pattern for panes: `pane_grid::State<HexPanel>` with movable, splittable, resizable panels.

**Default layout:** a vertical split with 75% hex matrix on the left and 25% inspector on the right.

**Panel types** (`HexPanelContent`):

- `Matrix`: the main hex byte display, always present
- `Inspector`: decoded value inspector, shown by default
- `PatternList`: pattern management panel, opened from the toolbar
- `Statistics`: byte statistics and entropy panel, opened from the toolbar

**Operations:**

- Split a panel with `SplitPane`, up to 8 total
- Reorder panels by drag with `PaneDragged`
- Close panels with `ClosePane`, with at least 1 left
- The focused panel receives keyboard input
- Resize dividers with the configurable `config.pane_gap`

## Minimap

An overview strip at the bottom of the matrix panel shows:

- **File structure:** each minimap row represents many file rows through a pixel-per-row mapping
- **Cursor marker:** the current position in the file
- **Dirty and diff pixels:** the regions that changed

The tool caches the strip as `RefCell<Option<MinimapCache>>` to avoid recomputing every frame.

## Theme system

All colours live in `HexEditorTheme`, a flat struct with 90+ fields. Two constants compile at build time:

- **`DARK_THEME`:** leather and amber, the original theme. Near-black background `#14110f`, warm text `#d4cabd` and amber accents.
- **`LIGHT_THEME`:** warm parchment. Background `#F5EDE0`, dark text `#3A3228` and blue selection.

Both themes meet WCAG AA contrast ratio (4.5:1) for normal-size text across all byte colouring schemes. `dim_nulls` for `0x00` is the exception.

State holds the theme as `&'static HexEditorTheme`, a reference to the static constant. A theme change only updates the pointer. The `ThemeVariant` enum tracks the active theme for the settings UI.

## Test structure

Files in `src/tests/` use `iced_test::Simulator` for end-to-end view-update-view pipeline tests:

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

The host provides a `SavePort` callback in `HexEditorConfig` when persistence should connect to mod recording or direct file saves. The callback receives a domain-level `SaveRequest` with the path and current bytes. It returns a task producing a `SaveOutcome`. It never constructs or matches on `HexEditorMessage`.

## Usage (standalone binaries)

```bash
# hexedit — Opens with a file dialog (no path given)
cargo run -p hexedit

# Opens a specific file from the command line
cargo run -p hexedit -- path/to/file.bin
```

## Build

```bash
cargo build -p hexedit                           # Build the library and binary
cargo test -p hexedit                            # Unit + integration tests
cargo test -p hexedit --test '' -- --ignored     # All tests (including slow ones)
```

## Feature flags

`hexedit` declares no crate-specific Cargo features.

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

## Dependencies

- **Iced 0.15:** UI framework with wgpu, advanced, canvas and lazy features
- **gui-widgets:** reusable Iced widgets, context menu, modal and paragraph cache
- **lru 0.12:** LRU cache for the inspector and internal use
- **tokio 1:** async file I/O
- **rfd 0.17:** file dialogs for the standalone binaries
- **encoding_rs:** text encoding for write modes and custom encodings
- **serde** and **serde_json:** pattern import and export serialization
- **once_cell:** lazy static for inspector entries

## LLM guidance: conventions and invariants

Keep these invariants when you change the code:

1. **Domain and UI separation:** Never import Iced widget types into `src/domain/`. Domain types hold pure data with no widget dependencies. Put UI code in `src/ui/`.

2. **State mutation discipline:** All state mutation happens in `update()`. `view()` stays pure. It never mutates state, apart from reading `Cell<Option<u64>>` through `Cell::take()` for one-frame events.

3. **Provider chain order:** The `fold_color()` chain in the matrix renderer has a fixed order: search, diff, dirty, pattern, scheme, dim and selection. A new provider must go in the right position. The order sets priority.

4. **Pattern lookup consistency:** After any pattern mutation (create, remove, colour change or annotation change), call `rebuild_pattern_lookup()` and `recompute_row_annotations()`. These keep address lookups and annotation rendering in sync.

5. **Vanilla diff freshness:** Call `recompute_vanilla_diff()` after every `provider.write()` call. The linear scan is cheap. If you skip it, the diff highlighting becomes stale.

6. **Statistics invalidation:** Call `invalidate_stats()` when file content or `bytes_per_row` changes. The tool caches statistics and computes them through async work.

7. **Error string handling:** Do not use `"."` or empty strings for errors. The host shows `status_msg` to the user. `error` records load-time failures. Both should be informative.

8. **Message return discipline:** `update()` returns `Task<HexEditorMessage>`. Synchronous mutations return `Task::none()`. Async work (file I/O, clipboard, analysis) returns `Task::perform(...)`. Do not return `Task::none()` from an update that should produce follow-up messages.

9. **Pane count limit:** The maximum is 8 panes. Before a split, the `SplitPane` handler checks `state.panes.len() < 8`.

10. **Bytes-per-row validation:** Any value in `MIN_BYTES_PER_ROW..=MAX_BYTES_PER_ROW` (1 to 64) is valid. The `SetBytesPerRow` handler enforces the range. The settings modal custom input goes through `parse_bpr()` and shows a status message for invalid values. A BPR change also re-lays out pattern annotations through `recompute_row_annotations()`. The tool keys annotations by `row * bpr`.

11. **ParagraphCache:** The app shares `state.cache` across frames. The widget gets a cheap clone each frame. Use it for shaped text that survives between frames.

12. **Test pattern:** Integration tests use `iced_test::Simulator`. Construct state, call `view()`, assert the widget tree with `ui.find()`, send messages through `update()` and check the view again. See `tests/` for examples.

13. **New decoder:**

    - Add a `dec_*` function and an optional `enc_*` function in `ui/inspector.rs`
    - Add an `entry()` call in the `ENTRIES` lazy static
    - Add a `call_decode` test in the `#[cfg(test)]` block
    - The host can also add entries through `config.extra_entries` without a code change

14. **New message variant:**

    - Add the variant to `HexEditorMessage` in `message.rs`
    - Add a handler arm to the `match` in `update.rs`
    - Connect async returns and state mutations to the handler
    - Write an iced_test for the message if it changes the view

## Project context

hexedit is part of the [Dispel RPG modding toolkit](https://github.com/piotr/dispel-extractor), a 6-crate Rust workspace: root `dispel-extractor`, `dispel-encodings`, `dispel-gui`, `dispel-macros`, `gui-widgets` and `hexedit`. It powers the hex editor tab inside `dispel-gui`. You can also use it standalone for general binary editing.
