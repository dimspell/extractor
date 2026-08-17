# Event.ini Documentation

## File Information

### Overview

Text file that defines event scripts with execution conditions, prerequisites, and repetition limits for the game's event system.

### File Structure

**Location**: `Event.ini`
**Encoding**: WINDOWS-1250 (Polish) — the comment header is written in Polish
**Format**: CSV (Comma-Separated Values) with comments
**Total Entries**: 2,251 event mappings (verified: IDs 0–2250)

> **Note on encoding**: The file's comment header is Polish text encoded in
> WINDOWS-1250 (e.g. `poprzedzający`, `wykonywane`). The parser declares
> `EUC_KR` in `src/references/event_ini.rs`, but this is harmless because all
> data fields are pure ASCII (numeric IDs, ASCII script filenames, `null`).
> Only the comment lines carry non-ASCII bytes, and comments are skipped
> during parsing. If the comment header were ever parsed, the encoding would
> need to be corrected to WINDOWS-1250.

### Format Specification

```ini
; Comment line explaining event types and execution conditions
event_id,required_event_id,event_type,script_filename,counter
0,0,0,null,0
1,0,2,script0001.scr,0
2,0,2,script0002.scr,0
...
```

### Field Definitions

| Field | Type | Description |
|-------|------|-------------|
| event_id | i32 | Unique event identifier (0-2250+) |
| required_event_id | i32 | Prerequisite event ID that must be completed first |
| event_type | i32 | Execution condition type (0-8) |
| script_filename | string | Script filename or "null" for no script |
| counter | i32 | Execution limit (0 = unlimited, N = max executions) |

### Event Type System

The file includes a detailed comment header (in Polish) explaining the event type system:

```ini
; Numer zdarzenia, poprzedzający identyfikator zdarzenia, typ:
; 0 - wykonuje bezwarunkowo jeden raz (ignoruje poprzedzające zdarzenie)
; 1 - wykonuje N razy bezwarunkowo (ignoruje poprzedzające zdarzenie)
; 2 - wykonuje bezwarunkowo (ignoruje poprzedzające zdarzenie)
; 3 - wykonywane raz, gdy poprzedzające zdarzenie jest niezadowalające
; 4 - wykonaj N razy w przypadku niezadowolenia
; 5 - kontynuuj wykonywanie, gdy zdarzenie poprzedzające nie jest spełnione
; 6 - wykonaj 1 raz, gdy zdarzenie poprzedzające jest spełnione
; 7 - wykonaj N razy, gdy zdarzenie poprzedzające jest spełnione
; 8 - kontynuuj wykonywanie, gdy zdarzenie poprzedzające jest spełnione
; skrypt nazwa pliku, ilość razy do wykonania (N)
```

The comment header documents the full 0–8 type range, but the codebase
`EventType` enum only maps the four values actually present in the file.

### Event Type Details

**Unconditional Execution (Types 0, 1, 2):**
- Execute regardless of previous event status
- Type 0: Execute once
- Type 1: Execute N times (uses counter)
- Type 2: Execute unconditionally

**Conditional Execution (Types 3-8):**
- Execution depends on previous event status
- Types 3-5: Execute when previous event unsatisfied
- Types 6-8: Execute when previous event satisfied
- Types 4, 7, 8: Use counter for repetition limits

### Verified Event Type Distribution

The `EventType` enum (`src/references/enums.rs`) maps only the values that
actually occur in the file:

| Value | Enum variant | Meaning | Count |
|-------|--------------|---------|-------|
| 0 | `Unknown` | Default / no condition | 1,729 |
| 2 | `Conditional` | Execute N times unconditionally | 504 |
| 5 | `ContinueOnUnsatisfied` | Continue when previous unsatisfied | 10 |
| 6 | `ExecuteOnSatisfied` | Execute once when previous satisfied | 8 |

**Verified facts:**
- All 2,251 entries have `counter = 0` (no repetition limit used in this file).
- 1,503 entries use the literal `null` script filename.
- No data field contains non-ASCII bytes.

### Special Values

- **"null"**: Literal string indicating no script filename
- **counter = 0**: No execution limit (infinite)
- **counter = N**: Maximum execution count
- **;**: Lines starting with semicolon are comments
- **Empty lines**: Ignored during processing

### Example Format

```ini
; Default data
0,0,0,null,0

; Initialization sequence
1,0,2,init_script.scr,0
2,1,6,post_init.scr,1

; Map transition
10,0,2,map_load.scr,0
11,10,7,transition.scr,3
```

### Technical Details

**Encoding**: WINDOWS-1250 (Polish)
- Supports Polish characters in comments
- Data fields are pure ASCII, so the parser's declared `EUC_KR` encoding
  does not affect parsing (comments are skipped)
- Requires proper encoding handling if the comment header is ever read

**File Processing**:
- Comments (lines starting with ";") are ignored
- Empty lines are skipped
- CSV format with comma delimiter
- "null" literal used for missing script filenames

**Database Integration**:
- Processed by `Event` struct in the codebase
- Uses `EventType` enum for type-safe event types
- Stored in database with all field mappings
- Referenced by other game systems (NPC, Extra objects)

### Event Type Enum

The codebase defines a type-safe enum for event types. Only the four values
present in the file are mapped; all others fall back to `Unknown`:

```rust
pub enum EventType {
    Unknown,               // 0 - Default / no condition
    Conditional,           // 2 - Execute N times unconditionally
    ContinueOnUnsatisfied, // 5 - Continue when previous unsatisfied
    ExecuteOnSatisfied,    // 6 - Execute once when previous satisfied
}
```

### Usage in Game

1. **Event System Initialization**: Game loads event mappings from Event.ini
2. **Quest Progression**: Events trigger based on completion status
3. **Script Execution**: Runs associated SCR files when conditions met
4. **State Management**: Tracks event completion with counters
5. **Prerequisite Checking**: Validates previous event requirements

### Event Chaining

The system supports complex event sequences:
- **Linear Progression**: Event A → Event B → Event C
- **Conditional Branching**: Different paths based on success/failure
- **Parallel Events**: Multiple independent event chains
- **Repeating Events**: Limited or unlimited execution cycles

### File Characteristics

- **Entry Count**: 2,251 event mappings (verified)
- **ID Range**: 0-2250 (contiguous, no gaps)
- **Comment Organization**: Logical grouping by function (Polish comments)
- **Encoding**: WINDOWS-1250 (Polish comments); data fields are ASCII
- **Format**: Strict CSV structure

### Notes

- File uses Windows-style line endings (\r\n)
- Comments provide detailed explanations in Polish
- Event system forms core of game progression mechanics
- Integrated with multiple game subsystems
- **No copyrighted game content** is reproduced or distributed

## Legal Notice

⚠️ **DISCLAIMER**: This documentation describes technical file format specifications only. It does not distribute any copyrighted game content, script files, or proprietary assets. All references to event systems are for **educational and research purposes** to document file organization and data structures.

**DISPEL®** is a registered trademark. This documentation is **not affiliated with, endorsed by, or sponsored by** the trademark owner.

## Legal Compliance

This documentation:
- Describes **file format specifications only**
- Does **not** distribute any script files or game content
- Focuses on **technical organization and event system design**
- Uses **generic examples** of event structures
- Maintains **nominal fair use** for trademark references

## Extractor

An extractor is available in `src/references/event_ini.rs` to parse this file format.

### How to Run

```bash
# Extract Event.ini to JSON
cargo run -- extract -i "fixtures/Dispel/Event.ini"

# Import to SQLite database
cargo run -- database import "fixtures/Dispel/" "database.sqlite"
```
