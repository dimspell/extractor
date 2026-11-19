use std::path::Path;
use std::sync::LazyLock;

use super::entries;
use super::types::{DetectResult, FileType};

/// The static registry of all supported file types.
pub static FILE_TYPES: LazyLock<Vec<FileType>> = LazyLock::new(|| {
    vec![
        // INI types
        entries::make_all_map_ini(),
        entries::make_map_ini(),
        entries::make_extra_ini(),
        entries::make_event_ini(),
        entries::make_monster_ini(),
        entries::make_npc_ini(),
        entries::make_wave_ini(),
        // DB types
        entries::make_weapons(),
        entries::make_monsters(),
        entries::make_magic(),
        entries::make_store(),
        entries::make_misc_item(),
        entries::make_heal_item(),
        entries::make_event_item(),
        entries::make_edit_item(),
        entries::make_party_level(),
        entries::make_party_ini(),
        entries::make_chdata(),
        // REF types
        entries::make_party_ref(),
        entries::make_draw_item(),
        entries::make_npc_ref(),
        entries::make_monster_ref(),
        entries::make_extra_ref(),
        entries::make_event_npc_ref(),
        // DLG/PGP types
        entries::make_dialog(),
        entries::make_dialog_text(),
        // SCR types
        entries::make_quest(),
        entries::make_message(),
        entries::make_event_scr(),
        // Map types (extract only, no patch)
        entries::make_map_file(),
        entries::make_gtl(),
        entries::make_btl(),
        // Sprite files (extract only, no patch)
        entries::make_sprite(),
        // Multiplayer file types
        entries::make_chest_spawn(),
        entries::make_monster_spawn(),
        entries::make_map_info_multi(),
        entries::make_tile_grid(),
        // Save files
        entries::make_save_file(),
        entries::make_save_ifo(),
    ]
});

/// Look up a file type by its key.
pub fn get_by_key(key: &str) -> Option<&'static FileType> {
    FILE_TYPES.iter().find(|ft| ft.key == key)
}

/// Detect the file type for a given path.
pub fn detect(path: &Path) -> DetectResult {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase());

    let ext = match ext {
        Some(e) => e,
        None => return DetectResult::None,
    };

    let candidates: Vec<&FileType> = FILE_TYPES
        .iter()
        .filter(|ft| {
            ft.extensions.iter().any(|&ext_match| {
                let normalized = ext_match.trim_start_matches('.');
                ext == normalized
            })
        })
        .collect();

    if candidates.is_empty() {
        return DetectResult::None;
    }

    if candidates.len() == 1 {
        return DetectResult::Single(candidates[0]);
    }

    // Try each candidate's detection function; first match wins
    for ft in &candidates {
        if ft.matches(path) {
            return DetectResult::Single(ft);
        }
    }

    // Multiple candidates, none matched by filename — caller must resolve via --type or _meta hint
    DetectResult::Ambiguous(candidates)
}

/// Suggest type keys similar to the given input (for helpful error messages).
pub fn suggest_similar_keys(input: &str) -> Vec<&'static str> {
    let input_lower = input.to_lowercase();
    FILE_TYPES
        .iter()
        .filter(|ft| {
            let key = ft.key;
            let name = ft.name.to_lowercase();
            key.contains(&input_lower)
                || input_lower.contains(key)
                || name.contains(&input_lower)
                || input_lower.contains(&name)
        })
        .map(|ft| ft.key)
        .collect()
}

/// Get a human-friendly list of all file types for error messages.
pub fn format_type_list() -> String {
    FILE_TYPES
        .iter()
        .map(|ft| {
            format!(
                "  {} ({}) — {}",
                ft.key,
                ft.extensions.join(", "),
                ft.description
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RecordLayout;
    use crate::commands::registry::types::{DetectKind, FieldType, fields_from_default};
    use crate::references::{
        all_map_ini::Map, chdata_db::ChData, edit_item_db::EditItem, event_item_db::EventItem,
        extra_ref::ExtraRef, heal_item_db::HealItem, magic_db::MagicSpell, misc_item_db::MiscItem,
        monster_db::Monster, monster_ref::MonsterRef, npc_ref::NPC, party_ini_db::PartyIniNpc,
        store_db::Store, weapons_db::WeaponItem,
    };

    #[test]
    fn test_detect_sample_path_returns_single_matching_key() {
        for ft in FILE_TYPES.iter() {
            let path = ft.sample_path();
            match detect(&path) {
                DetectResult::Single(detected) => assert_eq!(
                    detected.key,
                    ft.key,
                    "sample path {} detected as {}, expected {}",
                    path.display(),
                    detected.key,
                    ft.key
                ),
                DetectResult::Ambiguous(candidates) => panic!(
                    "sample path {} for '{}' is ambiguous: {:?}",
                    path.display(),
                    ft.key,
                    candidates.iter().map(|c| c.key).collect::<Vec<_>>()
                ),
                DetectResult::None => panic!(
                    "sample path {} for '{}' detected nothing",
                    path.display(),
                    ft.key
                ),
            }
        }
    }

    #[test]
    fn test_registry_keys_unique_and_fully_described() {
        let mut keys = std::collections::HashSet::new();
        for ft in FILE_TYPES.iter() {
            assert!(keys.insert(ft.key), "duplicate key '{}'", ft.key);
            assert!(!ft.editor.is_empty(), "empty editor for '{}'", ft.key);
            assert!(
                !ft.record_type.is_empty(),
                "empty record_type for '{}'",
                ft.key
            );
        }
    }

    #[test]
    fn test_extension_only_types_do_not_abuse_empty_db_kind() {
        for ft in FILE_TYPES.iter() {
            if let DetectKind::Db(names) = ft.detect_kind {
                assert!(!names.is_empty(), "empty Db() detection for '{}'", ft.key);
            }
        }
    }

    macro_rules! assert_layout_fields {
        ($key:expr, $ty:ty) => {{
            let ft = get_by_key($key).unwrap_or_else(|| panic!("missing key '{}'", $key));
            let names: Vec<&str> = ft.field_names().collect();
            for def in <$ty as RecordLayout>::LAYOUT.fields {
                assert!(
                    names.contains(&def.name),
                    "'{}': RecordLayout field '{}' missing from catalog",
                    $key,
                    def.name
                );
                let expected = FieldType::from_layout_type(def.ty);
                let actual = ft
                    .fields
                    .iter()
                    .find(|f| f.name == def.name)
                    .map(|f| f.ty)
                    .unwrap();
                assert_eq!(
                    actual, expected,
                    "'{}': wrong JSON type for layout field '{}'",
                    $key, def.name
                );
            }
        }};
    }

    #[test]
    fn test_catalog_fields_match_record_layout() {
        assert_layout_fields!("weapons", WeaponItem);
        assert_layout_fields!("monsters", Monster);
        assert_layout_fields!("magic", MagicSpell);
        assert_layout_fields!("misc_item", MiscItem);
        assert_layout_fields!("heal_item", HealItem);
        assert_layout_fields!("event_item", EventItem);
        assert_layout_fields!("edit_item", EditItem);
        assert_layout_fields!("chdata", ChData);
        assert_layout_fields!("party_ini", PartyIniNpc);
        assert_layout_fields!("npc_ref", NPC);
        assert_layout_fields!("monster_ref", MonsterRef);
        assert_layout_fields!("extra_ref", ExtraRef);
    }

    macro_rules! assert_serde_fields {
        ($key:expr, $ty:ty) => {{
            let ft = get_by_key($key).unwrap_or_else(|| panic!("missing key '{}'", $key));
            let expected: Vec<String> = fields_from_default::<$ty>()
                .into_iter()
                .map(|f| f.name)
                .collect();
            let actual: Vec<String> = ft.field_names().map(str::to_string).collect();
            assert_eq!(
                actual, expected,
                "'{}': catalog fields differ from serde",
                $key
            );
        }};
    }

    #[test]
    fn test_catalog_fields_match_serde_records() {
        assert_serde_fields!("all_maps", Map);
        assert_serde_fields!("store", Store);
        assert_serde_fields!("weapons", WeaponItem);
        assert_serde_fields!("save_ifo", crate::references::save_ifo::SaveIfo);
    }

    #[test]
    fn test_unknown_db_filename_is_ambiguous() {
        assert!(matches!(
            detect(Path::new("SomeFile.db")),
            DetectResult::Ambiguous(_)
        ));
    }
}
