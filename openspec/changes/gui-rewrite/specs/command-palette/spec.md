## ADDED Requirements

### Requirement: Command palette activation
Pressing Ctrl+P (or Cmd+P on macOS) SHALL open the command palette overlay.

#### Scenario: Open command palette
- **WHEN** user presses Ctrl+P
- **THEN** a search overlay appears with a text input and command list

### Requirement: Fuzzy command search
The command palette SHALL filter commands by fuzzy matching against the user's query.

#### Scenario: Fuzzy match
- **WHEN** user types "ext weap"
- **THEN** "Extract: weaponItem.db" appears in results

### Requirement: Command execution
Pressing Enter on a highlighted command SHALL execute the associated action.

#### Scenario: Execute extract command
- **WHEN** user selects "Extract: weaponItem.db" and presses Enter
- **THEN** the file is extracted to JSON

### Requirement: Recent actions
The command palette SHALL show recently executed actions at the top of the list.

#### Scenario: Recent action appears
- **WHEN** user just saved a file
- **THEN** "Save: weaponItem.db" appears at the top of the command list
