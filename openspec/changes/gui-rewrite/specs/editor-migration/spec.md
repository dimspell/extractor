## ADDED Requirements

### Requirement: EditableRecord trait implementation for all types
All 26 hand-written editor types SHALL implement the `EditableRecord` trait from `dispel-core`.

#### Scenario: Monster implements EditableRecord
- **WHEN** `Monster::field_descriptors()` is called
- **THEN** it returns descriptors for all 35 monster fields

#### Scenario: HealItem implements EditableRecord
- **WHEN** `HealItem::get_field("name")` is called on a record
- **THEN** it returns the record's name string

### Requirement: GenericEditorState type alias for each editor
Each editor's state file SHALL be a single type alias: `pub type XEditorState = GenericEditorState<X>;`

#### Scenario: WeaponEditor state
- **WHEN** `WeaponEditorState` is instantiated
- **THEN** it is equivalent to `GenericEditorState<WeaponItem>`

### Requirement: Generic view function for each editor
Each editor's view file SHALL call `build_editor_view()` or `build_multi_file_editor_view()` with appropriate parameters.

#### Scenario: MonsterRefEditor view
- **WHEN** `view_monster_ref_editor_tab()` is called
- **THEN** it returns the result of `build_multi_file_editor_view()` with monster ref parameters

### Requirement: Old hand-written editor files deleted
After migration, the original hand-written editor state and view files SHALL be removed.

#### Scenario: Cleanup after migration
- **WHEN** MonsterEditor is migrated to generic infrastructure
- **THEN** `monster_editor.rs` (346 lines) and `view/monster_editor.rs` (226 lines) are deleted
- **AND** replaced by 4-line state alias and 15-line view function

### Requirement: Lookup dropdowns for cross-referenced fields
Fields that reference other game data (e.g., `mon_id` → monster name) SHALL render as dropdowns populated from lookup data.

#### Scenario: Monster dropdown in MonsterRef
- **WHEN** editing `mon_id` in a MonsterRef record
- **THEN** a dropdown shows monster names loaded from `Monster.ini`

### Requirement: Migration order (easiest to hardest)
Editors SHALL be migrated in the specified order: HealItem → MiscItem → EditItem → EventItem → MagicSpell → PartyRef → PartyIni → NpcIni → Dialog → DialogueText → DrawItem → Event → EventNpcRef → Extra → ExtraRef → MapIni → Message → NpcRef → PartyLevelDb → Quest → WaveIni → ChData → Store → Map → Monster.

#### Scenario: First migration
- **WHEN** HealItem is migrated
- **THEN** it serves as the template for subsequent migrations
