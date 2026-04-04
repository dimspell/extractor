## ADDED Requirements

### Requirement: File type registry with static definitions
The system SHALL provide a static registry of all 29 supported file types, each defined with a key, name, description, extensions, detection function, extract function, and patch function.

#### Scenario: Registry contains all types
- **WHEN** the registry is queried for all types
- **THEN** it returns exactly 29 entries covering all reference file formats

#### Scenario: Registry lookup by key
- **WHEN** the registry is queried with key "weapons"
- **THEN** it returns the weapons file type definition

### Requirement: Auto-detection by extension
The registry SHALL support auto-detection of file types based on file extension (`.db`, `.ini`, `.ref`, `.dlg`, `.scr`, `.pgp`). Extension matching SHALL be case-insensitive.

#### Scenario: Detect .db extension
- **WHEN** a file with `.db` extension is passed to detection
- **THEN** candidate types with `.db` extension are returned

#### Scenario: Detect .ini extension
- **WHEN** a file with `.ini` extension is passed to detection
- **THEN** candidate types with `.ini` extension are returned

### Requirement: Content sniffing for ambiguous extensions
For file extensions shared by multiple types (e.g., `.scr` used by both quest and message), the registry SHALL use content sniffing to determine the correct type.

#### Scenario: Sniff quest .scr file
- **WHEN** a `.scr` file containing quest-format data is detected
- **THEN** the registry identifies it as the `quest` type

#### Scenario: Sniff message .scr file
- **WHEN** a `.scr` file containing message-format data is detected
- **THEN** the registry identifies it as the `message` type

### Requirement: Detection function signature
Each file type SHALL have a `detect_fn` with signature `fn(&Path) -> bool` that returns true if the given path matches that file type.

#### Scenario: Detection function for weapons
- **WHEN** the weapons detection function is called with `weaponItem.db`
- **THEN** it returns true

#### Scenario: Detection function mismatch
- **WHEN** the weapons detection function is called with `map.ini`
- **THEN** it returns false

### Requirement: Extract function bridge
Each file type SHALL have an `extract_fn` with signature `fn(&Path) -> Result<serde_json::Value>` that reads the game file and returns a JSON array of records.

#### Scenario: Extract function returns JSON array
- **WHEN** the weapons extract function is called with a valid path
- **THEN** it returns `Ok(Value::Array(...))` with weapon records

### Requirement: Patch function bridge
Each file type SHALL have a `patch_fn` with signature `fn(&serde_json::Value, &Path) -> Result<()>` that writes JSON data to a game file.

#### Scenario: Patch function writes file
- **WHEN** the weapons patch function is called with a JSON array and output path
- **THEN** the game file is written successfully

### Requirement: Registry lookup error messages
When a file type cannot be determined, the registry SHALL provide helpful error messages listing the closest matching types based on extension similarity.

#### Scenario: Unknown extension error
- **WHEN** detection is called with `file.xyz`
- **THEN** the error lists known extensions and suggests using `--type`
