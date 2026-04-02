# Npccat1.ref - NPC Placement Data

## File Information
- **Location**: `NpcInGame/Npccat1.ref`
- **Format**: Binary (Little-Endian)
- **Text Encoding**: WINDOWS-1250
- **Record Size**: 672 bytes (0x2A0)
- **Header**: 4-byte record count

## Structure

### Header
- `record_count`: i32 (4 bytes)

### Record Structure (672 bytes)
- `id`: i32 (4 bytes)
- `npc_id`: i32 (4 bytes) - NPC type ID
- `name`: 260 bytes (WINDOWS-1250, null-padded)
- `ignored_string`: 260 bytes (always zeros)
- `party_script_id`: i32 (4 bytes)
- `show_on_event`: i32 (4 bytes)
- `padding`: 4 bytes
- `goto1_filled`: i32 (4 bytes)
- `goto2_filled`: i32 (4 bytes)
- `goto3_filled`: i32 (4 bytes)
- `goto4_filled`: i32 (4 bytes)
- `goto1_x`: i32 (4 bytes)
- `goto2_x`: i32 (4 bytes)
- `goto3_x`: i32 (4 bytes)
- `goto4_x`: i32 (4 bytes)
- `padding`: 16 bytes
- `goto1_y`: i32 (4 bytes)
- `goto2_y`: i32 (4 bytes)
- `goto3_y`: i32 (4 bytes)
- `goto4_y`: i32 (4 bytes)
- `padding`: 16 bytes
- `looking_direction`: i32 (4 bytes) - 0=Up, 1=Right, 2=Down, 3=Left (clockwise)
- `padding`: 56 bytes
- `dialog_id`: i32 (4 bytes) - Pointer to `Dlgcat` or dialogue node
- `padding`: 4 bytes

## Looking Directions
- `0`: Up (North)
- `1`: Right (East)
- `2`: Down (South)
- `3`: Left (West)

## Waypoint System
- 4 waypoints per NPC
- `gotoN_filled`: 0=inactive, 1=active
- `gotoN_x`/`gotoN_y`: Tile coordinates
- Used for patrol routes and movement

## Special Values
- `show_on_event = 0`: Always visible
- `show_on_event > 0`: Event-triggered
- `dialog_id = 0`: No dialogue
- Fixed 260-byte string fields

## File Purpose
Defines NPC placements with waypoints, dialogue, and behavioral parameters. Used for populating maps with interactive characters.

## Related Files
- `Npccat2.ref`, `Npccat3.ref`, `Npccatp.ref`
- `Npcmap1.ref`, `Npcmap2.ref`, `Npcmap3.ref`
- `npcdun08.ref`, `npcdun19.ref`

## Implementation
- **Rust Module**: `src/references/npc_ref.rs`
- **Extractor**: `NPC` struct implementing `Extractor` trait
- **Database**: Saved to SQLite via `save_npc_refs` function

## Extractor

An extractor is available in `src/references/npc_ref.rs` to parse this file format.

### How to Run

```bash
# Extract Npccat1.ref to JSON
cargo run -- ref npc-ref "fixtures/Dispel/NpcInGame/Npccat1.ref"

# Import to SQLite database
cargo run -- database import "fixtures/Dispel/" "database.sqlite"
```
