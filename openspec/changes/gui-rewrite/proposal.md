## Why

The dispel-gui application has a 3,043-line monolithic `app.rs` god object, 94% of editors use hand-written duplicated code instead of the generic infrastructure, and the navigation model (34 hardcoded tabs) doesn't scale. Users face silent parse failures, no undo/redo, no global search, and no validation. The existing `GUI_REDESIGN_PLAN.md` and `IMPLEMENTATION_PLAYBOOK.md` documents define a clear path forward — this change executes that plan.

## What Changes

- **Split `app.rs`** (3,043 lines) into focused modules: `state.rs`, `navigation.rs`, `messages.rs`, `update/` domain handlers
- **Replace 34-tab sidebar** with file explorer tree + dynamic workspace tabs
- **Migrate all 26 hand-written editors** to `GenericEditorState` + `EditableRecord` pattern (2 → 34 editors using generic infrastructure)
- **Add spreadsheet view** for bulk editing with column sorting, filtering, inline editing, multi-select
- **Add field validation** with visual feedback (red borders, error tooltips) and pre-save validation
- **Add undo/redo** system with command pattern and Ctrl+Z/Ctrl+Y support
- **Add command palette** (Ctrl+P) for quick action access
- **Add global search** (Ctrl+Shift+F) across all loaded catalogs
- **Add auto-save drafts** with conflict detection for externally modified files
- **Remove** `Tab` enum in favor of workspace model (with backward-compatible feature flag during transition)
- **BREAKING**: Old tab-based navigation replaced by file-tree workspace (legacy mode available via feature flag)

## Capabilities

### New Capabilities
- `file-explorer`: Tree-view file browser with type icons, status badges, search, context menu, and double-click-to-open
- `workspace-model`: Dynamic tab management with open/close/reorder/pin, replacing fixed tab enumeration
- `spreadsheet-view`: Bulk table editing with sortable columns, inline editing, multi-select, and filter bar
- `field-validation`: Per-field validation on input with visual feedback and pre-save validation summary
- `undo-redo`: Command-pattern edit history with Ctrl+Z/Ctrl+Y and edit history panel
- `command-palette`: Ctrl+P fuzzy-search quick access to all actions, files, and settings
- `global-search`: Ctrl+Shift+F cross-catalog search with result navigation
- `auto-save`: Draft persistence with crash recovery and external file conflict detection
- `editor-migration`: All 26 hand-written editors migrated to generic `EditableRecord` infrastructure

### Modified Capabilities
- `extract-command`: No changes (existing capability, unchanged)
- `patch-command`: No changes (existing capability, unchanged)

## Impact

- **`dispel-gui/src/app.rs`**: 3,043 → < 200 lines (split into modules)
- **`dispel-gui/src/`**: New modules: `state.rs`, `navigation.rs`, `messages.rs`, `update/`, `components/`, `theme/`
- **`dispel-gui/src/*_editor.rs`**: 26 hand-written files replaced by 4-line type aliases
- **`dispel-gui/src/view/*_editor.rs`**: 26 hand-written views replaced by single-line `build_editor_view()` calls
- **`dispel-gui/src/types.rs`**: `Tab` enum deprecated (kept behind feature flag during transition)
- **`dispel-core/src/references/`**: 26 new `EditableRecord` impl files (one per type)
- **No changes** to `DbViewer`, `SpriteBrowser`, or `Map` tab (kept as-is)
- The code must be valid (`cargo check`, `cargo check -p dispel-gui`, `cargo clippy`) and well formatted (`cargo fmt`).
