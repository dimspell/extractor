# Dispel GUI — Implementation Playbook

## How to Use This Document

This is a **working guide** for implementing the GUI redesign and feature roadmap. Each section contains:
- **Concrete steps** — not goals, but actions
- **Code patterns** — exact patterns to follow
- **Decision checkpoints** — where to pause and validate
- **Risk flags** — what to watch out for

When starting any implementation task, find the relevant phase, read the steps, and follow the patterns.

---

## Phase 0: Pre-Flight Checklist

Before starting any refactoring:

```bash
# 1. Ensure clean state
git status
git diff --stat

# 2. Run full test suite
cargo test --workspace

# 3. Run clippy
cargo clippy --workspace -- -D warnings

# 4. Build both release and debug
cargo build --release
cargo build

# 5. Verify GUI launches
cargo run -p dispel-gui
```

**Rule:** Never start a refactor without a green baseline. If tests fail, fix them first.

---

## Phase 1: Split app.rs (Week 1-2)

### Goal: Reduce `app.rs` from 3,043 lines to < 200 lines

### Step 1.1: Extract State Struct

**File:** `dispel-gui/src/state.rs`

```rust
use crate::generic_editor::{GenericEditorState, MultiFileEditorState};
use crate::types::Tab;
use dispel_core::{/* all record types */};
use std::collections::HashMap;
use std::path::PathBuf;

pub struct AppState {
    // Navigation
    pub active_tab: Tab,
    pub shared_game_path: String,

    // Lookup data for dropdown fields
    pub lookups: HashMap<String, Vec<(String, String)>>,

    // Editor states — one per file type
    pub weapon_editor: Box<GenericEditorState<WeaponItem>>,
    pub monster_ref_editor: Box<MultiFileEditorState<MonsterRef>>,
    // ... (move all 27 editor fields here)

    // Legacy state (to be removed in Phase 2)
    pub map_op: MapOp,
    pub ref_op: RefOp,
    pub db_op: DbOp,
    pub log: String,
    pub is_running: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            active_tab: Tab::Map,
            shared_game_path: String::new(),
            lookups: HashMap::new(),
            weapon_editor: Box::default(),
            monster_ref_editor: Box::default(),
            // ...
        }
    }
}
```

**Action:**
1. Create `src/state.rs`
2. Move the `App` struct fields into `AppState`
3. In `app.rs`, replace `pub struct App` with `pub struct App { pub state: AppState }`
4. Update all field accesses: `self.weapon_editor` → `self.state.weapon_editor`
5. Verify: `cargo check` passes

### Step 1.2: Split Message Enum

**File:** `dispel-gui/src/messages.rs`

```rust
// Top-level message that wraps domain-specific messages
pub enum AppMessage {
    // Navigation
    TabSelected(Tab),
    GamePathChanged(String),

    // Domain messages (wrapped)
    Editor(EditorMessage),
    Explorer(ExplorerMessage),
    DbViewer(DbViewerMessage),
    Command(CommandMessage),

    // Legacy (to be removed)
    MapOp(MapOp),
    RefOp(RefOp),
}

// Editor domain messages
pub enum EditorMessage {
    // Generic editor messages
    ScanFiles { editor_id: EditorId },
    SelectFile { editor_id: EditorId, path: PathBuf },
    SelectRecord { editor_id: EditorId, index: usize },
    UpdateField { editor_id: EditorId, index: usize, field: String, value: String },
    AddRecord { editor_id: EditorId },
    RemoveRecord { editor_id: EditorId, index: usize },
    Save { editor_id: EditorId },
    CatalogLoaded { editor_id: EditorId, result: Result<Vec<Record>, String> },

    // Lookup messages
    LoadMonsterNames,
    MonsterNamesLoaded(Result<Vec<(String, String)>, String>),
}

// ... similar for ExplorerMessage, DbViewerMessage, CommandMessage
```

**Action:**
1. Create `src/messages.rs`
2. Move the `Message` enum there, rename to `AppMessage`
3. Group variants into domain enums
4. Update `app.rs` to use `AppMessage`
5. Verify: `cargo check` passes

### Step 1.3: Split Update Handlers

**Directory:** `dispel-gui/src/update/`

```
src/update/
├── mod.rs          # Dispatches AppMessage to domain handlers
├── editor.rs       # Handles EditorMessage
├── explorer.rs     # Handles ExplorerMessage (future)
├── db_viewer.rs    # Handles DbViewerMessage
└── commands.rs     # Handles CommandMessage (future)
```

**`mod.rs` pattern:**
```rust
use crate::state::AppState;
use crate::messages::AppMessage;
use iced::Task;

mod editor;
mod db_viewer;

pub fn update(state: &mut AppState, message: AppMessage) -> Task<AppMessage> {
    match message {
        AppMessage::TabSelected(tab) => {
            state.active_tab = tab;
            Task::none()
        }
        AppMessage::GamePathChanged(path) => {
            state.shared_game_path = path;
            Task::none()
        }
        AppMessage::Editor(msg) => editor::update(state, msg),
        AppMessage::DbViewer(msg) => db_viewer::update(state, msg),
        // ... legacy handlers stay in app.rs for now
        AppMessage::MapOp(_) | AppMessage::RefOp(_) => Task::none(),
    }
}
```

**`editor.rs` pattern:**
```rust
use crate::state::AppState;
use crate::messages::EditorMessage;
use iced::Task;

pub fn update(state: &mut AppState, message: EditorMessage) -> Task<AppMessage> {
    match message {
        EditorMessage::ScanFiles { editor_id } => {
            match editor_id {
                EditorId::MonsterRef => {
                    if state.shared_game_path.is_empty() {
                        state.monster_ref_editor.editor.status_msg =
                            "Please select game path first.".into();
                        return Task::none();
                    }
                    let path = PathBuf::from(&state.shared_game_path)
                        .join("MonsterInGame");
                    state.monster_ref_editor.scan_files(&path, "Mon*.ref");
                    state.monster_ref_editor.editor.status_msg = format!(
                        "Found {} monster ref files",
                        state.monster_ref_editor.file_list.len()
                    );
                }
                // ... other editors
            }
            Task::none()
        }
        EditorMessage::SelectRecord { editor_id, index } => {
            match editor_id {
                EditorId::MonsterRef => state.monster_ref_editor.select(index),
                // ...
            }
            Task::none()
        }
        // ...
    }
}
```

**Action:**
1. Create `src/update/` directory with `mod.rs`
2. Move editor-related `App::update` match arms to `update/editor.rs`
3. Move DbViewer match arms to `update/db_viewer.rs`
4. Update `app.rs` to call `update::update(&mut self.state, message)`
5. Verify: `cargo check` passes, GUI works identically

### Step 1.4: Split View Dispatch

**File:** `dispel-gui/src/view/mod.rs`

Current pattern (long if-else chain):
```rust
pub fn view(&self) -> Element<Message> {
    if self.active_tab == Tab::DbViewer {
        return self.view_db_viewer();
    }
    if self.active_tab == Tab::WeaponEditor {
        return self.view_weapon_editor_tab();
    }
    // ... 30 more if statements
}
```

New pattern (data-driven dispatch):
```rust
pub fn view(&self) -> Element<Message> {
    match self.active_tab {
        Tab::Map => self.view_map_tab(),
        Tab::Ref => self.view_ref_tab(),
        Tab::DbViewer => self.view_db_viewer(),
        Tab::WeaponEditor => self.view_weapon_editor_tab(),
        Tab::MonsterRefEditor => self.view_monster_ref_editor_tab(),
        // ... (still explicit, but in a match, not if-else)
    }
}
```

**Future pattern (Phase 3):** Registry-driven dispatch where new editors auto-register.

**Action:**
1. Convert if-else chain to match statement
2. Each tab gets its own `view_*` function
3. Verify: `cargo check` passes

### Decision Checkpoint 1

After Step 1.4, verify:
- [ ] `app.rs` is < 400 lines
- [ ] `cargo test` passes
- [ ] `cargo clippy` has 0 warnings
- [ ] GUI launches and all tabs work
- [ ] No behavioral changes from before

**If any check fails:** Stop and fix before proceeding.

---

## Phase 2: Complete Generic Editor Migration (Week 3-4)

### Goal: Migrate all 26 hand-written editors to `EditableRecord` + generic infrastructure

### Step 2.1: Implement `EditableRecord` for One Type

Start with `Monster` (Monster.db) — it has many fields, good stress test.

**File:** `dispel-core/src/references/monster_editor.rs`

```rust
use super::editable::{EditableRecord, FieldDescriptor, FieldKind};
use super::monster_db::Monster;

impl EditableRecord for Monster {
    fn field_descriptors() -> &'static [FieldDescriptor] {
        &[
            FieldDescriptor { name: "name", label: "Name:", kind: FieldKind::String },
            FieldDescriptor { name: "ai_type", label: "AI Type:", kind: FieldKind::Integer },
            FieldDescriptor { name: "health_points_min", label: "HP Min:", kind: FieldKind::Integer },
            FieldDescriptor { name: "health_points_max", label: "HP Max:", kind: FieldKind::Integer },
            // ... all 35 fields
        ]
    }

    fn get_field(&self, field: &str) -> String {
        match field {
            "name" => self.name.clone(),
            "ai_type" => self.ai_type.to_string(),
            "health_points_min" => self.health_points_min.to_string(),
            // ...
            _ => String::new(),
        }
    }

    fn set_field(&mut self, field: &str, value: String) -> bool {
        match field {
            "name" => { self.name = value; true }
            "ai_type" => { if let Ok(v) = value.parse() { self.ai_type = v; true } else { false } }
            // ...
            _ => false,
        }
    }

    fn list_label(&self) -> String {
        format!("[{}] {} - HP:{}/MP:{}",
            self.id, self.name, self.health_points_min, self.mana_points_min)
    }

    fn detail_title() -> &'static str { "Monster Details" }
    fn empty_selection_text() -> &'static str { "No monster selected" }
    fn save_button_label() -> &'static str { "Save Monsters" }
    fn detail_width() -> f32 { 380.0 }
}
```

**Action:**
1. Create `dispel-core/src/references/monster_editor.rs`
2. Implement `EditableRecord` for `Monster`
3. Add `mod monster_editor;` to `references/mod.rs`
4. Add `Default` derive to `Monster` struct if missing
5. Verify: `cargo check` in dispel-core passes

### Step 2.2: Create GUI Editor Stub

**File:** `dispel-gui/src/monster_editor.rs`

```rust
use crate::generic_editor::GenericEditorState;
use dispel_core::Monster;

pub type MonsterEditorState = GenericEditorState<Monster>;
```

**File:** `dispel-gui/src/view/monster_editor.rs`

```rust
use crate::app::App;
use crate::message::Message;
use crate::view::generic_editor::build_editor_view;
use iced::Element;

impl App {
    pub fn view_monster_editor_tab(&self) -> Element<'_, Message> {
        build_editor_view(
            self,
            &self.monster_editor,
            Message::MonsterOpScan,
            Message::MonsterOpSave,
            Message::MonsterOpSelect,
            Message::MonsterOpFieldChanged,
            &self.lookups,
        )
    }
}
```

**Action:**
1. Create the two stub files
2. Register module in `main.rs`
3. Add `monster_editor: Box<MonsterEditorState>` to `AppState`
4. Wire up in `view/mod.rs` match arm
5. Add message variants to `messages.rs`
6. Add handlers in `update/editor.rs`
7. Verify: Tab appears and works

### Step 2.3: Migrate Remaining Types

Repeat Steps 2.1-2.2 for each type, in this order (easiest to hardest):

| Priority | Type | File | Fields | Complexity |
|----------|------|------|--------|------------|
| 1 | `HealItem` | `HealItem.db` | 9 | Low |
| 2 | `MiscItem` | `MiscItem.db` | 19 | Low |
| 3 | `EditItem` | `EditItem.db` | 19 | Low |
| 4 | `EventItem` | `EventItem.db` | 19 | Low |
| 5 | `MagicSpell` | `Magic.db` | 10 | Low |
| 6 | `PartyRef` | `PartyRef.ref` | 14 | Medium |
| 7 | `PartyIniNpc` | `PrtIni.db` | 8 | Low |
| 8 | `NpcIni` | `Npc.ini` | 8 | Low |
| 9 | `Dialog` | `.dlg` files | 10 | Medium (multi-file) |
| 10 | `DialogueText` | `.pgp` files | 6 | Medium (multi-file) |
| 11 | `DrawItem` | `DRAWITEM.ref` | 8 | Low |
| 12 | `Event` | `Event.ini` | 6 | Low |
| 13 | `EventNpcRef` | `Eventnpc.ref` | 7 | Low |
| 14 | `Extra` | `Extra.ini` | 4 | Low |
| 15 | `ExtraRef` | `Ext*.ref` | 20 | High (multi-file, item catalog) |
| 16 | `MapIni` | `Map.ini` | 9 | Low |
| 17 | `Message` | `Message.scr` | 2 | Low |
| 18 | `NpcRef` | `Npccat*.ref` | 8 | Medium (multi-file) |
| 19 | `PartyLevelNpc` | `PrtLevel.db` | 4 | Medium (nested) |
| 20 | `Quest` | `Quest.scr` | 4 | Low |
| 21 | `WaveIni` | `Wave.ini` | 3 | Low |
| 22 | `ChData` | `ChData.db` | 12 | Low (single-record) |
| 23 | `Store` | `STORE.DB` | 8 | High (product sub-lists) |
| 24 | `Map` | `AllMap.ini` | 6 | Low |

**Rule:** After each migration, delete the old hand-written editor files. Never keep both.

### Step 2.4: Handle Complex Editors

**IMPORTANT: Selectable Fields (Dropdowns)**

Many fields should render as dropdowns, not text inputs. Use `FieldKind::Lookup` for fields that reference other game data:

```rust
FieldDescriptor {
    name: "mon_id",
    label: "Monster:",
    kind: FieldKind::Lookup("monster_names"),  // dropdown populated from Monster.ini
}
FieldDescriptor {
    name: "event_id",
    label: "Event:",
    kind: FieldKind::Lookup("event_names"),    // dropdown populated from Event.ini
}
FieldDescriptor {
    name: "loot1_item_type",
    label: "Loot 1 Type:",
    kind: FieldKind::Enum {
        variants: &["Weapon", "Healing", "Edit", "Event", "Misc", "Other"],
    },
}
```

Lookup data is loaded on-demand and stored in `state.lookups`:
```rust
state.lookups.insert("monster_names".to_string(),
    monsters.iter().map(|m| (m.id.to_string(), m.name.clone())).collect());
```

**StoreEditor** has product sub-lists — needs a custom view that wraps the generic editor:

```rust
// Store has products that reference other item types
// Keep the custom view for the product sub-list,
// but use GenericEditorState for the store records themselves
pub struct StoreEditorState {
    pub editor: GenericEditorState<Store>,
    pub product_catalog: ItemCatalog, // for name resolution
}
```

**ChestEditor** needs an item catalog for name lookup — use the lookup system:

```rust
// Load ItemCatalog into lookups on scan
state.lookups.insert("weapon_names".to_string(),
    catalog.weapons.iter().map(|w| (w.id.to_string(), w.name.clone())).collect());
state.lookups.insert("heal_names".to_string(),
    catalog.healing.iter().map(|h| (h.id.to_string(), h.name.clone())).collect());
// ... etc
```

**SpriteBrowser** and **DbViewer** are unique — keep as-is, they don't fit the record-editing pattern.

**⚠️ Map Rendering — Resource Constraints**

The `map.map` file produces a **300MB image** — full rendering is extremely resource-hungry. The visual map editor (Phase 5) must use **tile-level lazy loading**, not full-image rendering:

```rust
// WRONG: Load entire 300MB image into memory
let map_image = image::open("cat1.map.rendered.png"); // 300MB!

// CORRECT: Load tiles on-demand, cache visible viewport only
struct MapViewport {
    visible_tiles: HashMap<(i32, i32), TileCacheEntry>,
    zoom_level: f32,
    pan_offset: (f32, f32),
}

impl MapViewport {
    fn load_tile(&mut self, tile_x: i32, tile_y: i32) {
        // Only load tiles currently visible in viewport
        // Cache recently viewed tiles, evict old ones
        // Max cache size: 50MB (not 300MB)
    }
}
```

**Map rendering guidelines:**
- Never render the full map at once — use viewport-based tile loading
- Tile cache max: 50MB with LRU eviction
- Default zoom level should show the entire map at thumbnail resolution
- Zoom in progressively loads higher-resolution tiles
- Use iced's `canvas` API for efficient tile compositing
- Pre-generate tile atlas at multiple zoom levels (mipmapping)
- Consider using `image::imageops::resize` for thumbnail generation instead of full render

### Decision Checkpoint 2

After all migrations:
- [ ] All 26 editors use generic infrastructure
- [ ] Old hand-written editor files deleted
- [ ] `cargo test` passes
- [ ] `cargo clippy` has 0 warnings
- [ ] Each editor tab works identically to before
- [ ] Total editor code reduced from ~8,000 to ~500 lines

---

## Phase 3: Workspace Model (Week 5-6)

### Goal: Replace fixed 34-tab sidebar with dynamic file-tree workspace

### Step 3.1: Create File Explorer Widget

**File:** `dispel-gui/src/components/file_tree.rs`

```rust
use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Element, Length};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub enum FileTreeMessage {
    Expand(PathBuf),
    Collapse(PathBuf),
    Select(PathBuf),
    Search(String),
}

pub struct FileTree {
    root: PathBuf,
    expanded: HashSet<PathBuf>,
    selected: Option<PathBuf>,
    search_filter: String,
}

impl FileTree {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            expanded: HashSet::new(),
            selected: None,
            search_filter: String::new(),
        }
    }

    pub fn view(&self) -> Element<FileTreeMessage> {
        column![
            text_input("🔍 Filter files...", &self.search_filter)
                .on_input(FileTreeMessage::Search),
            scrollable(self.build_tree(&self.root))
        ]
        .spacing(8)
        .height(Length::Fill)
        .into()
    }

    fn build_tree(&self, path: &Path) -> Vec<Element<FileTreeMessage>> {
        // Recursively build tree nodes
        // Use registry to detect file types and show icons
        // ...
    }
}
```

### Step 3.2: Create Workspace State

```rust
pub struct Workspace {
    pub open_files: Vec<OpenFile>,
    pub active_file: Option<usize>,
}

pub struct OpenFile {
    pub path: PathBuf,
    pub file_type: &'static FileType, // from registry
    pub modified: bool,
    pub pinned: bool,
}
```

### Step 3.3: Dynamic Tab Bar

```rust
pub fn view_tab_bar(workspace: &Workspace) -> Element<WorkspaceMessage> {
    row![
        workspace.open_files.iter().enumerate().map(|(i, file)| {
            button(text(file.path.file_name().unwrap().to_string_lossy()))
                .on_press(WorkspaceMessage::SelectTab(i))
                .style(if Some(i) == workspace.active_file {
                    style::active_tab_button
                } else {
                    style::tab_button
                })
        })
    ]
}
```

### Decision Checkpoint 3

- [ ] File explorer shows game directory tree
- [ ] Double-clicking a file opens it in a new tab
- [ ] Tabs can be closed and reordered
- [ ] Old Tab enum still works as fallback
- [ ] Feature flag `workspace_mode` controls which UI is shown

---

## Phase 4: UX Enhancements (Week 7-8)

### Step 4.1: Undo/Redo

**Pattern:** Command pattern with a stack.

```rust
pub struct EditHistory {
    undo_stack: Vec<Box<dyn EditCommand>>,
    redo_stack: Vec<Box<dyn EditCommand>>,
    max_depth: usize,
}

pub trait EditCommand {
    fn execute(&self, state: &mut dyn Any);
    fn undo(&self, state: &mut dyn Any);
    fn description(&self) -> &str;
}

// Usage in editor:
pub fn update_field(&mut self, idx: usize, field: &str, value: String) {
    let old_value = /* capture current value */;
    let new_value = value.clone();

    self.history.push(Box::new(FieldEdit {
        editor_id: self.id,
        record_idx: idx,
        field: field.to_string(),
        old_value,
        new_value,
    }));

    // Apply the change
    // ...
}
```

### Step 4.2: Field Validation

**Pattern:** Validate on every keystroke, show visual feedback.

```rust
// In EditableRecord trait:
fn validate_field(&self, field: &str, value: &str) -> Result<(), String> {
    match field {
        "base_price" => {
            value.parse::<i32>()
                .map(|v| if v >= 0 { Ok(()) } else { Err("Price cannot be negative".into()) })
                .unwrap_or(Err("Invalid number".into()))
        }
        "name" => {
            if value.len() <= 30 { Ok(()) } else { Err("Name too long (max 30 chars)".into()) }
        }
        _ => Ok(()),
    }
}

// In view:
let is_valid = record.validate_field(field_name, &value).is_ok();
let input_style = if is_valid {
    style::valid_input
} else {
    style::invalid_input
};
```

### Step 4.3: Command Palette

**File:** `dispel-gui/src/view/command_palette.rs`

```rust
pub struct CommandPalette {
    visible: bool,
    query: String,
    commands: Vec<CommandEntry>,
}

struct CommandEntry {
    title: String,
    icon: &'static str,
    action: Box<dyn Fn() -> AppMessage>,
    score: f64, // fuzzy match score
}
```

---

## Phase 5: Advanced Features (Week 9+)

### Step 5.1: Derive Macro for `EditableRecord`

**File:** `dispel-core/derive/Cargo.toml`

```toml
[lib]
proc-macro = true

[dependencies]
proc-macro2 = "1.0"
quote = "1.0"
syn = { version = "2.0", features = ["full"] }
```

**File:** `dispel-core/derive/src/lib.rs`

```rust
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields};

#[proc_macro_derive(EditableRecord, attributes(field))]
pub fn derive_editable_record(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let Data::Struct(data) = &input.data else {
        panic!("EditableRecord can only be derived for structs");
    };

    let Fields::Named(fields) = &data.fields else {
        panic!("EditableRecord requires named fields");
    };

    // Parse #[field(...)] attributes
    // Generate field_descriptors(), get_field(), set_field()
    // ...

    TokenStream::from(quote! {
        impl EditableRecord for #name {
            // generated code
        }
    })
}
```

### Step 5.2: Diff Engine

```rust
pub enum DiffOp<T> {
    Modify { index: usize, field: String, from: T, to: T },
    Add { index: usize, record: T },
    Remove { index: usize, record: T },
}

pub fn diff<T: PartialEq + Clone>(old: &[T], new: &[T]) -> Vec<DiffOp<T>> {
    // Use a diff algorithm (Myers, LCS, etc.)
    // For simplicity, start with index-based comparison
}
```

### Step 5.3: Pipeline Support

```rust
// In unified.rs PatchCommand:
if self.args.stdin {
    let stdin = std::io::stdin();
    let mut buffer = String::new();
    stdin.read_to_string(&mut buffer)?;
    data = serde_json::from_str(&buffer)?;
}
```

---

## Testing Strategy

### Unit Tests (per module)
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_get_set_roundtrip() {
        let mut record = WeaponItem::default();
        record.set_field("base_price", "100".to_string());
        assert_eq!(record.get_field("base_price"), "100");
    }
}
```

### Integration Tests (per file type)
```rust
#[test]
fn round_trip_weapons_fixture() {
    let path = Path::new("fixtures/Dispel/CharacterInGame/weaponItem.db");
    let original = std::fs::read(path).unwrap();
    let records = WeaponItem::read_file(path).unwrap();
    let temp = tempfile::NamedTempFile::new().unwrap();
    WeaponItem::save_file(&records, temp.path()).unwrap();
    assert_eq!(original, std::fs::read(temp.path()).unwrap());
}
```

### GUI Tests (manual, for now)
```bash
# Test each tab
cargo run -p dispel-gui
# 1. Open each file type
# 2. Edit a field
# 3. Save
# 4. Re-open and verify change
# 5. Check .bak was created
```

---

## Code Conventions

### File Organization
```
dispel-core/src/references/
├── weapon_db.rs          # Parser + Extractor impl
├── weapon_editor.rs      # EditableRecord impl

dispel-gui/src/
├── weapon_editor.rs      # Type alias: GenericEditorState<WeaponItem>
└── view/
    └── weapon_editor.rs  # View function
```

### Naming
- State structs: `XEditorState`
- Message variants: `XOp*` (e.g., `WeaponOpScan`, `WeaponOpSave`)
- View functions: `view_x_editor_tab`
- Update handlers: in `update/editor.rs` match arm

### Error Handling
- Never use `.expect()` or `.unwrap()` on I/O
- Always use `.map_err(|e| format!("..."))` for user-facing errors
- Display errors in `status_msg`, never panic

### Styling
- All styles in `theme/styles.rs`
- Never hardcode colors in view code
- Use semantic names: `style::commit_button`, not `style::green_button`

---

## Risk Mitigation

| Risk | Mitigation |
|------|-----------|
| Breaking existing functionality | Run full test suite after every step |
| Losing user data | Always create timestamped backup before save |
| Feature creep | Stick to the phase plan; park new ideas in "Future" |
| Merge conflicts | Small PRs per step; never combine phases |
| Performance regression | Benchmark extract/patch times before and after |
| GUI freezes | All file I/O must be async via `Task::perform` |

---

## Progress Tracking

Use this checklist. Check off items as they're completed.

### Phase 1: Split app.rs
- [ ] 1.1 Extract `AppState` struct
- [ ] 1.2 Split `Message` enum into domains
- [ ] 1.3 Create `update/` module with handlers
- [ ] 1.4 Convert view dispatch to match
- [ ] **Checkpoint:** `app.rs` < 400 lines, all tests pass

### Phase 2: Generic Editor Migration
- [ ] 2.1 Implement `EditableRecord` for `Monster`
- [ ] 2.2 Create GUI stub for `MonsterEditor`
- [ ] 2.3 Migrate HealItem, MiscItem, EditItem, EventItem
- [ ] 2.4 Migrate Magic, PartyRef, PartyIni, NpcIni
- [ ] 2.5 Migrate Dialog, DialogueText, DrawItem
- [ ] 2.6 Migrate EventIni, EventNpcRef, ExtraIni
- [ ] 2.7 Migrate ExtraRef, MapIni, MessageScr, NpcRef
- [ ] 2.8 Migrate PartyLevelDb, QuestScr, WaveIni, ChData
- [ ] 2.9 Handle StoreEditor (product sub-lists)
- [ ] 2.10 Handle ChestEditor (item catalog)
- [ ] **Checkpoint:** 0 hand-written editors, all tests pass

### Phase 3: Workspace Model
- [ ] 3.1 Create `FileTree` component
- [ ] 3.2 Create `Workspace` state
- [ ] 3.3 Create dynamic tab bar
- [ ] 3.4 Wire file tree to editor opening
- [ ] 3.5 Add feature flag `workspace_mode`
- [ ] **Checkpoint:** Both old and new navigation work

### Phase 4: UX Enhancements
- [ ] 4.1 Undo/Redo system
- [ ] 4.2 Field validation with visual feedback
- [ ] 4.3 Command palette (Ctrl+P)
- [ ] 4.4 Global search (Ctrl+Shift+F)
- [ ] 4.5 Keyboard shortcuts
- [ ] **Checkpoint:** All UX features functional

### Phase 5: Advanced Features
- [ ] 5.1 Derive macro for `EditableRecord`
- [ ] 5.2 Diff engine
- [ ] 5.3 Pipeline support (CLI)
- [ ] 5.4 Visual map editor
- [ ] 5.5 Sprite animation timeline
- [ ] 5.6 Mod packaging
- [ ] 5.7 Audio playback
- [ ] 5.8 Fixture-based integration tests
- [ ] **Checkpoint:** All advanced features functional

---

## Quick Reference

### Adding a New File Type

1. **Parser** (if not exists): `dispel-core/src/references/new_type.rs`
2. **Extractor impl**: Implement `Extractor` trait
3. **EditableRecord impl**: `dispel-core/src/references/new_type_editor.rs`
4. **GUI state**: `dispel-gui/src/new_type_editor.rs` → `pub type NewTypeEditorState = GenericEditorState<NewType>;`
5. **GUI view**: `dispel-gui/src/view/new_type_editor.rs` → `build_editor_view(...)`
6. **Register in registry**: Add entry to `FILE_TYPES` in `dispel-core/src/commands/registry/entries.rs`
7. **Add tab**: Add variant to `Tab` enum
8. **Wire up**: Add to `AppState`, `messages.rs`, `update/editor.rs`, `view/mod.rs`

### Adding a New Field to an Existing Type

1. Add field to struct in `dispel-core/src/references/`
2. Update `read_file` and `save_file` in `Extractor` impl
3. Add `FieldDescriptor` in `EditableRecord` impl
4. Add `get_field` and `set_field` match arms
5. GUI updates automatically

### Debugging a Broken Editor

```bash
# 1. Check if the file type is registered
cargo run -- list --filter <type_key>

# 2. Try extracting manually
cargo run -- extract -i <path> --type <type_key>

# 3. Check if EditableRecord is implemented
grep -r "impl EditableRecord for" src/references/

# 4. Check if GUI state exists
grep -r "EditorState" dispel-gui/src/

# 5. Check if view is wired
grep -r "view_.*_editor_tab" dispel-gui/src/view/mod.rs
```
