# Pgpcat1.pgp - Dialogue Text

## File Information
- **Location**: `NpcInGame/*.pgp`
- **Format**: Commented pipe-delimited
- **Encoding**: WINDOWS-1250
- **Record Size**: Variable (text)

## Structure

### File Format
- Lines starting with `;` are comments
- Pipe-delimited format
- Empty lines are ignored

### Record Structure
- `id`: i32 - Unique dialogue text identifier
- `text`: String - Display text content
- `param1`: i32 - Logic parameter 1
- `param2`: i32 - Logic parameter 2

## Field Definitions
- `id`: Unique dialogue text identifier
- `text`: Display text content
- `param1`: Logic parameter 1
- `param2`: Logic parameter 2

## Parameter Usage
- `param1`: Dialogue branch conditions
- `param2`: Event triggers or requirements
- Special values: 0 = no condition

## Text Formatting
- "null" literal for empty text
- "$" literal interpreted as a line-break in game
- Pipe (|) delimiter between fields
- Semicolon (;) for comment lines
- Multi-line comments supported

## Special Values
- `param1 = 0`: Unconditional dialogue
- `param2 = 0`: No event trigger
- Empty text: "null" literal
- Comment lines preserved with ";" prefix

## File Purpose
Stores dialogue text content with developer comments and logical parameters. Used for displaying conversation text, branching dialogue, and triggering game events.

## Related Files
- `Pgpcat2.pgp`, `Pgpcat3.pgp`, `Pgpcatp.pgp`
- `Pgpmap1.pgp`, `Pgpmap2.pgp`, `Pgpmap3.pgp`
- `Pgpdun04.pgp`, `Pgpdun07.pgp`, `Pgpdun08.pgp`, `Pgpdun10.pgp`, `Pgpdun19.pgp`, `Pgpdun22.pgp`
- `PartyPgp.pgp`

## Implementation
- **Rust Module**: `src/references/dialogue_text.rs`
- **Extractor**: `DialogueText` struct implementing `Extractor` trait
- **Database**: Saved to SQLite via `save_dialogue_texts` function

## Extractor

An extractor is available in `src/references/dialogue_text.rs` to parse this file format.

### How to Run

```bash
# Extract Pgpmap1.pgp to JSON
cargo run -- ref dialog-texts "fixtures/Dispel/NpcInGame/Pgpmap1.pgp"

# Import to SQLite database
cargo run -- database import "fixtures/Dispel/" "database.sqlite"
```
