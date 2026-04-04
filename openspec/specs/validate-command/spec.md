## ADDED Requirements

### Requirement: Validate command with input and type flags
The CLI SHALL provide a `validate` command that accepts `--input` for the JSON file path and `--type` for the file type. Both flags are required.

#### Scenario: Validate with required flags
- **WHEN** user runs `dispel-extractor validate --input data.json --type weapons`
- **THEN** the command validates the JSON against the weapons format

#### Scenario: Missing required flags fails
- **WHEN** user runs `dispel-extractor validate` without `--input` or `--type`
- **THEN** the command exits with an error and shows usage information

### Requirement: Validate deserializes JSON into target record type
The validate command SHALL attempt to deserialize the JSON input into the target file type's record struct using serde. Deserialization errors SHALL be reported with field-level context.

#### Scenario: Valid JSON passes validation
- **WHEN** user validates correctly structured JSON for the specified type
- **THEN** the command prints a success message and exits with code 0

#### Scenario: Invalid field type fails validation
- **WHEN** user validates JSON with a string where an integer is expected
- **THEN** the command reports the field name, expected type, and actual value

### Requirement: Validate verbose mode
The `--verbose` flag SHALL produce detailed validation output including record indices and raw values for each error.

#### Scenario: Verbose error output
- **WHEN** user runs `dispel-extractor validate --input bad.json --type weapons --verbose`
- **THEN** each error includes the record index, field name, expected type, and actual value

### Requirement: Validate outputs structured JSON errors
When validation fails, the command SHALL output a structured JSON error object to stderr containing a `valid` boolean and an `errors` array with per-field details.

#### Scenario: Structured error output
- **WHEN** user validates invalid JSON
- **THEN** stderr contains `{"valid": false, "errors": [{"record_index": 5, "field": "attack", "expected": "i16", "got": "string", "value": "high"}]}`

### Requirement: Validate supports all registered file types
The validate command SHALL work with all 29 file types registered in the file type registry.

#### Scenario: Validate monster type
- **WHEN** user runs `dispel-extractor validate --input monsters.json --type monsters`
- **THEN** the command validates against the monster record format

#### Scenario: Validate unknown type fails
- **WHEN** user runs `dispel-extractor validate --input data.json --type nonexistent`
- **THEN** the command exits with an error listing available types
