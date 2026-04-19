use crate::components::editor::editable::EditableRecord;
use dispel_core::Extractor;
use dispel_core::{
    ChData, DrawItem, EditItem, Event, EventItem, EventNpcRef, Extra, HealItem, MagicSpell, Map,
    MapIni, Message as ScrMessage, MiscItem, Monster, NpcIni, PartyIniNpc, PartyLevelNpc, PartyRef,
    Quest, Store, WaveIni, WeaponItem,
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// A single indexed entry from a game file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedEntry {
    /// Display label shown in search results.
    pub label: String,
    /// The editor type this entry belongs to (e.g. "WeaponEditor").
    pub editor_type: String,
    /// The record index within the editor's catalog.
    pub record_idx: usize,
    /// Optional source file path (for file-based navigation).
    pub source_file: Option<String>,
}

/// Maps a file path to the editor type that handles it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMapping {
    pub file_path: String,
    pub editor_type: String,
}

/// The persistent search index.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SearchIndex {
    /// All indexed entries.
    pub entries: Vec<IndexedEntry>,
    /// File-to-editor mappings.
    pub file_mappings: Vec<FileMapping>,
    /// The game path this index was built for.
    pub game_path: Option<String>,
    /// Whether indexing is currently in progress.
    pub indexing: bool,
    /// Progress of indexing (0.0 to 1.0).
    pub progress: f32,
}

impl SearchIndex {
    pub fn new() -> Self {
        Self::default()
    }

    /// Search the index for entries matching the query.
    pub fn search(&self, query: &str) -> Vec<&IndexedEntry> {
        if query.is_empty() {
            return Vec::new();
        }
        let query_lower = query.to_lowercase();
        self.entries
            .iter()
            .filter(|e| e.label.to_lowercase().contains(&query_lower))
            .collect()
    }

    /// Search for file mappings matching the query.
    pub fn search_files(&self, query: &str) -> Vec<&FileMapping> {
        if query.is_empty() {
            return Vec::new();
        }
        let query_lower = query.to_lowercase();
        self.file_mappings
            .iter()
            .filter(|m| m.file_path.to_lowercase().contains(&query_lower))
            .collect()
    }

    /// Save the index to a JSON file.
    pub fn save(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let json = serde_json::to_string(self).map_err(|e| e.to_string())?;
        std::fs::write(path, json).map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Load the index from a JSON file.
    pub fn load(path: &Path) -> Result<Self, String> {
        let json = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        serde_json::from_str(&json).map_err(|e| e.to_string())
    }

    /// Get the config directory path for the index file.
    pub fn index_path() -> PathBuf {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("dispel-gui");
        path.push("search_index.json");
        path
    }

    /// Clear the index and reset state.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.file_mappings.clear();
        self.game_path = None;
        self.indexing = false;
        self.progress = 0.0;
    }
}

/// Known catalog file paths and their editor types.
/// Each entry: (relative_path, editor_type, catalog_type_name)
fn catalog_files() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        ("CharacterInGame/weaponItem.db", "Weapon", "weaponItem"),
        ("CharacterInGame/HealItem.db", "HealItem", "HealItem"),
        ("CharacterInGame/MiscItem.db", "MiscItem", "MiscItem"),
        ("CharacterInGame/EditItem.db", "EditItem", "EditItem"),
        ("CharacterInGame/EventItem.db", "EventItem", "EventItem"),
        ("MonsterInGame/Monster.db", "Monster", "Monster"),
        ("Npc.ini", "NpcIni", "NpcIni"),
        ("MagicInGame/Magic.db", "MagicSpell", "MagicSpell"),
        ("CharacterInGame/STORE.DB", "Store", "Store"),
        ("Ref/PartyRef.ref", "PartyRef", "PartyRef"),
        ("NpcInGame/PrtIni.db", "PartyIni", "PartyIni"),
        ("AllMap.ini", "Map", "Map"),
        ("Ref/DRAWITEM.ref", "DrawItem", "DrawItem"),
        ("Event.ini", "Event", "Event"),
        ("NpcInGame/Eventnpc.ref", "EventNpcRef", "EventNpcRef"),
        ("Extra.ini", "Extra", "Extra"),
        ("Ref/Map.ini", "MapIni", "MapIni"),
        ("ExtraInGame/Message.scr", "Message", "Message"),
        ("NpcInGame/PrtLevel.db", "PartyLevel", "PartyLevel"),
        ("ExtraInGame/Quest.scr", "Quest", "Quest"),
        ("Wave.ini", "WaveIni", "WaveIni"),
        ("CharacterInGame/ChData.db", "ChData", "ChData"),
    ]
}

/// Build a search index from the given game path.
pub async fn build_index(game_path: &Path) -> SearchIndex {
    let mut index = SearchIndex::new();
    index.game_path = Some(game_path.to_string_lossy().to_string());

    // Build file mappings
    for (rel_path, editor_type, _catalog_type) in catalog_files() {
        index.file_mappings.push(FileMapping {
            file_path: rel_path.to_string(),
            editor_type: editor_type.to_string(),
        });
    }

    // Index each catalog file
    for (rel_path, editor_type, _catalog_type) in catalog_files() {
        let full_path = game_path.join(rel_path);
        index_entries_for_file(&full_path, editor_type, &mut index.entries);
    }

    // Index sprite files
    index_sprites(game_path, &mut index.entries, &mut index.file_mappings);

    index
}

fn index_entries_for_file(path: &Path, editor_type: &str, entries: &mut Vec<IndexedEntry>) {
    if !path.exists() {
        return;
    }

    let source = path.to_string_lossy().to_string();

    match editor_type {
        "Weapon" => index_catalog::<WeaponItem>(path, editor_type, &source, entries),
        "HealItem" => index_catalog::<HealItem>(path, editor_type, &source, entries),
        "MiscItem" => index_catalog::<MiscItem>(path, editor_type, &source, entries),
        "EditItem" => index_catalog::<EditItem>(path, editor_type, &source, entries),
        "EventItem" => index_catalog::<EventItem>(path, editor_type, &source, entries),
        "Monster" => index_monsters(path, &source, entries),
        "NpcIni" => index_npc_ini(path, &source, entries),
        "MagicSpell" => index_catalog::<MagicSpell>(path, editor_type, &source, entries),
        "Store" => index_catalog::<Store>(path, editor_type, &source, entries),
        "PartyRef" => index_catalog::<PartyRef>(path, editor_type, &source, entries),
        "PartyIni" => index_party_ini(path, &source, entries),
        "Map" => index_all_map(path, &source, entries),
        "DrawItem" => index_catalog::<DrawItem>(path, editor_type, &source, entries),
        "Event" => index_catalog::<Event>(path, editor_type, &source, entries),
        "EventNpcRef" => index_catalog::<EventNpcRef>(path, editor_type, &source, entries),
        "Extra" => index_catalog::<Extra>(path, editor_type, &source, entries),
        "MapIni" => index_catalog::<MapIni>(path, editor_type, &source, entries),
        "Message" => index_catalog::<ScrMessage>(path, editor_type, &source, entries),
        "PartyLevel" => index_catalog::<PartyLevelNpc>(path, editor_type, &source, entries),
        "Quest" => index_catalog::<Quest>(path, editor_type, &source, entries),
        "WaveIni" => index_catalog::<WaveIni>(path, editor_type, &source, entries),
        "ChData" => index_catalog::<ChData>(path, editor_type, &source, entries),
        _ => {}
    }
}

fn index_catalog<R: EditableRecord + Extractor>(
    path: &Path,
    editor_type: &str,
    source: &str,
    entries: &mut Vec<IndexedEntry>,
) {
    if let Ok(records) = R::read_file(path) {
        for (idx, record) in records.iter().enumerate() {
            entries.push(IndexedEntry {
                label: record.list_label(),
                editor_type: editor_type.to_string(),
                record_idx: idx,
                source_file: Some(source.to_string()),
            });
        }
    }
}

fn index_monsters(path: &Path, source: &str, entries: &mut Vec<IndexedEntry>) {
    if let Ok(records) = Monster::read_file(path) {
        for (idx, monster) in records.iter().enumerate() {
            entries.push(IndexedEntry {
                label: format!("#{} {}", monster.id, monster.name),
                editor_type: "Monster".to_string(),
                record_idx: idx,
                source_file: Some(source.to_string()),
            });
        }
    }
}

fn index_npc_ini(path: &Path, source: &str, entries: &mut Vec<IndexedEntry>) {
    if let Ok(records) = NpcIni::read_file(path) {
        for (idx, npc) in records.iter().enumerate() {
            entries.push(IndexedEntry {
                label: format!("#{} {}", npc.id, npc.description),
                editor_type: "NpcIni".to_string(),
                record_idx: idx,
                source_file: Some(source.to_string()),
            });
        }
    }
}

fn index_party_ini(path: &Path, source: &str, entries: &mut Vec<IndexedEntry>) {
    if let Ok(records) = PartyIniNpc::read_file(path) {
        for (idx, npc) in records.iter().enumerate() {
            entries.push(IndexedEntry {
                label: format!("#{} {}", idx + 1, npc.name),
                editor_type: "PartyIni".to_string(),
                record_idx: idx,
                source_file: Some(source.to_string()),
            });
        }
    }
}

fn index_all_map(path: &Path, source: &str, entries: &mut Vec<IndexedEntry>) {
    if let Ok(records) = Map::read_file(path) {
        for (idx, map) in records.iter().enumerate() {
            entries.push(IndexedEntry {
                label: format!("#{} {}", map.id, map.map_name),
                editor_type: "Map".to_string(),
                record_idx: idx,
                source_file: Some(source.to_string()),
            });
        }
    }
}

fn index_sprites(
    game_path: &Path,
    entries: &mut Vec<IndexedEntry>,
    file_mappings: &mut Vec<FileMapping>,
) {
    let mut sprite_count = 0;
    find_sprites_recursive(game_path, entries, &mut sprite_count);
    if sprite_count > 0 {
        file_mappings.push(FileMapping {
            file_path: "*.spr".to_string(),
            editor_type: "SpriteViewer".to_string(),
        });
    }
}

fn find_sprites_recursive(dir: &Path, entries: &mut Vec<IndexedEntry>, count: &mut usize) {
    if let Ok(read_dir) = std::fs::read_dir(dir) {
        for entry in read_dir.flatten() {
            let path = entry.path();
            if path.is_dir() {
                find_sprites_recursive(&path, entries, count);
            } else if path.extension().is_some_and(|e| e == "spr") {
                if let Some(name) = path.file_stem().and_then(|n| n.to_str()) {
                    entries.push(IndexedEntry {
                        label: format!("[Sprite] {}", name),
                        editor_type: "SpriteViewer".to_string(),
                        record_idx: *count,
                        source_file: Some(path.to_string_lossy().to_string()),
                    });
                    *count += 1;
                }
            }
        }
    }
}
