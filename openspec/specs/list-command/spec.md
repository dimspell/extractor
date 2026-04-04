## ADDED Requirements

### Requirement: List command shows all supported file types
The CLI SHALL provide a `list` command that displays all 29 supported file types with their key, name, description, and file extensions.

#### Scenario: Default text output
- **WHEN** user runs `dispel-extractor list`
- **THEN** stdout displays a formatted table of all file types

### Requirement: List command with JSON format
The `--format json` flag SHALL produce machine-readable JSON output containing an array of type objects with `name`, `description`, `extensions`, `typical_paths`, `record_type`, and `fields` properties.

#### Scenario: JSON output
- **WHEN** user runs `dispel-extractor list --format json`
- **THEN** stdout contains a JSON object with a `types` array

### Requirement: List command with filter
The `--filter` flag SHALL restrict output to file types whose key, name, or description contains the filter string (case-insensitive).

#### Scenario: Filter by type name
- **WHEN** user runs `dispel-extractor list --filter "monster"`
- **THEN** output includes only types matching "monster" (monsters, monster_ref, monster_ini)

#### Scenario: Filter with no matches
- **WHEN** user runs `dispel-extractor list --filter "xyz"`
- **THEN** output shows an empty list or "no matching types" message

### Requirement: List JSON output structure for AI consumption
The JSON output SHALL follow a consistent structure suitable for AI tool context:

```json
{
  "types": [
    {
      "name": "weapons",
      "description": "Weapons & armor database",
      "extensions": [".db"],
      "typical_paths": ["CharacterInGame/weaponItem.db"],
      "record_type": "WeaponItem",
      "fields": ["id", "name", "description", "base_price"]
    }
  ]
}
```

#### Scenario: AI tool consumes list output
- **WHEN** an AI tool parses `dispel-extractor list --format json`
- **THEN** it can extract type names, extensions, and field lists for context
