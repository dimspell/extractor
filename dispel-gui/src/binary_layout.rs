//! Maps known game files to their immutable Hexedit structure overlays.

use std::path::Path;

use dispel_core::{RecordLayout, references::monster_db::Monster};
use hexedit::{BinaryLayout, FixedRecordBinaryLayout, FixedRecordField};

/// Return a layout only for an explicitly recognized game-relative path.
pub fn layout_for_path(path: &Path) -> Option<Box<dyn BinaryLayout>> {
    let normalized = path
        .to_string_lossy()
        .replace('\\', "/")
        .to_ascii_lowercase();
    if !normalized.ends_with("monsteringame/monster.db") {
        return None;
    }
    let definition = Monster::LAYOUT;
    let fields: Vec<FixedRecordField> = definition
        .fields
        .iter()
        .map(|field| FixedRecordField {
            name: field.name,
            offset: field.offset,
            size: field.size,
            ty: field.ty,
        })
        .collect();
    Some(Box::new(FixedRecordBinaryLayout::new(
        definition.type_name,
        definition.header_size,
        definition.record_size,
        fields,
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_for_path_recognizes_only_monster_db_path() {
        assert!(layout_for_path(Path::new("MonsterInGame/Monster.db")).is_some());
        assert!(layout_for_path(Path::new("Monster.db")).is_none());
        assert!(layout_for_path(Path::new("MonsterInGame/Other.db")).is_none());
    }

    #[test]
    fn test_monster_layout_resolves_second_field_in_first_record() {
        let layout = layout_for_path(Path::new("MonsterInGame/Monster.db")).unwrap();
        let field = layout.field_at(24, 160).unwrap();
        assert_eq!(field.name, "health_points_max");
        assert_eq!(field.record_index, 0);
        assert_eq!(field.range, 24..28);
    }
}
