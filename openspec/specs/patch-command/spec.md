## ADDED Requirements

### Requirement: Patch command with input and target flags
The CLI SHALL provide a `patch` command that accepts `--input` (short: `-i`) for the JSON file path and `--target` (short: `-t`) for the game file to patch. Both flags are required.

#### Scenario: Patch with required flags
- **WHEN** user runs `dispel-extractor patch --input data.json --target file.db`
- **THEN** the command reads the JSON and writes the game file to the target path

#### Scenario: Missing required flags fails
- **WHEN** user runs `dispel-extractor patch` without `--input` or `--target`
- **THEN** the command exits with an error and shows usage information

### Requirement: Patch writes to target by default
When no `--output` flag is provided, the patched game file SHALL be written to the `--target` path.

#### Scenario: Default target write
- **WHEN** user runs `dispel-extractor patch --input data.json --target file.db`
- **THEN** `file.db` is overwritten with the patched data

### Requirement: Patch with alternate output path
When `--output` (short: `-o`) flag is provided, the patched game file SHALL be written to the output path instead of the target path.

#### Scenario: Alternate output
- **WHEN** user runs `dispel-extractor patch --input data.json --target original.db --output modified.db`
- **THEN** `modified.db` is created with the patched data and `original.db` is unchanged

### Requirement: Patch dry-run mode
The `--dry-run` (short: `-n`) flag SHALL validate the JSON input against the target file format without writing any files.

#### Scenario: Successful dry-run
- **WHEN** user runs `dispel-extractor patch --input data.json --target file.db --dry-run`
- **THEN** the command validates the JSON and prints a success message without modifying any files

#### Scenario: Failed dry-run
- **WHEN** user runs `dispel-extractor patch --input invalid.json --target file.db --dry-run`
- **THEN** the command exits with an error describing the validation failure

### Requirement: Patch in-place mode with backup
The `--in-place` flag SHALL patch the target file directly, creating a `.bak` backup of the original file before writing. The `--no-backup` flag SHALL skip backup creation when used with `--in-place`.

#### Scenario: In-place with backup
- **WHEN** user runs `dispel-extractor patch --input data.json --target file.db --in-place`
- **THEN** `file.db.bak` is created with the original content and `file.db` is updated

#### Scenario: In-place without backup
- **WHEN** user runs `dispel-extractor patch --input data.json --target file.db --in-place --no-backup`
- **THEN** `file.db` is updated without creating a backup

### Requirement: Patch type override
The `--type` flag SHALL override auto-detection of the file type for the target file.

#### Scenario: Patch with type override
- **WHEN** user runs `dispel-extractor patch --input data.json --target file.db --type weapons`
- **THEN** the command uses the weapons format to write the file

### Requirement: Patch uses existing save_file implementations
The patch command SHALL use the existing `Extractor::save_file()` implementations from the `references` module for all 29 file types. No new serialization logic SHALL be introduced.

#### Scenario: Round-trip integrity
- **WHEN** user extracts a file to JSON, then patches it back without modifications
- **THEN** the resulting file is byte-identical to the original

### Requirement: Patch error handling
When patching fails, the command SHALL exit with a non-zero code and print a descriptive error to stderr. Specific cases:
- JSON parse failure: message includes the JSON error location
- Validation failure: message includes field-level errors
- File permission denied: message includes the path and suggestion
- Backup creation failure: command aborts with message

#### Scenario: Invalid JSON input
- **WHEN** user runs `dispel-extractor patch --input invalid.json --target file.db`
- **THEN** stderr contains the JSON parse error and exit code is non-zero

#### Scenario: Backup failure aborts
- **WHEN** user runs patch with `--in-place` on a read-only filesystem
- **THEN** the command aborts with an error message suggesting `--output` instead
