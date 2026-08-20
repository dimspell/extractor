# PrtLevel.db

`NpcInGame/PrtLevel.db` contains eight party-member progression tables, each
with 20 levels. It has no header: the game reads level `n` of party slot `s`
at `s * 0x2d0 + (n - 1) * 0x24`.

Each entry is 36 bytes (`0x24`).

| Offset | Size | Type | Field | Meaning |
|---:|---:|---|---|---|
| `0x00` | 1 | u8 | `magic_spell_id_1` | First magic-spell ID (`0xff` = absent) |
| `0x01` | 1 | u8 | `magic_spell_id_2` | Second magic-spell ID |
| `0x02` | 1 | u8 | `magic_spell_id_3` | Third magic-spell ID |
| `0x03` | 1 | u8 | `reserved_0x03` | Alignment byte |
| `0x04` | 4 | u32 | `strength` | Strength |
| `0x08` | 4 | u32 | `constitution` | Constitution |
| `0x0c` | 4 | u32 | `wisdom` | Wisdom |
| `0x10` | 2 | u16 | `health_points` | Maximum HP |
| `0x12` | 2 | u16 | `mana_points` | Maximum MP |
| `0x14` | 1 | u8 | `agility` | Agility |
| `0x15` | 3 | u8 | `reserved_0x15..17` | Reserved bytes |
| `0x18` | 1 | u8 | `attack` | Attack-related stat |
| `0x19` | 3 | u8 | `reserved_0x19..1b` | Reserved bytes |
| `0x1c` | 4 | u32 | `weapon_skill_level` | Party member's shared weapon proficiency |
| `0x20` | 4 | u32 | `tactical_action_chance` | Percentage threshold for a level-10+ tactical action |

The first three bytes were previously misidentified as a sentinel. They are
magic-spell references copied into runtime state. All
reserved bytes are preserved verbatim by the parser.

`weapon_skill_level` is not MP recharge: the game uses it in the same generic
weapon calculations that select one of the player's weapon-skill levels.
`tactical_action_chance` is not defence: it is compared to a random value from
0 to 99 before changing the party member's tactical action state.

DISPEL® is a registered trademark. This project is not affiliated with,
endorsed by, or sponsored by the trademark owner.
