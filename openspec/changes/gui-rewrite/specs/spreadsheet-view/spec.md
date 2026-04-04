## ADDED Requirements

### Requirement: Spreadsheet view for record lists
Every editor SHALL support a spreadsheet (table) view showing all records with sortable columns.

#### Scenario: Sort by column
- **WHEN** user clicks a column header
- **THEN** records are sorted by that column (ascending, then descending on second click)

#### Scenario: Inline cell editing
- **WHEN** user double-clicks a cell in spreadsheet mode
- **THEN** the cell becomes an editable text input
- **AND** pressing Enter commits the change

### Requirement: Filter bar with free-text search
A filter bar above the spreadsheet SHALL support free-text search across all visible fields.

#### Scenario: Filter by text
- **WHEN** user types "Sword" in the filter bar
- **THEN** only records containing "Sword" in any field are shown

### Requirement: Multi-select for batch operations
Users SHALL be able to select multiple records via Ctrl+click or Shift+click for batch operations.

#### Scenario: Select multiple records
- **WHEN** user Ctrl+clicks multiple rows
- **THEN** all clicked rows are highlighted as selected

### Requirement: Inspector panel for single-record editing
The editor SHALL support an inspector panel showing all fields of the selected record with labeled inputs.

#### Scenario: Switch to inspector view
- **WHEN** user toggles from spreadsheet to inspector mode
- **THEN** a single-record detail panel is shown with all fields
