# PrtIni.db

`NpcInGame/PrtIni.db` supplies the initial configuration for the eight recruitable party-character slots in DISPEL®.

## Format

- No header.
- Exactly eight 28-byte records.
- Records are addressed as `party_slot * 28`.
- `name` is a 20-byte, null-padded string. Names are ASCII-compatible.

| Offset | Size | Type   | Field               | Meaning                                  |
|-------:|-----:|--------|---------------------|------------------------------------------|
|    `0` |   20 | string | `name`              | Character name                           |
| `20` | 1 | u8 | `reserved_0x14` | Reserved; zero by default |
| `21` | 1 | u8 | `class_id` | Class identifier; observed values 21–24 |
|   `22` |    1 | u8     | `starting_level`    | Level used when creating the character   |
| `23` | 1 | u8 | `pathfinding_mode` | Map/path-query mode; observed value 7 |
| `24` | 4 | u32 | `character_variant` | Variant selector; observed values 0 and 1 |

The loader copies `class_id`, `starting_level`, `pathfinding_mode`, and
`character_variant` into runtime character state. It uses `class_id` for class-specific behavior and titles, and uses
`character_variant` to choose one of two class variants. The binary does not establish whether that variant is a gender,
portrait, or another presentation choice, so the parser uses the neutral name `character_variant`.

DISPEL® is a registered trademark. This project is not affiliated with, endorsed by, or sponsored by the trademark
owner.
