# Message.scr - UI Text Messages

## File Information
- **Location**: `ExtraInGame/Message.scr`
- **Format**: Pipe-delimited text
- **Encoding**: WINDOWS-1250
- **Record Size**: Variable (text)

## File Structure

### File Format
- Lines starting with `;` are comments
- Pipe-delimited format: `field1|field2|field3|field4`
- Empty lines are ignored

### Record Structure
- `id`: i32 - Unique message identifier
- `line1`: String - First text line (top)
- `line2`: String - Second text line (middle)
- `line3`: String - Third text line (bottom)

## Field Definitions

### id
- Unique message identifier
- References from game objects and events
- Used for displaying specific messages

### line1, line2, line3
- Three lines of text for UI display
- Each line can be up to 255 characters
- Lines are displayed vertically stacked
- Empty lines use "null" literal

## Special Values
- `"null"` literal for empty text lines
- Lines starting with `;` are comments
- Pipe (`|`) delimiter between fields
- Maximum 3 lines per message

## File Purpose
Stores multi-line text messages for UI elements such as:
- Signposts and plaques
- System notifications
- Environmental storytelling
- Player guidance and hints
- Object descriptions

## Message Types

### Signposts
- Location descriptions
- Area names and information

## Implementation
- **Rust Module**: `src/references/message_scr.rs`
- **Extractor**: `Message` struct implementing `Extractor` trait
- **Data Structure**: `Message` with ID and three text lines
- **Database**: Saved to SQLite via `save_messages` function

## Example Usage

### Extract and display messages:
```bash
cargo run -- extract -i "Dispel/ExtraInGame/Message.scr"
```

### Format Example
```
; Welcome message
1|Welcome|to the|town
; Danger warning
2|Danger|Ahead|!
; Quest hint
3|Find the|hidden|treasure
; Empty middle line
4|Top line||Bottom line
```

## Display System
- Messages are displayed in UI panels
- Three-line vertical layout
- Centered or aligned based on context
- WINDOWS-1250 character encoding support

## Related Files
- `Quest.scr` - Quest journal entries
- `Extra.ini` - Object definitions referencing messages
- Game scripts triggering message display

## Extractor

An extractor is available in `src/references/message_scr.rs` to parse this file format.

### How to Run

```bash
# Extract Message.scr to JSON
cargo run -- extract -i "fixtures/Dispel/ExtraInGame/Message.scr"

# Import to SQLite database
cargo run -- database import "fixtures/Dispel/" "database.sqlite"
```
