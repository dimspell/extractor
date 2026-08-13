# WeaponItem.db

`CharacterInGame/WeaponItem.db` defines weapons and armour for DISPEL®.

## Format

- Header: 4-byte little-endian `i32` record count.
- Records: `record_count` fixed-size records of 284 bytes (`0x11c`).
- Text: `name` and `description` are null-padded WINDOWS-1250 strings.

The game loader reads the complete record block, then overwrites each record's
`weapon_item_id` word with its zero-based record index. The file's stored value
at that offset is therefore not a stable persistent ID.

| Offset | Size | Type | Field | Meaning |
|---:|---:|---|---|---|
| `0x000` | 30 | string | `name` | Item name |
| `0x01e` | 202 | string | `description` | Item description |
| `0x0e8` | 4 | i32 | `base_price` | Shop value |
| `0x0ec` | 4 | i32 | `weapon_item_id` | Runtime ID; replaced with record index at load |
| `0x0f0` | 2 | i16 | `health_points` | HP bonus |
| `0x0f2` | 2 | i16 | `mana_points` | MP bonus |
| `0x0f4` | 2 | i16 | `strength` | Strength bonus |
| `0x0f6` | 2 | i16 | `agility` | Agility bonus |
| `0x0f8` | 2 | i16 | `wisdom` | Wisdom/magic bonus |
| `0x0fa` | 2 | i16 | `constitution` | Constitution bonus |
| `0x0fc` | 2 | i16 | `to_dodge` | Evasion bonus (`UNIK`) |
| `0x0fe` | 2 | i16 | `to_hit` | Accuracy bonus (`TRF`) |
| `0x100` | 2 | i16 | `attack` | Attack bonus |
| `0x102` | 2 | i16 | `defense` | Defence bonus |
| `0x104` | 2 | i16 | `magical_strength` | Magic-power bonus |
| `0x106` | 2 | i16 | `durability` | Item durability |
| `0x108` | 2 | i16 | `reserved_0x108` | Reserved; zero in shipped records |
| `0x10a` | 2 | i16 | `reserved_0x10a` | Reserved; zero in shipped records |
| `0x10c` | 2 | i16 | `req_strength` | Minimum strength |
| `0x10e` | 2 | i16 | `reserved_0x10e` | Reserved; zero in shipped records |
| `0x110` | 2 | i16 | `req_agility` | Minimum agility |
| `0x112` | 2 | i16 | `reserved_0x112` | Reserved; zero in shipped records |
| `0x114` | 2 | i16 | `req_wisdom` | Minimum wisdom |
| `0x116` | 2 | i16 | `reserved_0x116` | Reserved; zero in shipped records |
| `0x118` | 2 | i16 | `reserved_0x118` | Reserved; zero in shipped records |
| `0x11a` | 2 | i16 | `reserved_0x11a` | Reserved; zero in shipped records |

The seven reserved words are serialized and copied into save-game inventory
records. No code path in the supplied pseudocode reads them, and all 87
records in the supplied game database contain zero at those offsets. Preserve
their values when editing; their purpose is unknown.

DISPEL® is a registered trademark. This project is not affiliated with,
endorsed by, or sponsored by the trademark owner.
