# EditItem.db

`CharacterInGame/EditItem.db` defines consumable item-modification materials.
It is a little-endian binary file with a four-byte record count followed by fixed-size
268-byte records. Text is encoded as WINDOWS-1250.

## Record layout

| Offset | Size | Field | Meaning |
|---:|---:|---|---|
| `0x00` | 30 | `name` | Null-padded item name. |
| `0x1E` | 202 | `description` | Null-padded description. |
| `0xE8` | 4 | `base_price` | Shop price. |
| `0xEC` | 4 | `runtime_item_id` | Runtime ID assigned by the game loader. |
| `0xF0` | 2 | `health_points` | HP modifier. |
| `0xF2` | 2 | `mana_points` | MP modifier. |
| `0xF4` | 2 | `strength` | Strength modifier. |
| `0xF6` | 2 | `agility` | Agility modifier. |
| `0xF8` | 2 | `wisdom` | Wisdom modifier. |
| `0xFA` | 2 | `constitution` | Constitution modifier. |
| `0xFC` | 2 | `to_dodge` | Dodge modifier. |
| `0xFE` | 2 | `to_hit` | Hit modifier. |
| `0x100` | 2 | `offense` | Offense modifier. |
| `0x102` | 2 | `defense` | Defense modifier. |
| `0x104` | 2 | `magical_power` | Magical-power modifier. |
| `0x106` | 2 | `modification_resistance` | Resistance to modification; a negative value lowers resistance. |
| `0x108` | 1 | `reserved_byte` | Always zero in shipped data; no runtime use identified. |
| `0x109` | 1 | `modifies_item` | Whether this material can modify an item. |
| `0x10A` | 2 | `additional_effect` | Extra effect: none, fire, or mana drain. |

## Runtime item ID

The four bytes at `0xEC` are not two independent padding fields. The on-disk database
stores zero there, then the game loader overwrites the word with the zero-based record
index. Inventory and ground-item records preserve that value as `edit_item_id` to link
an instance back to its database definition.

When writing the original database, retain the stored value (normally zero). The game
will assign the runtime ID when it loads the file.

## Notes

- Record size: 268 bytes (`0x10C`).
- Header: signed 32-bit record count.
- All numeric fields are little-endian.
- `reserved_byte` is an explicit byte in the binary layout, not compiler alignment.

**DISPEL®** is a registered trademark. This project is not affiliated with, endorsed
by, or sponsored by the trademark owner.
