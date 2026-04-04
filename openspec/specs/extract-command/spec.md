## ADDED Requirements

### Requirement: Extract command with required input flag
The CLI SHALL provide an `extract` command that accepts a `--input` (short: `-i`) flag specifying the path to a game file. The `--input` flag is required.

#### Scenario: Extract with input flag
- **WHEN** user runs `dispel-extractor extract --input path/to/file.db`
- **THEN** the command reads the file and outputs JSON to stdout

#### Scenario: Missing input flag fails
- **WHEN** user runs `dispel-extractor extract` without `--input`
- **THEN** the command exits with an error and shows usage information

### Requirement: Extract outputs JSON to stdout by default
When no `--output` flag is provided, the extracted JSON SHALL be written to stdout.

#### Scenario: Default stdout output
- **WHEN** user runs `dispel-extractor extract --input file.db`
- **THEN** JSON is printed to stdout

### Requirement: Extract outputs JSON to file with output flag
When `--output` (short: `-o`) flag is provided, the extracted JSON SHALL be written to the specified file path.

#### Scenario: File output
- **WHEN** user runs `dispel-extractor extract --input file.db --output out.json`
- **THEN** JSON is written to `out.json`

### Requirement: Extract auto-detects file type
The extract command SHALL auto-detect the file type from the input file's extension and content. The `--type` flag MAY be used to override auto-detection.

#### Scenario: Auto-detect .db file
- **WHEN** user runs `dispel-extractor extract --input weaponItem.db`
- **THEN** the command detects the file as `weapons` type and extracts successfully

#### Scenario: Type override
- **WHEN** user runs `dispel-extractor extract --input unknown_file --type weapons`
- **THEN** the command uses the specified type and extracts successfully

#### Scenario: Unknown file type error
- **WHEN** user runs `dispel-extractor extract --input unknown.xyz` with no matching type
- **THEN** the command exits with an error listing closest matching file types

### Requirement: Extract pretty-print flag
The `--pretty` (short: `-p`) flag SHALL produce indented, human-readable JSON output. Without this flag, output format is determined by context (pretty for terminal, compact for pipes).

#### Scenario: Pretty-print to file
- **WHEN** user runs `dispel-extractor extract --input file.db --output out.json --pretty`
- **THEN** the output file contains indented JSON

### Requirement: Extract outputs JSON array of records
The extracted JSON SHALL be a JSON array where each element represents one record from the game file. Each record includes an `id` field representing its zero-based index.

#### Scenario: Weapons extraction output format
- **WHEN** user extracts a weapons database with 3 entries
- **THEN** stdout contains `[{ "id": 0, ... }, { "id": 1, ... }, { "id": 2, ... }]`

### Requirement: Extract error handling
When extraction fails, the command SHALL exit with a non-zero code and print a descriptive error message to stderr. Specific error cases:
- File not found: message includes the attempted path
- Parse failure: message includes line/record number and context
- Unknown file type: message lists closest matching types

#### Scenario: File not found
- **WHEN** user runs `dispel-extractor extract --input nonexistent.db`
- **THEN** stderr contains "File not found" with the path and exit code is non-zero

#### Scenario: Parse failure
- **WHEN** user runs extract on a corrupted file
- **THEN** stderr contains the record/line number and error context
