# Event.ini Documentation

> **DISPEL®** is a registered trademark. This project is not affiliated with, endorsed by, or sponsored by the trademark
> owner.

## File Information

### Overview

Text file that defines event scripts with execution conditions, prerequisites, and repetition limits for the game's
event system.

### File Structure

**Location**: `Event.ini`
**Encoding**: EUC-KR (Korean character encoding)
**Format**: CSV (Comma-Separated Values) with comments

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

| Field             | Type   | Description                                         |
|-------------------|--------|-----------------------------------------------------|
| event_id          | i32    | Unique event identifier                             |
| required_event_id | i32    | Prerequisite event ID that must be completed first  |
| event_type        | i32    | Execution condition type (0-8)                      |
| script_filename   | string | Script filename or "null" for no script             |
| counter           | i32    | Execution limit (0 = unlimited, N = max executions) |

### Event Type System

The file includes a detailed comment header explaining the event type system:

```ini
; Event number, preceding event identifier, type:
; 0 - Execute once unconditionally (ignores preceding event)
; 1 - Execute N times unconditionally (ignores preceding event)
; 2 - Execute unconditionally (ignores preceding event)
; 3 - Execute once if preceding event is unsatisfied
; 4 - Execute N times if preceding event is unsatisfied
; 5 - Continue executing if preceding event is unsatisfied
; 6 - Execute once if preceding event is satisfied
; 7 - Execute N times if preceding event is satisfied
; 8 - Continue executing if preceding event is satisfied
; Script filename, number of times to execute (N)
```

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

**Encoding**: EUC-KR (Extended Unix Code Korea)

- Supports Korean characters in comments
- Requires proper encoding handling for reading/writing

**File Processing**:

- Comments (lines starting with ";") are ignored
- Empty lines are skipped
- CSV format with comma delimiter
- "null" literal used for missing script filenames

### Database Integration

- Processed by the `Event` struct in `src/references/event_ini.rs`
- Uses the `EventType` enum from `src/references/enums.rs` for type-safe event types
- Stored in database with all field mappings
- Referenced by other game systems (NPC, Extra objects)

### Event Chaining

The system supports complex event sequences:

- **Linear Progression**: Event A → Event B → Event C
- **Conditional Branching**: Different paths based on success/failure
- **Parallel Events**: Multiple independent event chains
- **Repeating Events**: Limited or unlimited execution cycles

### Notes

- File uses Windows-style line endings (`\r\n`)
- Comments provide detailed explanations
- Event system forms core of game progression mechanics
- Integrated with multiple game subsystems

## Extractor

The file is parsed by the `Event` struct in `src/references/event_ini.rs`, which implements the `Extractor` trait.

### How to Run

```bash
# Extract Event.ini to JSON
cargo run -- extract -i "Event.ini"

# Import to SQLite database
cargo run -- database import "path/to/Dispel/" "database.sqlite"
```
