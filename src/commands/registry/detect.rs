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

    // Fall back to the first candidate in registry order
    DetectResult::Single(candidates[0])
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
