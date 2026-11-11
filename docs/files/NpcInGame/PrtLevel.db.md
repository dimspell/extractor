# PrtLevel.db

`NpcInGame/PrtLevel.db` contains eight party-member progression tables, each with 20 levels. It has no header: the game
reads level `n` of party slot `s`
at `s * 720 + (n - 1) * 36`.

Each entry is 36 bytes.

| Offset | Size | Type | Field                    | Meaning                                              |
|-------:|-----:|------|--------------------------|------------------------------------------------------|
|    `0` |    1 | u8   | `magic_spell_id_1`       | First magic-spell ID (255 = absent)                  |
|    `1` |    1 | u8   | `magic_spell_id_2`       | Second magic-spell ID                                |
|    `2` |    1 | u8   | `magic_spell_id_3`       | Third magic-spell ID                                 |
|    `3` |    1 | u8   | `reserved_0x03`          | Alignment byte                                       |
|    `4` |    4 | u32  | `strength`               | Strength                                             |
|    `8` |    4 | u32  | `constitution`           | Constitution                                         |
|   `12` |    4 | u32  | `wisdom`                 | Wisdom                                               |
|   `16` |    2 | u16  | `health_points`          | Maximum HP                                           |
|   `18` |    2 | u16  | `mana_points`            | Maximum MP                                           |
|   `20` |    1 | u8   | `agility`                | Agility                                              |
|   `21` |    3 | u8   | `reserved` (bytes 21–23) | Reserved bytes                                       |
|   `24` |    1 | u8   | `attack`                 | Attack-related stat                                  |
|   `25` |    3 | u8   | `reserved` (bytes 25–27) | Reserved bytes                                       |
|   `28` |    4 | u32  | `weapon_skill_level`     | Party member's shared weapon proficiency             |
|   `32` |    4 | u32  | `tactical_action_chance` | Percentage threshold for a level-10+ tactical action |

The first three bytes were previously misidentified as a sentinel. They are magic-spell references copied into runtime
state. All reserved bytes are preserved verbatim by the parser.

`weapon_skill_level` is not MP recharge: the game uses it in the same generic weapon calculations that select one of the
player's weapon-skill levels.
`tactical_action_chance` is not defence: it is compared to a random value from 0 to 99 before changing the party
member's tactical action state.

DISPEL® is a registered trademark. This project is not affiliated with, endorsed by, or sponsored by the trademark
owner.
