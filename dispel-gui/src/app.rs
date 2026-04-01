use crate::chest_editor;
use crate::db;
use crate::db_viewer_state::{DbViewerState, PAGE_SIZE};
use crate::edit_item_editor;
use crate::event_item_editor;
use crate::heal_item_editor;
use crate::magic_editor;
use crate::message::Message;
use crate::misc_item_editor;
use crate::monster_editor;
use crate::npc_ini_editor;
use crate::party_ini_editor;
use crate::party_ref_editor;
use crate::sprite_browser;
use crate::store_editor;
use crate::types::{DbOp, MapOp, RefOp, SpriteMode, Tab};
use crate::utils::{browse_file, browse_folder};
use crate::weapon_editor;
use dispel_core::commands::{self, Command, CommandFactory};
use dispel_core::{
    EditItem, EventItem, Extractor, HealItem, MagicSpell, MiscItem, Monster, NpcIni, PartyIniNpc,
    PartyRef, Store, WeaponItem,
};
use iced::{Element, Task};
use std::io::Seek;
use std::path::{Path, PathBuf};

pub struct App {
    pub active_tab: Tab,
    // Shared Game Path (set once, used by all editor tabs)
    pub shared_game_path: String,
    // Map fields
    pub map_op: Option<MapOp>,
    pub map_input: String,
    pub map_output: String,
    pub map_map_path: String,
    pub map_btl_path: String,
    pub map_gtl_path: String,
    pub map_save_sprites: bool,
    pub map_database: String,
    pub map_map_id: String,
    pub map_gtl_atlas: String,
    pub map_btl_atlas: String,
    pub map_atlas_columns: String,
    pub map_game_path: String,
    // Ref fields
    pub ref_op: Option<RefOp>,
    pub ref_input: String,
    // Database fields
    pub db_op: Option<DbOp>,
    // Sprite fields
    pub sprite_input: String,
    pub sprite_mode: Option<SpriteMode>,
    // Sound fields
    pub sound_input: String,
    pub sound_output: String,
    // Global
    pub extractor_path: String,
    pub log: String,
    pub is_running: bool,
    // DB Viewer
    pub viewer: Box<DbViewerState>,
    // Chest Editor
    pub chest_editor: Box<chest_editor::ChestEditorState>,
    // Weapon Editor
    pub weapon_editor: Box<weapon_editor::WeaponEditorState>,
    // Heal Item Editor
    pub heal_item_editor: Box<heal_item_editor::HealItemEditorState>,
    // Misc Item Editor
    pub misc_item_editor: Box<misc_item_editor::MiscItemEditorState>,
    // Edit Item Editor
    pub edit_item_editor: Box<edit_item_editor::EditItemEditorState>,
    // Event Item Editor
    pub event_item_editor: Box<event_item_editor::EventItemEditorState>,
    // Monster Editor
    pub monster_editor: Box<monster_editor::MonsterEditorState>,
    // NPC Ini Editor
    pub npc_ini_editor: Box<npc_ini_editor::NpcIniEditorState>,
    // Magic Editor
    pub magic_editor: Box<magic_editor::MagicEditorState>,
    // Store Editor
    pub store_editor: Box<store_editor::StoreEditorState>,
    // Party Ref Editor
    pub party_ref_editor: Box<party_ref_editor::PartyRefEditorState>,
    // Party Ini Editor
    pub party_ini_editor: Box<party_ini_editor::PartyIniEditorState>,
    // Sprite Browser
    pub sprite_browser: Box<sprite_browser::SpriteBrowserState>,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        (
            Self {
                active_tab: Tab::Map,
                shared_game_path: String::new(),
                map_op: Some(MapOp::Render),
                map_input: String::new(),
                map_output: String::from("map.png"),
                map_map_path: String::new(),
                map_btl_path: String::new(),
                map_gtl_path: String::new(),
                map_save_sprites: false,
                map_database: String::from("database.sqlite"),
                map_map_id: String::new(),
                map_gtl_atlas: String::new(),
                map_btl_atlas: String::new(),
                map_atlas_columns: String::from("48"),
                map_game_path: String::new(),
                ref_op: Some(RefOp::AllMaps),
                ref_input: String::new(),
                db_op: Some(DbOp::Import),
                sprite_input: String::new(),
                sprite_mode: Some(SpriteMode::Sprite),
                sound_input: String::new(),
                sound_output: String::new(),
                extractor_path: String::from("dispel-extractor"),
                log: String::new(),
                is_running: false,
                viewer: Box::default(),
                chest_editor: Box::default(),
                weapon_editor: Box::default(),
                heal_item_editor: Box::default(),
                misc_item_editor: Box::default(),
                edit_item_editor: Box::default(),
                event_item_editor: Box::default(),
                monster_editor: Box::default(),
                npc_ini_editor: Box::default(),
                magic_editor: Box::default(),
                store_editor: Box::default(),
                party_ref_editor: Box::default(),
                party_ini_editor: Box::default(),
                sprite_browser: Box::default(),
            },
            Task::none(),
        )
    }

    pub fn refresh_chests(&mut self) {
        let editor = &mut self.chest_editor;
        editor.filtered_chests = editor
            .all_records
            .iter()
            .enumerate()
            .filter(|(_, r)| r.object_type == dispel_core::ExtraObjectType::Chest)
            .map(|(i, r)| (i, r.clone()))
            .collect();
    }

    pub fn load_map_file(&mut self, path: PathBuf) -> Task<Message> {
        self.chest_editor.is_loading = true;
        Task::perform(
            async move { dispel_core::ExtraRef::read_file(&path) },
            |res: Result<Vec<dispel_core::ExtraRef>, std::io::Error>| {
                Message::ChestMapLoaded(res.map_err(|e| e.to_string()))
            },
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::TabSelected(tab) => {
                self.active_tab = tab;
                Task::none()
            }
            // Map
            Message::MapOpSelected(op) => {
                self.map_op = Some(op);
                Task::none()
            }
            Message::MapInputChanged(v) => {
                self.map_input = v;
                Task::none()
            }
            Message::MapOutputChanged(v) => {
                self.map_output = v;
                Task::none()
            }
            Message::MapMapPathChanged(v) => {
                self.map_map_path = v;
                Task::none()
            }
            Message::MapBtlPathChanged(v) => {
                self.map_btl_path = v;
                Task::none()
            }
            Message::MapGtlPathChanged(v) => {
                self.map_gtl_path = v;
                Task::none()
            }
            Message::MapSaveSpritesToggled(v) => {
                self.map_save_sprites = v;
                Task::none()
            }
            Message::MapDatabaseChanged(v) => {
                self.map_database = v;
                Task::none()
            }
            Message::MapMapIdChanged(v) => {
                self.map_map_id = v;
                Task::none()
            }
            Message::MapGtlAtlasChanged(v) => {
                self.map_gtl_atlas = v;
                Task::none()
            }
            Message::MapBtlAtlasChanged(v) => {
                self.map_btl_atlas = v;
                Task::none()
            }
            Message::MapAtlasColumnsChanged(v) => {
                self.map_atlas_columns = v;
                Task::none()
            }
            Message::MapGamePathChanged(v) => {
                self.map_game_path = v;
                Task::none()
            }
            // Shared Game Path
            Message::BrowseSharedGamePath => browse_folder("shared_game_path"),
            Message::LoadSharedGamePath => {
                if self.shared_game_path.is_empty() {
                    return browse_folder("shared_game_path");
                }
                Task::none()
            }
            // Browse buttons
            Message::BrowseMapInput => browse_file("map_input"),
            Message::BrowseMapMapPath => browse_file("map_map_path"),
            Message::BrowseMapBtlPath => browse_file("map_btl_path"),
            Message::BrowseMapGtlPath => browse_file("map_gtl_path"),
            Message::BrowseMapGtlAtlas => browse_file("map_gtl_atlas"),
            Message::BrowseMapBtlAtlas => browse_file("map_btl_atlas"),
            Message::BrowseMapGamePath => browse_folder("map_game_path"),
            Message::BrowseRefInput => browse_file("ref_input"),
            Message::BrowseSpriteInput => browse_file("sprite_input"),
            Message::BrowseSoundInput => browse_file("sound_input"),
            Message::BrowseSoundOutput => browse_file("sound_output"),
            Message::BrowseExtractorPath => browse_file("extractor_path"),
            Message::FileSelected { field, path } => {
                if let Some(p) = path {
                    let s = p.to_string_lossy().to_string();
                    match field.as_str() {
                        "shared_game_path" => self.shared_game_path = s.clone(),
                        "map_input" => self.map_input = s,
                        "map_map_path" => self.map_map_path = s,
                        "map_btl_path" => self.map_btl_path = s,
                        "map_gtl_path" => self.map_gtl_path = s,
                        "map_gtl_atlas" => self.map_gtl_atlas = s,
                        "map_btl_atlas" => self.map_btl_atlas = s,
                        "map_game_path" => {
                            self.map_game_path = s.clone();
                            self.shared_game_path = s;
                        }
                        "ref_input" => self.ref_input = s,
                        "sprite_input" => self.sprite_input = s,
                        "sound_input" => self.sound_input = s,
                        "sound_output" => self.sound_output = s,
                        "extractor_path" => self.extractor_path = s,
                        "viewer_db" => self.viewer.db_path = s,
                        "chest_game_path" => self.shared_game_path = s,
                        "chest_map_file" => self.chest_editor.current_map_file = s,
                        _ => {}
                    }
                }
                Task::none()
            }
            // Ref
            Message::RefOpSelected(op) => {
                self.ref_op = Some(op);
                Task::none()
            }
            Message::RefInputChanged(v) => {
                self.ref_input = v;
                Task::none()
            }
            // Database
            Message::DbOpSelected(op) => {
                self.db_op = Some(op);
                Task::none()
            }
            // Sprite
            Message::SpriteInputChanged(v) => {
                self.sprite_input = v;
                Task::none()
            }
            Message::SpriteModeSelected(m) => {
                self.sprite_mode = Some(m);
                Task::none()
            }
            // Sound
            Message::SoundInputChanged(v) => {
                self.sound_input = v;
                Task::none()
            }
            Message::SoundOutputChanged(v) => {
                self.sound_output = v;
                Task::none()
            }
            // Global
            Message::ExtractorPathChanged(v) => {
                self.extractor_path = v;
                Task::none()
            }
            Message::Run => {
                let Some(cmd) = self.build_internal_command() else {
                    self.log
                        .push_str("⚠ No command configured or supported in GUI yet.\n");
                    return Task::none();
                };
                self.log.push_str(&format!(
                    "▸ Running internal command: {} [{}]\n",
                    cmd.name(),
                    cmd.description()
                ));
                self.is_running = true;

                Task::perform(
                    async move {
                        tokio::task::spawn_blocking(move || {
                            let result =
                                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                                    cmd.execute()
                                }));
                            match result {
                                Ok(Ok(())) => Ok("Command finished successfully.\n".to_string()),
                                Ok(Err(e)) => Err(format!("Error: {}", e)),
                                Err(panic_err) => {
                                    if let Some(s) = panic_err.downcast_ref::<&str>() {
                                        Err(s.to_string())
                                    } else if let Some(s) = panic_err.downcast_ref::<String>() {
                                        Err(s.clone())
                                    } else {
                                        Err("Unknown panic occurred during execution".to_string())
                                    }
                                }
                            }
                        })
                        .await
                        .unwrap()
                    },
                    Message::CommandFinished,
                )
            }
            Message::CommandFinished(result) => {
                self.is_running = false;
                match result {
                    Ok(output) => {
                        self.log.push_str(&output);
                        self.log.push_str("✔ Done.\n\n");
                    }
                    Err(err) => {
                        self.log.push_str(&format!("✖ Error: {}\n\n", err));
                    }
                }
                Task::none()
            }
            Message::ClearLog => {
                self.log.clear();
                Task::none()
            }

            // ─── Chest Editor messages ──────────────────────────────
            Message::ChestOpBrowseGamePath => browse_folder("shared_game_path"),
            Message::ChestOpBrowseMapFile => browse_file("chest_map_file"),
            Message::ChestOpScanMaps => {
                if self.shared_game_path.is_empty() {
                    self.chest_editor.status_msg = "Please select game path first.".into();
                    return Task::none();
                }
                self.chest_editor.is_loading = true;
                let path = PathBuf::from(&self.shared_game_path).join("ExtraInGame");
                Task::perform(
                    async move {
                        let mut files = vec![];
                        if let Ok(entries) = std::fs::read_dir(path) {
                            for entry in entries.flatten() {
                                let p = entry.path();
                                if p.is_file()
                                    && p.extension().map(|e| e == "ref").unwrap_or(false)
                                    && p.file_name()
                                        .map(|n| n.to_string_lossy().starts_with("Ext"))
                                        .unwrap_or(false)
                                {
                                    files.push(p);
                                }
                            }
                        }
                        files.sort();
                        Ok(files)
                    },
                    Message::ChestMapsScanned,
                )
            }
            Message::ChestMapsScanned(res) => {
                self.chest_editor.is_loading = false;
                match res {
                    Ok(files) => {
                        self.chest_editor.map_files = files;
                        self.chest_editor.status_msg =
                            format!("Found {} map files.", self.chest_editor.map_files.len());
                    }
                    Err(e) => self.chest_editor.status_msg = format!("Error scanning maps: {}", e),
                }
                Task::none()
            }

            Message::ChestOpLoadCatalog => {
                if self.shared_game_path.is_empty() {
                    self.chest_editor.status_msg = "Please select game path first.".into();
                    return Task::none();
                }
                self.chest_editor.is_loading = true;
                let path = PathBuf::from(&self.shared_game_path);
                Task::perform(
                    async move { chest_editor::ItemCatalog::load_from_folder(&path) },
                    |res| Message::ChestCatalogLoaded(res.map_err(|e| e.to_string())),
                )
            }
            Message::ChestCatalogLoaded(res) => {
                self.chest_editor.is_loading = false;
                match res {
                    Ok(catalog) => {
                        self.chest_editor.catalog = Some(catalog);
                        self.chest_editor.status_msg = "Catalog loaded successfully.".into();
                    }
                    Err(e) => {
                        self.chest_editor.status_msg = format!("Error loading catalog: {}", e)
                    }
                }
                Task::none()
            }
            Message::ChestOpSelectMap => {
                if self.chest_editor.current_map_file.is_empty() {
                    self.chest_editor.status_msg = "No map file selected.".into();
                    return Task::none();
                }
                self.load_map_file(PathBuf::from(&self.chest_editor.current_map_file))
            }
            Message::ChestOpSelectMapFromFile(path) => {
                self.chest_editor.current_map_file = path.to_string_lossy().to_string();
                self.load_map_file(path)
            }
            Message::ChestMapLoaded(res) => {
                self.chest_editor.is_loading = false;
                match res {
                    Ok(records) => {
                        self.chest_editor.all_records = records;
                        self.chest_editor.status_msg = "Map loaded successfully.".into();
                        self.refresh_chests();
                    }
                    Err(e) => self.chest_editor.status_msg = format!("Error loading map: {}", e),
                }
                Task::none()
            }
            Message::ChestOpSelectChest(idx) => {
                self.chest_editor.selected_idx = Some(idx);
                if let Some((_, record)) = self.chest_editor.filtered_chests.get(idx) {
                    self.chest_editor.edit_name = record.name.clone();
                    self.chest_editor.edit_x = record.x_pos.to_string();
                    self.chest_editor.edit_y = record.y_pos.to_string();
                    self.chest_editor.edit_gold = record.gold_amount.to_string();
                    self.chest_editor.edit_item_count = record.item_count.to_string();
                    self.chest_editor.edit_item_id = record.item_id.to_string();
                    self.chest_editor.edit_item_type = (u8::from(record.item_type_id)).to_string();
                    self.chest_editor.edit_closed = record.closed.to_string();
                }
                Task::none()
            }
            Message::ChestOpFieldChanged(orig_idx, field, val) => {
                match field.as_str() {
                    "name" => self.chest_editor.edit_name = val.clone(),
                    "x" => self.chest_editor.edit_x = val.clone(),
                    "y" => self.chest_editor.edit_y = val.clone(),
                    "gold" => self.chest_editor.edit_gold = val.clone(),
                    "item_count" => self.chest_editor.edit_item_count = val.clone(),
                    "item_id" => self.chest_editor.edit_item_id = val.clone(),
                    "item_type" => self.chest_editor.edit_item_type = val.clone(),
                    "closed" => self.chest_editor.edit_closed = val.clone(),
                    _ => {}
                }
                if let Some(record) = self.chest_editor.all_records.get_mut(orig_idx) {
                    match field.as_str() {
                        "name" => record.name = val,
                        "x" => {
                            if let Ok(v) = val.parse() {
                                record.x_pos = v
                            }
                        }
                        "y" => {
                            if let Ok(v) = val.parse() {
                                record.y_pos = v
                            }
                        }
                        "gold" => {
                            if let Ok(v) = val.parse() {
                                record.gold_amount = v
                            }
                        }
                        "item_count" => {
                            if let Ok(v) = val.parse() {
                                record.item_count = v
                            }
                        }
                        "item_id" => {
                            if let Ok(v) = val.parse() {
                                record.item_id = v
                            }
                        }
                        "item_type" => {
                            if let Ok(v) = val.parse::<u8>() {
                                if let Some(t) = dispel_core::ItemTypeId::from_u8(v) {
                                    record.item_type_id = t;
                                }
                            }
                        }
                        "closed" => {
                            if let Ok(v) = val.parse() {
                                record.closed = v
                            }
                        }
                        _ => {}
                    }
                    self.refresh_chests();
                }
                Task::none()
            }
            Message::ChestOpSave => {
                if self.chest_editor.current_map_file.is_empty()
                    || self.chest_editor.all_records.is_empty()
                {
                    return Task::none();
                }
                self.chest_editor.is_loading = true;

                let path = PathBuf::from(&self.chest_editor.current_map_file);

                // Copy the original file with a timestamp (before file extension) as a backup
                if path.exists() {
                    let timestamp = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_secs())
                        .unwrap_or(0);

                    let stem = path.file_stem().unwrap_or_default().to_string_lossy();
                    let ext = path.extension().unwrap_or_default().to_string_lossy();

                    let mut backup_path = path.clone();
                    backup_path.set_file_name(format!("{}_{}.{}", stem, timestamp, ext));

                    if let Err(e) = std::fs::copy(&path, &backup_path) {
                        return Task::perform(
                            async move { Err(format!("Failed to backup: {}", e)) },
                            Message::ChestSaved,
                        );
                    }
                }

                let records = self.chest_editor.all_records.clone();
                Task::perform(
                    async move { dispel_core::ExtraRef::save_file(&records, &path) },
                    |res: Result<(), std::io::Error>| {
                        Message::ChestSaved(res.map_err(|e| e.to_string()))
                    },
                )
            }
            Message::ChestSaved(res) => {
                self.chest_editor.is_loading = false;
                match res {
                    Ok(_) => self.chest_editor.status_msg = "Map saved successfully.".into(),
                    Err(e) => self.chest_editor.status_msg = format!("Error saving map: {}", e),
                }
                Task::none()
            }
            Message::ChestOpAdd => Task::none(),
            Message::ChestOpDelete(_) => Task::none(),

            // ─── Weapon Editor messages ──────────────────────────────
            Message::WeaponOpBrowseGamePath => browse_folder("weapon_game_path"),
            Message::WeaponOpScanWeapons => {
                if self.shared_game_path.is_empty() {
                    self.weapon_editor.status_msg = "Please select game path first.".into();
                    return Task::none();
                }
                self.weapon_editor.is_loading = true;
                self.weapon_editor.status_msg = "Scanning weapons...".into();
                let path = PathBuf::from(&self.shared_game_path);
                Task::perform(
                    async move {
                        WeaponItem::read_file(&path.join("CharacterInGame").join("weaponItem.db"))
                            .map_err(|e: std::io::Error| e.to_string())
                    },
                    |res| Message::WeaponCatalogLoaded(res),
                )
            }
            Message::WeaponCatalogLoaded(res) => {
                self.weapon_editor.is_loading = false;
                match res {
                    Ok(catalog) => {
                        self.weapon_editor.catalog = Some(catalog.clone());
                        self.weapon_editor.status_msg =
                            format!("Weapon catalog loaded: {} weapons", catalog.len()).into();
                        self.weapon_editor.refresh_weapons();
                    }
                    Err(e) => {
                        self.weapon_editor.status_msg =
                            format!("Error loading weapon catalog: {}", e)
                    }
                }
                Task::none()
            }
            Message::WeaponOpSelectWeapon(idx) => {
                self.weapon_editor.select_weapon(idx);
                Task::none()
            }
            Message::WeaponOpFieldChanged(idx, field, val) => {
                self.weapon_editor.update_field(idx, &field, val);
                Task::none()
            }
            Message::WeaponOpSave => {
                if self.shared_game_path.is_empty() {
                    self.weapon_editor.status_msg = "Please select game path first.".into();
                    return Task::none();
                }
                self.weapon_editor.is_loading = true;
                let result = self.weapon_editor.save_weapons(&self.shared_game_path);
                self.weapon_editor.is_loading = false;
                match result {
                    Ok(_) => self.weapon_editor.status_msg = "Weapons saved successfully.".into(),
                    Err(e) => {
                        self.weapon_editor.status_msg = format!("Error saving weapons: {}", e)
                    }
                }
                Task::none()
            }
            Message::HealItemOpBrowseGamePath => browse_folder("shared_game_path"),
            Message::HealItemOpBrowseSpritePath => browse_folder("heal_item_sprite_path"),
            Message::HealItemOpScanItems => {
                if self.shared_game_path.is_empty() {
                    self.heal_item_editor.status_msg = "Please select game path first.".into();
                    return Task::none();
                }
                self.heal_item_editor.is_loading = true;
                self.heal_item_editor.status_msg = "Scanning heal items...".into();
                let path = PathBuf::from(&self.shared_game_path);
                Task::perform(
                    async move {
                        HealItem::read_file(&path.join("CharacterInGame").join("HealItem.db"))
                            .map_err(|e: std::io::Error| e.to_string())
                    },
                    |res| Message::HealItemCatalogLoaded(res),
                )
            }
            Message::HealItemOpLoadCatalog => {
                if self.shared_game_path.is_empty() {
                    self.heal_item_editor.status_msg = "Please select game path first.".into();
                    return Task::none();
                }
                self.heal_item_editor.is_loading = true;
                let path = PathBuf::from(&self.shared_game_path);
                Task::perform(
                    async move {
                        HealItem::read_file(&path.join("CharacterInGame").join("HealItem.db"))
                            .map_err(|e: std::io::Error| e.to_string())
                    },
                    |res| Message::HealItemCatalogLoaded(res),
                )
            }
            Message::HealItemCatalogLoaded(res) => {
                self.heal_item_editor.is_loading = false;
                match res {
                    Ok(catalog) => {
                        self.heal_item_editor.catalog = Some(catalog.clone());
                        self.heal_item_editor.status_msg =
                            format!("Heal item catalog loaded: {} items", catalog.len()).into();
                        self.heal_item_editor.refresh_items();
                    }
                    Err(e) => {
                        self.heal_item_editor.status_msg =
                            format!("Error loading heal item catalog: {}", e)
                    }
                }
                Task::none()
            }
            Message::HealItemOpSelectItem(idx) => {
                self.heal_item_editor.select_item(idx);
                Task::none()
            }
            Message::HealItemOpFieldChanged(idx, field, val) => {
                self.heal_item_editor.update_field(idx, &field, val);
                Task::none()
            }
            Message::HealItemOpSave => {
                if self.shared_game_path.is_empty() {
                    self.heal_item_editor.status_msg = "Please select game path first.".into();
                    return Task::none();
                }
                self.heal_item_editor.is_loading = true;
                let result = self.heal_item_editor.save_items(&self.shared_game_path);
                self.heal_item_editor.is_loading = false;
                match result {
                    Ok(_) => {
                        self.heal_item_editor.status_msg = "Heal items saved successfully.".into()
                    }
                    Err(e) => {
                        self.heal_item_editor.status_msg = format!("Error saving heal items: {}", e)
                    }
                }
                Task::none()
            }
            Message::MiscItemOpBrowseGamePath => browse_folder("shared_game_path"),
            Message::MiscItemOpLoadCatalog | Message::MiscItemOpScanItems => {
                if self.shared_game_path.is_empty() {
                    self.misc_item_editor.status_msg = "Please select game path first.".into();
                    return Task::none();
                }
                self.misc_item_editor.is_loading = true;
                let path = PathBuf::from(&self.shared_game_path);
                Task::perform(
                    async move {
                        MiscItem::read_file(&path.join("CharacterInGame").join("MiscItem.db"))
                            .map_err(|e: std::io::Error| e.to_string())
                    },
                    |res| Message::MiscItemCatalogLoaded(res),
                )
            }
            Message::MiscItemCatalogLoaded(res) => {
                self.misc_item_editor.is_loading = false;
                match res {
                    Ok(catalog) => {
                        self.misc_item_editor.catalog = Some(catalog.clone());
                        self.misc_item_editor.status_msg =
                            format!("Misc item catalog loaded: {} items", catalog.len()).into();
                        self.misc_item_editor.refresh_items();
                    }
                    Err(e) => {
                        self.misc_item_editor.status_msg =
                            format!("Error loading misc item catalog: {}", e)
                    }
                }
                Task::none()
            }
            Message::MiscItemOpSelectItem(idx) => {
                self.misc_item_editor.select_item(idx);
                Task::none()
            }
            Message::MiscItemOpFieldChanged(idx, field, val) => {
                self.misc_item_editor.update_field(idx, &field, val);
                Task::none()
            }
            Message::MiscItemOpSave => {
                if self.shared_game_path.is_empty() {
                    self.misc_item_editor.status_msg = "Please select game path first.".into();
                    return Task::none();
                }
                self.misc_item_editor.is_loading = true;
                let result = self.misc_item_editor.save_items(&self.shared_game_path);
                self.misc_item_editor.is_loading = false;
                match result {
                    Ok(_) => {
                        self.misc_item_editor.status_msg = "Misc items saved successfully.".into()
                    }
                    Err(e) => {
                        self.misc_item_editor.status_msg = format!("Error saving misc items: {}", e)
                    }
                }
                Task::none()
            }
            Message::EditItemOpBrowseGamePath => browse_folder("shared_game_path"),
            Message::EditItemOpLoadCatalog | Message::EditItemOpScanItems => {
                if self.shared_game_path.is_empty() {
                    self.edit_item_editor.status_msg = "Please select game path first.".into();
                    return Task::none();
                }
                self.edit_item_editor.is_loading = true;
                let path = PathBuf::from(&self.shared_game_path);
                Task::perform(
                    async move {
                        EditItem::read_file(&path.join("CharacterInGame").join("EditItem.db"))
                            .map_err(|e: std::io::Error| e.to_string())
                    },
                    |res| Message::EditItemCatalogLoaded(res),
                )
            }
            Message::EditItemCatalogLoaded(res) => {
                self.edit_item_editor.is_loading = false;
                match res {
                    Ok(catalog) => {
                        self.edit_item_editor.catalog = Some(catalog.clone());
                        self.edit_item_editor.status_msg =
                            format!("Edit item catalog loaded: {} items", catalog.len()).into();
                        self.edit_item_editor.refresh_items();
                    }
                    Err(e) => {
                        self.edit_item_editor.status_msg =
                            format!("Error loading edit item catalog: {}", e)
                    }
                }
                Task::none()
            }
            Message::EditItemOpSelectItem(idx) => {
                self.edit_item_editor.select_item(idx);
                Task::none()
            }
            Message::EditItemOpFieldChanged(idx, field, val) => {
                self.edit_item_editor.update_field(idx, &field, val);
                Task::none()
            }
            Message::EditItemOpSave => {
                if self.shared_game_path.is_empty() {
                    self.edit_item_editor.status_msg = "Please select game path first.".into();
                    return Task::none();
                }
                self.edit_item_editor.is_loading = true;
                let result = self.edit_item_editor.save_items(&self.shared_game_path);
                self.edit_item_editor.is_loading = false;
                match result {
                    Ok(_) => {
                        self.edit_item_editor.status_msg = "Edit items saved successfully.".into()
                    }
                    Err(e) => {
                        self.edit_item_editor.status_msg = format!("Error saving edit items: {}", e)
                    }
                }
                Task::none()
            }
            Message::EventItemOpBrowseGamePath => browse_folder("shared_game_path"),
            Message::EventItemOpLoadCatalog | Message::EventItemOpScanItems => {
                if self.shared_game_path.is_empty() {
                    self.event_item_editor.status_msg = "Please select game path first.".into();
                    return Task::none();
                }
                self.event_item_editor.is_loading = true;
                let path = PathBuf::from(&self.shared_game_path);
                Task::perform(
                    async move {
                        EventItem::read_file(&path.join("CharacterInGame").join("EventItem.db"))
                            .map_err(|e: std::io::Error| e.to_string())
                    },
                    |res| Message::EventItemCatalogLoaded(res),
                )
            }
            Message::EventItemCatalogLoaded(res) => {
                self.event_item_editor.is_loading = false;
                match res {
                    Ok(catalog) => {
                        self.event_item_editor.catalog = Some(catalog.clone());
                        self.event_item_editor.status_msg =
                            format!("Event item catalog loaded: {} items", catalog.len()).into();
                        self.event_item_editor.refresh_items();
                    }
                    Err(e) => {
                        self.event_item_editor.status_msg =
                            format!("Error loading event item catalog: {}", e)
                    }
                }
                Task::none()
            }
            Message::EventItemOpSelectItem(idx) => {
                self.event_item_editor.select_item(idx);
                Task::none()
            }
            Message::EventItemOpFieldChanged(idx, field, val) => {
                self.event_item_editor.update_field(idx, &field, val);
                Task::none()
            }
            Message::EventItemOpSave => {
                if self.shared_game_path.is_empty() {
                    self.event_item_editor.status_msg = "Please select game path first.".into();
                    return Task::none();
                }
                self.event_item_editor.is_loading = true;
                let result = self.event_item_editor.save_items(&self.shared_game_path);
                self.event_item_editor.is_loading = false;
                match result {
                    Ok(_) => {
                        self.event_item_editor.status_msg = "Event items saved successfully.".into()
                    }
                    Err(e) => {
                        self.event_item_editor.status_msg =
                            format!("Error saving event items: {}", e)
                    }
                }
                Task::none()
            }
            Message::MonsterOpBrowseGamePath => browse_folder("shared_game_path"),
            Message::MonsterOpLoadCatalog | Message::MonsterOpScanMonsters => {
                if self.shared_game_path.is_empty() {
                    self.monster_editor.status_msg = "Please select game path first.".into();
                    return Task::none();
                }
                self.monster_editor.is_loading = true;
                let path = PathBuf::from(&self.shared_game_path);
                Task::perform(
                    async move {
                        Monster::read_file(&path.join("MonsterInGame").join("Monster.db"))
                            .map_err(|e: std::io::Error| e.to_string())
                    },
                    |res| Message::MonsterCatalogLoaded(res),
                )
            }
            Message::MonsterCatalogLoaded(res) => {
                self.monster_editor.is_loading = false;
                match res {
                    Ok(catalog) => {
                        self.monster_editor.catalog = Some(catalog.clone());
                        self.monster_editor.status_msg =
                            format!("Monster catalog loaded: {} monsters", catalog.len()).into();
                        self.monster_editor.refresh_monsters();
                    }
                    Err(e) => {
                        self.monster_editor.status_msg =
                            format!("Error loading monster catalog: {}", e)
                    }
                }
                Task::none()
            }
            Message::MonsterOpSelectMonster(idx) => {
                self.monster_editor.select_monster(idx);
                Task::none()
            }
            Message::MonsterOpFieldChanged(idx, field, val) => {
                self.monster_editor.update_field(idx, &field, val);
                Task::none()
            }
            Message::MonsterOpSave => {
                if self.shared_game_path.is_empty() {
                    self.monster_editor.status_msg = "Please select game path first.".into();
                    return Task::none();
                }
                self.monster_editor.is_loading = true;
                let result = self.monster_editor.save_monsters(&self.shared_game_path);
                self.monster_editor.is_loading = false;
                match result {
                    Ok(_) => self.monster_editor.status_msg = "Monsters saved successfully.".into(),
                    Err(e) => {
                        self.monster_editor.status_msg = format!("Error saving monsters: {}", e)
                    }
                }
                Task::none()
            }
            Message::NpcIniOpBrowseGamePath => browse_folder("shared_game_path"),
            Message::NpcIniOpLoadCatalog | Message::NpcIniOpScanNpcs => {
                if self.shared_game_path.is_empty() {
                    self.npc_ini_editor.status_msg = "Please select game path first.".into();
                    return Task::none();
                }
                self.npc_ini_editor.is_loading = true;
                let path = PathBuf::from(&self.shared_game_path);
                Task::perform(
                    async move {
                        NpcIni::read_file(&path.join("Npc.ini"))
                            .map_err(|e: std::io::Error| e.to_string())
                    },
                    |res| Message::NpcIniCatalogLoaded(res),
                )
            }
            Message::NpcIniCatalogLoaded(res) => {
                self.npc_ini_editor.is_loading = false;
                match res {
                    Ok(catalog) => {
                        self.npc_ini_editor.catalog = Some(catalog.clone());
                        self.npc_ini_editor.status_msg =
                            format!("NPC catalog loaded: {} npcs", catalog.len()).into();
                        self.npc_ini_editor.refresh_npcs();
                    }
                    Err(e) => {
                        self.npc_ini_editor.status_msg = format!("Error loading NPC catalog: {}", e)
                    }
                }
                Task::none()
            }
            Message::NpcIniOpSelectNpc(idx) => {
                self.npc_ini_editor.select_npc(idx);
                Task::none()
            }
            Message::NpcIniOpFieldChanged(idx, field, val) => {
                self.npc_ini_editor.update_field(idx, &field, val);
                Task::none()
            }
            Message::NpcIniOpSave => {
                if self.shared_game_path.is_empty() {
                    self.npc_ini_editor.status_msg = "Please select game path first.".into();
                    return Task::none();
                }
                self.npc_ini_editor.is_loading = true;
                let result = self.npc_ini_editor.save_npcs(&self.shared_game_path);
                self.npc_ini_editor.is_loading = false;
                match result {
                    Ok(_) => self.npc_ini_editor.status_msg = "NPCs saved successfully.".into(),
                    Err(e) => self.npc_ini_editor.status_msg = format!("Error saving NPCs: {}", e),
                }
                Task::none()
            }
            Message::MagicOpBrowseGamePath => browse_folder("shared_game_path"),
            Message::MagicOpLoadCatalog | Message::MagicOpScanSpells => {
                if self.shared_game_path.is_empty() {
                    self.magic_editor.status_msg = "Please select game path first.".into();
                    return Task::none();
                }
                self.magic_editor.is_loading = true;
                let path = PathBuf::from(&self.shared_game_path);
                Task::perform(
                    async move {
                        MagicSpell::read_file(&path.join("MagicInGame").join("Magic.db"))
                            .map_err(|e: std::io::Error| e.to_string())
                    },
                    |res| Message::MagicCatalogLoaded(res),
                )
            }
            Message::MagicCatalogLoaded(res) => {
                self.magic_editor.is_loading = false;
                match res {
                    Ok(catalog) => {
                        self.magic_editor.catalog = Some(catalog.clone());
                        self.magic_editor.status_msg =
                            format!("Magic catalog loaded: {} spells", catalog.len()).into();
                        self.magic_editor.refresh_spells();
                    }
                    Err(e) => {
                        self.magic_editor.status_msg = format!("Error loading magic catalog: {}", e)
                    }
                }
                Task::none()
            }
            Message::MagicOpSelectSpell(idx) => {
                self.magic_editor.select_spell(idx);
                Task::none()
            }
            Message::MagicOpFieldChanged(idx, field, val) => {
                self.magic_editor.update_field(idx, &field, val);
                Task::none()
            }
            Message::MagicOpSave => {
                if self.shared_game_path.is_empty() {
                    self.magic_editor.status_msg = "Please select game path first.".into();
                    return Task::none();
                }
                self.magic_editor.is_loading = true;
                let result = self.magic_editor.save_spells(&self.shared_game_path);
                self.magic_editor.is_loading = false;
                match result {
                    Ok(_) => self.magic_editor.status_msg = "Spells saved successfully.".into(),
                    Err(e) => self.magic_editor.status_msg = format!("Error saving spells: {}", e),
                }
                Task::none()
            }
            Message::StoreOpBrowseGamePath => browse_folder("shared_game_path"),
            Message::StoreOpLoadCatalog | Message::StoreOpScanStores => {
                if self.shared_game_path.is_empty() {
                    self.store_editor.status_msg = "Please select game path first.".into();
                    return Task::none();
                }
                self.store_editor.is_loading = true;
                self.store_editor.status_msg = "Loading item catalogs...".into();
                let path = PathBuf::from(&self.shared_game_path);
                let char_path = path.join("CharacterInGame");
                let weapons_path = char_path.join("weaponItem.db");
                let heals_path = char_path.join("HealItem.db");
                let misc_path = char_path.join("MiscItem.db");
                let edit_path = char_path.join("EditItem.db");
                let store_path = char_path.join("STORE.DB");

                Task::perform(
                    async move {
                        let weapons = WeaponItem::read_file(&weapons_path).ok();
                        let heals = HealItem::read_file(&heals_path).ok();
                        let misc = MiscItem::read_file(&misc_path).ok();
                        let edit = EditItem::read_file(&edit_path).ok();
                        let stores = Store::read_file(&store_path)
                            .map_err(|e: std::io::Error| e.to_string())?;
                        Ok((weapons, heals, misc, edit, stores))
                    },
                    |res| Message::StoreCatalogWithItemsLoaded(res),
                )
            }
            Message::StoreCatalogWithItemsLoaded(res) => {
                match res {
                    Ok((weapons, heals, misc, edit, stores)) => {
                        self.weapon_editor.catalog = weapons.clone();
                        self.weapon_editor.refresh_weapons();
                        self.heal_item_editor.catalog = heals.clone();
                        self.heal_item_editor.refresh_items();
                        self.misc_item_editor.catalog = misc.clone();
                        self.misc_item_editor.refresh_items();
                        self.edit_item_editor.catalog = edit.clone();
                        self.edit_item_editor.refresh_items();
                        self.store_editor.catalog = Some(stores.clone());
                        let weapons_count = weapons.as_ref().map(|w| w.len()).unwrap_or(0);
                        let heals_count = heals.as_ref().map(|h| h.len()).unwrap_or(0);
                        let misc_count = misc.as_ref().map(|m| m.len()).unwrap_or(0);
                        let edit_count = edit.as_ref().map(|e| e.len()).unwrap_or(0);
                        self.store_editor.status_msg = format!(
                            "Loaded: {} stores, {} weapons, {} heals, {} misc, {} edit items",
                            stores.len(), weapons_count, heals_count, misc_count, edit_count
                        )
                        .into();
                        self.store_editor.refresh_stores();
                    }
                    Err(e) => {
                        self.store_editor.status_msg = format!("Error loading store catalog: {}", e)
                    }
                }
                self.store_editor.is_loading = false;
                Task::none()
            }
            Message::StoreCatalogLoaded(res) => {
                self.store_editor.is_loading = false;
                match res {
                    Ok(catalog) => {
                        self.store_editor.catalog = Some(catalog.clone());
                        self.store_editor.status_msg =
                            format!("Store catalog loaded: {} stores", catalog.len()).into();
                        self.store_editor.refresh_stores();
                    }
                    Err(e) => {
                        self.store_editor.status_msg = format!("Error loading store catalog: {}", e)
                    }
                }
                Task::none()
            }
            Message::StoreOpSelectStore(idx) => {
                self.store_editor.select_store(idx);
                Task::none()
            }
            Message::StoreOpFieldChanged(idx, field, val) => {
                self.store_editor.update_field(idx, &field, val);
                Task::none()
            }
            Message::StoreOpSelectProduct(idx) => {
                self.store_editor.select_product(idx);
                Task::none()
            }
            Message::StoreOpAddProduct => {
                self.store_editor.add_product();
                Task::none()
            }
            Message::StoreOpRemoveProduct(idx) => {
                self.store_editor.remove_product(idx);
                Task::none()
            }
            Message::StoreOpProductFieldChanged(prod_idx, field, val) => {
                self.store_editor.update_product(prod_idx, &field, val);
                Task::none()
            }
            Message::StoreOpSave => {
                if self.shared_game_path.is_empty() {
                    self.store_editor.status_msg = "Please select game path first.".into();
                    return Task::none();
                }
                self.store_editor.is_loading = true;
                let result = self.store_editor.save_stores(&self.shared_game_path);
                self.store_editor.is_loading = false;
                match result {
                    Ok(_) => self.store_editor.status_msg = "Stores saved successfully.".into(),
                    Err(e) => self.store_editor.status_msg = format!("Error saving stores: {}", e),
                }
                Task::none()
            }
            Message::PartyRefOpBrowseGamePath => browse_folder("shared_game_path"),
            Message::PartyRefOpLoadCatalog | Message::PartyRefOpScanParty => {
                if self.shared_game_path.is_empty() {
                    self.party_ref_editor.status_msg = "Please select game path first.".into();
                    return Task::none();
                }
                self.party_ref_editor.is_loading = true;
                let path = PathBuf::from(&self.shared_game_path);
                Task::perform(
                    async move {
                        PartyRef::read_file(&path.join("Ref").join("PartyRef.ref"))
                            .map_err(|e: std::io::Error| e.to_string())
                    },
                    |res| Message::PartyRefCatalogLoaded(res),
                )
            }
            Message::PartyRefCatalogLoaded(res) => {
                self.party_ref_editor.is_loading = false;
                match res {
                    Ok(catalog) => {
                        self.party_ref_editor.catalog = Some(catalog.clone());
                        self.party_ref_editor.status_msg =
                            format!("Party catalog loaded: {} members", catalog.len()).into();
                        self.party_ref_editor.refresh_party();
                    }
                    Err(e) => {
                        self.party_ref_editor.status_msg =
                            format!("Error loading party catalog: {}", e)
                    }
                }
                Task::none()
            }
            Message::PartyRefOpSelectMember(idx) => {
                self.party_ref_editor.select_member(idx);
                Task::none()
            }
            Message::PartyRefOpFieldChanged(idx, field, val) => {
                self.party_ref_editor.update_field(idx, &field, val);
                Task::none()
            }
            Message::PartyRefOpSave => {
                if self.shared_game_path.is_empty() {
                    self.party_ref_editor.status_msg = "Please select game path first.".into();
                    return Task::none();
                }
                self.party_ref_editor.is_loading = true;
                let result = self.party_ref_editor.save_party(&self.shared_game_path);
                self.party_ref_editor.is_loading = false;
                match result {
                    Ok(_) => {
                        self.party_ref_editor.status_msg = "Party refs saved successfully.".into()
                    }
                    Err(e) => {
                        self.party_ref_editor.status_msg = format!("Error saving party refs: {}", e)
                    }
                }
                Task::none()
            }
            Message::PartyIniOpBrowseGamePath => browse_folder("shared_game_path"),
            Message::PartyIniOpLoadCatalog | Message::PartyIniOpScanNpcs => {
                if self.shared_game_path.is_empty() {
                    self.party_ini_editor.status_msg = "Please select game path first.".into();
                    return Task::none();
                }
                self.party_ini_editor.is_loading = true;
                let path = PathBuf::from(&self.shared_game_path);
                Task::perform(
                    async move {
                        PartyIniNpc::read_file(&path.join("NpcInGame").join("PrtIni.db"))
                            .map_err(|e: std::io::Error| e.to_string())
                    },
                    |res| Message::PartyIniCatalogLoaded(res),
                )
            }
            Message::PartyIniCatalogLoaded(res) => {
                self.party_ini_editor.is_loading = false;
                match res {
                    Ok(catalog) => {
                        self.party_ini_editor.catalog = Some(catalog.clone());
                        self.party_ini_editor.status_msg =
                            format!("Party ini catalog loaded: {} npcs", catalog.len()).into();
                        self.party_ini_editor.refresh_npcs();
                    }
                    Err(e) => {
                        self.party_ini_editor.status_msg =
                            format!("Error loading party ini catalog: {}", e)
                    }
                }
                Task::none()
            }
            Message::PartyIniOpSelectNpc(idx) => {
                self.party_ini_editor.select_npc(idx);
                Task::none()
            }
            Message::PartyIniOpFieldChanged(idx, field, val) => {
                self.party_ini_editor.update_field(idx, &field, val);
                Task::none()
            }
            Message::PartyIniOpSave => {
                if self.shared_game_path.is_empty() {
                    self.party_ini_editor.status_msg = "Please select game path first.".into();
                    return Task::none();
                }
                self.party_ini_editor.is_loading = true;
                let result = self.party_ini_editor.save_npcs(&self.shared_game_path);
                self.party_ini_editor.is_loading = false;
                match result {
                    Ok(_) => {
                        self.party_ini_editor.status_msg = "Party ini saved successfully.".into()
                    }
                    Err(e) => {
                        self.party_ini_editor.status_msg = format!("Error saving party ini: {}", e)
                    }
                }
                Task::none()
            }

            // ─── DB Viewer messages ─────────────────────────────────
            Message::ViewerDbPathChanged(v) => {
                self.viewer.db_path = v;
                Task::none()
            }
            Message::ViewerBrowseDb => crate::utils::browse_file("viewer_db"),
            Message::ViewerConnect => {
                self.viewer.is_loading = true;
                self.viewer.status_msg = "Connecting…".into();
                let path = self.viewer.db_path.clone();
                Task::perform(
                    async move { db::list_tables(&path) },
                    Message::ViewerTablesLoaded,
                )
            }
            Message::ViewerTablesLoaded(result) => {
                self.viewer.is_loading = false;
                match result {
                    Ok(tables) => {
                        self.viewer.status_msg = format!("Connected – {} tables", tables.len());
                        self.viewer.tables = tables;
                        self.viewer.active_table = None;
                        self.viewer.rows.clear();
                        self.viewer.columns.clear();
                    }
                    Err(e) => {
                        self.viewer.status_msg = format!("✖ {}", e);
                    }
                }
                Task::none()
            }
            Message::ViewerSelectTable(t) => {
                self.viewer.active_table = Some(t.clone());
                self.viewer.page = 0;
                self.viewer.search.clear();
                self.viewer.sort_col = None;
                self.viewer.pending_edits.clear();
                self.viewer.editing_cell = None;
                self.viewer.sql_mode = false;
                self.viewer.sql_query = format!("SELECT * FROM \"{}\"", t);
                self.fetch_viewer_data()
            }
            Message::ViewerDataLoaded(result) => {
                self.viewer.is_loading = false;
                match result {
                    Ok(qr) => {
                        self.viewer.columns = qr.columns;
                        self.viewer.rows = qr.rows;
                        self.viewer.total_rows = qr.total_rows;
                        let page_start = self.viewer.page * PAGE_SIZE + 1;
                        let page_end =
                            (page_start - 1 + self.viewer.rows.len()).max(page_start - 1);
                        self.viewer.status_msg = format!(
                            "Showing {}-{} of {} rows",
                            page_start, page_end, self.viewer.total_rows
                        );
                    }
                    Err(e) => {
                        self.viewer.status_msg = format!("✖ Query error: {}", e);
                    }
                }
                Task::none()
            }
            Message::ViewerSearch(v) => {
                self.viewer.search = v;
                self.viewer.page = 0;
                self.fetch_viewer_data()
            }
            Message::ViewerSortColumn(idx) => {
                if self.viewer.sort_col == Some(idx) {
                    self.viewer.sort_dir = self.viewer.sort_dir.toggle();
                } else {
                    self.viewer.sort_col = Some(idx);
                    self.viewer.sort_dir = db::SortDir::Asc;
                }
                self.viewer.page = 0;
                self.fetch_viewer_data()
            }
            Message::ViewerNextPage => {
                let max_page = self.viewer.total_rows.saturating_sub(1) / PAGE_SIZE;
                if self.viewer.page < max_page {
                    self.viewer.page += 1;
                    return self.fetch_viewer_data();
                }
                Task::none()
            }
            Message::ViewerPrevPage => {
                if self.viewer.page > 0 {
                    self.viewer.page -= 1;
                    return self.fetch_viewer_data();
                }
                Task::none()
            }
            Message::ViewerCellClick(r, c) => {
                // Confirm previous edit if any
                if let Some((pr, pc)) = self.viewer.editing_cell {
                    if !self.viewer.edit_buffer.is_empty()
                        || self
                            .viewer
                            .rows
                            .get(pr)
                            .and_then(|row| row.get(pc).map(|v| v.as_str()))
                            != Some(&self.viewer.edit_buffer)
                    {
                        let original = self
                            .viewer
                            .rows
                            .get(pr)
                            .and_then(|row| row.get(pc))
                            .cloned()
                            .unwrap_or_default();
                        if self.viewer.edit_buffer != original {
                            self.viewer
                                .pending_edits
                                .insert((pr, pc), self.viewer.edit_buffer.clone());
                        }
                    }
                }
                let val = self
                    .viewer
                    .rows
                    .get(r)
                    .and_then(|row| row.get(c))
                    .cloned()
                    .unwrap_or_default();
                self.viewer.editing_cell = Some((r, c));
                self.viewer.edit_buffer = val;
                Task::none()
            }
            Message::ViewerCellEdit(v) => {
                self.viewer.edit_buffer = v;
                Task::none()
            }
            Message::ViewerCellConfirm => {
                if let Some((r, c)) = self.viewer.editing_cell {
                    let original = self
                        .viewer
                        .rows
                        .get(r)
                        .and_then(|row| row.get(c))
                        .cloned()
                        .unwrap_or_default();
                    if self.viewer.edit_buffer != original {
                        self.viewer
                            .pending_edits
                            .insert((r, c), self.viewer.edit_buffer.clone());
                    }
                }
                self.viewer.editing_cell = None;
                Task::none()
            }
            Message::ViewerCellCancel => {
                self.viewer.editing_cell = None;
                Task::none()
            }
            Message::ViewerCommit => {
                if self.viewer.pending_edits.is_empty() {
                    self.viewer.status_msg = "Nothing to commit.".into();
                    return Task::none();
                }
                let path = self.viewer.db_path.clone();
                let table = self.viewer.active_table.clone().unwrap_or_default();
                let cols = self.viewer.columns.clone();
                let rows = self.viewer.rows.clone();
                let edits = self.viewer.pending_edits.clone();
                self.viewer.is_loading = true;
                Task::perform(
                    async move { db::commit_edits(&path, &table, &cols, &rows, &edits) },
                    Message::ViewerCommitDone,
                )
            }
            Message::ViewerCommitDone(result) => {
                self.viewer.is_loading = false;
                match result {
                    Ok(n) => {
                        // Apply edits to local rows
                        for ((r, c), val) in &self.viewer.pending_edits {
                            if let Some(row) = self.viewer.rows.get_mut(*r) {
                                if let Some(cell) = row.get_mut(*c) {
                                    *cell = val.clone();
                                }
                            }
                        }
                        self.viewer.pending_edits.clear();
                        self.viewer.status_msg = format!("✔ Committed {} row(s)", n);
                    }
                    Err(e) => {
                        self.viewer.status_msg = format!("✖ Commit failed: {}", e);
                    }
                }
                Task::none()
            }
            Message::ViewerToggleSql => {
                self.viewer.sql_mode = !self.viewer.sql_mode;
                Task::none()
            }
            Message::ViewerSqlChanged(v) => {
                self.viewer.sql_query = v;
                Task::none()
            }
            Message::ViewerRunSql => {
                self.viewer.page = 0;
                self.viewer.pending_edits.clear();
                self.viewer.editing_cell = None;
                self.fetch_viewer_data_sql()
            }
            Message::ViewerExportCsv => {
                let cols = self.viewer.columns.clone();
                let rows = self.viewer.rows.clone();
                Task::perform(
                    async move {
                        let handle = rfd::AsyncFileDialog::new()
                            .set_file_name("export.csv")
                            .add_filter("CSV", &["csv"])
                            .save_file()
                            .await;
                        match handle {
                            Some(h) => {
                                let path = h.path().to_string_lossy().to_string();
                                db::export_csv(&path, &cols, &rows).map(|_| path)
                            }
                            None => Err("Cancelled".into()),
                        }
                    },
                    Message::ViewerCsvSaved,
                )
            }
            Message::ViewerCsvSaved(result) => {
                match result {
                    Ok(p) => self.viewer.status_msg = format!("✔ Exported to {}", p),
                    Err(e) => self.viewer.status_msg = format!("✖ Export: {}", e),
                }
                Task::none()
            }
            Message::ViewerRevertEdits => {
                self.viewer.pending_edits.clear();
                self.viewer.editing_cell = None;
                self.viewer.status_msg = "Reverted all pending edits.".into();
                Task::none()
            }
            // Sprite Browser
            Message::SpriteBrowserOpBrowsePath => browse_folder("shared_game_path"),
            Message::SpriteBrowserOpScan => {
                if self.shared_game_path.is_empty() {
                    self.sprite_browser.status_msg = "Please select game path first.".into();
                    return Task::none();
                }
                self.sprite_browser.is_loading = true;
                self.sprite_browser.status_msg = "Scanning for sprites...".into();
                let path = PathBuf::from(&self.shared_game_path);
                Task::perform(
                    async move {
                        let mut entries = Vec::new();
                        Self::find_sprites_recursive(&path, &mut entries);
                        Ok(entries)
                    },
                    Message::SpriteBrowserScanned,
                )
            }
            Message::SpriteBrowserScanned(res) => {
                self.sprite_browser.is_loading = false;
                match res {
                    Ok(entries) => {
                        self.sprite_browser.sprites = entries;
                        self.sprite_browser.filter_sprites();
                        self.sprite_browser.status_msg =
                            format!("Found {} sprite files", self.sprite_browser.sprites.len())
                                .into();
                    }
                    Err(e) => {
                        self.sprite_browser.status_msg =
                            format!("Error scanning sprites: {}", e).into();
                    }
                }
                Task::none()
            }
            Message::SpriteBrowserOpSearch(query) => {
                self.sprite_browser.search_query = query;
                self.sprite_browser.filter_sprites();
                Task::none()
            }
            Message::SpriteBrowserOpSelectSprite(filtered_idx) => {
                self.sprite_browser.select_sprite_filtered(filtered_idx);
                Task::none()
            }
            Message::SpriteBrowserOpSelectSequence(seq_idx) => {
                self.sprite_browser.select_sequence(seq_idx);
                Task::none()
            }
            Message::SpriteBrowserOpSelectFrame(frame_idx) => {
                self.sprite_browser.select_frame(frame_idx);
                Task::none()
            }
        }
    }

    fn find_sprites_recursive(dir: &Path, results: &mut Vec<sprite_browser::SpriteEntry>) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                if path.is_dir() {
                    Self::find_sprites_recursive(&path, results);
                } else if let Some(ext) = path.extension() {
                    if ext.to_string_lossy().to_lowercase() == "spr" {
                        let name = path
                            .file_stem()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_default();
                        let (seq_count, frame_counts) =
                            Self::analyze_sprite_file(&path);
                        results.push(sprite_browser::SpriteEntry {
                            path,
                            name,
                            sequence_count: seq_count,
                            frame_counts,
                        });
                    }
                }
            }
        }
    }

    fn analyze_sprite_file(path: &Path) -> (usize, Vec<usize>) {
        use std::fs;
        use std::io::BufReader;

        let mut frame_counts = Vec::new();
        if let Ok(file) = fs::File::open(path) {
            let file_len = file.metadata().map(|m| m.len()).unwrap_or(0);
            let mut reader = BufReader::new(file);

            if std::io::Seek::seek(&mut reader, std::io::SeekFrom::Start(268)).is_ok() {
                loop {
                    let pos = reader.stream_position().unwrap_or(0);
                    if pos >= file_len {
                        break;
                    }
                    if let Ok(valid) =
                        dispel_core::sprite::seek_next_sequence(&mut reader, pos, file_len)
                    {
                        if valid {
                            if let Ok(info) = dispel_core::sprite::get_sequence_info(&mut reader) {
                                frame_counts.push(info.frame_count as usize);
                            } else {
                                break;
                            }
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
            }
        }
        (frame_counts.len(), frame_counts)
    }

    /// Fetch data using the built table query (filters + sorting).
    pub fn fetch_viewer_data(&mut self) -> Task<Message> {
        let table = match &self.viewer.active_table {
            Some(t) => t.clone(),
            None => return Task::none(),
        };
        self.viewer.is_loading = true;

        // First get column info, then build query
        let path = self.viewer.db_path.clone();
        let search = self.viewer.search.clone();
        let sort_col = self.viewer.sort_col;
        let sort_dir = self.viewer.sort_dir;
        let page = self.viewer.page;

        Task::perform(
            async move {
                let cols = db::table_columns(&path, &table)?;
                let sql = db::build_table_query(&table, &cols, &search, sort_col, sort_dir);
                let mut result = db::execute_query(&path, &sql, PAGE_SIZE, page * PAGE_SIZE)?;
                // Merge proper column info
                result.columns = cols;
                Ok(result)
            },
            Message::ViewerDataLoaded,
        )
    }

    /// Fetch data using the custom SQL query.
    pub fn fetch_viewer_data_sql(&mut self) -> Task<Message> {
        self.viewer.is_loading = true;
        let path = self.viewer.db_path.clone();
        let sql = self.viewer.sql_query.clone();
        let page = self.viewer.page;

        Task::perform(
            async move { db::execute_query(&path, &sql, PAGE_SIZE, page * PAGE_SIZE) },
            Message::ViewerDataLoaded,
        )
    }

    pub fn build_internal_command(&self) -> Option<Box<dyn Command>> {
        let factory = CommandFactory::new();
        match self.active_tab {
            Tab::Map => {
                let op = self.map_op?;
                let subcommand = match op {
                    MapOp::Tiles => commands::map::MapSubcommand::Tiles {
                        input: self.map_input.clone(),
                        output: if self.map_output.is_empty() {
                            "out".to_string()
                        } else {
                            self.map_output.clone()
                        },
                    },
                    MapOp::Atlas => commands::map::MapSubcommand::Atlas {
                        input: self.map_input.clone(),
                        output: self.map_output.clone(),
                    },
                    MapOp::Render => commands::map::MapSubcommand::Render {
                        map: self.map_map_path.clone(),
                        btl: self.map_btl_path.clone(),
                        gtl: self.map_gtl_path.clone(),
                        output: self.map_output.clone(),
                        save_sprites: self.map_save_sprites,
                    },
                    MapOp::FromDb => commands::map::MapSubcommand::FromDb {
                        database: self.map_database.clone(),
                        map_id: self.map_map_id.clone(),
                        gtl_atlas: self.map_gtl_atlas.clone(),
                        btl_atlas: self.map_btl_atlas.clone(),
                        atlas_columns: self.map_atlas_columns.parse().unwrap_or(48),
                        output: self.map_output.clone(),
                        game_path: if self.map_game_path.is_empty() {
                            None
                        } else {
                            Some(self.map_game_path.clone())
                        },
                    },
                    MapOp::ToDb => commands::map::MapSubcommand::ToDb {
                        database: self.map_database.clone(),
                        map: self.map_map_path.clone(),
                    },
                    MapOp::Sprites => commands::map::MapSubcommand::Sprites {
                        input: self.map_input.clone(),
                        output: if self.map_output.is_empty() {
                            "out".to_string()
                        } else {
                            self.map_output.clone()
                        },
                    },
                };
                Some(Box::new(factory.create_map_command(subcommand)))
            }
            Tab::Ref => {
                let op = self.ref_op?;
                let input = self.ref_input.clone();
                let subcommand = match op {
                    RefOp::AllMaps => commands::ref_command::RefSubcommand::AllMaps { input },
                    RefOp::Map => commands::ref_command::RefSubcommand::Map { input },
                    RefOp::Extra => commands::ref_command::RefSubcommand::Extra { input },
                    RefOp::Event => commands::ref_command::RefSubcommand::Event { input },
                    RefOp::Monster => commands::ref_command::RefSubcommand::Monster { input },
                    RefOp::Npc => commands::ref_command::RefSubcommand::Npc { input },
                    RefOp::Wave => commands::ref_command::RefSubcommand::Wave { input },
                    RefOp::PartyRef => commands::ref_command::RefSubcommand::PartyRef { input },
                    RefOp::DrawItem => commands::ref_command::RefSubcommand::DrawItem { input },
                    RefOp::PartyDialog => {
                        commands::ref_command::RefSubcommand::PartyDialog { input }
                    }
                    RefOp::Dialog => commands::ref_command::RefSubcommand::Dialog { input },
                    RefOp::Weapons => commands::ref_command::RefSubcommand::Weapons { input },
                    RefOp::MultiMagic => commands::ref_command::RefSubcommand::MultiMagic { input },
                    RefOp::Store => commands::ref_command::RefSubcommand::Store { input },
                    RefOp::NpcRef => commands::ref_command::RefSubcommand::NpcRef { input },
                    RefOp::MonsterRef => commands::ref_command::RefSubcommand::MonsterRef { input },
                    RefOp::Monsters => commands::ref_command::RefSubcommand::Monsters { input },
                    RefOp::MiscItem => commands::ref_command::RefSubcommand::MiscItem { input },
                    RefOp::HealItems => commands::ref_command::RefSubcommand::HealItems { input },
                    RefOp::ExtraRef => commands::ref_command::RefSubcommand::ExtraRef { input },
                    RefOp::EventItems => commands::ref_command::RefSubcommand::EventItems { input },
                    RefOp::EditItems => commands::ref_command::RefSubcommand::EditItems { input },
                    RefOp::PartyLevel => commands::ref_command::RefSubcommand::PartyLevel { input },
                    RefOp::PartyIni => commands::ref_command::RefSubcommand::PartyIni { input },
                    RefOp::EventNpcRef => {
                        commands::ref_command::RefSubcommand::EventNpcRef { input }
                    }
                    RefOp::Magic => commands::ref_command::RefSubcommand::Magic { input },
                    RefOp::Quest => commands::ref_command::RefSubcommand::Quest { input },
                    RefOp::Message => commands::ref_command::RefSubcommand::Message { input },
                    RefOp::ChData => commands::ref_command::RefSubcommand::ChData { input },
                };
                Some(Box::new(factory.create_ref_command(subcommand)))
            }
            Tab::Database => {
                // let op = self.db_op?;
                // let subcommand = match op {
                //     DbOp::Import => commands::database::DatabaseSubcommand::Import,
                //     DbOp::DialogTexts => commands::database::DatabaseSubcommand::DialogTexts,
                //     DbOp::Maps => commands::database::DatabaseSubcommand::Maps,
                //     DbOp::Databases => commands::database::DatabaseSubcommand::Databases,
                //     DbOp::Refs => commands::database::DatabaseSubcommand::Refs,
                //     DbOp::Rest => commands::database::DatabaseSubcommand::Rest,
                // };
                // Some(Box::new(factory.create_database_command(subcommand)))
                None
            }
            Tab::Sprite => {
                let mode = match self.sprite_mode {
                    Some(SpriteMode::Animation) => commands::sprite::SpriteMode::Animation,
                    _ => commands::sprite::SpriteMode::Sprite,
                };
                Some(Box::new(
                    factory.create_sprite_command(self.sprite_input.clone(), mode),
                ))
            }
            Tab::Sound => Some(Box::new(
                factory.create_sound_command(self.sound_input.clone(), self.sound_output.clone()),
            )),
            Tab::DbViewer | Tab::ChestEditor => None,
            Tab::WeaponEditor | Tab::HealItemEditor => None,
            Tab::MiscItemEditor
            | Tab::EditItemEditor
            | Tab::EventItemEditor
            | Tab::MonsterEditor
            | Tab::NpcIniEditor
            | Tab::MagicEditor
            | Tab::StoreEditor
            | Tab::PartyRefEditor
            | Tab::PartyIniEditor
            | Tab::SpriteBrowser => None,
        }
    }
}

pub fn view(app: &App) -> Element<'_, Message> {
    app.view()
}
