# PrtIni.db

`NpcInGame/PrtIni.db` supplies the initial configuration for the eight
recruitable party-character slots in DISPEL®.

## Format

- No header.
- Exactly eight 28-byte (`0x1c`) records.
- The game reads record `party_slot * 0x1c` directly.
- `name` is a 20-byte, null-padded string. Shipped names are ASCII-compatible.

| Offset | Size | Type | Field | Meaning |
|---:|---:|---|---|---|
| `0x00` | 20 | string | `name` | Character name |
| `0x14` | 1 | u8 | `reserved_0x14` | Reserved; zero in shipped records |
| `0x15` | 1 | u8 | `class_id` | Class identifier; shipped values 21–24 |
| `0x16` | 1 | u8 | `starting_level` | Level used when creating the character |
| `0x17` | 1 | u8 | `pathfinding_mode` | Map/path-query mode; shipped value 7 |
| `0x18` | 4 | u32 | `character_variant` | Variant selector; shipped values 0 and 1 |

The loader copies `class_id`, `starting_level`, `pathfinding_mode`, and
`character_variant` into runtime character state. It uses `class_id` for
class-specific behavior and titles, and uses `character_variant` to choose one
of two class variants. The binary does not establish whether that variant is a
gender, portrait, or another presentation choice, so the parser uses the
neutral name `character_variant`.

DISPEL® is a registered trademark. This project is not affiliated with,
endorsed by, or sponsored by the trademark owner.
