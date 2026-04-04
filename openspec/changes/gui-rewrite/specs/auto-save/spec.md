## ADDED Requirements

### Requirement: Auto-save drafts
Unsaved changes SHALL be persisted to a temporary draft file periodically (default: every 30 seconds).

#### Scenario: Auto-save triggers
- **WHEN** 30 seconds pass with unsaved changes
- **THEN** changes are written to a draft file in the temp directory

### Requirement: Draft restoration on restart
On application startup, if draft files exist, the user SHALL be prompted to restore them.

#### Scenario: Restore after crash
- **WHEN** the application restarts after a crash with pending drafts
- **THEN** a dialog offers to restore unsaved changes

### Requirement: External file conflict detection
If a file has been modified externally since it was last loaded, the user SHALL be warned before saving.

#### Scenario: External modification detected
- **WHEN** user saves a file that was modified externally
- **THEN** a conflict dialog shows the differences and offers: overwrite, keep external, or merge
