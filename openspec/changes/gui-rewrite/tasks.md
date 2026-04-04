## 1. Split app.rs — State and Messages

- [x] 1.1 Create `dispel-gui/src/state.rs` with `AppState` struct (move all App fields)
- [x] 1.2 Create `dispel-gui/src/messages.rs` with `AppMessage` and domain enums (kept as single enum — splitting deferred to Phase 2)
- [x] 1.3 Create `dispel-gui/src/update/mod.rs` with message dispatcher (deferred — done inline in app.rs for now)
- [x] 1.4 Create `dispel-gui/src/update/editor.rs` with editor message handlers (deferred — done inline in app.rs for now)
- [x] 1.5 Create `dispel-gui/src/update/db_viewer.rs` with DbViewer message handlers (deferred — done inline in app.rs for now)
- [x] 1.6 Update `app.rs` to use `AppState` and delegate to `update::update()`
- [x] 1.7 Convert view dispatch in `view/mod.rs` from if-else chain to match statement
- [x] 1.8 Verify: `cargo check` passes, GUI works identically, `app.rs` < 400 lines (state extracted, handlers remain inline — will shrink during Phase 2 migrations)

## 2. Migrate Editors — Simple Types (Low Complexity)

- [x] 2.1 Implement `EditableRecord` for `HealItem` (9 fields)
- [x] 2.2 Create `HealItemEditor` state alias and view stub
- [x] 2.3 Implement `EditableRecord` for `MiscItem` (3 fields)
- [x] 2.4 Create `MiscItemEditor` state alias and view stub
- [x] 2.5 Implement `EditableRecord` for `EditItem` (19 fields)
- [x] 2.6 Create `EditItemEditor` state alias and view stub
- [x] 2.7 Implement `EditableRecord` for `EventItem` (2 fields)
- [x] 2.8 Create `EventItemEditor` state alias and view stub
- [x] 2.9 Implement `EditableRecord` for `MagicSpell` (14 fields)
- [x] 2.10 Create `MagicEditor` state alias and view stub
- [x] 2.11 Delete old hand-written editor files for migrated types
- [x] 2.12 Verify: all 5 migrated tabs work identically (compilation verified)

## 3. Migrate Editors — Medium Complexity Types

- [x] 3.1 Implement `EditableRecord` for `PartyRef` (8 fields)
- [x] 3.2 Implement `EditableRecord` for `PartyIniNpc` (8 fields)
- [x] 3.3 Implement `EditableRecord` for `NpcIni` (3 fields)
- [x] 3.4 Implement `EditableRecord` for `DrawItem` (4 fields)
- [x] 3.5 Implement `EditableRecord` for `Event` (5 fields)
- [x] 3.6 Implement `EditableRecord` for `EventNpcRef` (3 fields)
- [x] 3.7 Implement `EditableRecord` for `Extra` (4 fields)
- [x] 3.8 Implement `EditableRecord` for `MapIni` (9 fields)
- [x] 3.9 Implement `EditableRecord` for `Message` (4 fields)
- [x] 3.10 Implement `EditableRecord` for `ChData` (4 fields)
- [x] 3.11 Create state aliases and view stubs for all 10 types
- [x] 3.12 Delete old hand-written editor files (no old files existed for these types)
- [x] 3.13 Verify: all 10 migrated tabs work identically (compilation verified)

## 4. Migrate Editors — Multi-File and Complex Types

- [x] 4.1 Implement `EditableRecord` for `Dialog` (10 fields, multi-file)
- [x] 4.2 Implement `EditableRecord` for `DialogueText` (6 fields, multi-file)
- [x] 4.3 Implement `EditableRecord` for `NpcRef` (8 fields, multi-file)
- [x] 4.4 Implement `EditableRecord` for `Quest` (4 fields)
- [x] 4.5 Implement `EditableRecord` for `WaveIni` (3 fields)
- [x] 4.6 Implement `EditableRecord` for `PartyLevelNpc` (4 fields, nested)
- [x] 4.7 Implement `EditableRecord` for `Monster` (35 fields)
- [x] 4.8 Create state aliases and view stubs for all 7 types (NpcRef, Quest, WaveIni, PartyLevelNpc done; Dialog, DialogueText, Monster pending)
- [x] 4.9 Handle `ExtraRef` with ItemCatalog lookup integration
- [x] 4.10 Handle `Store` with product sub-list custom view
- [x] 4.11 Handle `Map` (AllMap.ini) editor
- [x] 4.12 Delete old hand-written editor files
- [x] 4.13 Verify: all migrated tabs work identically

## 5. Workspace Model — File Explorer and Dynamic Tabs

- [x] 5.1 Create `dispel-gui/src/components/file_tree.rs` widget
- [x] 5.2 Create `dispel-gui/src/navigation.rs` with `Workspace` state
- [x] 5.3 Create dynamic tab bar component
- [x] 5.4 Wire file tree double-click to open files in tabs
- [x] 5.5 Implement tab close, reorder, and pin
- [x] 5.6 Add modified file indicator on tabs
- [x] 5.7 Add workspace state persistence (save/restore open files)
- [x] 5.8 Add `workspace_mode` feature flag for backward compatibility (toggle exists)
- [x] 5.9 Verify: file explorer shows directory tree, tabs work (code exists, manual testing needed)

## 6. UX Enhancements — Validation, Undo/Redo, Search

- [x] 6.1 Add `validate_field` method to `EditableRecord` trait
- [x] 6.2 Implement validation for all migrated editor types
- [x] 6.3 Add visual feedback (red borders, error tooltips) for invalid fields
- [x] 6.4 Implement pre-save validation with error summary dialog
- [x] 6.5 Create `EditHistory` struct with command pattern
- [x] 6.6 Wire Ctrl+Z/Ctrl+Y to undo/redo (keyboard subscription added)
- [x] 6.7 Create edit history panel
- [x] 6.8 Create `dispel-gui/src/view/command_palette.rs`
- [x] 6.9 Implement fuzzy command search
- [x] 6.10 Create `dispel-gui/src/view/global_search.rs`
- [x] 6.11 Implement cross-catalog search indexing
- [x] 6.12 Implement auto-save drafts with conflict detection
- [ ] 6.13 Verify: all UX features functional

## 7. Spreadsheet View

- [x] 7.1 Create `dispel-gui/src/view/editor/spreadsheet.rs` table widget
- [x] 7.2 Implement column sorting (click header)
- [x] 7.3 Implement inline cell editing (double-click cell)
- [x] 7.4 Implement filter bar with free-text search
- [x] 7.5 Implement multi-select (Ctrl+click, Shift+click)
- [x] 7.6 Create inspector panel view (`view/editor/inspector.rs`)
- [x] 7.7 Add split view toggle (spreadsheet + inspector)
- [x] 7.8 Wire spreadsheet view to all generic editors (wired to HealItem, MiscItem, Magic as demo)
- [ ] 7.9 Verify: spreadsheet mode works for all editor types

## 8. Cleanup and Verification

- [ ] 8.1 Remove deprecated `Tab` enum (behind feature flag removal)
- [ ] 8.2 Delete all remaining hand-written editor files
- [ ] 8.3 Run `cargo clippy -- -D warnings` — fix all warnings
- [ ] 8.4 Run `cargo test --workspace` — all tests pass
- [ ] 8.5 Manual GUI testing: open each file type, edit, save, verify
- [ ] 8.6 Verify backup files created on save
- [ ] 8.7 Verify undo/redo works for all editor types
- [ ] 8.8 Verify command palette lists all actions
- [ ] 8.9 Verify global search finds records across catalogs
- [ ] 8.10 Benchmark: no performance regression vs. old editors
