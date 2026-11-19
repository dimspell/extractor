# Eventnpc.ref - Event NPC Definitions

> DISPEL® is a registered trademark. This project is not affiliated with,
> endorsed by, or sponsored by the trademark owner.

## File Information

- **Location**: `NpcInGame/Eventnpc.ref`
- **Format**: CSV with comments
- **Encoding**: WINDOWS-1250
- **Record Size**: Variable (text), 40 columns per row
- **Header**: None

## Structure

### File Format

- Lines starting with `;` are comments
- Rows are comma-separated, 40 columns each
- Rows are terminated by CRLF
- Empty lines are ignored

### Record Structure

A row holds the full placement record of one NPC. It mirrors the 672-byte
binary `NpcInGame/Npccat*.ref` record.

| Column | Field                        | Type      | Meaning                                                  |
|-------:|------------------------------|-----------|----------------------------------------------------------|
|      0 | `id`                         | i32       | Row index used by the event opcodes to address this NPC  |
|      1 | `npc_ini_id`                 | i32       | NPC visual-archetype ID from `Npc.ini`                   |
|      2 | `name`                       | String    | Display name shown by the game                           |
|      3 | `role_description`           | String    | Role or descriptive text stored with the NPC             |
|      4 | `role_result`                | i32       | Role behavior selected when the NPC is interacted with   |
|      5 | `show_on_event`              | i32       | Event ID condition required to spawn the NPC             |
|      6 | `movement_mode`              | i32       | Movement pattern                                         |
|      7 | `waypoint1_filled`           | i32       | Waypoint 1 definition flag (0 or 1)                      |
|      8 | `waypoint1_x`                | i32       | Waypoint 1 X target                                      |
|      9 | `waypoint1_y`                | i32       | Waypoint 1 Y target                                      |
|     10 | `waypoint1_wait_time`        | i32       | Wait time before leaving waypoint 1                      |
|     11 | `waypoint1_facing_direction` | i32       | Facing direction at waypoint 1                           |
|     12 | `waypoint1_reserved`         | i32       | Reserved, observed as zero                               |
|     13 | `waypoint2_filled`           | i32       | Waypoint 2 definition flag                               |
|     14 | `waypoint2_x`                | i32       | Waypoint 2 X target                                      |
|     15 | `waypoint2_y`                | i32       | Waypoint 2 Y target                                      |
|     16 | `waypoint2_wait_time`        | i32       | Wait time before leaving waypoint 2                      |
|     17 | `waypoint2_facing_direction` | i32       | Facing direction at waypoint 2                           |
|     18 | `waypoint2_reserved`         | i32       | Reserved, observed as zero                               |
|     19 | `waypoint3_filled`           | i32       | Waypoint 3 definition flag                               |
|     20 | `waypoint3_x`                | i32       | Waypoint 3 X target                                      |
|     21 | `waypoint3_y`                | i32       | Waypoint 3 Y target                                      |
|     22 | `waypoint3_wait_time`        | i32       | Wait time before leaving waypoint 3                      |
|     23 | `waypoint3_facing_direction` | i32       | Facing direction at waypoint 3                           |
|     24 | `waypoint3_reserved`         | i32       | Reserved, observed as zero                               |
|     25 | `waypoint4_filled`           | i32       | Waypoint 4 definition flag                               |
|     26 | `waypoint4_x`                | i32       | Waypoint 4 X target                                      |
|     27 | `waypoint4_y`                | i32       | Waypoint 4 Y target                                      |
|     28 | `waypoint4_wait_time`        | i32       | Wait time before leaving waypoint 4                      |
|     29 | `waypoint4_facing_direction` | i32       | Facing direction at waypoint 4                           |
|     30 | `waypoint4_reserved`         | i32       | Reserved, observed as zero                               |
|     31 | `activation_rect_x1`         | i32       | Activation rectangle X1 coordinate                       |
|     32 | `activation_rect_y1`         | i32       | Activation rectangle Y1 coordinate                       |
|     33 | `activation_rect_x2`         | i32       | Activation rectangle X2 coordinate                       |
|     34 | `activation_rect_y2`         | i32       | Activation rectangle Y2 coordinate                       |
|     35 | `interaction_mode`           | i32       | Special interaction-result behavior                      |
|     36 | `interaction_result`         | i32       | Packed interaction result                                |
|     37 | `interaction_range_offset`   | i32       | Value added to one for the distance comparison           |
|     38 | `dialog_id`                  | i32       | Dialogue triggered when the NPC is clicked               |
|     39 | `dialogue_face_sprite_id`    | i32       | Portrait sprite shown in dialogue windows                |

### Enum Values

`role_result` (`NpcRoleResult`)

| Value | Meaning              |
|------:|----------------------|
|     0 | NormalDialogue       |
|   1-8 | PartyMemberDialogue1-8 |
|     9 | WeaponShop           |
|    10 | HealMiscShop         |
|    11 | EditItemShop         |
|    12 | Inn                  |

`movement_mode` (`NpcMovementMode`)

| Value | Meaning                  |
|------:|--------------------------|
|     0 | Static                   |
|     1 | Waypoints                |
|     2 | RandomInActivationRect   |

`waypointN_facing_direction` (`NpcLookingDirection`)

| Value | Meaning   |
|------:|-----------|
|     0 | Up        |
|     1 | UpRight   |
|     2 | Right     |
|     3 | DownRight |
|     4 | Down      |
|     5 | DownLeft  |
|     6 | Left      |
|     7 | UpLeft    |

`interaction_mode` (`NpcInteractionMode`)

| Value | Meaning                |
|------:|------------------------|
|     0 | Default                |
|     1 | RandomResult           |
|     2 | ConfiguredThenRandom   |

`interaction_result` is a packed 32-bit value. The low 16 bits hold the item
id, the high 16 bits the parameter - the same split the `Npccat*.ref` records use.

## Event Linking

- `show_on_event` links to `Event.ini` entries
- NPCs appear only during specific events
- Removed after event completion
- Can trigger quests and dialogues

## Special Values

- Lines starting with `;` are comments
- Empty lines ignored
- The `id` column is the row index. The game itself ignores the value and
  selects rows by position.

## Known Data Defect

The shipped `Eventnpc.ref` holds one row whose comma between the
`npc_ini_id` and the `name` column is missing:

```text
2,69Character Name,guard,0,0,1,1,301,251,0,4,0,...,0,6
```

The row therefore carries 39 columns. The game reads the leading digits of
`69Character Name` as the archetype id and loses the name. The extractor splits
the glued column back apart, so the record keeps the id `69` and the name
`Character Name`. Saving the file writes the repaired 40-column row.

## Extractor

An extractor is available in `src/references/event_npc_ref.rs` to parse this file format.

### How to Run

```bash
# Extract Eventnpc.ref to JSON
cargo run -- extract -i "fixtures/Dispel/NpcInGame/Eventnpc.ref"
```
