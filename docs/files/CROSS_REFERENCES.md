# Dispel game files: cross references

## Overview

This document lists the links between game data files. The tool reads and writes the files that you supply. Each link connects a field in one file with a field in another file. Some links point to a file name instead of a field.

## File dependency map

This section groups files by role. A family name such as `DlgMapFiles.dlg` stands for a set of real files. A real file name such as `Dlgcat1.dlg` names one file on disk.

### Core configuration files (INI)

- `AllMap.ini`: Master map index. It lists each map and its resources.
- `Ref/Map.ini`: Map setup with spawn coordinates and file links. Each row holds one entrance.
- `Monster.ini`: Monster visuals and animation sequences.
- `Npc.ini`: NPC visuals and sprite file names.
- `Extra.ini`: Interactive object definitions.
- `Event.ini`: Event conditions and script file names.
- `Wave.ini`: Sound effect names that point to `.snf` files.

### Database files (DB)

- `CharacterInGame/WeaponItem.db`: Weapons and armor with combat stats.
- `CharacterInGame/HealItem.db`: Consumable healing items.
- `CharacterInGame/EditItem.db`: Editable items that the player can change.
- `CharacterInGame/EventItem.db`: Quest and event items.
- `CharacterInGame/MiscItem.db`: Utility items.
- `CharacterInGame/STORE.DB`: Shops and inns with product lists.
- `CharacterInGame/ChData.db`: One record of initial class statistics.
- `MagicInGame/Magic.db`: Spell definitions and effects.
- `NpcInGame/PrtLevel.db`: Party growth records for eight slots and twenty levels.
- `MonsterInGame/Monster.db`: Monster combat statistics.

### Reference files (REF)

- `Ref/PartyRef.ref`: Party member definitions and dialogue links.
- `MonsterInGame/Mondun01.ref`: Monster placements. `MondunMonmap.ref` is the family name.
- `NpcInGame/Npccat1.ref`: NPC placements. `NpcMapFiles.ref` is the family name.
- `ExtraInGame/Extdun01.ref`: Interactive object placements. `Extra.ref` is the family name.
- `NpcInGame/Eventnpc.ref`: NPCs that appear during events.
- `Ref/DRAWITEM.ref`: Map item drops. This is a parenthesized CSV file with EUC-KR text.
- `ExtraInGame/fogdata.dat`: Fade tables for map lighting. See [ExtraInGame/fogdata.dat.md](ExtraInGame/fogdata.dat.md).

### Script files (SCR/DLG/PGP)

- `ExtraInGame/Message.scr`: UI text messages.
- `ExtraInGame/Quest.scr`: Quest journal entries.
- `NpcInGame/Dlgcat1.dlg`: Dialogue scripts. `DlgMapFiles.dlg` is the family name.
- `NpcInGame/Pgpcat1.pgp`: Dialogue text and parameters. `PgpMapFiles.pgp` is the family name.

### Visual, audio and map assets

- Map.map: Map geometry, tile layers and embedded sprites.
- Tileset files: Ground and roof tiles. Each map stem names its own pair of `.gtl` and `.btl` files.
- `.spr` files: Sprite art. File names come from `Monster.ini`, `Npc.ini` and `Extra.ini`.
- `.snf` files: Audio data. `Wave.ini` names these files.

## Relationship diagrams

### 1. Map system

```
AllMap.ini
    ├── id → row key
    ├── map_filename → Map.map stem
    ├── pgp_filename → Pgpcat1.pgp (dialogue text)
    ├── dlg_filename → Dlgcat1.dlg (dialogue scripts)
    └── lighting → MapLighting code

Ref/Map.ini
    ├── id → entrance row key
    ├── map_id → AllMap.ini.id
    ├── event_id_on_camera_move → event index
    ├── start_pos_x / start_pos_y → spawn point
    ├── monsters_filename → Mondun01.ref
    ├── npc_filename → Npccat1.ref
    ├── extra_filename → Extdun01.ref
    └── cd_music_track_number → audio track
```

### 2. Map.map layers

```
Map.map
    ├── ground GTL index = (word >> 10) & 0x7FFF (Tile and Access words)
    ├── BTL IDs → u16 array in the Tiled Objects block
    ├── roof BTL index → bits 0..14 of the Access-Ref grid
    ├── event IDs → bits 0..13 of a u32 (resolved via Map.ini and AllMap.ini)
    └── sprites → embedded sprite block
```

### 3. Character data

```
Ref/PartyRef.ref
    ├── npc_id → Npc.ini.id (visual type)
    ├── root_map_id → AllMap.ini.id
    ├── dlg_when_not_in_party → PartyDlg.dlg paragraph
    └── dlg_when_in_party → PartyDlg.dlg paragraph

NpcInGame/PrtLevel.db
    ├── 8 slots × 20 levels, with 36-byte records
    ├── magic_spell_id_1 / 2 / 3 → spell indices
    └── weapon_skill_level → skill level

CharacterInGame/ChData.db
    └── one 84-byte record of initial class statistics
```

### 4. Monster data

```
Monster.ini
    └── sprite_filename → .spr file under MonsterInGame/

MonsterInGame/Monster.db
    ├── known_spell_slot1 / 2 / 3 → spell indices (-1 = SQL NULL)
    └── combat statistics

MonsterInGame/Mondun01.ref
    ├── monster_db_id → Monster.db.id
    ├── loot_item_1 / 2 / 3 → items lookup (type:ID)
    └── event_id_on_kill → event index
```

### 5. NPC and dialogue data

```
NpcInGame/Npccat1.ref
    ├── npc_ini_id → Npc.ini.id
    ├── dialog_id → Dlgcat1.dlg script
    └── show_on_event → event index

NpcInGame/Eventnpc.ref
    ├── npc_ini_id → Npc.ini.id
    ├── show_on_event → event index
    └── dialog_id → Dlgcat1.dlg script

NpcInGame/Dlgcat1.dlg (on-disk labels in parentheses)
    ├── required_event_id (prev_event) → event index
    ├── next_dialog_to_check (next_dlg) → Dlg script
    ├── dialog_id (dlg_id) → Pgpcat1.pgp paragraph
    └── triggered_event_id (event_id) → event index

NpcInGame/Pgpcat1.pgp
    └── dialogue text and parameters
```

### 6. Interactive object data

```
ExtraInGame/Extra.ini
    └── sprite_filename → .spr file under ExtraInGame/

ExtraInGame/Extdun01.ref
    ├── extra_definition_id → Extra.ini.id
    ├── map_object_id → map object number
    ├── interaction_event_id → event index
    ├── interaction_message_id → Message.scr.id
    ├── required_item / required_item2 → items lookup (type:ID)
    ├── loot_item → items lookup (type:ID)
    ├── additional_loot_1 / additional_loot_2 → item IDs
    ├── trap_spell_id → spell index
    └── open_sfx_id → Wave.ini index

Ref/DRAWITEM.ref
    ├── map_id → AllMap.ini.id
    ├── x_coord / y_coord → map position
    └── item → items lookup (type:ID)
```

### 7. Events and audio

```
Event.ini
    ├── required_event_id → event prerequisite
    ├── event_filename → event script file
    └── event_type → 0, 2, 5 or 6

Wave.ini
    └── snf_filename → .snf audio file

NpcInGame/Pgpcat1.pgp
    └── wave_ini_entry_id (column 3) → Wave.ini index
```

### 8. Item keys and store products

```
ItemTypeId (composite key type:ID)
    ├── 1 Weapon → WeaponItem.db
    ├── 2 Healing → HealItem.db
    ├── 3 Edit → EditItem.db
    ├── 4 Event → EventItem.db
    ├── 5 Misc → MiscItem.db
    └── -1 NoItem → 0xFF on disk, 255 in SQL and GUI keys

ProductType (Store.db slots)
    ├── 1 Weapon → WeaponItem.db
    ├── 2 Healing → HealItem.db
    ├── 3 EditItem → EditItem.db
    └── 4 MiscItem → MiscItem.db

Note: 0 ends the list. A store holds at most 15 products.
```

## Complete cross-reference table

| Source file | Source field | Target file | Target field | Relationship |
|---|---|---|---|---|
| AllMap.ini | map_filename | Map.map | stem | Map geometry |
| AllMap.ini | pgp_filename | Pgpcat1.pgp | file name | Dialogue text |
| AllMap.ini | dlg_filename | Dlgcat1.dlg | file name | Dialogue scripts |
| Ref/Map.ini | map_id | AllMap.ini | id | Map link |
| Ref/Map.ini | monsters_filename | Mondun01.ref | file name | Monster placements |
| Ref/Map.ini | npc_filename | Npccat1.ref | file name | NPC placements |
| Ref/Map.ini | extra_filename | Extdun01.ref | file name | Object placements |
| Ref/Map.ini | cd_music_track_number | CD audio track | number | Music track |
| Mondun01.ref | monster_db_id | Monster.db | id | Monster type |
| Mondun01.ref | loot_item_1 / 2 / 3 | items lookup | type:ID | Loot |
| Mondun01.ref | event_id_on_kill | event index | - | Kill trigger |
| Monster.db | known_spell_slot1 / 2 / 3 | spell index | - | Spells (NULL for -1) |
| Npccat1.ref | npc_ini_id | Npc.ini | id | NPC type |
| Npccat1.ref | dialog_id | Dlgcat1.dlg | id | Dialogue |
| Npccat1.ref | show_on_event | event index | - | Visibility |
| Eventnpc.ref | npc_ini_id | Npc.ini | id | NPC type |
| Eventnpc.ref | show_on_event | event index | - | Trigger |
| Eventnpc.ref | dialog_id | Dlgcat1.dlg | id | Dialogue |
| Dlgcat1.dlg | required_event_id | event index | - | Prerequisite |
| Dlgcat1.dlg | next_dialog_to_check | Dlgcat1.dlg | id | Chain |
| Dlgcat1.dlg | dialog_id | Pgpcat1.pgp | paragraph | Text |
| Dlgcat1.dlg | triggered_event_id | event index | - | Trigger |
| Pgpcat1.pgp | wave_ini_entry_id | Wave.ini | index | Voice audio |
| PartyRef.ref | npc_id | Npc.ini | id | Visual type |
| PartyRef.ref | root_map_id | AllMap.ini | id | Location |
| PartyRef.ref | dlg_when_not_in_party | PartyDlg.dlg | paragraph | Dialogue |
| PartyRef.ref | dlg_when_in_party | PartyDlg.dlg | paragraph | Dialogue |
| Extdun01.ref | extra_definition_id | Extra.ini | id | Object type |
| Extdun01.ref | interaction_event_id | event index | - | Interaction |
| Extdun01.ref | interaction_message_id | Message.scr | id | Sign text |
| Extdun01.ref | required_item / required_item2 | items lookup | type:ID | Required item |
| Extdun01.ref | loot_item | items lookup | type:ID | Loot |
| Extdun01.ref | trap_spell_id | spell index | - | Trap spell |
| Extdun01.ref | open_sfx_id | Wave.ini | index | Open sound |
| DRAWITEM.ref | map_id | AllMap.ini | id | Map location |
| DRAWITEM.ref | item | items lookup | type:ID | Dropped item |
| Map.map | ground GTL index | per-map .gtl file | tile index | Ground tiles |
| Map.map | BTL IDs | per-map .btl file | tile index | Roof tiles |
| Map.map | event ID bits | event index | - | Tile event |
| STORE.DB | product | item databases | product type:ID | Product |
| PrtLevel.db | magic_spell_id_1 / 2 / 3 | spell index | - | Spells |
| Monster.ini / Npc.ini / Extra.ini | sprite_filename | .spr | file name | Sprites |
| Event.ini | event_filename | event script | file name | Event logic |
| Wave.ini | snf_filename | .snf | file name | Audio |

## Data flow summary

1. Map data: `AllMap.ini` → `Ref/Map.ini` → `Map.map`, `Mondun01.ref`, `Npccat1.ref` and `Extdun01.ref`.
2. Map rendering: `Map.map` → ground `.gtl` tiles, roof `.btl` tiles and the embedded sprite block.
3. NPC data: `Npccat1.ref` → `Npc.ini` → `.spr` files, and `Dlgcat1.dlg` → `Pgpcat1.pgp`.
4. Monster data: `Mondun01.ref` → `Monster.db` and `Monster.ini`, plus spell indices in `Monster.db`.
5. Object data: `Extdun01.ref` → `Extra.ini`, `Event.ini` and `Message.scr`.
6. Item drops: `DRAWITEM.ref` → item databases through the items lookup.
7. Party data: `PartyRef.ref` → `Npc.ini` and `PartyDlg.dlg`, with growth in `PrtLevel.db`.
8. Commerce data: `STORE.DB` → item databases through product types.
9. Audio data: `Wave.ini` → `.snf` files, with indices from `Pgpcat1.pgp`.
10. Event data: `Event.ini` → event scripts, with indices in placements and dialogue.

## Item type section

### Item type enum

The tool uses `ItemTypeId` for composite item keys. A key holds a type code and an item ID.

- 1 Weapon → `WeaponItem.db`
- 2 Healing → `HealItem.db`
- 3 Edit → `EditItem.db`
- 4 Event → `EventItem.db`
- 5 Misc → `MiscItem.db`
- -1 NoItem → empty slot (`0xFF` on disk, `255` in SQL and GUI keys)

There is no item type 0.

### Product types in stores

`STORE.DB` uses a different enum named `ProductType`. Each product slot holds a type code and an item ID.

- 1 Weapon → `WeaponItem.db`
- 2 Healing → `HealItem.db`
- 3 EditItem → `EditItem.db`
- 4 MiscItem → `MiscItem.db`
- 0 ends the product list, with at most 15 products per store

Unknown type codes fall back to MiscItem. EventItem has no product type. NoItem and 255 do not appear in `STORE.DB`.

## File encoding summary

Some files mix text and binary data. `Monster.db` is binary, and its 24-byte name field is EUC-KR. `MiscItem.db` has a WINDOWS-1250 name and an EUC-KR description.

| Encoding | Files |
|---|---|
| EUC-KR | `Event.ini`, `Extra.ini`, `Npc.ini`, `Wave.ini`, `Map.ini`, DLG files, `HealItem.db` descriptions, `MiscItem.db` description, `Monster.db` name field, `DRAWITEM.ref` |
| WINDOWS-1250 | `AllMap.ini`, `Monster.ini`, `WeaponItem.db`, `EditItem.db`, `EventItem.db`, `Message.scr`, `Quest.scr`, PGP files, `MiscItem.db` name, `STORE.DB` text fields, `Extra.ref` names, NPC refs, `PartyRef.ref`, `Eventnpc.ref` |
| Binary | `WeaponItem.db`, `HealItem.db`, `MiscItem.db`, `EditItem.db`, `EventItem.db`, `STORE.DB`, `Monster.db`, `ChData.db`, `PrtLevel.db`, `Magic.db`, `Map.map`, `.gtl`, `.btl`, `.spr`, `.snf` |

## Scope

The tool reads and writes the game files that you supply. This document lists the field links between those files. The extractor commands below use sample paths.

## Extractor commands

The tool in `src/` parses and extracts data from game files. Run these commands from the repository root.

### Quick reference

```bash
# Build the extractor
cargo build --release

# Extract INI files to JSON
cargo run -- extract -i "path/to/AllMap.ini"
cargo run -- extract -i "path/to/Ref/Map.ini"
cargo run -- extract -i "path/to/Event.ini"
cargo run -- extract -i "path/to/Extra.ini"
cargo run -- extract -i "path/to/Monster.ini"
cargo run -- extract -i "path/to/Npc.ini"
cargo run -- extract -i "path/to/Wave.ini"

# Extract database files to JSON
cargo run -- extract -i "path/to/CharacterInGame/weaponItem.db"
cargo run -- extract -i "path/to/MonsterInGame/Monster.db"
cargo run -- extract -i "path/to/CharacterInGame/HealItem.db"
cargo run -- extract -i "path/to/CharacterInGame/MiscItem.db"
cargo run -- extract -i "path/to/CharacterInGame/EditItem.db"
cargo run -- extract -i "path/to/CharacterInGame/EventItem.db"
cargo run -- extract -i "path/to/CharacterInGame/STORE.DB"
cargo run -- extract -i "path/to/MagicInGame/Magic.db"
cargo run -- extract -i "path/to/NpcInGame/PrtLevel.db"
cargo run -- extract -i "path/to/CharacterInGame/ChData.db"

# Extract reference files to JSON
cargo run -- extract -i "path/to/Ref/PartyRef.ref"
cargo run -- extract -i "path/to/Ref/DRAWITEM.ref"
cargo run -- extract -i "path/to/NpcInGame/Npccat1.ref"
cargo run -- extract -i "path/to/MonsterInGame/Mondun01.ref"
cargo run -- extract -i "path/to/ExtraInGame/Extdun01.ref"
cargo run -- extract -i "path/to/NpcInGame/Eventnpc.ref"

# Extract script files to JSON
cargo run -- extract -i "path/to/NpcInGame/Dlgcat1.dlg"
cargo run -- extract -i "path/to/NpcInGame/Pgpcat1.pgp"
cargo run -- extract -i "path/to/ExtraInGame/Message.scr"
cargo run -- extract -i "path/to/ExtraInGame/Quest.scr"

# Extract sprites
cargo run -- sprite "path/to/file.spr" "output_name"
cargo run -- sprite "path/to/file.spr" --mode animation

# Extract map data
cargo run -- map tiles "path/to/file.gtl" --output "output_dir"
cargo run -- map atlas "path/to/file.gtl" "output.png"
cargo run -- map render --map "file.map" --btl "file.btl" --gtl "file.gtl" --output "map.png"
cargo run -- map sprites "path/to/file.map"

# Convert audio
cargo run -- sound to-wav "path/to/file.snf" "output.wav"

# Import to SQLite database
cargo run -- database import "path/to/Dispel/" "database.sqlite"
```
