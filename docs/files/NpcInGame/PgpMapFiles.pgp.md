# Pgpcat1.pgp - Dialogue Text

> DISPEL® is a registered trademark. This project is not affiliated with,
> endorsed by, or sponsored by the trademark owner.

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
- `comment`: String - Developer notes (accumulated from preceding `;` comment lines, joined with ` | `)
- `param1`: i32 - Dialogue branch conditions
- `wave_ini_entry_id`: i32 - ID of sound from `Wave.ini`, played at start of dialogue

## Field Definitions

- `id`: Unique dialogue text identifier
- `text`: Display text content
- `comment`: Developer notes (accumulated from preceding `;` comment lines)
- `param1`: Dialogue branch conditions
- `wave_ini_entry_id`: Sound effect ID from `Wave.ini`

## Parameter Usage

- `param1`: Dialogue branch conditions
- `wave_ini_entry_id`: Sound effect played at dialogue start; 0 = none

## Text Formatting

- "null" literal for empty text
- "$" literal interpreted as a line-break in game
- Pipe (|) delimiter between fields
- Semicolon (;) for comment lines
- Multi-line comments supported

## Special Values

- `param1 = 0`: Unconditional dialogue
- `wave_ini_entry_id = 0`: No sound trigger
- Empty text: `"null"` literal (mapped to empty string on parse)
- Comment lines preserved with `;` prefix

## File Purpose

Stores dialogue text content with developer comments and logical parameters. Used for displaying conversation text,
branching dialogue, and triggering game events.

## Related Files

- `Pgpcat2.pgp`, `Pgpcat3.pgp`, `Pgpcatp.pgp`
- `Pgpmap1.pgp`, `Pgpmap2.pgp`, `Pgpmap3.pgp`
- `Pgpdun04.pgp`, `Pgpdun07.pgp`, `Pgpdun08.pgp`, `Pgpdun10.pgp`, `Pgpdun19.pgp`, `Pgpdun22.pgp`
- `PartyPgp.pgp`

## Implementation

- **Rust Module**: `src/references/dialogue_paragraph.rs`
- **Extractor**: `DialogueParagraph` struct implementing `Extractor` trait
- **Database**: Saved to SQLite via `save_dialogue_paragraphs` function

## Extractor

An extractor is available in `src/references/dialogue_paragraph.rs` to parse this file format.

### How to Run

```bash
# Extract Pgpmap1.pgp to JSON
cargo run -- extract -i "fixtures/Dispel/NpcInGame/Pgpmap1.pgp"

# Import to SQLite database
cargo run -- database import "fixtures/Dispel/" "database.sqlite"
```
