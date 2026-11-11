# Magic.db — Spell Database

> DISPEL® is a registered trademark. This project is not affiliated with,
> endorsed by, or sponsored by the trademark owner.

## File Information

- **Location**: `MagicInGame/Magic.db`
- **Format**: Binary (Little-Endian)
- **Record Size**: 88 bytes
- **No Header**: Record count derived from file size

## Record Structure (88 bytes)

All fields are little-endian `u32`; the record ID is its zero-based position.

| Offset | Field                                            | Meaning                                                                                                                                             |
|-------:|--------------------------------------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------|
|      0 | `enabled`                                        | Spell availability flag                                                                                                                             |
|      4 | `effect_visual_blends_with_background`           | `0`=Off, `1`=On — uses blended rendering for the initial spell visual instead of direct blitting                                                    |
|      8 | `base_damage`                                    | Base damage used by the spell-damage calculation                                                                                                    |
|     12 | `base_success_rate`                              | Base casting-success chance before skill adjustment                                                                                                 |
|     16 | `mana_cost`                                      | Base mana cost before skill reduction; effective cost is at least 5                                                                                 |
| 20, 24 | `reserved_0x14`, `reserved_0x18`                 | Reserved words; zero by default                                                                                                                     |
|     28 | `effect_animation_repeats`                       | `0`=Off, `1`=On — repeats the target-effect animation while the target remains valid                                                                |
|     32 | `range`                                          | Maximum target distance checked by casting code                                                                                                     |
|     36 | `reserved_0x24`                                  | Reserved word; zero by default                                                                                                                      |
|     40 | `cast_duration`                                  | Maximum casting/action progress counter                                                                                                             |
|     44 | `animation_data_index`                           | Index into the animation data table, resolved to a pointer after file load                                                                          |
|  48–56 | `effect_value`, `effect_type`, `effect_modifier` | Effect configuration; exact semantics are not yet established                                                                                       |
|     60 | `reserved_0x3c`                                  | Reserved word; zero by default                                                                                                                      |
|     64 | `magic_type`                                     | Magic type (0=Magic, 1=HolyMagic, 2=DarkMagic) — selects which character magic-skill attribute drives damage, success, and mana-cost calculations |
|     68 | `target_animation_blends_with_background`        | `0`=Off, `1`=On — uses blended rendering for the target animation instead of direct blitting                                                        |
|     72 | `animation_set_id`                               | Cast-animation set                                                                                                                                  |
|     76 | `effect_visual_id`                               | Visual/projectile mapping selected when casting                                                                                                     |
|     80 | `icon_id`                                        | UI icon ID (inferred from its use as a UI-facing ID)                                                                                                |
|     84 | `targeting_mode`                                 | Targeting-mode configuration; exact value meanings need confirmation                                                                                |

## Field semantics

- Effective mana cost is derived from `mana_cost` and the caster's magic type skill, clamped to a minimum of 5.
- Effective success chance is derived from `base_success_rate` and that same skill category.
- `cast_duration` controls the progress limit used by the casting/action state.
- `effect_visual_id` selects the mapping used to create the spell visual or projectile.
- The flags at offsets 4 and 68 select the renderer's palette-blending path; clear values use direct pixel copies.
- `effect_animation_repeats` keeps the target-effect animation alive after its last frame.
- `animation_data_index` (offset 44) is resolved to a pointer after the file is loaded.

## File Purpose

Defines spell combat parameters, cast timing, and visual configuration. Effect-configuration words at offsets 48–56
remain intentionally offset-named until their runtime behavior is confirmed.

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
