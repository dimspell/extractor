# PartyRef.ref: party characters

## File Information

- **Location**: `Ref/PartyRef.ref`
- **Format**: CSV with comments
- **Encoding**: WINDOWS-1250
- **Record Size**: Variable (text)

## File Structure

### File Format

- Lines starting with `;` are comments
- CSV format with comma delimiter
- The parser ignores empty lines.

### Record Structure

- `id`: i32 - Unique character identifier
- `name`: String - Character display name or "null"
- `job`: String - Character class/job or "null"
- `map_id`: i32 - Origin map ID where character is found
- `npc_id`: i32 - Linked NPC record ID
- `dlg_out`: i32 - Dialog ID when not in party
- `dlg_in`: i32 - Dialog ID when in party
- `in_party`: i32 - Boolean flag (0/1) marking the character as currently in the party

## Field Definitions

### id

- Unique character identifier
- Party member index
- The tool uses this ID for character selection.

### name

- Character display name
- Shown in UI and dialogues
- "null" for unnamed characters
- WINDOWS-1250 encoding for special characters

### map_id

- Origin map identifier
- Where character is initially found
- Links to map files and locations
- Records the origin map ID.

### npc_id

- Linked NPC record ID
- Connects to NPC definitions
- Determines appearance and behavior
- Links to `Npc.ini` entries

### dlg_out

- Dialog ID when not in party
- Conversation text outside the party
- Recruitment dialogue
- Initial interaction text

### dlg_in

- Dialog ID when in party
- Conversation text inside the party
- Party member dialogue
- Later interaction text

### in_party

- Boolean flag (0/1) marking the character as currently in the party
- A single byte. Non-zero marks the name as highlighted in the party member list.
- `0`: character not in the party (default)
- `1`: character is in the party (name shown highlighted)

## Special Values

- `"null"` literal for missing name/job fields
- Lines starting with `;` are comments
- CSV format with comma delimiter
- Empty lines ignored

## File Purpose

The tool reads each row as a character record with a name, a job, an origin map and dialogue IDs. The data supports:

- Party membership flags
- Character origin records
- Dialogue references
- UI text and portraits
- Character progression records

## Implementation

- **Rust Module**: `src/references/party_ref.rs`
- **Extractor**: `PartyRef` struct implementing `Extractor` trait
- **Data Structure**: `PartyRef` with character attributes
- **Database**: saved to SQLite by the `save_party_refs` function

## Example Usage

### Extract and display party characters:

```bash
cargo run -- extract -i "fixtures/Dispel/Ref/PartyRef.ref"
```

### Format Example

```
; Party character definitions
; id,name,job,map_id,npc_id,dlg_out,dlg_in,in_party
1,Hero,null,1,1,100,101,1
2,Warrior,Fighter,1,2,102,103,0
3,Mage,Sorcerer,2,3,104,105,0
4,Rogue,Thief,3,4,106,107,0
```

## Related Files

- `Npc.ini`: NPC definitions
- `PrtIni.db`: party initialization data
- `PrtLevel.db`: character progression data
- Dialogue files (`.dlg`)
- Map files (`.map`)

## Extractor

An extractor is available in `src/references/party_ref.rs` to parse this file format.

### How to Run

```bash
# Extract PartyRef.ref to JSON
cargo run -- extract -i "fixtures/Dispel/Ref/PartyRef.ref"

# Import to SQLite database
cargo run -- database import "fixtures/Dispel/" "database.sqlite"
```
