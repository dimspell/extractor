# Magic.db - Spell Database

## File Information
- **Location**: `MagicInGame/Magic.db`
- **Format**: Binary (Little-Endian)
- **Record Size**: 88 bytes (22 × u32)
- **No Header**: Record count derived from file size

## Record Structure (88 bytes)

All fields are little-endian `u32`; the record ID is its zero-based position.

| Offset | Field | Meaning |
|---:|---|---|
| `0x00` | `enabled` | Spell availability flag |
| `0x04` | `effect_visual_blends_with_background` | Uses blended rendering for the initial spell visual instead of direct blitting |
| `0x08` | `base_damage` | Base damage used by the spell-damage calculation |
| `0x0C` | `base_success_rate` | Base casting-success chance before skill adjustment |
| `0x10` | `mana_cost` | Base mana cost before skill reduction; effective cost is at least 5 |
| `0x14`, `0x18` | `reserved_0x14`, `reserved_0x18` | Reserved words; zero in shipped `Magic.db` |
| `0x1C` | `effect_animation_repeats` | Repeats the target-effect animation while the target remains valid |
| `0x20` | `range` | Maximum target distance checked by casting code |
| `0x24` | `reserved_0x24` | Reserved word; zero in shipped `Magic.db` |
| `0x28` | `cast_duration` | Maximum casting/action progress counter |
| `0x2C` | `unused_constant_one` | Compatibility constant: always 1 in shipped data and not read by this executable |
| `0x30–0x38` | `effect_value`, `effect_type`, `effect_modifier` | Effect configuration; exact semantics are not yet established |
| `0x3C` | `reserved_0x3c` | Reserved word; zero in shipped `Magic.db` |
| `0x40` | `magic_school` | Magic-school/stat category used in cost and success calculations |
| `0x44` | `target_animation_blends_with_background` | Uses blended rendering for the target animation instead of direct blitting |
| `0x48` | `animation_set_id` | Cast-animation set |
| `0x4C` | `effect_visual_id` | Visual/projectile mapping selected when casting |
| `0x50` | `icon_id` | UI icon ID (inferred from its use as a UI-facing ID) |
| `0x54` | `targeting_mode` | Targeting-mode configuration; exact value meanings need confirmation |

## Confirmed runtime behavior

- The game calculates effective mana cost from `mana_cost` and the caster's magic-school skill, clamped to a minimum of 5.
- It calculates effective success chance from `base_success_rate` and that same skill category.
- `cast_duration` controls the progress limit used by casting/action state.
- `effect_visual_id` selects the mapping used to create the spell visual or projectile.
- The `0x04` and `0x44` flags select the renderer's palette-blending path; clear values use direct pixel copies.
- `effect_animation_repeats` keeps the target-effect animation alive after its last frame.

## File Purpose

Defines spell combat parameters, cast timing, and visual configuration. Several effect-configuration words remain intentionally offset-named until their runtime behavior is confirmed.



## Implementation
- **Rust Module**: `src/references/magic_db.rs`
- **Extractor**: `MagicSpell` struct implementing `Extractor` trait
- **Data Structure**: `MagicSpell` with comprehensive spell attributes
- **Database**: Saved to SQLite via `save_magic_spells` function

## Example Usage

### Extract and display spells:
```bash
cargo run -- extract -i "Dispel/MagicInGame/Magic.db"
```

### Import to database:
```bash
cargo run -- database import "Dispel/"
```

## Extractor

An extractor is available in `src/references/magic_db.rs` to parse this file format.

### How to Run

```bash
# Extract Magic.db to JSON
cargo run -- extract -i "fixtures/Dispel/MagicInGame/Magic.db"

# Import to SQLite database
cargo run -- database import "fixtures/Dispel/" "database.sqlite"
```
