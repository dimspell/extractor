# PartyRef.ref - Party Characters

## File Information
- **Location**: `Ref/PartyRef.ref`
- **Format**: CSV with comments
- **Encoding**: WINDOWS-1250
- **Record Size**: Variable (text)

## File Structure

### File Format
- Lines starting with `;` are comments
- CSV format with comma delimiter
- Empty lines are ignored

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
- Used for character selection and management

### name
- Character display name
- Shown in UI and dialogues
- "null" for unnamed characters
- WINDOWS-1250 encoding for special characters

### map_id
- Origin map identifier
- Where character is initially found
- Links to map files and locations
- Determines recruitment location

### npc_id
- Linked NPC record ID
- Connects to NPC definitions
- Determines appearance and behavior
- Links to `Npc.ini` entries

### dlg_out
- Dialog ID when not in party
- Conversation when character is roaming
- Recruitment dialogue
- Initial interaction text

### dlg_in
- Dialog ID when in party
- Conversation when character is recruited
- Party member dialogue
- Ongoing interaction text

### in_party
- Boolean flag (0/1) marking the character as currently in the party
- The game reads this as a single byte and uses it to highlight the character's
  name in the party member list when non-zero
- `0`: character not in the party (default)
- `1`: character is in the party (name shown highlighted)

## Special Values
- `"null"` literal for missing name/job fields
- Lines starting with `;` are comments
- CSV format with comma delimiter
- Empty lines ignored

## File Purpose
Defines all party characters with their names, classes, origin locations, dialog references, and visual representations. Used for:
- Party management system
- Character recruitment
- Dialogue interactions
- UI display and portraits
- Character progression tracking

## Implementation
- **Rust Module**: `src/references/party_ref.rs`
- **Extractor**: `PartyRef` struct implementing `Extractor` trait
- **Data Structure**: `PartyRef` with character attributes
- **Database**: Saved to SQLite via `save_party_refs` function

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

## Character Classes

### Warrior Types
- Fighter: Melee combat specialist
- Knight: Heavy armor, high defense
- Berserker: High damage, low defense
- Paladin: Holy warrior, balanced

### Mage Types
- Sorcerer: Offensive magic
- Cleric: Healing and support
- Necromancer: Dark magic
- Elementalist: Element-based spells

### Rogue Types
- Thief: Stealth and agility
- Assassin: Critical hits
- Ranger: Ranged combat
- Bard: Support and buffs

### Hybrid Types
- Battle Mage: Magic and melee
- Spellblade: Sword and spell combo
- Monk: Unarmed combat
- Druid: Nature magic

## Party Management

### Recruitment
- Characters found at specific locations
- Dialogue-based recruitment
- Quest requirements may apply
- Limited party size

### Dialogue System
- Different dialogues based on party status
- Context-sensitive conversations
- Character-specific interactions
- Story progression through dialogue

### UI Integration
- Character portraits in menus
- Status displays
- Inventory management
- Skill and ability interfaces

## Game Mechanics

### Character Progression
- Experience gain and leveling
- Skill development
- Equipment and gear
- Stat growth and improvement

### Party Dynamics
- Character relationships
- Synergy and combinations
- Conflict resolution
- Group strategies

### Quest Integration
- Character-specific quests
- Party-based objectives
- Story-driven interactions
- Reward distribution

## Related Files
- `Npc.ini` - NPC definitions
- `PrtIni.db` - Party initialization data
- `PrtLevel.db` - Character progression data
- Dialogue files (`.dlg`)
- Map files (`.map`)

## Data Analysis
The file enables analysis of:
- Character distribution and balance
- Class representation
- Recruitment patterns
- Dialogue coverage
- Party composition options
- Character progression paths

## Extractor

An extractor is available in `src/references/party_ref.rs` to parse this file format.

### How to Run

```bash
# Extract PartyRef.ref to JSON
cargo run -- extract -i "fixtures/Dispel/Ref/PartyRef.ref"

# Import to SQLite database
cargo run -- database import "fixtures/Dispel/" "database.sqlite"
```
