# AGENTS.md — Dispel Game File Extractor

**A game modding toolkit for Dispel RPG.** Reads/writes game files in native binary, INI, and DB formats. All edits are persisted directly—no intermediate database.

---

## Project Structure

```
dispel-extractor/
├── src/ (dispel-core)    Parsers, binary readers, EditableRecord trait
├── main.rs               CLI binary wrapping dispel-core
└── dispel-gui/           Desktop GUI (Iced 0.14, Elm/MVU)
```

**Separation of concerns:**
- `dispel-core`: game logic only. Zero GUI/presentation code.
- `dispel-gui`: UI consumer of dispel-core. Zero game logic.
- `dispel-extractor`: thin CLI wrapper.

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

**Encoding is critical:** `Event.ini`, `Npc.ini`, `Monster.db` (desc) → **EUC-KR**. `Monster.ini`, `Store.db`, `WeaponItem.db` → **WINDOWS-1250**.

### Item Type Enum
| Value | Type | Database |
|-------|------|----------|
| 0/1 | Weapon/Armor | WeaponItem.db |
| 2 | Heal | HealItem.db |
| 3 | Misc | MiscItem.db |
| 4 | Edit | EditItem.db |
| 5 | Event | EventItem.db |

### File Dependencies
- **AllMap.ini** → Map.ini (per-map config) → Map.map (geometry, sprites, events)
- **Monster.ini** (visual) + **Monster.db** (combat stats)
- **Npc.ini** (visual) + **DlgMapFiles.dlg** (dialogue)
- See `/docs` for full dependency graphs.

---

## Binary Format Essentials

- **All integers**: little-endian
- **Colors**: RGB 565 (16-bit)
- **Map tiles**: seeked from EOF, event/GTL/BTL layers
- **Sprites**: start at byte 268; validation: `ints[11] * ints[12] == ints[13]`
- **Tilesets**: contiguous 32×32 tiles, RGB 565 (2 bytes/pixel)
- **Map rendering**: isometric projection, 62×32 display size. **Warning:** full map = ~300MB; use viewport + LRU cache (≤50MB).
- **SNF audio**: custom PCM header, prepend RIFF WAVE header for conversion.

See `/docs` for detailed format specs.

---

## GUI Architecture

### Tech Stack
- **UI**: Iced 0.14 (GPU via wgpu, Elm/MVU)
- **Async**: Tokio
- **Core**: dispel_core (sibling crate)
- **Search**: nucleo-matcher (fuzzy)
- **SQLite**: optional (DbViewer only)

### Flow
`user action → Message → update/ handler (mutates AppState) → view/ (pure render)`

### Editor System
**27 editor types**, one per game data category. All use:
```rust
pub struct GenericEditorState<R: EditableRecord> {
    pub catalog: Option<Vec<R>>,
    pub selected_idx: Option<usize>,
    pub edit_history: EditHistory,
}
```

**`EditableRecord` trait** (dispel-core):
```rust
fn field_descriptors() -> &'static [FieldDescriptor];
fn get_field(&self, field: &str) -> String;
fn set_field(&mut self, field: &str, value: String) -> bool;
fn list_label(&self) -> String;
```

**Field kinds:** String, Integer, Enum, Lookup (runtime dropdown).

### Key Files

| File | Role |
|------|------|
| `app.rs` | App struct, AppState, init |
| `workspace.rs` | Tab management, recent files |
| `generic_editor.rs` | GenericEditorState<R>, filtering, history |
| `message/` | Message enum + routing macro |
| `update/` | Domain handlers |
| `view/` | Pure render functions (one per editor) |
| `state/` | Per-editor state structs (27 types) |

### Adding a New Editor

1. Implement `EditableRecord` in `dispel-core/src/references/`
2. Create state alias: `pub type MyEditorState = GenericEditorState<MyType>;`
3. Create view: `impl App { pub fn view_my_editor_tab(&self) -> Element { ... } }`
4. Add field to `AppState`
5. Add `Message` variants (use `define_message_ext!` macro)
6. Add handler in `update/editor/`
7. Wire `EditorType::from_path()` for file extension detection
8. Wire `view/mod.rs` dispatch
9. Delete old editor code—never keep both

### Conventions

- **Async**: Use `iced::Task` + Tokio. Never block the UI thread.
- **State mutation**: `update/` only. Views are pure.
- **Views**: defined as `impl App` blocks in `view/*.rs`, not `app.rs`.
- **Messages**: use `define_message_ext!` macro.
- **Scan vs Browse**: "Scan" = load from `shared_game_path`; "Browse" = file picker.
- **SQLite**: only `DbViewer`. Other editors read/write game files directly.

### Iced 0.14 Lessons

| Problem | Fix |
|---------|-----|
| Views in both `app.rs` and `view/mod.rs` | Keep only in `view/*.rs` |
| `center_x()` compile error | Use `container(...).align_x(Horizontal::Center)` |
| Deprecated patterns | Check Iced 0.14 docs |

---

## Development Best Practices

### Error Handling
- `dispel-core`: `thiserror` for enumerable error types
- GUI/CLI: `anyhow` for contextual bubbling
- Never `.unwrap()` on file I/O—show errors in `status_msg`

### State & Async
- Use **enums over booleans**: `LoadingState<T> { Idle, Loading, Loaded(T), Failed(String) }`
- Always use `Task::perform` for async work—never block

### Code Quality
- Clippy: zero warnings (`cargo clippy --workspace --all-features -- -D warnings`)
- Format: `cargo fmt --all` before commit
- Validate all binary bounds before indexing

### Testing
- **dispel-core**: unit test every new parser with hardcoded byte slices
- **dispel-gui**: test state transitions, not visuals
- **Round-trip tests**: read → parse → write → verify byte-for-byte match
- Run before every commit: `cargo test --workspace --all-features`

### Tools
- `cargo check --message-format=short`: fast compile errors
- `rtk cargo test`: optimized test runs
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
```

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
- **Hardcoded paths**: Use `dirs` crate for config/cache
- **Text encoding**: Check encoding table before reading—wrong codec = corruption
- **Map memory**: Never load full rendered map (~300MB); use viewport + LRU cache

---

## Quick Commands

```bash
cargo build --workspace                           # Build all
cargo test --workspace --all-features             # Test all
cargo clippy --workspace -- -D warnings           # Lint
cargo fmt --all                                   # Format
cargo check -p dispel-gui --message-format=short # Fast errors
```

---

*Last updated: 2026-04-18*  
**DISPEL®** is a registered trademark. This project is **not affiliated with, endorsed by, or sponsored by** the trademark owner.
