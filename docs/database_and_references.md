# Database and entities

The tool populates a SQLite database (`database.sqlite`) through `database.rs`.

## Schema overview

Tables come from SQL files in `src/queries/`. The tool creates the tables when it imports data.

### Core entities

#### Maps (`save_maps`)

- Table: `create_table_maps.sql`
- Fields: ID, Filename, Name, PGP Filename, DLG Filename, Is Light.

#### Map tiles (`save_map_tiles`)

- The tool stores one row per coordinate (X, Y).
- Fields: MapID, X, Y, GTL_ID (ground), BTL_ID (building), Collision (Bool), EventID.

#### Events and dialogs

- Events: ID, Previous ID, Type, Filename, Counter.
- Dialogs: ID, Prev Event ID, Next Dialog, Type, Owner, DLG ID, Event ID.

#### Items

The tool uses separate tables for item categories:

- Weapons: Stats (Str, Agi, Wis, Atk, Def), Price, Requirements.
- HealItems: HP, MP and status recovery effects.
- MiscItems: Basic description and price.
- EventItems: Name and description.
- DrawItems: Map items with MapID, X, Y and ItemID.
- Store: Shop definitions and product lists.

#### Characters and monsters

- Monsters: HP, MP, Atk, Def, Hit and Dodge minimum and maximum values, plus XP, Gold, Spells and AI type.
- Monster refs: Placed monster instances on maps with Pos X, Y and Loot.
- Monster inis: Visual and sprite references.
- NPCs:
  - NPC refs: Placed NPCs with Pos, Scripts and Dialog IDs.
  - NPC inis: Visual and sprite references.
- Party:
  - Party refs: Joinable components and Job names.

#### Other

- Extras: Sprite file and description.
- Waves: SNF filenames.

## Data import

The CLI imports game data with `cargo run -- database import <dir> <db.sqlite>`. The command reads files from the given directory with the parsers in `src/references/`. It calls the matching `save_*` function in `database.rs`.

Each `save_*` function does three things:

1. It runs a `create_table_*.sql` file.
2. It iterates over the parsed records.
3. It runs an `insert_*.sql` file for each record.

The inserts for `chest_spawns`, `monster_spawns`, `map_info_multi` and `tile_grids` live inline in `src/references/chest_spawn.rs`, `src/references/monster_spawn.rs`, `src/references/map_info_multi.rs` and `src/references/tile_grid.rs`. They call `conn.execute` directly instead of a prepared `include_str!` statement. No `insert_*.sql` files exist for those four tables. Do not re-add them.
