# Dispel GUI — Redesign Plan

## Vision

A modern, intuitive, and highly moddable game data editor for the Dispel game. The application should feel like a professional tool — think **SQLite Browser** meets **Unity Inspector** — with discoverable navigation, instant feedback, and zero friction between browsing data and editing it.

---

## Current State Summary

| Metric | Value |
|--------|-------|
| Total tabs | 34 |
| `app.rs` size | 3,043 lines (god object) |
| Total code | ~14,600 lines |
| Generic editors | 2 of 34 (6%) |
| Hand-written editors | ~26 |
| Disabled features | Database tab |
| Undo/redo | None |
| Search | Per-tab only (SpriteBrowser) |
| Validation | None (silent parse failures) |

**What works:** Generic editor infrastructure, DbViewer, timestamped backups, lookup dropdowns, async I/O, consistent message naming.

**What's broken:** Monolithic `app.rs`, 94% of editors not using generic infrastructure, no global search, no undo, no validation, inconsistent navigation patterns, Database tab disabled.

---

## Design Principles

### 1. Data-Driven, Not Code-Driven
Every file type should be defined by **data** (registry entry), not **code** (hand-written editor). The registry already exists in `dispel-core` — the GUI should consume it entirely.

### 2. Progressive Disclosure
Show what the user needs, when they need it. Start with a file browser, drill into records, then edit fields. Don't overwhelm with 34 tabs upfront.

### 3. Consistent Interaction Model
One way to browse files, one way to edit records, one way to save. No special cases.

### 4. Zero Data Loss
Auto-save drafts, undo/redo, conflict detection, backup management.

### 5. Discoverable
Global search, command palette, contextual help, inline documentation.

---

## Architecture Redesign

### 3-Layer Architecture

```
┌─────────────────────────────────────────┐
│              Presentation               │
│  (iced widgets, themes, animations)     │
├─────────────────────────────────────────┤
│              Application                │
│  (state machines, commands, navigation) │
├─────────────────────────────────────────┤
│              Domain                     │
│  (EditableRecord, lookups, validation)  │
└─────────────────────────────────────────┘
```

**Key changes:**
- `app.rs` → split into focused modules: `state.rs`, `navigation.rs`, `commands.rs`
- Message enum → split into per-domain enums with a top-level `AppMessage` wrapper
- Tab enum → replaced by a **workspace** model with dynamic tab creation

### Workspace Model

Instead of a fixed 34-tab sidebar, use a **workspace** with dynamic tabs:

```
┌──────────────────────────────────────────────────────────┐
│  [📁 Explorer] [🔍 Search] [⚙️ Settings]                 │
├──────────┬───────────────────────────────────────────────┤
│          │                                               │
│ Explorer │  [weaponItem.db] [Monster.db] [Map.ini]  ✕    │
│          │  ─────────────────────────────────────────    │
│ 📂 Dispel│                                               │
│  ├─ 📂 CharacterInGame                                  │
│  │  ├─ 📄 weaponItem.db  ← click → opens tab            │
│  │  ├─ 📄 HealItem.db                                   │
│  │  └─ ...                                              │
│  ├─ 📂 MonsterInGame                                    │
│  ├─ 📂 Ref                                              │
│  └─ 📂 NpcInGame                                        │
│          │                                               │
│          │  Active tab content (editor, viewer, etc.)    │
│          │                                               │
└──────────┴───────────────────────────────────────────────┘
```

**Benefits:**
- No hardcoded tab list — new file types appear automatically
- Multiple files open simultaneously
- File tree mirrors actual game directory structure
- Users learn the game's file organization naturally

---

## UI/UX Redesign

### 1. File Explorer (replaces sidebar tabs)

A tree view of the game directory with:
- **Icons per file type** (database, INI, REF, script, map, sprite)
- **Color coding** by file type
- **Search/filter** box at top
- **Context menu** on right-click: Open, Extract, Validate, Show in OS
- **Recent files** quick-access section
- **File type badges** showing extractable/patchable status

```
📂 fixtures/Dispel/
  📂 CharacterInGame/
    🗃️ weaponItem.db        [extract ✓] [patch ✓]
    🗃️ HealItem.db          [extract ✓] [patch ✓]
    🗃️ STORE.DB             [extract ✓] [patch ✓]
  📂 MonsterInGame/
    🗃️ Monster.db           [extract ✓] [patch ✓]
    📄 Monster.ini           [extract ✓] [patch ✓]
    📄 Mondun01.ref          [extract ✓] [patch ✓]
  📂 Map/
    🗺️  cat1.map             [extract ✓] [patch ✗]
    🖼️  cat1.gtl             [extract ✓] [patch ✗]
    🖼️  cat1.btl             [extract ✓] [patch ✗]
```

### 2. Unified Editor View

All file types use the same editor layout, configured by the registry:

```
┌─────────────────────────────────────────────────────────────┐
│ weaponItem.db — 87 records                    [💾 Save] [↩] │
├──────────────┬──────────────────────────────────────────────┤
│ 🔍 Filter... │  ID  Name              Price  ATK  DEF  MAG  │
│ ──────────── │ ──────────────────────────────────────────── │
│ ──────────── │ > 0   Short Sword        10g   5    3    0   │
│ [All]        │   1   Long Sword         25g  12    8    0   │
│ [Weapons]    │   2   Broad Sword        45g  18   14    0   │
│ [Armor]      │   3   Leather Armor      15g   0   10    0   │
│              │   4   Chain Mail         80g   0   25    0   │
│              │   ...                                        │
│              │                                              │
│ ──────────── ├──────────────────────────────────────────────┤
│              │  ┌─ Record #0: Short Sword ────────────────┐ │
│              │  │ Name:        [Short Sword__________]    │ │
│              │  │ Description: [A basic sword_________]    │ │
│              │  │ Base Price:  [10____________________] g  │ │
│              │  │ Attack:      [5_____________________]    │ │
│              │  │ Defense:     [3_____________________]    │ │
│              │  │ Req STR:     [2_____________________]    │ │
│              │  │ Req AGI:     [1_____________________]    │ │
│              │  │ Req WIS:     [0_____________________]    │ │
│              │  │ HP Bonus:    [0_____________________]    │ │
│              │  │ MP Bonus:    [0_____________________]    │ │
│              │  │ Durability:  [100__________________]    │ │
│              │  └─────────────────────────────────────────┘ │
└──────────────┴──────────────────────────────────────────────┘
```

**Features:**
- **Spreadsheet view** for bulk editing (like DbViewer but for any file type)
- **Inspector panel** for detailed single-record editing
- **Split view** toggle (list + inspector side by side)
- **Column sorting** by clicking headers
- **Inline editing** in spreadsheet mode
- **Multi-select** for batch operations
- **Filter bar** with quick filters and free-text search

### 3. Multi-File Editor (for .ref, .dlg, .pgp types)

```
┌─────────────────────────────────────────────────────────────┐
│ MonsterRef — Mondun01.ref                      [💾 Save]    │
├────────────┬──────────────────┬─────────────────────────────┤
│ 📁 Files   │ 📋 Records (24)  │  ┌─ Record #3 ───────────┐ │
│            │                  │  │ File ID:    [1______]   │ │
│ Mondun01.ref│ ─────────────── │  │ Monster:    [Goblin▼]   │ │
│ Mondun02.ref│ ID  Monster  XY │  │ Pos X:      [12____]   │ │
│>Mondun03.ref│ ─────────────── │  │ Pos Y:      [8_____]   │ │
│ Monmap1.ref │ 0   Goblin  5,3 │  │ Event ID:   [0_____]   │ │
│ Monmap2.ref │ 1   Orc    12,7 │  │ Loot 1:     [5▼][3__] │ │
│            │ 2   Skeleton 8,2 │  │ Loot 2:     [0▼][0__] │ │
│            │ 3   Goblin  15,1 │  │ Loot 3:     [0▼][0__] │ │
│            │ ...              │  └────────────────────────┘ │
│ [+ Scan]   │ [+] [−]          │                             │
└────────────┴──────────────────┴─────────────────────────────┘
```

### 4. Command Palette (Ctrl+P / Cmd+P)

Quick access to any action:

```
┌─────────────────────────────────────────────────┐
│ > extract weaponItem.db                         │
│                                                 │
│ 📄 Extract: weaponItem.db                       │
│ 📄 Extract: HealItem.db                         │
│ 📄 Extract: Monster.db                          │
│ 🔧 Patch: weaponItem.db                         │
│ 📋 Validate: weapons.json                       │
│ 📊 Open: DbViewer                               │
│ 🗺️  Render: cat1.map                            │
└─────────────────────────────────────────────────┘
```

### 5. Global Search (Ctrl+Shift+F)

Search across all loaded data:

```
┌─────────────────────────────────────────────────┐
│ 🔍 "Short Sword"                                │
│                                                 │
│ weaponItem.db — Record #0                       │
│   Name: "Short Sword"                           │
│                                                 │
│ STORE.DB — Record #12                           │
│   product: WeaponItem.db #0 (Short Sword)       │
│                                                 │
│ 2 results in 2 files                            │
└─────────────────────────────────────────────────┘
```

---

## Technical Implementation Plan

### Phase 1: Foundation (Week 1-2)

**Goal:** Split `app.rs`, establish workspace model

1. **Split `app.rs`** into:
   - `state.rs` — App state struct, initialization
   - `navigation.rs` — Tab/workspace management, routing
   - `commands.rs` — Async task execution, file I/O
   - `messages.rs` — Message enum (split into domains)

2. **Workspace model:**
   - Replace `Tab` enum with `Workspace` struct
   - Dynamic tab creation from file paths
   - Tab close, reorder, pin

3. **File Explorer:**
   - Tree view of game directory
   - File type icons and status badges
   - Double-click to open in editor

### Phase 2: Generic Editor Completion (Week 3-4)

**Goal:** Migrate all 26 hand-written editors to generic infrastructure

1. **Implement `EditableRecord`** for all remaining types:
   - Monster, HealItem, MiscItem, EditItem, EventItem
   - Magic, Store, PartyRef, PartyIni, NpcIni
   - Dialog, DialogueText, DrawItem, EventIni, EventNpcRef
   - ExtraIni, ExtraRef, MapIni, MessageScr, NpcRef
   - PartyLevelDb, QuestScr, WaveIni, ChData

2. **Enhance generic view:**
   - Spreadsheet mode (bulk table editing)
   - Column sorting and filtering
   - Multi-select and batch operations

3. **Validation framework:**
   - Field-level validation on input
   - Visual feedback (red borders, error tooltips)
   - Pre-save validation with error summary

### Phase 3: UX Enhancements (Week 5-6)

**Goal:** Polish the user experience

1. **Undo/Redo:**
   - Command pattern for all edits
   - Ctrl+Z / Ctrl+Y support
   - Edit history panel

2. **Auto-save drafts:**
   - Save unsaved changes to temp files
   - Restore on app restart
   - Conflict detection if file changed externally

3. **Command palette:**
   - Ctrl+P quick access
   - Fuzzy matching
   - Recent actions

4. **Global search:**
   - Index all loaded catalogs
   - Cross-file search results
   - Click to navigate to result

### Phase 4: Advanced Features (Week 7-8)

**Goal:** Power-user features

1. **Diff view:**
   - Compare current state with file on disk
   - Compare with backup version
   - Visual diff highlighting

2. **Batch operations:**
   - Find & replace across records
   - Bulk value changes
   - Import from CSV/JSON

3. **Plugin system:**
   - Custom field renderers
   - Custom validators
   - Custom export formats

4. **Settings panel:**
   - Theme selection (light/dark/custom)
   - Editor preferences (font size, tab width)
   - Backup retention policy
   - File association settings

---

## Component Specifications

### File Explorer

| Property | Value |
|----------|-------|
| Component | `FileTree` |
| Data source | `std::fs::read_dir` + registry type detection |
| Features | Expand/collapse, search, context menu, drag-drop |
| State | Expanded paths, selected path, search filter |

### Editor View

| Property | Value |
|----------|-------|
| Component | `EditorView<R: EditableRecord>` |
| Layout modes | Spreadsheet, Inspector, Split |
| Data source | `GenericEditorState<R>` or `MultiFileEditorState<R>` |
| Features | Sort, filter, inline edit, multi-select, undo |

### Command Palette

| Property | Value |
|----------|-------|
| Component | `CommandPalette` |
| Trigger | Ctrl+P |
| Data source | Registered commands + recent files + file types |
| Features | Fuzzy search, keyboard navigation, action preview |

---

## Migration Strategy

### Backward Compatibility
- Keep existing `Tab` enum during transition
- Old tab URLs work via redirect to workspace
- All existing editors continue to function

### Incremental Migration
1. Migrate simple editors first (WeaponEditor, HealItemEditor)
2. Migrate multi-file editors (MonsterRefEditor, NpcRefEditor)
3. Migrate complex editors (StoreEditor, ChestEditor, SpriteBrowser)
4. Migrate DbViewer (keep as-is, it's already excellent)

### Rollout
- Feature flag: `workspace_mode = false` by default
- Users can opt-in via settings
- After stabilization, make default

---

## File Structure (After Refactor)

```
dispel-gui/src/
├── main.rs                    # Entry point (60 lines)
├── app/
│   ├── mod.rs                 # App struct, init
│   ├── state.rs               # State management
│   ├── navigation.rs          # Workspace, tabs, routing
│   ├── messages.rs            # Message enums
│   └── update/                # Update handlers by domain
│       ├── mod.rs
│       ├── editor.rs
│       ├── explorer.rs
│       ├── db_viewer.rs
│       └── commands.rs
├── view/
│   ├── mod.rs                 # Root view dispatcher
│   ├── sidebar.rs             # File explorer tree
│   ├── editor/
│   │   ├── mod.rs             # Editor view factory
│   │   ├── spreadsheet.rs     # Table view
│   │   ├── inspector.rs       # Detail panel
│   │   └── generic.rs         # Field input builders
│   ├── db_viewer.rs           # SQLite viewer (keep as-is)
│   ├── sprite_browser.rs      # Sprite browser (keep as-is)
│   ├── command_palette.rs     # Ctrl+P quick access
│   └── settings.rs            # Settings panel
├── components/
│   ├── mod.rs
│   ├── file_tree.rs           # Directory tree widget
│   ├── data_table.rs          # Sortable/filterable table
│   ├── status_bar.rs          # Bottom status bar
│   └── toolbar.rs             # Action toolbar
├── theme/
│   ├── mod.rs
│   ├── palette.rs             # Color definitions
│   └── styles.rs              # Component styles
├── generic_editor.rs           # GenericEditorState (from dispel-core)
└── utils.rs                    # Helper functions
```

---

## Success Metrics

| Metric | Current | Target |
|--------|---------|--------|
| `app.rs` lines | 3,043 | < 200 |
| Generic editors | 2 (6%) | 34 (100%) |
| Hand-written editor code | ~8,000 lines | ~500 lines (EditableRecord impls only) |
| Tabs to find a file | Up to 34 clicks | 2 clicks (tree → file) |
| Undo support | None | Full |
| Validation | None | Per-field + pre-save |
| Cross-file search | None | Global |
| Build warnings | 3+ | 0 |

---

## Future Feature Roadmap

### Core Library Enhancements

#### 1. Derive Macro for `EditableRecord`

Instead of hand-writing 23-field `get_field`/`set_field` match blocks for every record type, a procedural derive macro generates them automatically:

```rust
#[derive(EditableRecord)]
#[field(name, label = "Name:", widget = String)]
#[field(description, label = "Description:", widget = String)]
#[field(base_price, label = "Base Price:", widget = Integer)]
#[field(mon_id, label = "Monster:", widget = Lookup("monster_names"))]
#[field(loot1_item_type, label = "Loot 1 Type:", widget = Enum)]
pub struct MonsterRef { ... }
```

**What it generates:**
- `field_descriptors()` — static array of `FieldDescriptor` from `#[field(...)]` attributes
- `get_field()` — match block mapping field names to string values
- `set_field()` — match block parsing strings back into typed fields
- `list_label()` — default format string, overridable with `#[label(format = "...")]`

**Implementation approach:**
- Use `proc-macro2`, `quote`, and `syn` crates
- Parse struct fields and their `#[field(...)]` attributes
- Generate the `EditableRecord` trait impl
- Handle enum fields with `widget = Enum` by referencing the enum's variants
- Handle lookup fields with `widget = Lookup("key")` by storing the lookup key

**Impact:** Eliminates ~2,000 lines of boilerplate across all 28 editor impls. Each editor becomes a 10-15 line derive annotation block.

#### 2. Round-Trip Integrity Testing

Property-based tests that verify extract→patch produces byte-identical output for every file type, using both synthetic and real fixture data:

```rust
#[cfg(test)]
mod round_trip_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn round_trip_weapons_synthetic(items in prop::collection::vec(any::<WeaponItem>(), 0..100)) {
            let temp = tempfile::NamedTempFile::new().unwrap();
            WeaponItem::save_file(&items, temp.path()).unwrap();
            let loaded = WeaponItem::read_file(temp.path()).unwrap();
            prop_assert_eq!(items, loaded);
        }

        #[test]
        fn round_trip_weapons_fixture() {
            let path = "fixtures/Dispel/CharacterInGame/weaponItem.db";
            let original = std::fs::read(path).unwrap();
            let data = WeaponItem::read_file(path).unwrap();
            let temp = tempfile::NamedTempFile::new().unwrap();
            WeaponItem::save_file(&data, temp.path()).unwrap();
            assert_eq!(original, std::fs::read(temp.path()).unwrap());
        }
    }
}
```

**Coverage strategy:**
- Every file type gets a fixture round-trip test (real game files)
- Binary types (.db, .ref) get synthetic property tests
- Text types (.ini, .scr) get encoding round-trip tests
- CI runs all tests on every PR

**Impact:** Catches silent data corruption before it reaches users. Ensures no regression when parsers or serializers change.

#### 3. Diff Engine

Compare two versions of any file type and produce a minimal, human-readable diff — then apply that diff as a patch:

```bash
# Compare two binary files
dispel-extractor diff original.db modified.db --format text

# Output:
# --- original.db
# +++ modified.db
# @@ Record #0 @@
# -  attack: 5
# +  attack: 50
# @@ Record #3 @@
# -  name: "Rusty Sword"
# +  name: "Iron Sword"
# -  base_price: 5
# +  base_price: 25

# Generate a JSON patch file
dispel-extractor diff original.db modified.db --format json > changes.patch

# Apply the patch to a third file
dispel-extractor apply-patch base.db changes.patch --output new.db

# Apply with dry-run (preview only)
dispel-extractor apply-patch base.db changes.patch --dry-run
```

**Patch format (JSON):**
```json
{
  "file_type": "weapons",
  "operations": [
    { "op": "modify", "index": 0, "field": "attack", "from": 5, "to": 50 },
    { "op": "modify", "index": 3, "field": "name", "from": "Rusty Sword", "to": "Iron Sword" },
    { "op": "modify", "index": 3, "field": "base_price", "from": 5, "to": 25 },
    { "op": "add", "index": 150, "record": { "id": 150, "name": "New Item", ... } },
    { "op": "remove", "index": 42 }
  ]
}
```

**Implementation approach:**
- Deserialize both files into `Vec<T>`
- Compute diff using a record-level comparison (by `id` field when available)
- Generate minimal operations (modify, add, remove)
- Apply operations with validation before writing

**Impact:** Enables precise change tracking, mod sharing, and conflict resolution.

#### 6. Pipeline Support (CLI)

Chain commands with stdin/stdout for complex data transformations, enabling powerful one-liners:

```bash
# Extract, filter with jq, and patch back
dispel-extractor extract -i weapons.db \
  | jq '{ _meta: ._meta, data: [.data[] | select(.attack > 50)] }' \
  | dispel-extractor patch -t weapons.db --stdin --in-place

# Extract multiple files and merge into a single JSON
for f in fixtures/Dispel/CharacterInGame/*.db; do
  dispel-extractor extract -i "$f"
done | jq -s 'add' > all_items.json

# Bulk rename: change all items with "Sword" in name to "Blade"
dispel-extractor extract -i weapons.db \
  | jq '{ _meta: ._meta, data: [.data[] | .name = (.name | gsub("Sword"; "Blade"))] }' \
  | dispel-extractor patch -t weapons.db --stdin

# Export to CSV for spreadsheet editing, then re-import
dispel-extractor extract -i weapons.db \
  | jq -r '.data[] | [.id, .name, .base_price, .attack] | @csv' \
  > weapons.csv
# ... edit in LibreOffice ...
cat weapons.csv | csv2json | dispel-extractor patch -t weapons.db --stdin
```

**Implementation approach:**
- `--stdin` flag on `patch` reads JSON from stdin instead of a file
- `--format csv` on `extract` outputs CSV instead of JSON
- `--format tsv` for tab-separated output
- `--quiet` flag suppresses stderr status messages for clean piping
- Auto-detect stdin vs file when `--input` is `-`

**Impact:** Enables power users to combine dispel-extractor with standard Unix tools (jq, grep, sed, awk, csvkit) for complex batch operations.

---

### Visual & Interactive Features

#### 8. Visual Map Editor

Render the actual isometric map with tiles, then click to place, move, or edit monsters, NPCs, chests, and events directly on the map canvas:

```
┌─────────────────────────────────────────────────────────────┐
│ 🗺️  cat1.map                                    [Layers ▼] │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  ░░░▓▓▓░░░▒▒▒░░░▓▓▓░░░                            │   │
│  │  ░░▓👾▓▓░▒🧙▒▒░░▓▓▓░░   ← Click 👾 to edit props  │   │
│  │  ░░░▓▓▓░░░▒👤▒░░░▓▓▓░░                             │   │
│  │  ░░░░░░░░░░░░░░░░░░░░░                             │   │
│  │  ░░📦░░░░░░░░░░░░░░░░░   ← Click 📦 to edit chest  │   │
│  │  ░░░░░░░░░░░░░░░░░░░░░                             │   │
│  │  ░░░░░░░░░░░🚪░░░░░░░░░   ← Click 🚪 for event    │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  ┌─ Selected: 👾 Goblin at (12, 8) ──────────────────────┐ │
│  │ Monster: [Goblin ▼]  Event: [0____]                   │ │
│  │ Loot 1: [Short Sword ▼]  Loot 2: [—]  Loot 3: [—]     │ │
│  │ [Delete] [Duplicate] [Move to...]                     │ │
│  └───────────────────────────────────────────────────────┘ │
│                                                             │
│  Layers: [✓ Ground] [✓ Buildings] [✓ Monsters] [✓ NPCs]    │
│          [✓ Chests] [✓ Events] [✗ Collision]               │
└─────────────────────────────────────────────────────────────┘
```

**Features:**
- **Isometric rendering** using viewport-based tile loading (NOT full-image rendering — see constraints below)
- **Entity overlays** — monsters (red), NPCs (green), chests (blue), events (yellow)
- **Click to select** — opens inspector panel for the entity
- **Drag to move** — reposition entities on the map
- **Layer toggles** — show/hide entity types, collision grid, event zones
- **Mini-map** — overview with viewport rectangle
- **Zoom & pan** — mouse wheel zoom, middle-click drag pan
- **Grid snap** — optional tile alignment
- **Multi-select** — box-select multiple entities for batch edit

**Implementation approach:**
- Use iced's `canvas` API for isometric tile rendering
- **Viewport-based tile loading** — never render the full map (300MB+). Load only tiles visible in the current viewport at the current zoom level.
- **Tile cache with LRU eviction** — max 50MB cache. Evict tiles that scroll out of view.
- **Mipmap levels** — pre-generate tile atlases at multiple zoom levels (1x, 0.5x, 0.25x, 0.125x). Start with lowest resolution, progressively load higher as user zooms in.
- **Thumbnail mode** — at maximum zoom-out, render a single low-res overview image (not individual tiles).
- Reuse existing `map::tileset::extract()` for individual tile pixel data
- Overlay entities as interactive canvas shapes
- Inspector panel reuses generic editor infrastructure
- Entity data comes from `MonsterRef`, `NpcRef`, `ExtraRef` records

**⚠️ Resource constraints:**
- `map.map` files produce 300MB+ rendered images — **never load the full image into memory**
- Target memory usage: < 200MB for map rendering (including tile cache)
- Target frame rate: 30fps during pan/zoom (viewport updates only)
- Use `image::imageops::resize` for thumbnail generation, not full render pipeline
- Consider background thread for tile decoding to avoid blocking the UI thread

**Impact:** Transforms the editing experience from abstract table rows to visual, intuitive map manipulation — the single most impactful feature for modders.

#### 9. Sprite Animation Timeline

Preview `.spr` files with playback controls, frame inspection, and export:

```
┌─────────────────────────────────────────────────────────────┐
│ 🎬 ShieldB1.spr — 3 sequences, 24 frames       [▶ Play 1x] │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │                                                     │   │
│  │                                                     │   │
│  │              [Sprite Frame Preview]                 │   │
│  │                                                     │   │
│  │                                                     │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  Sequence: [Attack ▼]    Frame: 3 / 8    Size: 64×64      │
│                                                             │
│  ◀◀  ◀  ⏸  ▶  ▶▶     [🔁 Loop]  [📐 Show Origin]          │
│  ──●────●────●────●────●────●────●────●──                  │
│  0    1    2    3    4    5    6    7                      │
│                                                             │
│  ┌─ Frame Properties ────────────────────────────────────┐ │
│  │ Width: 64   Height: 64   Origin: (32, 48)             │ │ │
│  │ File offset: 0x1A40   Pixel data: 8,192 bytes         │ │ │
│  │ [Export PNG] [Export All Frames] [Copy to Clipboard]   │ │
│  └───────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

**Features:**
- **Sequence navigation** — dropdown or tabs for animation sequences
- **Frame scrubber** — draggable timeline with frame thumbnails
- **Playback controls** — play, pause, step forward/back, loop, speed
- **Origin point visualization** — crosshair showing anchor offset
- **Frame inspector** — dimensions, origin, file offset, pixel data size
- **Export options** — single frame PNG, all frames as sprite sheet, GIF animation
- **Batch preview** — grid view of all sequences at once

**Implementation approach:**
- Use existing `sprite.rs` parser for sequence/frame data
- Render frames using iced's `image::Handle` from in-memory pixel buffers
- Timeline widget built with iced's `canvas` or custom widget
- Export via `image` crate (PNG, GIF)

**Impact:** Essential for understanding sprite animations, debugging visual issues, and creating custom sprite content.

#### 10. Mod Packaging & Distribution

Create, validate, version, and share mods directly from the GUI:

```
┌─────────────────────────────────────────────────────────────┐
│ 📦 Mod Manager                                              │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Installed Mods:                              [+ New Mod]   │
│  ┌───────────────────────────────────────────────────────┐ │
│  │ ⚔️  Weapon Rebalance v1.2                [✓ Active]  │ │
│  │    Files: weaponItem.db, HealItem.db                  │ │
│  │    Changes: 23 records modified                       │ │
│  │    Conflicts: None                                    │ │
│  │    [Disable] [Edit] [Share]                           │ │
│  ├───────────────────────────────────────────────────────┤ │
│  │ 🗺️  New Dungeon Map v0.3               [✗ Inactive] │ │
│  │    Files: cat1.map, Mondun26.ref, Event.ini           │ │
│  │    Changes: 1 new map, 15 monster placements          │ │
│  │    Conflicts: Overwrites Mondun01.ref                 │ │
│  │    [Enable] [Edit] [Share]                            │ │
│  ├───────────────────────────────────────────────────────┤ │
│  │ 🎨 HD Tileset v2.0                     [✓ Active]   │ │
│  │    Files: cat1.gtl, cat1.btl                          │ │
│  │    Changes: 2 tileset files replaced                  │ │
│  │    Conflicts: None                                    │ │
│  │    [Disable] [Edit] [Share]                           │ │
│  └───────────────────────────────────────────────────────┘ │
│                                                             │
│  ┌─ Create New Mod ──────────────────────────────────────┐ │
│  │ Name: [Weapon Rebalance________]                      │ │
│  │ Version: [1.2____]  Author: [YourName____]            │ │
│  │ Description: [Rebalances weapon stats for...]         │ │
│  │                                                       │ │
│  │ Tracked files:                                        │ │
│  │  ✓ CharacterInGame/weaponItem.db (23 changes)         │ │
│  │  ✓ CharacterInGame/HealItem.db (5 changes)            │ │
│  │                                                       │ │
│  │ [Validate] [Package as .zip] [Publish to Community]   │ │
│  └───────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

**Features:**
- **Change tracking** — automatically tracks which files have been modified since last save
- **Conflict detection** — warns when two mods modify the same file
- **Enable/disable toggle** — activates or deactivates mods without deleting data
- **Mod validation** — checks that all referenced files exist and are valid
- **Packaging** — exports mod as a `.zip` with manifest (`mod.json`)
- **Import** — installs mods from `.zip` files, with conflict warnings
- **Version management** — semantic versioning, changelog support

**Mod manifest format (`mod.json`):**
```json
{
  "name": "Weapon Rebalance",
  "version": "1.2.0",
  "author": "YourName",
  "description": "Rebalances weapon stats for better late-game progression",
  "dependencies": [],
  "files": [
    { "path": "CharacterInGame/weaponItem.db", "hash": "sha256:abc123..." },
    { "path": "CharacterInGame/HealItem.db", "hash": "sha256:def456..." }
  ]
}
```

**Impact:** Enables a modding community by providing a standardized way to create, share, and install mods with safety checks.

#### 12. Audio Playback

Play `.snf` files directly in the GUI with a waveform visualizer and export to WAV:

```
┌─────────────────────────────────────────────────────────────┐
│ 🔊 Audio — Wave.ini                                         │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  🔍 Filter: attack                                           │
│                                                             │
│  ID  Filename              Duration    [▶ Play]  [💾 Export]│
│  ───────────────────────────────────────────────────────────│
│  0   bgm_title.snf         2:34        [▶]       [💾]       │
│  1   sfx_attack.snf        0:03        [▶]       [💾]       │
│  2   sfx_hit.snf           0:01        [▶]       [💾]       │
│  3   sfx_magic.snf         0:02        [▶]       [💾]       │
│  4   bgm_dungeon.snf       3:12        [▶]       [💾]       │
│  ...                                                         │
│                                                             │
│  ┌─ Now Playing: sfx_attack.snf ─────────────────────────┐ │
│  │ ▂▃▅▆▇██▇▆▅▃▂▁▁▂▃▅▆▇██▇▆▅▃▂▁                          │ │
│  │ 0:00 ─────────────●────────────── 0:03                │ │
│  │                                                         │ │
│  │ ⏮  ⏸  ⏭    🔊 ████████░░    [🔁 Loop]  [💾 Export WAV]│ │
│  └───────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

**Features:**
- **Inline playback** — click ▶ to play any `.snf` file without leaving the list
- **Waveform visualization** — mini waveform preview for each file
- **Now playing bar** — persistent playback controls at bottom
- **Loop toggle** — repeat playback for testing
- **Volume control** — adjustable playback volume
- **Export to WAV** — one-click conversion to standard WAV file
- **Batch export** — select multiple files and export all at once

**Implementation approach:**
- Use existing `snf.rs` parser to extract PCM data
- Use `cpal` or `rodio` crate for audio playback
- Generate waveform data from PCM samples for visualization
- Export via existing `snf::extract()` WAV writer

**Impact:** Essential for audio modding — lets modders preview sound effects and music before deciding which files to modify or replace.

---

### Developer Experience

#### 13. Fixture-Based Integration Tests

Every file type gets a test that verifies extract→patch round-trip against real fixture files, ensuring no regression when parsers or serializers change:

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::path::Path;

    fn test_round_trip<T: Extractor + PartialEq + std::fmt::Debug>(
        fixture_path: &str,
    ) {
        let path = Path::new(fixture_path);
        assert!(path.exists(), "Fixture missing: {}", fixture_path);

        let original_bytes = std::fs::read(path).unwrap();
        let records = T::read_file(path).unwrap();
        assert!(!records.is_empty(), "No records parsed from {}", fixture_path);

        let temp = tempfile::NamedTempFile::new().unwrap();
        T::save_file(&records, temp.path()).unwrap();
        let patched_bytes = std::fs::read(temp.path()).unwrap();

        assert_eq!(
            original_bytes, patched_bytes,
            "Round-trip mismatch for {}: original {} bytes, patched {} bytes",
            fixture_path, original_bytes.len(), patched_bytes.len()
        );
    }

    #[test] fn round_trip_weapons() {
        test_round_trip::<WeaponItem>("fixtures/Dispel/CharacterInGame/weaponItem.db");
    }
    #[test] fn round_trip_monsters() {
        test_round_trip::<Monster>("fixtures/Dispel/MonsterInGame/Monster.db");
    }
    #[test] fn round_trip_magic() {
        test_round_trip::<MagicSpell>("fixtures/Dispel/Ref/Magic.db");
    }
    // ... one test per file type ...
}
```

**Test matrix:**

| File Type | Fixture File | Round-Trip | Extract Only |
|-----------|-------------|------------|--------------|
| `weapons` | `weaponItem.db` | ✓ | |
| `monsters` | `Monster.db` | ✓ | |
| `magic` | `Magic.db` | ✓ | |
| `store` | `STORE.DB` | ✓ | |
| `map_file` | `cat1.map` | | ✓ |
| `gtl` | `cat1.gtl` | | ✓ |
| `sprite` | `ShieldB1.spr` | | ✓ |

**CI integration:**
- Tests run on every PR
- Fixture files committed to repository
- Missing fixtures cause test skip (not failure) with a warning
- Round-trip failures block merge

**Impact:** Guarantees that no parser or serializer change silently corrupts data. Provides living documentation of which file types are fully supported vs. extract-only.
