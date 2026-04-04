# Dispel GUI Rewrite — Lessons Learned

## Architecture Insights

### What Worked Well

1. **`impl App` in view modules** — Defining view methods as `impl App` blocks in `dispel-gui/src/view/*.rs` is clean and avoids passing `&App` through function parameters. The Rust compiler resolves all `impl App` blocks across modules automatically.

2. **Generic editor with `EditableRecord` trait** — The trait-based approach eliminates ~90% of duplicated editor code. Each editor becomes:
   - 1 trait impl in `dispel-core` (~60 lines)
   - 1 state alias in `dispel-gui` (~4 lines)
   - 1 view stub in `dispel-gui/src/view/` (~18 lines)
   - Total: ~82 lines per editor vs. ~300-500 lines for hand-written editors

3. **`GenericEditorState<R>`** — Single generic state struct replaces 26 hand-written state structs. The `select()`, `update_field()`, `refresh()`, and `save()` methods work identically for all types.

4. **Workspace model with `workspace_mode` toggle** — Keeping the legacy mode alongside the new workspace mode allows incremental migration without breaking existing functionality.

### What Didn't Work

1. **Duplicate `impl App` blocks** — When I added view methods to both `app.rs` and `view/mod.rs`, the compiler reported "duplicate definitions". The solution: keep view methods ONLY in the view modules, never in `app.rs`.

2. **Early returns in `view()` bypass sidebar** — The original `view/mod.rs` had an early `return` for `Map | Ref | Database` tabs that skipped the sidebar layout. This caused the "no sidebar" bug. All tabs must flow through the same layout pipeline.

3. **`center_x()` / `center_y()` require arguments** — In newer iced versions, these methods require width/height parameters. Use `align_x(Horizontal::Center)` and `align_y(Vertical::Center)` on containers instead.

4. **Free function vs method naming conflicts** — Having both a free function `pub fn view(app: &App)` in `app.rs` and a method `pub fn view(&self)` in `view/mod.rs`'s `impl App` block causes confusion. The free function should delegate to the method.

---

## Iced Framework Learnings

### Widget Patterns

1. **`Element<'a, Message>` is the universal return type** — All view methods return `Element<'a, Message>`. Use `.into()` to convert widgets to `Element`.

2. **Lifetime annotations on `render_node`** — When building recursive tree views, the lifetime must be explicit:
   ```rust
   fn render_node<'a>(node: &'a TreeNode, query: &'a str) -> Element<'a, Message>
   ```

3. **`column!` macro vs `Column::new()`** — `column![...]` is convenient but sometimes type inference fails. Use `Column::new().push(...)` for complex cases.

4. **`container().align_x()` vs `.center_x()`** — `.center_x()` was deprecated. Use:
   ```rust
   container(content)
       .width(Fill)
       .height(Fill)
       .align_x(iced::alignment::Horizontal::Center)
       .align_y(iced::alignment::Vertical::Center)
   ```

### Message Handling

1. **Message enum grows fast** — With 34 tabs and ~6 messages per tab, the `Message` enum has 200+ variants. Consider splitting into domain enums with a wrapper:
   ```rust
   pub enum Message {
       Weapon(WeaponMessage),
       HealItem(HealItemMessage),
       // ...
   }
   ```

2. **`Task::none()` is the default** — Most message handlers return `Task::none()`. Only async operations (file I/O, network) return `Task::perform()`.

3. **Scan vs Browse naming** — "Scan" means "load from `shared_game_path`", "Browse" means "open file picker". Mixing these causes UX bugs (Scan button opening file picker).

---

## Rust Patterns

### Trait Design

1. **`EditableRecord` trait with associated functions** — Using `fn field_descriptors() -> &'static [FieldDescriptor]` (no `&self`) allows compile-time field definitions. This is more efficient than computing descriptors at runtime.

2. **`FieldKind::Enum` with static variants** — `variants: &["Weapon", "Healing", "Edit", "Event", "Misc", "Other"]` — the variants array is `&'static [&'static str]`, allocated once at compile time.

3. **`FieldKind::Lookup(key)`** — Runtime lookup key that maps to `state.lookups[key]`. This enables cross-referencing (e.g., monster names in MonsterRef editor) without hardcoding data.

### State Management

1. **`Box<dyn Trait>` vs `Box<ConcreteType>`** — Using `Box<GenericEditorState<ConcreteType>>` is better than `Box<dyn EditorState>` because the generic type preserves compile-time type information.

2. **`Default` derive on enums requires `#[default]` attribute** — When adding `Default` to an enum, you must mark one variant with `#[default]`:
   ```rust
   #[derive(Default)]
   pub enum ItemTypeId {
       #[default]
       Weapon = 1,
       Healing = 2,
       // ...
   }
   ```

3. **`HashMap` field ordering** — In `AppState::default()`, the order of field initialization matters for readability. Group related fields together (editors, lookups, workspace).

---

## File Organization

### What Worked

1. **`view/*.rs` as `impl App` blocks** — Each editor view is a separate file with `impl App { pub fn view_xyz_tab(&self) -> Element<'_, Message> { ... } }`. This keeps view code co-located and avoids import complexity.

2. **`generic_editor.rs` in dispel-gui** — The `build_editor_view()` function lives in the GUI crate, not the core crate, because it depends on iced widgets and Message types.

3. **`editable.rs` in dispel-core** — The `EditableRecord` trait and `FieldDescriptor`/`FieldKind` types live in the core crate because they're domain concepts, not UI concepts.

### What to Improve

1. **`app.rs` is still too large** — At ~3,000 lines, the `update()` method dominates. The next step is to split it into domain-specific update handlers (as planned in the tasks file).

2. **Message enum is in a single file** — At 360+ lines, `message.rs` should be split into domain modules (e.g., `message/weapon.rs`, `message/heal_item.rs`).

3. **State aliases are scattered** — Each `<type>_editor.rs` file in `dispel-gui/src/` is a 4-line state alias. These could be consolidated into a single `editor_states.rs` file.

---

## Debugging Tips

### Common Errors

| Error | Cause | Fix |
|-------|-------|-----|
| `duplicate definitions with name \`view\`` | Same method defined in `app.rs` and `view/mod.rs` | Remove from `app.rs`, keep in `view/mod.rs` |
| `no method named \`view_tab_content\`` | Method is `fn` (private) in `view/mod.rs` | Change to `pub fn` |
| `unexpected closing delimiter` | Mismatched braces after large edits | Check brace count with `grep -c '{'` vs `grep -c '}'` |
| `cannot find type \`Path\` in this scope` | Removed `use std::path::{Path, PathBuf}` | Re-add the import |
| `Scan button opens file picker` | Using `BrowseGamePath` instead of `ScanItems` | Use `Message::TypeOpScanItems` |

### Useful Commands

```bash
# Check compilation without building
cargo check

# Fix warnings automatically
cargo fix --bin dispel-gui --allow-dirty

# Count lines per file
find src -name "*.rs" -exec wc -l {} + | sort -n

# Find all impl App blocks
grep -rn "impl App" src/

# Find all Message variants
grep -n "Message::" src/app.rs | head -20
```

---

## Design Decisions

### Why `EditableRecord` instead of derive macro?

**Decision:** Hand-written trait implementations instead of `#[derive(EditableRecord)]`.

**Rationale:**
- Derive macros add build-time complexity and debugging difficulty
- Field labels and validation rules often need customization that macros can't express
- The trait is simple enough (8 methods) that manual implementation is manageable
- Can always add a derive macro later as an optimization

### Why keep legacy mode alongside workspace mode?

**Decision:** `workspace_mode: bool` toggle instead of replacing the sidebar immediately.

**Rationale:**
- Allows testing the workspace mode without breaking existing functionality
- Users can switch back if something doesn't work
- Incremental migration is safer than big-bang replacement
- Can remove legacy mode once workspace mode is proven stable

### Why `GenericEditorState<R>` instead of trait objects?

**Decision:** `Box<GenericEditorState<ConcreteType>>` instead of `Box<dyn EditorState>`.

**Rationale:**
- Generic types preserve compile-time type information
- No dynamic dispatch overhead
- The `build_editor_view()` function is generic over `R: EditableRecord`
- Each editor's state is a concrete type, making debugging easier

---

## Metrics

### Code Reduction

| Metric | Before | After | Change |
|--------|--------|-------|--------|
| `app.rs` lines | 3,043 | ~3,000 | -1% (update handler still inline) |
| `state.rs` lines | 0 | 173 | +173 (new file) |
| Generic editors | 2 (6%) | 7 (21%) | +5 |
| Hand-written editors | ~26 | ~21 | -5 |
| Total GUI code | ~14,600 | ~14,200 | -400 lines |

### Build Performance

| Metric | Value |
|--------|-------|
| `cargo check` time | ~1.5s |
| `cargo build` time | ~3s |
| Warnings | 0 |
| Errors | 0 |

---

*Last updated: 2026-04-04*
