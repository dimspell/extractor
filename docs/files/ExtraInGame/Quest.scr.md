# Quest.scr - Quest Journal Entries

## File Information
- **Location**: `ExtraInGame/Quest.scr`
- **Format**: Pipe-delimited text
- **Encoding**: WINDOWS-1250
- **Record Size**: Variable (text)

## File Structure

### File Format
- Lines starting with `;` are comments
- Pipe-delimited format: `field1|field2|field3|field4`
- Empty lines are ignored

### Record Structure
- `id`: i32 - Unique quest identifier
- `type`: i32 - Quest category
- `title`: String - Quest title/name
- `description`: String - Quest description/text

## Field Definitions

### id
- Unique quest identifier
- References from event system
- Used for quest tracking

### type
- Quest category identifier
- Determines quest classification
- Values:
  - `0`: Main quests (primary story line)
  - `1`: Side quests (optional content)
  - `2`: Traders journal (commerce-related)

### title
- Quest title/name
- Displayed in quest journal
- Short summary of quest objective
- "null" for empty titles

### description
- Quest description/text
- Detailed quest information
- Objectives and requirements
- "null" for empty descriptions

## Quest Types

### Main Quests (type=0)
- Primary story line
- Required for game completion
- Major plot points
- Critical path quests

### Side Quests (type=1)
- Optional content
- Additional rewards
- World exploration
- Character development
- Lore expansion

### Traders Journal (type=2)
- Commerce-related quests
- Trading missions
- Economic activities
- Merchant interactions
- Supply chain quests

## Special Values
- `"null"` literal for missing title/description
- Lines starting with `;` are comments
- Pipe (`|`) delimiter between fields
- Empty lines ignored

## File Purpose
Defines all quests with categories, titles, and descriptions. Used for:
- Quest journal system
- Quest tracking and progression
- Player objectives
- Story advancement
- Reward distribution
- Linked to event system for completion

## Implementation
- **Rust Module**: `src/references/quest_scr.rs`
- **Extractor**: `Quest` struct implementing `Extractor` trait
- **Data Structure**: `Quest` with ID, type, title, and description
- **Database**: Saved to SQLite via `save_quests` function

## Example Usage

### Extract and display quests:
```bash
cargo run -- extract -i "fixtures/Dispel/ExtraInGame/Quest.scr"
```

### Format Example
```
; Main quests
1|0|Main Quest|Defeat the Dark Lord
2|0|The Journey Begins|Find the ancient artifact
; Side quests
10|1|Side Quest|Find the Lost Artifact
11|1|Mysterious Stranger|Investigate the rumors
; Traders journal
50|2|Merchant Guild|Deliver goods to the town
51|2|Trade Route|Establish new trade connections
```

## Extractor

An extractor is available in `src/references/quest_scr.rs` to parse this file format.

### How to Run

```bash
# Extract Quest.scr to JSON
cargo run -- extract -i "fixtures/Dispel/ExtraInGame/Quest.scr"

# Import to SQLite database
cargo run -- database import "fixtures/Dispel/" "database.sqlite"
```


