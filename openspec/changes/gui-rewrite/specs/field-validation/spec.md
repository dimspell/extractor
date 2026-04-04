## ADDED Requirements

### Requirement: Field validation on input
Every field input SHALL validate its value on every keystroke and display visual feedback for invalid values.

#### Scenario: Valid integer input
- **WHEN** user types "100" in an integer field
- **THEN** the input shows a valid border (green)

#### Scenario: Invalid integer input
- **WHEN** user types "abc" in an integer field
- **THEN** the input shows an invalid border (red) with an error tooltip

### Requirement: Pre-save validation
Before saving, the editor SHALL validate all records and display a summary of errors if any exist.

#### Scenario: Save with validation errors
- **WHEN** user clicks Save with invalid fields
- **THEN** a validation summary dialog appears listing all errors
- **AND** the save is aborted

#### Scenario: Save with valid data
- **WHEN** user clicks Save with all fields valid
- **THEN** the file is saved normally

### Requirement: Validation in EditableRecord trait
The `EditableRecord` trait SHALL include a `validate_field` method that returns `Result<(), String>` for each field.

#### Scenario: Validate string length
- **WHEN** `validate_field("name", "a".repeat(31))` is called
- **THEN** it returns `Err("Name too long (max 30 chars)")`
