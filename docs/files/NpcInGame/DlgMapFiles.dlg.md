# Dlgcat1.dlg: dialogue scripts

## File Information

- **Location**: `NpcInGame/Dlgcat1.dlg` (and other .dlg files)
- **Format**: CSV with comments
- **Encoding**: EUC-KR
- **Record Size**: Variable (text)

## Structure

### File Format

- Lines starting with `;` are comments
- CSV format with comma delimiter
- The parser ignores empty lines.

### Record Structure

Each line has exactly 10 comma-separated fields:

- `id`: i32 - Unique dialogue line identifier
- `prev_event`: i32 - Required event ID to trigger
- `next_dlg`: i32 - Next dialogue ID in chain
- `type`: i32 - 0=normal, 1=choice dialog
- `owner`: i32 - 0=player, 1=NPC
- `dlg_id`: i32 - Reference to PGP text content
- `opt0`: i32 - Next dialogue option 1 (choice dialogs) or next dialog in linear type
- `opt1`: i32 - next dialogue option 2 for choice dialogs. Zero means absent.
- `opt2`: i32 - next dialogue option 3 for choice dialogs. Zero means absent.
- `event_id`: i32 - Event triggered by dialogue

## Field Definitions

- `id`: Unique dialogue line identifier
- `prev_event`: Required event ID to trigger
- `next_dlg`: Next dialogue ID in chain
- `type`: 0=normal, 1=choice dialog
- `owner`: 0=player, 1=NPC
- `dlg_id`: Reference to PGP text content
- `opt0` to `opt2`: choice-dialog targets. Unused options are 0.
- `event_id`: Event triggered by dialogue

## Dialogue Types

- `0`: Normal dialogue (linear conversation)
- `1`: Choice dialogue (branching options)

## Dialogue Owners

- `0`: Main character/player speaking
- `1`: NPC character speaking

## Special Values

- Optional fields are `0` when absent (not `"null"`)
- Lines starting with `;` are comments
- CSV format with comma delimiter
- The parser ignores empty lines.

## File Purpose

Defines dialogue scripts with branching conversations, event triggers, and text references. Used for NPC interactions,
quest dialogues, and story progression systems.

## Related Files

- `Dlgcat2.dlg`, `Dlgcat3.dlg`, `Dlgcatp.dlg`
- `Dlgmap1.dlg`, `Dlgmap2.dlg`, `Dlgmap3.dlg`
- `Dlgdun04.dlg`, `Dlgdun07.dlg`, `Dlgdun08.dlg`, `Dlgdun10.dlg`, `Dlgdun19.dlg`, `Dlgdun22.dlg`
- `PartyDlg.dlg`

## Implementation

- **Rust Module**: `src/references/dialogue_script.rs`
- **Extractor**: `DialogueScript` struct implementing `Extractor` trait
- **Database**: saved to SQLite by the `save_dialogs` function

## Extractor

An extractor is available in `src/references/dialogue_script.rs` to parse this file format.

### How to Run

```bash
# Extract Dlgcat1.dlg to JSON
cargo run -- extract -i "fixtures/Dispel/NpcInGame/Dlgcat1.dlg"

# Import to SQLite database
cargo run -- database import "fixtures/Dispel/" "database.sqlite"
```
