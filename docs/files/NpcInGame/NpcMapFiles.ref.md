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

| Offset | Field | Meaning |
|---:|---|---|
| 0x000 | file_record_id | File-local value; the map loader uses the record index for runtime identity. |
| 0x004 | npc_ini_id | NPC visual-archetype ID from `Npc.ini`. |
| 0x008 | name | 260-byte Windows-1250 display name. |
| 0x10c | role_description | 260-byte Windows-1250 role/description text. |
| 0x210 | party_member_slot | Values 1–8 select party/recruitment NPC behavior. |
| 0x214 | show_on_event | Event condition controlling visibility. |
| 0x218 | movement_mode | 0 static, 1 waypoint patrol, 2 random movement in activation rectangle. |
| 0x21c | goto1_filled..goto4_filled | Four waypoint-enabled flags. |
| 0x22c | goto1_x..goto4_x | Waypoint X coordinates. |
| 0x23c | goto1_y..goto4_y | Waypoint Y coordinates. |
| 0x24c | waypoint1_wait_time..waypoint4_wait_time | Delay at each waypoint. |
| 0x25c | waypoint1_facing_direction..waypoint4_facing_direction | Facing direction at each waypoint. |
| 0x26c | waypoint1_reserved..waypoint4_reserved | Per-waypoint reserved values, observed as zero. |
| 0x27c | activation_rect_x1..activation_rect_y2 | Rectangle used by random movement. |
| 0x28c | interaction_mode | 0 default, 1 random result, 2 configured result once then random. |
| 0x290 | interaction_result_item + interaction_result_parameter | Packed 32-bit interaction result. |
| 0x294 | interaction_range_offset | The game adds one before comparing interaction distance. |
| 0x298 | dialog_id | Dialogue node ID. |
| 0x29c | dialogue_face_sprite_id | Face sprite ID. |

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
- `interaction_range_offset`: Runtime interaction range is this value plus one.
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
cargo run -- extract -i "fixtures/Dispel/NpcInGame/Npccat1.ref"

# Import to SQLite database
cargo run -- database import "fixtures/Dispel/" "database.sqlite"
```
