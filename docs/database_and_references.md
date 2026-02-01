# Database & Entities

The extractor populates a SQLite database (`database.sqlite`) using `database.rs`.

## Schema Overview
Tables are created on the fly using SQL files found in `src/queries/`.

### Core Entities

#### Maps (`save_maps`)
- **Table**: `create_table_maps.sql`
- **Fields**: ID, Filename, Name, PGP Filename, DLG Filename, Is Light.

#### Map Tiles (`save_map_tiles`)
- Stored per coordinate (X, Y).
- **Fields**: MapID, X, Y, GTL_ID (Ground), BTL_ID (Building), Collision (Bool), EventID.

#### Events & Dialogs
- **Events**: ID, Previous ID, Type, Filename, Counter.
- **Dialogs**: ID, Prev Event ID, Next Dialog, Type, Owner, DLG ID, Event ID.
- **Party PGPs**: ID, Dialog Text.

#### Items
Separate tables for different item categories:
- **Weapons**: Stats (Str, Agi, Wis, Atk, Def), Price, Requirements.
- **HealItems**: HP/MP/Status recovery effects.
- **MiscItems**: Basic description/price.
- **EventItems**: Name/Description.
- **DrawItems**: Map items (MapID, X, Y, ItemID).
- **Store**: Shop definitions and product lists.

#### Characters & Monsters
- **Monsters**: Huge stat block (HP/MP min/max, Atk/Def/Hit/Dodge min/max, XP/Gold, Spells, AI type).
  - **Monster Refs**: Placed monster instances on maps (Pos X, Y, Loot).
  - **Monster Inis**: Visual/Sprite references.
- **NPCs**:
  - **NPC Refs**: Placed NPCs (Pos, Scripts, Dialog IDs).
  - **NPC Inis**: Visual/Sprite references.
- **Party**:
  - **Party Refs**: Joinable components, Job names.

#### Other
- **Extras**: Interactive objects? (Sprite file, Description).
- **Waves**: SNF filenames (Sound/Music triggers?).

## Data Injection Logic
The `save_all` function in `main.rs` reads from hardcoded paths (e.g., `fixtures/Dispel/Ref/Map.ini`) using parsers in the `references` module, then calls the corresponding `save_*` function in `database.rs`.

Each `save_*` function:
1.  Executes a `create_table_*.sql`.
2.  Iterates over the parsed data vector.
3.  Executes an `insert_*.sql` for each record.
