## ADDED Requirements

### Requirement: File tree displays game directory structure
The file explorer SHALL display a recursive tree view of the configured game directory, with expandable/collapsible directory nodes.

#### Scenario: Expand directory node
- **WHEN** user clicks a directory node
- **THEN** the directory's children are displayed
- **AND** the node icon changes from collapsed to expanded

#### Scenario: Collapse directory node
- **WHEN** user clicks an expanded directory node
- **THEN** the directory's children are hidden
- **AND** the node icon changes from expanded to collapsed

### Requirement: File type icons and status badges
Each file in the tree SHALL display an icon based on its detected type (from the file type registry) and badges indicating extractable/patchable status.

#### Scenario: Database file display
- **WHEN** a `.db` file is displayed
- **THEN** it shows a database icon with `[extract ✓] [patch ✓]` badges

#### Scenario: Map file display
- **WHEN** a `.map` file is displayed
- **THEN** it shows a map icon with `[extract ✓] [patch ✗]` badges

### Requirement: Double-click opens file in editor
Double-clicking a file in the tree SHALL open it in a new workspace tab using the appropriate editor based on file type detection.

#### Scenario: Open weapon database
- **WHEN** user double-clicks `weaponItem.db`
- **THEN** a new tab opens with the weapons editor

#### Scenario: Open monster ref file
- **WHEN** user double-clicks `Mondun01.ref`
- **THEN** a new tab opens with the multi-file monster ref editor

### Requirement: File tree search filter
A search input at the top of the file tree SHALL filter visible nodes by name match.

#### Scenario: Filter by filename
- **WHEN** user types "weapon" in the search box
- **THEN** only files and directories containing "weapon" in their name are visible

### Requirement: Context menu on right-click
Right-clicking a file SHALL show a context menu with: Open, Extract, Validate, Show in OS.

#### Scenario: Extract from context menu
- **WHEN** user right-clicks a file and selects "Extract"
- **THEN** the file is extracted to JSON using the unified extract command
