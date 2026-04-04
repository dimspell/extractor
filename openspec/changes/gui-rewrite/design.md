## Context

The dispel-gui application currently uses:
- A 3,043-line `app.rs` god object handling state, messages, update logic, and view dispatch
- 34 hardcoded tabs in a `Tab` enum with a 30+ if-else chain for view routing
- 26 hand-written editor implementations (~8,000 lines) with massive duplication
- Only 2 editors (WeaponEditor, MonsterRefEditor) using the existing generic infrastructure
- No validation, undo/redo, global search, or command palette
- Silent parse failures on invalid input

Two planning documents already exist:
- `docs/GUI_REDESIGN_PLAN.md` — vision, architecture, UI mockups, component specs
- `docs/IMPLEMENTATION_PLAYBOOK.md` — step-by-step implementation guide with code patterns

This change executes those plans across 5 phases.

## Goals / Non-Goals

**Goals:**
- Reduce `app.rs` from 3,043 to < 200 lines through module splitting
- Migrate 100% of editors to generic `EditableRecord` infrastructure
- Replace hardcoded 34-tab sidebar with file explorer + dynamic workspace
- Add validation, undo/redo, command palette, global search, auto-save
- Maintain backward compatibility during transition via feature flag
- Preserve existing DbViewer, SpriteBrowser, and Map tab functionality

**Non-Goals:**
- No changes to dispel-core parsers, Extractor trait, or registry
- No visual map editor (Phase 5 feature — deferred)
- No sprite animation timeline (Phase 5 feature — deferred)
- No mod packaging system (Phase 5 feature — deferred)
- No derive macro for EditableRecord (Phase 5 feature — deferred)

## Decisions

### 1. Incremental Migration with Feature Flag

**Decision:** Keep the existing `Tab` enum during transition. New workspace model is opt-in via `workspace_mode` feature flag.

**Rationale:** Allows testing the new UI without breaking existing workflows. Users can switch back if issues arise.

**Alternatives considered:**
- Big-bang migration — too risky, no rollback path
- Dual-maintenance of both systems — too much overhead

### 2. Domain-Split Message Enums

**Decision:** Top-level `AppMessage` wraps domain-specific enums (`EditorMessage`, `ExplorerMessage`, `DbViewerMessage`).

**Rationale:** Keeps each domain's messages focused. The update dispatcher routes to the correct handler. Easier to test individual domains.

**Alternatives considered:**
- Single flat enum — becomes unwieldy at 200+ variants
- Trait-based message handling — too much indirection for iced's update model

### 3. GenericEditorState Wraps MultiFileEditorState

**Decision:** `MultiFileEditorState<R>` contains a `GenericEditorState<R>` as its `editor` field, rather than duplicating methods.

**Rationale:** Zero code duplication. Multi-file editors get all generic editor features (refresh, select, update_field, save) for free. Only file-list-specific methods are added.

**Alternatives considered:**
- Separate trait with shared methods — more boilerplate
- Inheritance via macro — harder to debug

### 4. Viewport-Based Map Rendering (Not Full Image)

**Decision:** Map editor loads tiles on-demand into an LRU cache (max 50MB), never renders the full 300MB+ image.

**Rationale:** 300MB images would crash or freeze the GUI. Viewport loading keeps memory predictable and enables smooth pan/zoom.

**Alternatives considered:**
- Full image with downscaling — still requires loading 300MB into memory
- Server-side rendering — adds infrastructure complexity

### 5. EditableRecord Trait Over Derive Macro (Initially)

**Decision:** Implement `EditableRecord` by hand for all 26 types first. Add derive macro in Phase 5.

**Rationale:** Derive macros add build-time complexity and debugging difficulty. Hand-written impls are clearer for the initial migration. The derive macro can replace them later without changing the trait contract.

**Alternatives considered:**
- Derive macro first — faster to write, harder to debug issues
- Code generation script — adds external dependency

### 6. Command Pattern for Undo/Redo

**Decision:** Each field edit creates an `EditCommand` with `execute()` and `undo()` methods, stored in a bounded stack.

**Rationale:** Clean separation between state mutation and history. Easy to add "describe" for undo history panel. Bounded stack prevents unbounded memory growth.

**Alternatives considered:**
- Full state snapshots — too memory-intensive for large catalogs
- Operational transforms — overkill for single-user editing

## Risks / Trade-offs

| Risk | Mitigation |
|------|-----------|
| Breaking existing editor behavior during migration | Run full test suite after each migration; keep old editors until new ones verified |
| Feature flag complexity | Single boolean flag; default to `false` (old behavior); remove flag after stabilization |
| Performance regression with generic editors | Benchmark extract/patch times; generic editors add negligible overhead (trait dispatch) |
| Undo/redo memory usage | Bounded stack (default 100 operations); configurable via settings |
| File explorer tree performance on large directories | Lazy loading — only read directory contents when node is expanded |
| iced framework limitations for spreadsheet view | Start with basic table; enhance incrementally; fall back to inspector-only if needed |
| Migration timeline creep | Strict phase boundaries; park new ideas in "Future" section of plan |
