# Eventnpc.ref - Event NPC Definitions

## File Information
- **Location**: `NpcInGame/Eventnpc.ref`
- **Format**: CSV with comments
- **Encoding**: WINDOWS-1250
- **Record Size**: Variable (text)

## Structure

### File Format
- Lines starting with `;` are comments
- CSV format with comma delimiter
- Empty lines are ignored

### Record Structure
- `id`: i32 - Unique event NPC identifier
- `event_id`: i32 - Linked event script ID
- `name`: String - NPC display name

## Field Definitions
- `id`: Unique event NPC identifier
- `event_id`: Linked event script ID
- `name`: NPC display name

## NPC Categories
- 1-50: Quest-related NPCs
- 51-100: Story-critical NPCs
- 101-150: Temporary event NPCs
- 151-200: Special merchants/traders
- 201-250: Hidden/secret NPCs

## Event Linking
- `event_id` links to Event.ini entries
- NPCs appear only during specific events
- Removed after event completion
- Can trigger quests and dialogues

## Special Values
- Lines starting with `;` are comments
- CSV format with comma delimiter
- Empty lines ignored

## File Purpose
Defines NPCs that appear only during specific scripted events. Used for quest-related characters, temporary merchants, and story-critical encounters.

## Implementation
- **Rust Module**: `src/references/event_npc_ref.rs`
- **Extractor**: `EventNpcRef` struct implementing `Extractor` trait
- **Database**: Saved to SQLite via `save_event_npc_refs` function

## Example Usage
```bash
cargo run -- ref event-npc-ref "fixtures/Dispel/NpcInGame/Eventnpc.ref"
```
