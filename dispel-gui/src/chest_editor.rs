use dispel_core::{
    EditItem, EventItem, ExtraRef, Extractor, HealItem, ItemTypeId, MiscItem, WeaponItem,
};
use std::path::{Path, PathBuf};

/// A consolidated record of all items in the game databases.
#[derive(Debug, Clone, Default)]
pub struct ItemCatalog {
    pub weapons: Vec<WeaponItem>,
    pub healing: Vec<HealItem>,
    pub misc: Vec<MiscItem>,
    pub event: Vec<EventItem>,
    pub edit: Vec<EditItem>,
}

impl ItemCatalog {
    pub fn load_from_folder(game_path: &Path) -> Result<Self, String> {
        let char_path = game_path.join("CharacterInGame");

        // Case-insensitive loader helper for macOS compatibility
        let load_db = |file_name: &str| -> Result<PathBuf, String> {
            let exact_p = char_path.join(file_name);
            if exact_p.exists() {
                return Ok(exact_p);
            }

            // Re-read directory and search case-insensitive
            if let Ok(entries) = std::fs::read_dir(&char_path) {
                let target = file_name.to_lowercase();
                for entry in entries.filter_map(Result::ok) {
                    if let Some(name) = entry.file_name().to_str() {
                        if name.to_lowercase() == target {
                            return Ok(entry.path());
                        }
                    }
                }
            }
            Err(format!(
                "Missing file ignoring case: {} in {}",
                file_name,
                char_path.display()
            ))
        };

        Ok(ItemCatalog {
            weapons: WeaponItem::read_file(&load_db("weaponItem.db")?)
                .map_err(|e| e.to_string())?,
            healing: HealItem::read_file(&load_db("HealItem.db")?).map_err(|e| e.to_string())?,
            misc: MiscItem::read_file(&load_db("MiscItem.db")?).map_err(|e| e.to_string())?,
            event: EventItem::read_file(&load_db("EventItem.db")?).map_err(|e| e.to_string())?,
            edit: EditItem::read_file(&load_db("EditItem.db")?).map_err(|e| e.to_string())?,
        })
    }

    /// Retrieve an item name by combining item_type_id and item_id.
    pub fn get_item_name(&self, type_id: ItemTypeId, id: u8) -> Option<String> {
        match type_id {
            ItemTypeId::Weapon => self.weapons.get(id as usize).map(|i| i.name.clone()),
            ItemTypeId::Healing => self.healing.get(id as usize).map(|i| i.name.clone()),
            ItemTypeId::Misc => self.misc.get(id as usize).map(|i| i.name.clone()),
            ItemTypeId::Edit => self.edit.get(id as usize).map(|i| i.name.clone()),
            ItemTypeId::Event => self.event.get(id as usize).map(|i| i.name.clone()),
            ItemTypeId::Other => {
                if id == 15 {
                    Some("-".into())
                } else {
                    None
                }
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ChestEditorState {
    pub catalog: Option<ItemCatalog>,
    pub current_map_file: String,
    pub all_records: Vec<ExtraRef>,
    pub map_files: Vec<PathBuf>,
    pub filtered_chests: Vec<(usize, ExtraRef)>, // (original_index, record)
    pub selected_idx: Option<usize>,             // Index into filtered_chests

    // String buffers for text inputs (iced lifetime requirement)
    pub edit_name: String,
    pub edit_x: String,
    pub edit_y: String,
    pub edit_gold: String,
    pub edit_item_count: String,
    pub edit_item_id: String,
    pub edit_item_type: String,
    pub edit_closed: String,

    pub status_msg: String,
    pub is_loading: bool,
}
