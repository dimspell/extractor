# WeaponItem.db

> DISPEL® is a registered trademark. This project is not affiliated with, endorsed by, or sponsored by the trademark
> owner.

`CharacterInGame/WeaponItem.db` defines weapons and armour.

## Format

- Header: 4-byte little-endian `i32` record count.
- Records: `record_count` fixed-size records of 284 bytes.
- Text: `name` and `description` are null-padded WINDOWS-1250 strings.

The game loader reads the complete record block, then overwrites each record's
`weapon_item_id` word with its zero-based record index. The file's stored value at that offset is therefore not a stable
persistent ID.

| Offset | Size | Type   | Field              | Meaning                                        |
|-------:|-----:|--------|--------------------|------------------------------------------------|
|      0 |   30 | string | `name`             | Item name                                      |
|     30 |  202 | string | `description`      | Item description                               |
|    232 |    4 | i32    | `base_price`       | Shop value                                     |
|    236 |    4 | i32    | `weapon_item_id`   | Runtime ID; replaced with record index at load |
|    240 |    2 | i16    | `health_points`    | HP bonus                                       |
|    242 |    2 | i16    | `mana_points`      | MP bonus                                       |
|    244 |    2 | i16    | `strength`         | Strength bonus                                 |
|    246 |    2 | i16    | `agility`          | Agility bonus                                  |
|    248 |    2 | i16    | `wisdom`           | Wisdom/magic bonus                             |
|    250 |    2 | i16    | `constitution`     | Constitution bonus                             |
|    252 |    2 | i16    | `to_dodge`         | Evasion bonus (`UNIK`)                         |
|    254 |    2 | i16    | `to_hit`           | Accuracy bonus (`TRF`)                         |
|    256 |    2 | i16    | `attack`           | Attack bonus                                   |
|    258 |    2 | i16    | `defense`          | Defence bonus                                  |
|    260 |    2 | i16    | `magical_strength` | Magic-power bonus                              |
|    262 |    2 | i16    | `durability`       | Item durability                                |
|    264 |    2 | i16    | `reserved_0x108`   | Reserved; zero by default              |
|    266 |    2 | i16    | `reserved_0x10a`   | Reserved; zero by default              |
|    268 |    2 | i16    | `req_strength`     | Minimum strength                               |
|    270 |    2 | i16    | `reserved_0x10e`   | Reserved; zero by default              |
|    272 |    2 | i16    | `req_agility`      | Minimum agility                                |
|    274 |    2 | i16    | `reserved_0x112`   | Reserved; zero by default              |
|    276 |    2 | i16    | `req_wisdom`       | Minimum wisdom                                 |
|    278 |    2 | i16    | `reserved_0x116`   | Reserved; zero by default              |
|    280 |    2 | i16    | `reserved_0x118`   | Reserved; zero by default              |
|    282 |    2 | i16    | `reserved_0x11a`   | Reserved; zero by default              |

The seven reserved words are serialized and copied into save-game inventory records. No observed reader touches
them, and all known records contain zero at those offsets. Preserve their values when editing; their purpose is
unknown.
