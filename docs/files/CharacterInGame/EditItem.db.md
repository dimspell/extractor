# EditItem.db

> DISPEL® is a registered trademark. This project is not affiliated with, endorsed by, or sponsored by the trademark
> owner.

`CharacterInGame/EditItem.db` defines consumable item-modification materials. It is a little-endian binary file with a
four-byte record count followed by fixed-size 268-byte records. Text is encoded as WINDOWS-1250.

## Record layout

| Offset | Size | Field                     | Meaning                                                         |
|-------:|-----:|---------------------------|-----------------------------------------------------------------|
|      0 |   30 | `name`                    | Null-padded item name.                                          |
|     30 |  202 | `description`             | Null-padded description.                                        |
|    232 |    4 | `base_price`              | Shop price.                                                     |
|    236 |    4 | `runtime_item_id`         | Runtime ID assigned by the game loader.                         |
|    240 |    2 | `health_points`           | HP modifier.                                                    |
|    242 |    2 | `mana_points`             | MP modifier.                                                    |
|    244 |    2 | `strength`                | Strength modifier.                                              |
|    246 |    2 | `agility`                 | Agility modifier.                                               |
|    248 |    2 | `wisdom`                  | Wisdom modifier.                                                |
|    250 |    2 | `constitution`            | Constitution modifier.                                          |
|    252 |    2 | `to_dodge`                | Dodge modifier.                                                 |
|    254 |    2 | `to_hit`                  | Hit modifier.                                                   |
|    256 |    2 | `offense`                 | Offense modifier.                                               |
|    258 |    2 | `defense`                 | Defense modifier.                                               |
|    260 |    2 | `magical_power`           | Magical-power modifier.                                         |
|    262 |    2 | `modification_resistance` | Resistance to modification; a negative value lowers resistance. |
| 264 | 1 | `reserved_byte` | Always zero by default; purpose unknown. |
|    265 |    1 | `modifies_item`           | Whether this material can modify an item.                       |
|    266 |    2 | `additional_effect`       | Extra effect: none, fire, or mana drain.                        |

## Runtime item ID

The four bytes at byte offset 236 are not two independent padding fields. The on-disk database stores zero there, then
the game loader overwrites the word with the zero-based record index. Inventory and ground-item records preserve that
value as `edit_item_id` to link an instance back to its database definition.

When writing the original database, retain the stored value (normally zero). The game will assign the runtime ID when it
loads the file.

## Notes

- Record size: 268 bytes.
- Header: signed 32-bit record count.
- All numeric fields are little-endian.
- `reserved_byte` is an explicit byte in the binary layout, not compiler alignment.
