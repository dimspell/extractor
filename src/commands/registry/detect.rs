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

    #[test]
    fn detect_uniquely_named_ini_returns_single() {
        let result = detect(Path::new("AllMap.ini"));
        assert!(matches!(result, DetectResult::Single(ft) if ft.key == "all_maps"));
    }

    #[test]
    fn detect_unknown_extension_returns_none() {
        assert!(matches!(detect(Path::new("file.xyz")), DetectResult::None));
    }

    #[test]
    fn detect_no_extension_returns_none() {
        assert!(matches!(detect(Path::new("noext")), DetectResult::None));
    }

    #[test]
    fn detect_case_insensitive_extension() {
        let result1 = detect(Path::new("AllMap.INI"));
        let result2 = detect(Path::new("AllMap.ini"));
        assert!(matches!(result1, DetectResult::Single(_)));
        assert!(matches!(result2, DetectResult::Single(_)));
    }

    #[test]
    fn detect_ambiguous_db_returns_ambiguous() {
        let result = detect(Path::new("SomeFile.db"));
        // .db maps to multiple types (weapons, monsters, magic, store, misc, heal, event, edit, party_level, party_ini, chdata)
        // without a unique filename match, should be Ambiguous
        assert!(matches!(result, DetectResult::Ambiguous(_)));
    }

    #[test]
    fn detect_ambiguous_ref_returns_ambiguous_or_single() {
        let result = detect(Path::new("Unknown.ref"));
        // .ref maps to multiple types; PartyRef.ref should be Single due to filename match
        assert!(matches!(
            result,
            DetectResult::Ambiguous(_) | DetectResult::Single(_)
        ));
    }

    #[test]
    fn detect_unique_ref_by_name_returns_single() {
        let result = detect(Path::new("PartyRef.ref"));
        assert!(matches!(result, DetectResult::Single(ft) if ft.key == "party_ref"));
    }

    #[test]
    fn get_by_key_weapons_exists() {
        let ft = get_by_key("weapons");
        assert!(ft.is_some());
        assert_eq!(ft.unwrap().key, "weapons");
    }

    #[test]
    fn get_by_key_monsters_exists() {
        let ft = get_by_key("monsters");
        assert!(ft.is_some());
        assert_eq!(ft.unwrap().key, "monsters");
    }

    #[test]
    fn get_by_key_unknown_returns_none() {
        assert!(get_by_key("does_not_exist").is_none());
    }

    #[test]
    fn suggest_similar_keys_weapon_returns_weapons() {
        let results = suggest_similar_keys("weapon");
        assert!(results.contains(&"weapons"));
    }

    #[test]
    fn suggest_similar_keys_partial_match() {
        let results = suggest_similar_keys("map");
        assert!(!results.is_empty());
        assert!(results.iter().any(|k| k.contains("map")));
    }

    #[test]
    fn suggest_similar_keys_no_match_returns_empty() {
        let results = suggest_similar_keys("zzzznotakey");
        assert!(results.is_empty());
    }

    #[test]
    fn suggest_similar_keys_case_insensitive() {
        let results1 = suggest_similar_keys("weapon");
        let results2 = suggest_similar_keys("WEAPON");
        assert_eq!(results1, results2);
    }

    #[test]
    fn get_by_key_all_file_types_have_distinct_keys() {
        let keys: Vec<_> = FILE_TYPES.iter().map(|ft| ft.key).collect();
        let unique_keys: std::collections::HashSet<_> = keys.iter().copied().collect();
        assert_eq!(
            keys.len(),
            unique_keys.len(),
            "Duplicate keys found in registry"
        );
    }

    #[test]
    fn format_type_list_contains_descriptions() {
        let list = format_type_list();
        assert!(!list.is_empty());
        assert!(list.contains("—"));
        assert!(list.contains("\n"));
    }
}
