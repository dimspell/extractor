## ADDED Requirements

### Requirement: Global search activation
Pressing Ctrl+Shift+F SHALL open the global search panel.

#### Scenario: Open global search
- **WHEN** user presses Ctrl+Shift+F
- **THEN** a search panel appears with a text input

### Requirement: Cross-catalog search
The global search SHALL search across all loaded file catalogs (all open editors).

#### Scenario: Search across files
- **WHEN** user searches for "Short Sword"
- **THEN** results show matches from weaponItem.db, STORE.DB, and any other loaded catalogs

### Requirement: Result navigation
Clicking a search result SHALL navigate to the corresponding file and highlight the matching record.

#### Scenario: Navigate to result
- **WHEN** user clicks a search result for "STORE.DB — Record #12"
- **THEN** the STORE.DB tab is opened/focused and record #12 is highlighted
