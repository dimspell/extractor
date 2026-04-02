# Dispel Game File System - Cross References

## Overview

This document maps the relationships and cross-references between all game data files in the Dispel game system. The files form a complex interconnected database that drives the game's mechanics, visuals, and interactions.

## File Dependency Map

### Core Configuration Files (INI)
- **AllMap.ini**: Master map index linking maps to resources
- **Map.ini** (Ref/): Map initialization with positions and resource links
- **Monster.ini** (CharacterInGame/): Monster visual definitions and animations
- **Npc.ini**: NPC visual definitions and sprite references
- **Extra.ini**: Interactive object definitions
- **Event.ini**: Event scripts with conditions and triggers
- **Wave.ini**: Sound effect mappings

### Database Files (DB)
- **Monster.db** (MonsterInGame/): Monster combat statistics and AI
- **WeaponItem.db** (CharacterInGame/): Weapons and armor with stats
- **HealItem.db** (CharacterInGame/): Consumable healing items
- **MiscItem.db** (CharacterInGame/): Generic utility items
- **EditItem.db** (CharacterInGame/): Modifiable equipment items
- **EventItem.db** (CharacterInGame/): Quest and event items
- **Store.db** (CharacterInGame/): Shops and inns with inventories
- **ChData.db** (CharacterInGame/): Character statistics tracking
- **PrtLevel.db** (NpcInGame/): Character progression (levels 1-20)
- **Magic.db** (Ref/): Spell database with effects
- **DrawItem.db** (Ref/): Object placements on maps

### Reference Files (REF)
- **PartyRef.ref** (Ref/): Party character definitions
- **Eventnpc.ref** (NpcInGame/): Event-triggered NPC definitions
- **NpcMapFiles.ref** (NpcInGame/): NPC placements with waypoints
- **Extra.ref** (ExtraInGame/): Interactive object placements
- **MondunMonmap.ref** (MonsterInGame/): Monster placements on maps

### Script Files (SCR/DLG/PGP)
- **Message.scr** (ExtraInGame/): UI text messages
- **Quest.scr** (ExtraInGame/): Quest journal entries
- **DlgMapFiles.dlg** (NpcInGame/): Dialogue conversation scripts
- **PgpMapFiles.pgp** (NpcInGame/): Dialogue text content

### Visual/Audio Assets
- **Sprites.spr** (Map/): Sprite files for characters, monsters, objects
- **SNF files** (referenced by Wave.ini): Sound effect files
- **Map.map** (Map/): Map geometry and tile data
- **Map.gtl** (Map/): Ground tileset files
- **Map.btl** (Map/): Building/roof tileset files

### Map Tileset Relationships
```
Map.map files
    ├── gtl_tile_id → Map.gtl (ground tiles)
    ├── btl_tile_id → Map.btl (building tiles)
    └── event_id → Event.ini (tile events)

Map.gtl (ground tiles)
    └── Referenced by Map.map for terrain

Map.btl (building tiles)
    └── Referenced by Map.map for structures
```

## Cross-Reference Relationships

### 1. Map System Relationships

```
AllMap.ini (master index)
    ├── map_file → Map.map files
    ├── pgp → PgpMapFiles.pgp (dialogue text)
    └── dlg → DlgMapFiles.dlg (dialogue scripts)

Map.ini (Ref/)
    ├── id → AllMap.ini.id
    ├── monsters → MondunMonmap.ref (monster placements)
    ├── npcs → NpcMapFiles.ref (NPC placements)
    ├── extras → Extra.ref (object placements)
    ├── camera_event → Event.ini.event_id
    └── cd_track → Audio track number

Map.map (map geometry)
    ├── gtl_tile_id → Map.gtl (ground tiles)
    ├── btl_tile_id → Map.btl (building tiles)
    ├── event_id → Event.ini (tile events)
    └── Contains embedded sprites

Map.gtl (ground tiles)
    └── Referenced by Map.map for terrain

Map.btl (building tiles)
    └── Referenced by Map.map for structures
```

### 2. Character System Relationships

```
PartyRef.ref (Ref/)
    ├── npc_id → Npc.ini.id (visual appearance)
    ├── map_id → AllMap.ini.id (recruitment location)
    ├── dlg_out → DlgMapFiles.dlg.id (dialogue when not in party)
    └── dlg_in → DlgMapFiles.dlg.id (dialogue when in party)

Npc.ini (visual definitions)
    └── sprite_filename → Sprites.spr files

PrtLevel.db (character progression)
    └── 8 NPC slots × 20 levels = character stat growth

ChData.db (character statistics)
    └── Tracks overall game/player statistics

Character System Cross-References:
    ├── PartyRef.ref → PrtLevel.db (character progression)
    ├── PrtLevel.db → WeaponItem.db (equipment requirements)
    ├── PrtLevel.db → Magic.db (spell access)
    └── ChData.db → Overall game statistics
```

### 3. Monster System Relationships

```
Monster.ini (visual definitions)
    ├── sprite_filename → Sprites.spr files
    ├── attack_seq → Sprite animation sequence
    ├── hit_seq → Sprite animation sequence
    ├── death_seq → Sprite animation sequence
    ├── walk_seq → Sprite animation sequence
    └── cast_seq → Sprite animation sequence

Monster.db (combat statistics)
    ├── known_spell_slot1 → Magic.db.id
    ├── known_spell_slot2 → Magic.db.id
    ├── known_spell_slot3 → Magic.db.id
    └── AI type determines combat behavior

MondunMonmap.ref (monster placements)
    ├── mon_id → Monster.db.id (monster type)
    ├── loot1/2/3_item_id → Item databases (loot drops)
    └── Places monsters from Monster.db/ini on specific maps
```

### 4. NPC System Relationships

```
NpcMapFiles.ref (NPC placements)
    ├── npc_id → Npc.ini.id (visual type)
    ├── dialog_id → DlgMapFiles.dlg.id (dialogue script)
    ├── show_on_event → Event.ini.event_id (visibility trigger)
    └── Waypoints for patrol routes

Eventnpc.ref (event NPCs)
    ├── event_id → Event.ini.event_id (trigger condition)
    └── Temporary NPCs appearing during specific events

DlgMapFiles.dlg (dialogue scripts)
    ├── prev_event → Event.ini.event_id (prerequisite)
    ├── next_dlg → DlgMapFiles.dlg.id (conversation chain)
    ├── dlg_id → PgpMapFiles.pgp.id (text content)
    ├── event_id → Event.ini.event_id (triggered by dialogue)
    └── Referenced by NpcMapFiles.ref and PartyRef.ref

PgpMapFiles.pgp (dialogue text)
    └── Contains actual dialogue text and parameters
```

### 5. Interactive Object System Relationships

```
Extra.ini (object definitions)
    ├── sprite_filename → Sprites.spr files
    └── flag (0=standard, 1=special/quest)

Extra.ref (object placements)
    ├── ext_id → Extra.ini.id (object type)
    ├── event_id → Event.ini.event_id (interaction trigger)
    ├── message_id → Message.scr.id (sign text)
    ├── required_item_id → Item database
    ├── required_item_type_id → Item type enum
    ├── item_id → Item database (contents)
    └── item_type_id → Item type enum

DrawItem.db (object placements)
    ├── map_id → AllMap.ini.id
    └── item_id → Extra.ini.id or object database
```

### 6. Event System Relationships

```
Event.ini (event definitions)
    ├── previous_event_id → Event.ini.id (prerequisite)
    ├── script_filename → Script files (.scr)
    └── event_type (0-8) determines execution logic

Events are referenced by:
    ├── Map.ini.camera_event
    ├── NpcMapFiles.ref.show_on_event
    ├── DlgMapFiles.dlg.event_id
    ├── DlgMapFiles.dlg.prev_event
    ├── Extra.ref.event_id
    └── Eventnpc.ref.event_id
```

### 7. Item/Equipment System Relationships

```
WeaponItem.db (weapons/armor)
    ├── Referenced by Store.db products (type=0,1)
    ├── Referenced by Extra.ref contents (item_type_id=0,1)
    └── Referenced by MondunMonmap.ref loot (loot*_item_type)

HealItem.db (consumable healing)
    ├── Referenced by Store.db products (type=2)
    ├── Referenced by Extra.ref contents (item_type_id=2)
    └── Referenced by MondunMonmap.ref loot (loot*_item_type)

MiscItem.db (generic utility items)
    ├── Referenced by Store.db products (type=3)
    ├── Referenced by Extra.ref contents (item_type_id=3)
    └── Referenced by MondunMonmap.ref loot (loot*_item_type)

EditItem.db (modifiable equipment)
    ├── Referenced by Store.db products (type=4)
    ├── Referenced by Extra.ref contents (item_type_id=4)
    └── Referenced by MondunMonmap.ref loot (loot*_item_type)

EventItem.db (quest items)
    ├── Referenced by Extra.ref contents (item_type_id=5)
    └── Quest progression tracking

Store.db (shops/inns)
    ├── products reference item databases
    └── Commerce system with haggling and dialogue

Item Type Enum (used in Extra.ref and Store.db):
    ├── 0: Weapon → WeaponItem.db
    ├── 1: Armor → WeaponItem.db
    ├── 2: Heal → HealItem.db
    ├── 3: Misc → MiscItem.db
    ├── 4: Edit → EditItem.db
    ├── 5: Event → EventItem.db
    └── 6: Extra → Extra item type
```

### 8. Audio System Relationships

```
Wave.ini (sound mappings)
    └── snf_filename → .snf audio files

Sound IDs referenced by:
    ├── dialogue_text.rs (wave_ini_entry_id)
    └── Game events and interactions
```

### 9. Quest System Relationships

```
Quest.scr (quest definitions)
    ├── type (0=main, 1=side, 2=traders journal)
    └── Referenced by event system

Message.scr (UI messages)
    └── Referenced by Extra.ref.message_id for signs
```

## Complete Cross-Reference Table

| Source File | Source Field | Target File | Target Field | Relationship |
|-------------|--------------|-------------|--------------|--------------|
| AllMap.ini | map_file | Map.map | filename | Map geometry |
| AllMap.ini | pgp | PgpMapFiles.pgp | filename | Dialogue text |
| AllMap.ini | dlg | DlgMapFiles.dlg | filename | Dialogue scripts |
| Map.ini | monsters | MondunMonmap.ref | filename | Monster placements |
| Map.ini | npcs | NpcMapFiles.ref | filename | NPC placements |
| Map.ini | extras | Extra.ref | filename | Object placements |
| Map.ini | camera_event | Event.ini | event_id | Camera trigger |
| Monster.ini | sprite_filename | Sprites.spr | filename | Visual assets |
| Npc.ini | sprite_filename | Sprites.spr | filename | Visual assets |
| Extra.ini | sprite_filename | Sprites.spr | filename | Visual assets |
| PartyRef.ref | npc_id | Npc.ini | id | Visual appearance |
| PartyRef.ref | map_id | AllMap.ini | id | Location |
| PartyRef.ref | dlg_out | DlgMapFiles.dlg | id | Dialogue |
| PartyRef.ref | dlg_in | DlgMapFiles.dlg | id | Dialogue |
| NpcMapFiles.ref | npc_id | Npc.ini | id | NPC type |
| NpcMapFiles.ref | dialog_id | DlgMapFiles.dlg | id | Dialogue |
| NpcMapFiles.ref | show_on_event | Event.ini | event_id | Visibility |
| Eventnpc.ref | event_id | Event.ini | event_id | Trigger |
| DlgMapFiles.dlg | prev_event | Event.ini | event_id | Prerequisite |
| DlgMapFiles.dlg | next_dlg | DlgMapFiles.dlg | id | Chain |
| DlgMapFiles.dlg | dlg_id | PgpMapFiles.pgp | id | Text |
| DlgMapFiles.dlg | event_id | Event.ini | event_id | Trigger |
| Extra.ref | ext_id | Extra.ini | id | Object type |
| Extra.ref | event_id | Event.ini | event_id | Interaction |
| Extra.ref | message_id | Message.scr | id | Sign text |
| Extra.ref | required_item_type_id | Item type enum | - | Key type |
| Extra.ref | item_type_id | Item type enum | - | Content type |
| Monster.db | known_spell_slot1-3 | Magic.db | id | Spells |
| DrawItem.db | map_id | AllMap.ini | id | Map location |
| DrawItem.db | item_id | Extra.ini | id | Object type |
| Map.map | gtl_tile_id | Map.gtl | id | Ground tiles |
| Map.map | btl_tile_id | Map.btl | id | Building tiles |
| Map.map | event_id | Event.ini | event_id | Tile events |
| Store.db | products | Item databases | id | Inventory |
| Store.db | products (type=0,1) | WeaponItem.db | id | Weapons/Armor |
| Store.db | products (type=2) | HealItem.db | id | Healing items |
| Store.db | products (type=3) | MiscItem.db | id | Misc items |
| Store.db | products (type=4) | EditItem.db | id | Modifiable items |
| Extra.ref | item_type_id=0,1 | WeaponItem.db | id | Weapons/Armor |
| Extra.ref | item_type_id=2 | HealItem.db | id | Healing items |
| Extra.ref | item_type_id=3 | MiscItem.db | id | Misc items |
| Extra.ref | item_type_id=4 | EditItem.db | id | Modifiable items |
| Extra.ref | item_type_id=5 | EventItem.db | id | Quest items |
| MondunMonmap.ref | loot*_item_type | Item databases | id | Monster loot |
| Wave.ini | snf_filename | .snf files | filename | Audio |

## Data Flow Summary

1. **Map Loading**: AllMap.ini → Map.ini → Load map geometry (.map), place monsters (.ref), NPCs (.ref), objects (.ref)
2. **Map Rendering**: Map.map → Map.gtl (ground tiles) + Map.btl (building tiles) + Sprites.spr (objects/NPCs)
3. **NPC Interaction**: NpcMapFiles.ref → Npc.ini → Sprites.spr + DlgMapFiles.dlg → PgpMapFiles.pgp
4. **Monster Combat**: MondunMonmap.ref → Monster.ini → Sprites.spr + Monster.db → Magic.db
5. **Object Interaction**: Extra.ref → Extra.ini → Sprites.spr + Event.ini + Message.scr
6. **Quest Progression**: Event.ini → Script files → Quest.scr
7. **Character Development**: PartyRef.ref → PrtLevel.db → WeaponItem.db + Magic.db
8. **Commerce**: Store.db → Item databases (WeaponItem.db, HealItem.db, etc.)
9. **Audio**: Wave.ini → .snf files (referenced during events/dialogue)
10. **Monster Loot**: MondunMonmap.ref → Item databases (loot drops)

## Item Database Relationships

### Item Type Enum (used in Extra.ref and Store.db)
```
0: Weapon → WeaponItem.db
1: Armor → WeaponItem.db
2: Heal → HealItem.db
3: Misc → MiscItem.db
4: Edit → EditItem.db
5: Event → EventItem.db
6: Extra → Extra item type
```

### Item Database Cross-References
```
WeaponItem.db (weapons/armor)
    ├── Referenced by Store.db products (type=0,1)
    ├── Referenced by Extra.ref contents (item_type_id=0,1)
    └── Referenced by MondunMonmap.ref loot (loot*_item_type)

HealItem.db (consumable healing)
    ├── Referenced by Store.db products (type=2)
    ├── Referenced by Extra.ref contents (item_type_id=2)
    └── Referenced by MondunMonmap.ref loot (loot*_item_type)

MiscItem.db (generic utility items)
    ├── Referenced by Store.db products (type=3)
    ├── Referenced by Extra.ref contents (item_type_id=3)
    └── Referenced by MondunMonmap.ref loot (loot*_item_type)

EditItem.db (modifiable equipment)
    ├── Referenced by Store.db products (type=4)
    ├── Referenced by Extra.ref contents (item_type_id=4)
    └── Referenced by MondunMonmap.ref loot (loot*_item_type)

EventItem.db (quest items)
    ├── Referenced by Extra.ref contents (item_type_id=5)
    └── Quest progression tracking
```

## File Encoding Summary

| Encoding | Files |
|----------|-------|
| EUC-KR | Event.ini, Extra.ini, Npc.ini, Wave.ini, Map.ini, Monster.db, DlgMapFiles.dlg, HealItem.db (descriptions) |
| WINDOWS-1250 | AllMap.ini, Monster.ini, Store.db, WeaponItem.db, HealItem.db (names), MiscItem.db, EditItem.db, EventItem.db, Message.scr, Quest.scr, PgpMapFiles.pgp, Extra.ref, NpcMapFiles.ref, PartyRef.ref, Eventnpc.ref |
| Binary | Monster.db, WeaponItem.db, HealItem.db, MiscItem.db, EditItem.db, EventItem.db, Store.db, ChData.db, PrtLevel.db, Magic.db, DrawItem.db, Map.map, Map.gtl, Map.btl, Sprites.spr |

## Legal Compliance

This cross-reference documentation:
- Describes **technical relationships between file formats only**
- Does **not distribute any copyrighted game content**
- Focuses on **data structure organization and system design**
- Uses **generic descriptions** of game mechanics
- Maintains **nominal fair use** for trademark references
- Is intended for **educational and research purposes** only

## Extractor Commands

The extractor tool (`src/`) provides commands to parse and extract data from all game files.

### Quick Reference

```bash
# Build the extractor
cargo build --release

# Extract INI files to JSON
cargo run -- ref all-maps "path/to/AllMap.ini"
cargo run -- ref event "path/to/Event.ini"
cargo run -- ref extra "path/to/Extra.ini"
cargo run -- ref monster "path/to/Monster.ini"
cargo run -- ref npc "path/to/Npc.ini"
cargo run -- ref wave "path/to/Wave.ini"
cargo run -- ref map "path/to/Map.ini"

# Extract database files to JSON
cargo run -- ref weapons "path/to/weaponItem.db"
cargo run -- ref monsters "path/to/Monster.db"
cargo run -- ref heal-items "path/to/HealItem.db"
cargo run -- ref misc-item "path/to/MiscItem.db"
cargo run -- ref edit-items "path/to/EditItem.db"
cargo run -- ref event-items "path/to/EventItem.db"
cargo run -- ref store "path/to/STORE.DB"
cargo run -- ref magic "path/to/Magic.db"
cargo run -- ref party-level "path/to/PrtLevel.db"
cargo run -- ref chdata "path/to/ChData.db"

# Extract reference files to JSON
cargo run -- ref party-ref "path/to/PartyRef.ref"
cargo run -- ref draw-item "path/to/DrawItem.ref"
cargo run -- ref npc-ref "path/to/Npccat1.ref"
cargo run -- ref monster-ref "path/to/Mondun01.ref"
cargo run -- ref extra-ref "path/to/Extdun01.ref"
cargo run -- ref event-npc-ref "path/to/Eventnpc.ref"

# Extract script files to JSON
cargo run -- ref dialog "path/to/Dlgcat1.dlg"
cargo run -- ref message "path/to/Message.scr"
cargo run -- ref quest "path/to/Quest.scr"

# Extract sprites
cargo run -- sprite "path/to/file.spr" "output_name"
cargo run -- sprite "path/to/file.spr" --mode animation

# Extract map data
cargo run -- map tiles "path/to/file.gtl" --output "output_dir"
cargo run -- map atlas "path/to/file.gtl" "output.png"
cargo run -- map render --map "file.map" --btl "file.btl" --gtl "file.gtl" --output "map.png"
cargo run -- map sprites "path/to/file.map"

# Convert audio
cargo run -- sound "path/to/file.snf" "output.wav"

# Import to SQLite database
cargo run -- database import "path/to/Dispel/" "database.sqlite"
```

---

**DISPEL®** is a registered trademark. This documentation is **not affiliated with, endorsed by, or sponsored by** the trademark owner.
