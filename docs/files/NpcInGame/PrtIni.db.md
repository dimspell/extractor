# PrtIni.db

`NpcInGame/PrtIni.db` supplies the initial configuration for the eight recruitable party-character slots in DISPEL®.

## Format

- No header.
- Exactly eight 28-byte records.
- The parser addresses records as `party_slot * 28`.
- `name` is a 20-byte, null-padded string. Names are ASCII-compatible.

| Offset | Size | Type   | Field               | Meaning                                  |
|-------:|-----:|--------|---------------------|------------------------------------------|
|    `0` |   20 | string | `name`              | Character name                           |
| `20` | 1 | u8 | `reserved_0x14` | Reserved; zero by default |
| `21` | 1 | u8 | `class_id` | Class identifier; observed values 21–24 |
|   `22` |    1 | u8     | `starting_level`    | Level used when creating the character   |
| `23` | 1 | u8 | `pathfinding_mode` | Map/path-query mode; observed value 7 |
| `24` | 4 | u32 | `character_variant` | Variant selector; observed values 0 and 1 |

The tool reads `class_id`, `starting_level`, `pathfinding_mode` and `character_variant` into character state. The supplied data does not establish what `character_variant` represents. The parser uses that neutral name.


