## ADDED Requirements

### Requirement: Undo last edit
Pressing Ctrl+Z SHALL undo the most recent field change.

#### Scenario: Undo single edit
- **WHEN** user changes a field value then presses Ctrl+Z
- **THEN** the field reverts to its previous value

#### Scenario: Undo multiple edits
- **WHEN** user makes 3 edits then presses Ctrl+Z three times
- **THEN** all 3 edits are undone in reverse order

### Requirement: Redo undone edit
Pressing Ctrl+Y (or Ctrl+Shift+Z) SHALL redo the most recently undone change.

#### Scenario: Redo after undo
- **WHEN** user undoes an edit then presses Ctrl+Y
- **THEN** the edit is reapplied

### Requirement: Bounded undo stack
The undo stack SHALL have a configurable maximum depth (default: 100 operations).

#### Scenario: Stack overflow
- **WHEN** user makes 101 edits (with default stack size)
- **THEN** the oldest edit is discarded from the stack

### Requirement: Edit history panel
A panel SHALL display the undo/redo history with descriptions of each operation.

#### Scenario: View history
- **WHEN** user opens the edit history panel
- **THEN** a list of recent operations is shown with timestamps
