# Dispel GUI Rewrite — Project Context

## Overview

This document provides comprehensive context for the Dispel GUI rewrite project. It covers the architecture, decisions, completed work, remaining tasks, and patterns for future development.

**Project Goal:** Transform the monolithic 3,000+ line `app.rs` into a modern, modular GUI with workspace-based navigation, generic editors driven by data, and professional UX features.

---

## Architecture

### Three-Layer Design

```
┌─────────────────────────────────────────┐
│              Presentation               │
│  (iced widgets, themes, view modules)   │
├─────────────────────────────────────────┤
│              Application                │
│  (state, messages, update handlers)     │
├─────────────────────────────────────────┤
│              Domain                     │
│  (EditableRecord, lookups, validation)  │
└─────────────────────────────────────────┘
```

### Key Components

| Component | Location | Purpose |
|-----------|----------|---------|
| `AppState` | `dispel-gui/src/state.rs` | All mutable application state |
| `Message` | `dispel-gui/src/message.rs` | All UI events and commands |
| `update()` | `dispel-gui/src/app.rs` | Message handler (inline for now) |
| `view()` | `dispel-gui/src/view/mod.rs` | Root view dispatcher |
| `Workspace` | `dispel-gui/src/workspace.rs` | Dynamic tab management |
| `FileTree` | `dispel-gui/src/file_tree.rs` | Directory tree widget |
| `TabBar` | `dispel-gui/src/tab_bar.rs` | Tab bar with close/pin |
| `GenericEditorState<R>` | `dispel-gui/src/generic_editor.rs` | Generic editor state |
| `EditableRecord` | `dispel-core/src/references/editable.rs` | Trait for data-driven editors |

### Data Flow

```
User Action → Message → update() → State Mutation → view() → UI Update
```

---

## Completed Work

### Phase 1: Foundation ✅
- ✅ Extracted `AppState` struct (167 lines)
- ✅ Converted `App` to `pub state: AppState`
- ✅ Updated all field accesses to `self.state.FIELD`
- ✅ Converted view dispatch from if-else chain to `match` statement
- ✅ Added `workspace: Workspace` to `AppState`

### Phase 2: Generic Editor Migration (Partial) ✅
- ✅ Implemented `EditableRecord` for 5 types:
  - `HealItem` (9 fields)
  - `MiscItem` (3 fields)
  - `EditItem` (19 fields)
  - `EventItem` (2 fields)
  - `MagicSpell` (14 fields)
- ✅ Created state aliases: `HealItemEditorState`, `MiscItemEditorState`, etc.
- ✅ Created view stubs using `build_editor_view()`
- ✅ Fixed `Scan` button behavior (was opening file picker, now uses `shared_game_path`)

### Phase 3: Workspace Model ✅
- ✅ Created `Workspace` struct with `open()`, `close()`, `active()` methods
- ✅ Created `WorkspaceTab` with `id`, `label`, `path`, `modified`, `pinned`
- ✅ Created `FileTree` component with:
  - Recursive directory scanning (3 levels deep)
  - Expand/collapse directories
  - Search filter
  - File type icons (🗃️📄📋📜💬📝🗺️🖼️🎨🔊)
- ✅ Created `TabBar` component with close buttons and modified indicators
- ✅ Added `workspace_mode` toggle (Legacy ↔ Workspace)
- ✅ Wired file tree clicks to open files in workspace tabs
- ✅ Auto-detect file type by extension+stem when opening files

### Phase 4: UX Enhancements ✅
- ✅ Added `validate_field()` to `EditableRecord` trait
- ✅ Implemented validation for all migrated editor types
- ✅ Added visual feedback (red borders, error tooltips)
- ✅ Implemented pre-save validation with error summary

### Bug Fixes ✅
- ✅ Fixed sidebar visibility (Map/Ref/Database tabs had early return bypassing sidebar layout)
- ✅ Fixed Scan button in HealItem/EventItem/EditItem/MiscItem editors (was calling BrowseGamePath instead of ScanItems)

---

## Remaining Work

### Phase 2: Complete Generic Editor Migration
- [ ] Implement `EditableRecord` for 10 more types:
  - `PartyRef` (14 fields)
  - `PartyIniNpc` (8 fields)
  - `NpcIni` (8 fields)
  - `DrawItem` (8 fields)
  - `Event` (6 fields)
  - `EventNpcRef` (7 fields)
  - `Extra` (4 fields)
  - `MapIni` (9 fields)
  - `Message` (2 fields)
  - `ChData` (12 fields)
- [ ] Create state aliases and view stubs for all 10 types
- [ ] Delete old hand-written editor files
- [ ] Verify all migrated tabs work identically

### Phase 4: Advanced UX Features
- [ ] Command palette (Ctrl+P)
- [ ] Global search (Ctrl+Shift+F)
- [ ] Undo/Redo system
- [ ] Auto-save drafts
- [ ] Edit history panel

### Phase 5: Spreadsheet View
- [ ] Create table widget with column sorting
- [ ] Implement inline cell editing
- [ ] Add filter bar with free-text search
- [ ] Implement multi-select
- [ ] Create inspector panel view
- [ ] Add split view toggle

### Phase 6: Cleanup
- [ ] Remove deprecated `Tab` enum
- [ ] Delete remaining hand-written editor files
- [ ] Run `cargo clippy -- -D warnings`
- [ ] Run `cargo test --workspace`
- [ ] Manual GUI testing for all file types

---

## Key Patterns

### Adding a New Generic Editor

1. **Implement `EditableRecord`** in `dispel-core/src/references/<type>_editor.rs`:
   ```rust
   impl EditableRecord for MyType {
       fn field_descriptors() -> &'static [FieldDescriptor] { ... }
       fn get_field(&self, field: &str) -> String { ... }
       fn set_field(&mut self, field: &str, value: String) -> bool { ... }
       fn list_label(&self) -> String { ... }
       fn detail_title() -> &'static str { ... }
       fn empty_selection_text() -> &'static str { ... }
       fn save_button_label() -> &'static str { ... }
       fn detail_width() -> f32 { ... }
   }
   ```

2. **Create state alias** in `dispel-gui/src/<type>_editor.rs`:
   ```rust
   use crate::generic_editor::GenericEditorState;
   use dispel_core::MyType;
   pub type MyTypeEditorState = GenericEditorState<MyType>;
   ```

3. **Create view stub** in `dispel-gui/src/view/<type>_editor.rs`:
   ```rust
   use crate::app::App;
   use crate::message::Message;
   use crate::view::generic_editor::build_editor_view;
   use iced::Element;

   impl App {
       pub fn view_my_type_editor_tab(&self) -> Element<'_, Message> {
           build_editor_view(
               self,
               &self.state.my_type_editor,
               Message::MyTypeOpScanItems,
               Message::MyTypeOpSave,
               Message::MyTypeOpSelectItem,
               Message::MyTypeOpFieldChanged,
               &self.state.lookups,
           )
       }
   }
   ```

4. **Add to `AppState`** in `dispel-gui/src/state.rs`:
   ```rust
   pub my_type_editor: Box<my_type_editor::MyTypeEditorState>,
   ```

5. **Add to `AppState::default()`**:
   ```rust
   my_type_editor: Box::default(),
   ```

6. **Add messages** in `dispel-gui/src/message.rs`:
   ```rust
   MyTypeOpBrowseGamePath,
   MyTypeOpScanItems,
   MyTypeOpSelectItem(usize),
   MyTypeOpFieldChanged(usize, String, String),
   MyTypeOpSave,
   MyTypeOpLoadCatalog,
   MyTypeCatalogLoaded(Result<Vec<MyType>, String>),
   ```

7. **Add handlers** in `dispel-gui/src/app.rs` `update()` method.

8. **Add to view dispatch** in `dispel-gui/src/view/mod.rs`:
   ```rust
   Tab::MyTypeEditor => self.view_my_type_editor_tab(),
   ```

### Field Types

| `FieldKind` | Widget | Example |
|-------------|--------|---------|
| `String` | Text input | Name, description |
| `Integer` | Text input (parsed) | ID, price, stats |
| `Enum { variants }` | Dropdown | Item type, ghost face |
| `Lookup(key)` | Dropdown (from lookups) | Monster name, event name |

---

## Known Issues

### Current
1. **Workspace mode is incomplete** — File tree opens tabs but doesn't load file data into editors yet
2. **Legacy mode still uses Tab enum** — Full migration to workspace-only navigation pending
3. **No undo/redo** — All edits are immediate with no history
4. **No auto-save** — Changes are lost if app closes without saving
5. **No spreadsheet view** — All editors use inspector panel only

### Resolved
- ~~Sidebar not showing on Map/Ref/Database tabs~~ — Fixed by removing early return
- ~~Scan button opening file picker~~ — Fixed to use `shared_game_path`
- ~~Duplicate view method definitions~~ — Removed duplicates from `app.rs`

---

## File Structure

```
dispel-gui/src/
├── main.rs                    # Entry point (63 lines)
├── app.rs                     # App struct, update handler, workspace view (~3,000 lines)
├── state.rs                   # AppState struct (173 lines)
├── message.rs                 # Message enum (360+ lines)
├── workspace.rs               # Workspace + WorkspaceTab (96 lines)
├── file_tree.rs               # FileTree component (218 lines)
├── tab_bar.rs                 # TabBar component (58 lines)
├── generic_editor.rs          # GenericEditorState + MultiFileEditorState (248 lines)
├── style.rs                   # Theme and styling
├── utils.rs                   # Helper functions
├── view/
│   ├── mod.rs                 # Root view dispatcher (277 lines)
│   ├── generic_editor.rs      # build_editor_view() function
│   ├── weapon_editor.rs       # Generic view stub (18 lines)
│   ├── heal_item_editor.rs    # Generic view stub (18 lines)
│   ├── misc_item_editor.rs    # Generic view stub (18 lines)
│   ├── edit_item_editor.rs    # Generic view stub (18 lines)
│   ├── event_item_editor.rs   # Generic view stub (18 lines)
│   ├── magic_editor.rs        # Generic view stub (105 lines)
│   ├── [20+ hand-written editors...]
│   └── ...
└── [20+ hand-written editor state files...]

dispel-core/src/references/
├── editable.rs                # EditableRecord trait + FieldDescriptor + FieldKind
├── weapon_editor.rs           # EditableRecord impl for WeaponItem
├── heal_item_editor.rs        # EditableRecord impl for HealItem
├── misc_item_editor.rs        # EditableRecord impl for MiscItem
├── edit_item_editor.rs        # EditableRecord impl for EditItem
├── event_item_editor.rs       # EditableRecord impl for EventItem
├── magic_editor.rs            # EditableRecord impl for MagicSpell
└── [remaining types...]
```

---

## Build Status

```bash
cargo check  # ✅ Clean (0 errors, 0 warnings)
cargo build  # ✅ Success
```

---

## Next Steps

1. **Complete Phase 2** — Migrate remaining 10 editor types to generic infrastructure
2. **Delete old files** — Remove hand-written editor state and view files
3. **Complete workspace wiring** — Make file tree clicks actually load file data into editors
4. **Add command palette** — Ctrl+P quick access to all actions
5. **Add global search** — Ctrl+Shift+F cross-catalog search
6. **Add undo/redo** — Command pattern with history stack
7. **Add spreadsheet view** — Table widget for bulk editing

---

*Last updated: 2026-04-04*
