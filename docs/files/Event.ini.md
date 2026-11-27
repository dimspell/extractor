# Event.ini documentation

## File information

### Overview

The tool reads `Event.ini` as a CSV file with comments. Each row holds an event ID, a prerequisite ID, a type code, a script filename and a counter.

### File structure

- **Location**: `Event.ini`
- **Encoding**: EUC-KR (Korean character encoding)
- **Format**: CSV (Comma-Separated Values) with comments

### Format specification

```ini
; Comment line explaining event types and execution conditions
event_id,required_event_id,event_type,script_filename,counter
0,0,0,null,0
1,0,2,script0001.scr,0
2,0,2,script0002.scr,0
...
```

### Field definitions

| Field             | Type   | Description                                         |
|-------------------|--------|-----------------------------------------------------|
| event_id          | i32    | Unique event identifier                             |
| required_event_id | i32    | Prerequisite event ID that must be completed first  |
| event_type        | i32    | Execution condition type (0-8)                      |
| script_filename   | string | Script filename or "null" for no script             |
| counter           | i32    | Execution limit (0 = unlimited, N = max executions) |

### Event type system

The file carries a comment header that defines the type codes:

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

### Event type details

**Unconditional execution (types 0, 1 and 2):**

- Type 0 uses one execution and ignores the prerequisite.
- Type 1 uses `N` executions through the counter and ignores the prerequisite.
- Type 2 has no counter and ignores the prerequisite.

**Conditional execution (types 3 to 8):**

- Types 3 to 5 use an unsatisfied prerequisite.
- Types 6 to 8 use a satisfied prerequisite.
- Types 4, 7 and 8 use the counter for repetition limits.

### Special values

- **"null"**: The tool reads this string when no script filename is present.
- **counter = 0**: No execution limit.
- **counter = N**: Maximum execution count.
- **`;`**: Lines starting with this mark are comments.
- **Empty lines**: The parser ignores them.

### Example format

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

### Technical details

**Encoding**: EUC-KR (Extended Unix Code Korea)

- The encoding covers Korean characters in comments.
- The reader and writer must use this encoding.

**File processing:**

- The parser ignores comment lines that start with `;`.
- It skips empty lines.
- It splits rows on commas.
- It reads `null` as a missing script filename.

### Database integration

The `Event` struct in `src/references/event_ini.rs` parses the file. The `EventType` enum in `src/references/enums.rs` holds the type value. The tool stores every field in the database. Other file formats reference event IDs.

### Event chains

- Linear progression: Event A → Event B → Event C
- Conditional branching: paths based on a prerequisite state
- Parallel events: independent event chains
- Repeated events: limited or unlimited execution cycles

### Notes

- The file uses Windows-style line endings (`\r\n`).
- The comment header defines the type codes.
- Event IDs link to other file formats.

## Extractor

The `Event` struct in `src/references/event_ini.rs` parses the file. It implements the `Extractor` trait.

### How to run

```bash
# Extract Event.ini to JSON
cargo run -- extract -i "Event.ini"

# Import to SQLite database
cargo run -- database import "path/to/Dispel/" "database.sqlite"
```
