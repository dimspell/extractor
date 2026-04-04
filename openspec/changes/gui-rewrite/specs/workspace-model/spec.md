## ADDED Requirements

### Requirement: Dynamic tab creation from file paths
The workspace SHALL support opening files as tabs dynamically, without a predefined tab enumeration.

#### Scenario: Open file creates tab
- **WHEN** a file is opened from the file explorer
- **THEN** a new tab appears in the tab bar with the file's name

#### Scenario: Open same file twice
- **WHEN** user opens a file that is already open
- **THEN** the existing tab is focused (no duplicate tab created)

### Requirement: Tab close, reorder, and pin
Each tab SHALL be closable via an ✕ button, reorderable via drag, and pinnable to prevent accidental closure.

#### Scenario: Close tab
- **WHEN** user clicks the ✕ button on a tab
- **THEN** the tab is removed and the next tab is focused

#### Scenario: Close last tab
- **WHEN** user closes the last open tab
- **THEN** the workspace shows an empty state with file explorer

### Requirement: Modified file indicator
Tabs for files with unsaved changes SHALL display a visual indicator (dot or asterisk).

#### Scenario: Edit creates dirty indicator
- **WHEN** user modifies a field in an editor
- **THEN** the tab shows a modified indicator

#### Scenario: Save clears indicator
- **WHEN** user saves the file
- **THEN** the modified indicator is removed

### Requirement: Workspace state persistence
The workspace SHALL save and restore the list of open files on application restart.

#### Scenario: Restore workspace on restart
- **WHEN** user reopens the application
- **THEN** previously open files are restored as tabs
