use std::collections::HashMap;
use std::path::{Path, PathBuf};

use dispel_core::{EditItem, EventItem, Extractor, HealItem, ItemTypeId, MiscItem, WeaponItem};

/// Ensure the `"items"` lookup is populated. Idempotent — skips if already
/// loaded or if the game path is empty. Call this before any
/// `compute_all_caches` that may contain `CompositeItem` fields.
pub fn ensure_item_lookups(game_path: &str, lookups: &mut HashMap<String, Vec<(String, String)>>) {
    if !game_path.is_empty() && !lookups.contains_key("items") {
        let path = PathBuf::from(game_path);
        populate_item_lookups(&path, lookups);
    }
}

/// Populate `lookups["items"]` with all items from all DB files.
///
/// Each entry: `(composite_key, display_name)` where
/// `composite_key = "{type_byte}:{item_id}"` (e.g. `"1:5"` for Weapon ID 5)
/// and `display_name = "[Type] ItemName"`.
///
/// Also adds `"255:15"` → `"[-]"` for the "unset" sentinel value.
///
/// Missing DB files are silently skipped — each one is loaded independently.
pub fn populate_item_lookups(
    game_path: &Path,
    lookups: &mut HashMap<String, Vec<(String, String)>>,
) {
    let char_path = game_path.join("CharacterInGame");

    let find_db = |file_name: &str| -> Option<std::path::PathBuf> {
        let exact = char_path.join(file_name);
        if exact.exists() {
            return Some(exact);
        }
        // macOS case-insensitive fallback (skipped on case-sensitive systems)
        if cfg!(target_os = "macos") {
            if let Ok(entries) = std::fs::read_dir(&char_path) {
                let target = file_name.to_lowercase();
                for entry in entries.filter_map(Result::ok) {
                    if let Some(name) = entry.file_name().to_str() {
                        if name.to_lowercase() == target {
                            return Some(entry.path());
                        }
                    }
                }
            }
        }
        None
    };

    let mut entries: Vec<(String, String)> = Vec::new();

    // Each DB is loaded independently — a missing/corrupt file only skips
    // that item type instead of failing the entire catalog.

    // Weapon (type 1)
    if let Some(path) = find_db("weaponItem.db") {
        if let Ok(items) = WeaponItem::read_file(&path) {
            for item in items.iter() {
                entries.push((
                    format!("{}:{}", ItemTypeId::Weapon.value(), item.id),
                    format!("[Weapon] {}", item.name),
                ));
            }
        }
    }

    // Healing (type 2)
    if let Some(path) = find_db("HealItem.db") {
        if let Ok(items) = HealItem::read_file(&path) {
            for item in items.iter() {
                entries.push((
                    format!("{}:{}", ItemTypeId::Healing.value(), item.id),
                    format!("[Healing] {}", item.name),
                ));
            }
        }
    }

    // Edit (type 3)
    if let Some(path) = find_db("EditItem.db") {
        if let Ok(items) = EditItem::read_file(&path) {
            for item in items.iter() {
                entries.push((
                    format!("{}:{}", ItemTypeId::Edit.value(), item.index),
                    format!("[Edit] {}", item.name),
                ));
            }
        }
    }

    // Event (type 4)
    if let Some(path) = find_db("EventItem.db") {
        if let Ok(items) = EventItem::read_file(&path) {
            for item in items.iter() {
                entries.push((
                    format!("{}:{}", ItemTypeId::Event.value(), item.id),
                    format!("[Event] {}", item.name),
                ));
            }
        }
    }

    // Misc (type 5)
    if let Some(path) = find_db("MiscItem.db") {
        if let Ok(items) = MiscItem::read_file(&path) {
            for item in items.iter() {
                entries.push((
                    format!("{}:{}", ItemTypeId::Misc.value(), item.id),
                    format!("[Misc] {}", item.name),
                ));
            }
        }
    }

    // "Other" sentinel — 255:15 means "no item"
    entries.push((
        format!("{}:{}", ItemTypeId::Other.value(), 15u8),
        "[-]".to_string(),
    ));

    // Sort by type then by id for stable display
    entries.sort_by(|a, b| {
        let a_parts: Vec<&str> = a.0.split(':').collect();
        let b_parts: Vec<&str> = b.0.split(':').collect();
        let a_type = a_parts
            .first()
            .and_then(|s| s.parse::<u8>().ok())
            .unwrap_or(0);
        let b_type = b_parts
            .first()
            .and_then(|s| s.parse::<u8>().ok())
            .unwrap_or(0);
        let a_id = a_parts
            .get(1)
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(0);
        let b_id = b_parts
            .get(1)
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(0);
        a_type.cmp(&b_type).then(a_id.cmp(&b_id))
    });

    lookups.insert("items".to_string(), entries);
}
