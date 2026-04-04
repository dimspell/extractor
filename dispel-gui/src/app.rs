use crate::chest_editor::ItemCatalog;
use crate::command_palette::CommandPalette;
use crate::db;
use crate::db_viewer_state::PAGE_SIZE;
use crate::edit_history::EditHistory;
use crate::file_tree::FileTree;
use crate::message::{Message, WorkspaceMessage};
use crate::sprite_browser::SpriteEntry;
use crate::state::AppState;
use crate::style;
use crate::tab_bar::TabBarMessage;
use crate::types::{MapOp, RefOp, Tab};
use crate::utils::{browse_file, browse_folder, horizontal_rule, horizontal_space};
use dispel_core::commands::{self, CommandFactory};
use dispel_core::{
    ChData, Dialog, DialogueText, DrawItem, EditItem, Event, EventItem, EventNpcRef, Extra,
    ExtraRef, Extractor, HealItem, MagicSpell, Map, MapIni, Message as ScrMessage, MiscItem,
    Monster, MonsterIni, NpcIni, PartyIniNpc, PartyLevelNpc, PartyRef, Quest, Store, WaveIni,
    WeaponItem, NPC,
};
use iced::widget::{button, column, container, row, text};
use iced::{Element, Fill, Length, Subscription, Task};
use std::path::{Path, PathBuf};

pub use dispel_core::commands::Command;

pub struct App {
    pub state: AppState,
    pub file_tree: FileTree,
    pub workspace_mode: bool,
    pub window_id: iced::window::Id,
    pub history_panel_visible: bool,
    pub empty_edit_history: EditHistory,
    pub command_palette: Option<CommandPalette>,
    pub global_search: crate::global_search::GlobalSearch,
    pub draft_manager: crate::auto_save::DraftManager,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let mut state = AppState::default();
        if let Err(e) = state.load_workspace() {
            eprintln!("Failed to load workspace: {}", e);
        }
        let game_path = state.workspace.game_path.clone();
        let file_tree = if let Some(ref path) = game_path {
            FileTree::scan(path)
        } else {
            FileTree::default()
        };
        (
            Self {
                state,
                file_tree,
                workspace_mode: true,
                window_id: iced::window::Id::unique(),
                history_panel_visible: false,
                empty_edit_history: EditHistory::default(),
                command_palette: None,
                global_search: crate::global_search::GlobalSearch::new(),
                draft_manager: crate::auto_save::DraftManager::new(),
            },
            Task::none(),
        )
    }

    pub fn get_active_edit_history(&self) -> &EditHistory {
        use crate::generic_editor::UndoRedo;
        match self.state.active_tab {
            Tab::HealItemEditor => self.state.heal_item_editor.edit_history(),
            Tab::MiscItemEditor => self.state.misc_item_editor.edit_history(),
            Tab::EditItemEditor => self.state.edit_item_editor.edit_history(),
            Tab::EventItemEditor => self.state.event_item_editor.edit_history(),
            Tab::MagicEditor => self.state.magic_editor.edit_history(),
            Tab::WeaponEditor => self.state.weapon_editor.edit_history(),
            Tab::MonsterRefEditor => self.state.monster_ref_editor.edit_history(),
            Tab::ExtraRefEditor => self.state.extra_ref_editor.edit_history(),
            Tab::NpcRefEditor => self.state.npc_ref_editor.edit_history(),
            Tab::DialogEditor => self.state.dialog_editor.edit_history(),
            Tab::DialogueTextEditor => self.state.dialogue_text_editor.edit_history(),
            Tab::DrawItemEditor => self.state.draw_item_editor.edit_history(),
            Tab::EventIniEditor => self.state.event_ini_editor.edit_history(),
            Tab::EventNpcRefEditor => self.state.event_npc_ref_editor.edit_history(),
            Tab::ExtraIniEditor => self.state.extra_ini_editor.edit_history(),
            Tab::MapIniEditor => self.state.map_ini_editor.edit_history(),
            Tab::MessageScrEditor => self.state.message_scr_editor.edit_history(),
            Tab::PartyLevelDbEditor => self.state.party_level_db_editor.edit_history(),
            Tab::QuestScrEditor => self.state.quest_scr_editor.edit_history(),
            Tab::WaveIniEditor => self.state.wave_ini_editor.edit_history(),
            Tab::AllMapIniEditor => self.state.all_map_ini_editor.edit_history(),
            Tab::ChDataEditor => self.state.chdata_editor.edit_history(),
            Tab::PartyRefEditor => self.state.party_ref_editor.edit_history(),
            Tab::PartyIniEditor => self.state.party_ini_editor.edit_history(),
            Tab::NpcIniEditor => self.state.npc_ini_editor.edit_history(),
            Tab::StoreEditor => self.state.store_editor.edit_history(),
            _ => &self.empty_edit_history,
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        use iced::keyboard::{self, key::Named, Key};
        use iced::window;

        let close = window::close_requests().map(|_| Message::CloseRequested);

        let keyboard_sub = keyboard::listen().filter_map(|event| {
            if let keyboard::Event::KeyPressed { key, modifiers, .. } = event {
                if modifiers.control() || modifiers.command() {
                    if let Key::Character(c) = key.as_ref() {
                        let ch = c.chars().next()?;
                        return match ch {
                            'z' => Some(Message::Undo),
                            'y' => Some(Message::Redo),
                            'h' => Some(Message::ToggleHistoryPanel),
                            'p' => Some(Message::ToggleCommandPalette),
                            'f' => Some(Message::ToggleGlobalSearch),
                            _ => None,
                        };
                    }
                }
                if let Key::Named(named) = key.as_ref() {
                    match named {
                        Named::Escape => Some(Message::CommandPaletteClose),
                        Named::Enter => Some(Message::CommandPaletteConfirm),
                        Named::ArrowUp => Some(Message::CommandPaletteArrowUp),
                        Named::ArrowDown => Some(Message::CommandPaletteArrowDown),
                        _ => None,
                    }
                } else {
                    None
                }
            } else {
                None
            }
        });

        Subscription::batch([close, keyboard_sub])
    }

    pub fn update_workspace(&mut self, msg: WorkspaceMessage) -> Task<Message> {
        match msg {
            WorkspaceMessage::FileTree(ft_msg) => self.handle_file_tree(ft_msg),
            WorkspaceMessage::TabBar(tb_msg) => self.handle_tab_bar(tb_msg),
            WorkspaceMessage::ToggleWorkspaceMode => {
                self.workspace_mode = !self.workspace_mode;
                Task::none()
            }
        }
    }

    fn handle_file_tree(&mut self, msg: crate::file_tree::FileTreeMessage) -> Task<Message> {
        use crate::file_tree::FileTreeMessage as FTM;
        match msg {
            FTM::ToggleDir(path) => {
                self.file_tree.toggle(&path);
                Task::none()
            }
            FTM::OpenFile(path) => self.open_file_in_workspace(&path),
            FTM::Search(query) => {
                self.file_tree.search_query = query;
                Task::none()
            }
        }
    }

    fn handle_tab_bar(&mut self, msg: TabBarMessage) -> Task<Message> {
        match msg {
            TabBarMessage::SelectTab(idx) => {
                self.state.workspace.active_tab = Some(idx);
                self.save_workspace();
                Task::none()
            }
            TabBarMessage::CloseTab(idx) => {
                self.state.workspace.close(idx);
                self.save_workspace();
                Task::none()
            }
            TabBarMessage::TogglePin(idx) => {
                if let Some(tab) = self.state.workspace.tabs.get_mut(idx) {
                    tab.pinned = !tab.pinned;
                }
                self.save_workspace();
                Task::none()
            }
        }
    }

    fn save_workspace(&self) {
        if let Err(e) = self.state.save_workspace() {
            eprintln!("Failed to save workspace: {}", e);
        }
    }

    fn open_file_in_workspace(&mut self, path: &Path) -> Task<Message> {
        let label = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        self.state.workspace.open(label, Some(path.to_path_buf()));
        self.save_workspace();

        // Auto-load the file based on extension
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            let stem = path
                .file_stem()
                .map(|s| s.to_string_lossy().to_lowercase())
                .unwrap_or_default();
            match ext {
                "db" => match stem.as_str() {
                    "weaponitem" => self.load_editor_auto("weapons", path),
                    "monster" => self.load_editor_auto("monsters", path),
                    "healitem" => self.load_editor_auto("heal_items", path),
                    "miscitem" => self.load_editor_auto("misc_items", path),
                    "edititem" => self.load_editor_auto("edit_items", path),
                    "eventitem" => self.load_editor_auto("event_items", path),
                    "store" => self.load_editor_auto("stores", path),
                    "magic" => self.load_editor_auto("magic", path),
                    "chdata" => self.load_editor_auto("chdata", path),
                    "prtlevel" => self.load_editor_auto("party_levels", path),
                    "prtini" => self.load_editor_auto("party_ini", path),
                    _ => {}
                },
                "ini" => match stem.as_str() {
                    "allmap" => self.load_editor_auto("all_maps", path),
                    "map" => self.load_editor_auto("map_ini", path),
                    "extra" => self.load_editor_auto("extra_ini", path),
                    "event" => self.load_editor_auto("event_ini", path),
                    "monster" => self.load_editor_auto("monster_ini", path),
                    "npc" => self.load_editor_auto("npc_ini", path),
                    "wave" => self.load_editor_auto("wave_ini", path),
                    _ => {}
                },
                "ref" => match stem.as_str() {
                    "partyref" => self.load_editor_auto("party_ref", path),
                    "drawitem" => self.load_editor_auto("draw_items", path),
                    "eventnpc" => self.load_editor_auto("event_npc_ref", path),
                    _ => {
                        if stem.starts_with("npc") {
                            self.load_editor_auto("npc_ref", path);
                        } else if stem.starts_with("mon") {
                            self.load_editor_auto("monster_ref", path);
                        } else if stem.starts_with("ext") {
                            self.load_editor_auto("extra_ref", path);
                        }
                    }
                },
                "scr" => match stem.as_str() {
                    "quest" => self.load_editor_auto("quests", path),
                    "message" => self.load_editor_auto("messages", path),
                    _ => {}
                },
                "dlg" => self.load_editor_auto("dialogs", path),
                "pgp" => self.load_editor_auto("dialogue_texts", path),
                _ => {}
            }
        }

        Task::none()
    }

    fn load_editor_auto(&mut self, _type_name: &str, _path: &Path) {
        // For now, just open the tab. Loading will happen when the user
        // navigates to the appropriate editor tab.
        // TODO: Auto-load the file into the appropriate editor state.
    }

    pub fn refresh_chests(&mut self) {
        let editor = &mut self.state.chest_editor;
        editor.filtered_chests = editor
            .all_records
            .iter()
            .enumerate()
            .filter(|(_, r)| r.object_type == dispel_core::ExtraObjectType::Chest)
            .map(|(i, r)| (i, r.clone()))
            .collect();
    }

    pub fn load_map_file(&mut self, path: PathBuf) -> Task<Message> {
        self.state.chest_editor.is_loading = true;
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
                self.state.active_tab = tab;
                Task::none()
            }
            // Map
            Message::MapOpSelected(op) => {
                self.state.map_op = Some(op);
                Task::none()
            }
            Message::MapInputChanged(v) => {
                self.state.map_input = v;
                Task::none()
            }
            Message::MapOutputChanged(v) => {
                self.state.map_output = v;
                Task::none()
            }
            Message::MapMapPathChanged(v) => {
                self.state.map_map_path = v;
                Task::none()
            }
            Message::MapBtlPathChanged(v) => {
                self.state.map_btl_path = v;
                Task::none()
            }
            Message::MapGtlPathChanged(v) => {
                self.state.map_gtl_path = v;
                Task::none()
            }
            Message::MapSaveSpritesToggled(v) => {
                self.state.map_save_sprites = v;
                Task::none()
            }
            Message::MapDatabaseChanged(v) => {
                self.state.map_database = v;
                Task::none()
            }
            Message::MapMapIdChanged(v) => {
                self.state.map_map_id = v;
                Task::none()
            }
            Message::MapGtlAtlasChanged(v) => {
                self.state.map_gtl_atlas = v;
                Task::none()
            }
            Message::MapBtlAtlasChanged(v) => {
                self.state.map_btl_atlas = v;
                Task::none()
            }
            Message::MapAtlasColumnsChanged(v) => {
                self.state.map_atlas_columns = v;
                Task::none()
            }
            Message::MapGamePathChanged(v) => {
                self.state.map_game_path = v;
                Task::none()
            }
            // Shared Game Path
            Message::BrowseSharedGamePath => {
                if self.workspace_mode {
                    browse_folder("workspace_game_path")
                } else {
                    browse_folder("shared_game_path")
                }
            }
            Message::LoadSharedGamePath => {
                if self.state.shared_game_path.is_empty() {
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
            Message::BrowseExtractorPath => browse_file("extractor_path"),
            Message::FileSelected { field, path } => {
                if let Some(p) = path {
                    let s = p.to_string_lossy().to_string();
                    match field.as_str() {
                        "shared_game_path" => self.state.shared_game_path = s.clone(),
                        "workspace_game_path" => {
                            let pathbuf = PathBuf::from(&s);
                            self.state.workspace.game_path = Some(pathbuf.clone());
                            self.state.shared_game_path = s.clone();
                            self.file_tree = FileTree::scan(&pathbuf);
                            self.save_workspace();
                        }
                        "map_input" => self.state.map_input = s,
                        "map_map_path" => self.state.map_map_path = s,
                        "map_btl_path" => self.state.map_btl_path = s,
                        "map_gtl_path" => self.state.map_gtl_path = s,
                        "map_gtl_atlas" => self.state.map_gtl_atlas = s,
                        "map_btl_atlas" => self.state.map_btl_atlas = s,
                        "map_game_path" => {
                            self.state.map_game_path = s.clone();
                            self.state.shared_game_path = s;
                        }
                        "ref_input" => self.state.ref_input = s,
                        "extractor_path" => self.state.extractor_path = s,
                        "viewer_db" => self.state.viewer.db_path = s,
                        "chest_game_path" => self.state.shared_game_path = s,
                        "chest_map_file" => self.state.chest_editor.current_map_file = s,
                        "monster_ref_file" => {
                            let path = PathBuf::from(&s);
                            self.state.monster_ref_editor.select_file(path);
                        }
                        _ => {}
                    }
                }
                Task::none()
            }
            // Ref
            Message::RefOpSelected(op) => {
                self.state.ref_op = Some(op);
                Task::none()
            }
            Message::RefInputChanged(v) => {
                self.state.ref_input = v;
                Task::none()
            }
            // Database
            Message::DbOpSelected(op) => {
                self.state.db_op = Some(op);
                Task::none()
            }
            // Global
            Message::ExtractorPathChanged(v) => {
                self.state.extractor_path = v;
                Task::none()
            }
            Message::Run => {
                let Some(cmd) = self.build_internal_command() else {
                    self.state
                        .log
                        .push_str("⚠ No command configured or supported in GUI yet.\n");
                    return Task::none();
                };
                self.state.log.push_str(&format!(
                    "▸ Running internal command: {} [{}]\n",
                    cmd.name(),
                    cmd.description()
                ));
                self.state.is_running = true;

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
                self.state.is_running = false;
                match result {
                    Ok(output) => {
                        self.state.log.push_str(&output);
                        self.state.log.push_str("✔ Done.\n\n");
                    }
                    Err(err) => {
                        self.state.log.push_str(&format!("✖ Error: {}\n\n", err));
                    }
                }
                Task::none()
            }
            Message::ClearLog => {
                self.state.log.clear();
                Task::none()
            }

            // ─── Chest Editor messages ──────────────────────────────
            Message::ChestOpBrowseGamePath => browse_folder("shared_game_path"),
            Message::ChestOpBrowseMapFile => browse_file("chest_map_file"),
            Message::ChestOpScanMaps => {
                if self.state.shared_game_path.is_empty() {
                    self.state.chest_editor.status_msg = "Please select game path first.".into();
                    return Task::none();
                }
                self.state.chest_editor.is_loading = true;
                let path = PathBuf::from(&self.state.shared_game_path).join("ExtraInGame");
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
                self.state.chest_editor.is_loading = false;
                match res {
                    Ok(files) => {
                        self.state.chest_editor.map_files = files;
                        self.state.chest_editor.status_msg = format!(
                            "Found {} map files.",
                            self.state.chest_editor.map_files.len()
                        );
                    }
                    Err(e) => {
                        self.state.chest_editor.status_msg = format!("Error scanning maps: {}", e)
                    }
                }
                // Also load the catalog for human-friendly item names
                if self.state.shared_game_path.is_empty() {
                    Task::none()
                } else {
                    self.state.chest_editor.is_loading = true;
                    let path = PathBuf::from(&self.state.shared_game_path);
                    Task::perform(async move { ItemCatalog::load_from_folder(&path) }, |res| {
                        Message::ChestCatalogLoaded(res.map_err(|e| e.to_string()))
                    })
                }
            }

            Message::ChestOpLoadCatalog => {
                if self.state.shared_game_path.is_empty() {
                    self.state.chest_editor.status_msg = "Please select game path first.".into();
                    return Task::none();
                }
                self.state.chest_editor.is_loading = true;
                let path = PathBuf::from(&self.state.shared_game_path);
                Task::perform(async move { ItemCatalog::load_from_folder(&path) }, |res| {
                    Message::ChestCatalogLoaded(res.map_err(|e| e.to_string()))
                })
            }
            Message::ChestCatalogLoaded(res) => {
                self.state.chest_editor.is_loading = false;
                match res {
                    Ok(catalog) => {
                        self.state.chest_editor.catalog = Some(catalog);
                        self.state.chest_editor.status_msg = "Catalog loaded successfully.".into();
                    }
                    Err(e) => {
                        self.state.chest_editor.status_msg = format!("Error loading catalog: {}", e)
                    }
                }
                Task::none()
            }
            Message::ChestOpSelectMap => {
                if self.state.chest_editor.current_map_file.is_empty() {
                    self.state.chest_editor.status_msg = "No map file selected.".into();
                    return Task::none();
                }
                self.load_map_file(PathBuf::from(&self.state.chest_editor.current_map_file))
            }
            Message::ChestOpSelectMapFromFile(path) => {
                self.state.chest_editor.current_map_file = path.to_string_lossy().to_string();
                self.load_map_file(path)
            }
            Message::ChestMapLoaded(res) => {
                self.state.chest_editor.is_loading = false;
                match res {
                    Ok(records) => {
                        self.state.chest_editor.all_records = records;
                        self.state.chest_editor.status_msg = "Map loaded successfully.".into();
                        self.refresh_chests();
                    }
                    Err(e) => {
                        self.state.chest_editor.status_msg = format!("Error loading map: {}", e)
                    }
                }
                Task::none()
            }
            Message::ChestOpSelectChest(idx) => {
                self.state.chest_editor.selected_idx = Some(idx);
                if let Some((_, record)) = self.state.chest_editor.filtered_chests.get(idx) {
                    self.state.chest_editor.edit_name = record.name.clone();
                    self.state.chest_editor.edit_x = record.x_pos.to_string();
                    self.state.chest_editor.edit_y = record.y_pos.to_string();
                    self.state.chest_editor.edit_gold = record.gold_amount.to_string();
                    self.state.chest_editor.edit_item_count = record.item_count.to_string();
                    self.state.chest_editor.edit_item_id = record.item_id.to_string();
                    self.state.chest_editor.edit_item_type =
                        (u8::from(record.item_type_id)).to_string();
                    self.state.chest_editor.edit_closed = record.closed.to_string();
                }
                Task::none()
            }
            Message::ChestOpFieldChanged(orig_idx, field, val) => {
                match field.as_str() {
                    "name" => self.state.chest_editor.edit_name = val.clone(),
                    "x" => self.state.chest_editor.edit_x = val.clone(),
                    "y" => self.state.chest_editor.edit_y = val.clone(),
                    "gold" => self.state.chest_editor.edit_gold = val.clone(),
                    "item_count" => self.state.chest_editor.edit_item_count = val.clone(),
                    "item_id" => self.state.chest_editor.edit_item_id = val.clone(),
                    "item_type" => self.state.chest_editor.edit_item_type = val.clone(),
                    "closed" => self.state.chest_editor.edit_closed = val.clone(),
                    _ => {}
                }
                if let Some(record) = self.state.chest_editor.all_records.get_mut(orig_idx) {
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
                if self.state.chest_editor.current_map_file.is_empty()
                    || self.state.chest_editor.all_records.is_empty()
                {
                    return Task::none();
                }
                self.state.chest_editor.is_loading = true;

                let path = PathBuf::from(&self.state.chest_editor.current_map_file);

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

                let records = self.state.chest_editor.all_records.clone();
                Task::perform(
                    async move { dispel_core::ExtraRef::save_file(&records, &path) },
                    |res: Result<(), std::io::Error>| {
                        Message::ChestSaved(res.map_err(|e| e.to_string()))
                    },
                )
            }
            Message::ChestSaved(res) => {
                self.state.chest_editor.is_loading = false;
                match res {
                    Ok(_) => self.state.chest_editor.status_msg = "Map saved successfully.".into(),
                    Err(e) => {
                        self.state.chest_editor.status_msg = format!("Error saving map: {}", e)
                    }
                }
                Task::none()
            }
            Message::ChestOpAdd => Task::none(),
            Message::ChestOpDelete(_) => Task::none(),

            // ─── Weapon Editor messages ──────────────────────────────
            Message::WeaponOpBrowseGamePath => browse_folder("weapon_game_path"),
            Message::WeaponOpScanWeapons => {
                if self.state.shared_game_path.is_empty() {
                    self.state.weapon_editor.status_msg = "Please select game path first.".into();
                    return Task::none();
                }
                self.state.weapon_editor.is_loading = true;
                self.state.weapon_editor.status_msg = "Scanning weapons...".into();
                let path = PathBuf::from(&self.state.shared_game_path);
                Task::perform(
                    async move {
                        WeaponItem::read_file(&path.join("CharacterInGame").join("weaponItem.db"))
                            .map_err(|e: std::io::Error| e.to_string())
                    },
                    |res| Message::WeaponCatalogLoaded(res),
                )
            }
            Message::WeaponCatalogLoaded(res) => {
                self.state.weapon_editor.is_loading = false;
                match res {
                    Ok(catalog) => {
                        self.state.weapon_editor.catalog = Some(catalog.clone());
                        self.state.weapon_editor.status_msg =
                            format!("Weapon catalog loaded: {} weapons", catalog.len()).into();
                        self.state.weapon_editor.refresh();
                    }
                    Err(e) => {
                        self.state.weapon_editor.status_msg =
                            format!("Error loading weapon catalog: {}", e)
                    }
                }
                Task::none()
            }
            Message::WeaponOpSelectWeapon(idx) => {
                self.state.weapon_editor.select(idx);
                Task::none()
            }
            Message::WeaponOpFieldChanged(idx, field, val) => {
                self.state.weapon_editor.update_field(idx, &field, val);
                Task::none()
            }
            Message::WeaponOpSave => {
                if self.state.shared_game_path.is_empty() {
                    self.state.weapon_editor.status_msg = "Please select game path first.".into();
                    return Task::none();
                }
                self.state.weapon_editor.is_loading = true;
                let result = self.state.weapon_editor.save(
                    &self.state.shared_game_path,
                    "CharacterInGame/weaponItem.db",
                );
                self.state.weapon_editor.is_loading = false;
                match result {
                    Ok(_) => {
                        self.state.weapon_editor.status_msg = "Weapons saved successfully.".into()
                    }
                    Err(e) => {
                        self.state.weapon_editor.status_msg = format!("Error saving weapons: {}", e)
                    }
                }
                Task::none()
            }
            Message::HealItemOpBrowseGamePath => browse_folder("shared_game_path"),
            Message::HealItemOpBrowseSpritePath => browse_folder("heal_item_sprite_path"),
            Message::HealItemOpScanItems => {
                if self.state.shared_game_path.is_empty() {
                    self.state.heal_item_editor.status_msg =
                        "Please select game path first.".into();
                    return Task::none();
                }
                self.state.heal_item_editor.is_loading = true;
                self.state.heal_item_editor.status_msg = "Scanning heal items...".into();
                let path = PathBuf::from(&self.state.shared_game_path);
                Task::perform(
                    async move {
                        HealItem::read_file(&path.join("CharacterInGame").join("HealItem.db"))
                            .map_err(|e: std::io::Error| e.to_string())
                    },
                    |res| Message::HealItemCatalogLoaded(res),
                )
            }
            Message::HealItemCatalogLoaded(res) => {
                self.state.heal_item_editor.is_loading = false;
                match res {
                    Ok(catalog) => {
                        self.state.heal_item_editor.catalog = Some(catalog.clone());
                        self.state.heal_item_editor.status_msg =
                            format!("Heal item catalog loaded: {} items", catalog.len()).into();
                        self.state.heal_item_editor.refresh();
                    }
                    Err(e) => {
                        self.state.heal_item_editor.status_msg =
                            format!("Error loading heal item catalog: {}", e)
                    }
                }
                Task::none()
            }
            Message::HealItemOpSelectItem(idx) => {
                self.state.heal_item_editor.select(idx);
                Task::none()
            }
            Message::HealItemOpFieldChanged(idx, field, val) => {
                self.state.heal_item_editor.update_field(idx, &field, val);
                Task::none()
            }
            Message::HealItemOpSave => {
                if self.state.shared_game_path.is_empty() {
                    self.state.heal_item_editor.status_msg =
                        "Please select game path first.".into();
                    return Task::none();
                }
                // Pre-save validation
                let validation_errors = self.state.heal_item_editor.validate_all();
                if !validation_errors.is_empty() {
                    let error_summary: Vec<String> = validation_errors
                        .iter()
                        .take(5)
                        .map(|(idx, errs)| {
                            let record_label = format!("Record #{}", idx);
                            let field_errors: String = errs
                                .iter()
                                .map(|(f, e)| format!("{}: {}", f, e))
                                .collect::<Vec<_>>()
                                .join(", ");
                            format!("{}: {}", record_label, field_errors)
                        })
                        .collect();
                    let error_msg = if validation_errors.len() > 5 {
                        format!(
                            "Found {} records with validation errors:\n{}\n... and {} more",
                            validation_errors.len(),
                            error_summary.join("\n"),
                            validation_errors.len() - 5
                        )
                    } else {
                        format!(
                            "Found {} records with validation errors:\n{}",
                            validation_errors.len(),
                            error_summary.join("\n")
                        )
                    };
                    use rfd::MessageDialog;
                    MessageDialog::new()
                        .set_title("Validation Errors")
                        .set_description(&error_msg)
                        .set_buttons(rfd::MessageButtons::Ok)
                        .show();
                    return Task::none();
                }
                self.state.heal_item_editor.is_loading = true;
                let result = self
                    .state
                    .heal_item_editor
                    .save(&self.state.shared_game_path, "CharacterInGame/HealItem.db");
                self.state.heal_item_editor.is_loading = false;
                match result {
                    Ok(_) => {
                        self.state.heal_item_editor.status_msg =
                            "Heal items saved successfully.".into()
                    }
                    Err(e) => {
                        self.state.heal_item_editor.status_msg =
                            format!("Error saving heal items: {}", e)
                    }
                }
                Task::none()
            }
            Message::MiscItemOpBrowseGamePath => browse_folder("shared_game_path"),
            Message::MiscItemOpLoadCatalog | Message::MiscItemOpScanItems => {
                if self.state.shared_game_path.is_empty() {
                    self.state.misc_item_editor.status_msg =
                        "Please select game path first.".into();
                    return Task::none();
                }
                self.state.misc_item_editor.is_loading = true;
                let path = PathBuf::from(&self.state.shared_game_path);
                Task::perform(
                    async move {
                        MiscItem::read_file(&path.join("CharacterInGame").join("MiscItem.db"))
                            .map_err(|e: std::io::Error| e.to_string())
                    },
                    |res| Message::MiscItemCatalogLoaded(res),
                )
            }
            Message::MiscItemCatalogLoaded(res) => {
                self.state.misc_item_editor.is_loading = false;
                match res {
                    Ok(catalog) => {
                        self.state.misc_item_editor.catalog = Some(catalog.clone());
                        self.state.misc_item_editor.status_msg =
                            format!("Misc item catalog loaded: {} items", catalog.len()).into();
                        self.state.misc_item_editor.refresh();
                    }
                    Err(e) => {
                        self.state.misc_item_editor.status_msg =
                            format!("Error loading misc item catalog: {}", e)
                    }
                }
                Task::none()
            }
            Message::MiscItemOpSelectItem(idx) => {
                self.state.misc_item_editor.select(idx);
                Task::none()
            }
            Message::MiscItemOpFieldChanged(idx, field, val) => {
                self.state.misc_item_editor.update_field(idx, &field, val);
                Task::none()
            }
            Message::MiscItemOpSave => {
                if self.state.shared_game_path.is_empty() {
                    self.state.misc_item_editor.status_msg =
                        "Please select game path first.".into();
                    return Task::none();
                }
                self.state.misc_item_editor.is_loading = true;
                let result = self
                    .state
                    .misc_item_editor
                    .save(&self.state.shared_game_path, "CharacterInGame/MiscItem.db");
                self.state.misc_item_editor.is_loading = false;
                match result {
                    Ok(_) => {
                        self.state.misc_item_editor.status_msg =
                            "Misc items saved successfully.".into()
                    }
                    Err(e) => {
                        self.state.misc_item_editor.status_msg =
                            format!("Error saving misc items: {}", e)
                    }
                }
                Task::none()
            }
            Message::EditItemOpBrowseGamePath => browse_folder("shared_game_path"),
            Message::EditItemOpLoadCatalog | Message::EditItemOpScanItems => {
                if self.state.shared_game_path.is_empty() {
                    self.state.edit_item_editor.status_msg =
                        "Please select game path first.".into();
                    return Task::none();
                }
                self.state.edit_item_editor.is_loading = true;
                let path = PathBuf::from(&self.state.shared_game_path);
                Task::perform(
                    async move {
                        EditItem::read_file(&path.join("CharacterInGame").join("EditItem.db"))
                            .map_err(|e: std::io::Error| e.to_string())
                    },
                    |res| Message::EditItemCatalogLoaded(res),
                )
            }
            Message::EditItemCatalogLoaded(res) => {
                self.state.edit_item_editor.is_loading = false;
                match res {
                    Ok(catalog) => {
                        self.state.edit_item_editor.catalog = Some(catalog.clone());
                        self.state.edit_item_editor.status_msg =
                            format!("Edit item catalog loaded: {} items", catalog.len()).into();
                        self.state.edit_item_editor.refresh();
                    }
                    Err(e) => {
                        self.state.edit_item_editor.status_msg =
                            format!("Error loading edit item catalog: {}", e)
                    }
                }
                Task::none()
            }
            Message::EditItemOpSelectItem(idx) => {
                self.state.edit_item_editor.select(idx);
                Task::none()
            }
            Message::EditItemOpFieldChanged(idx, field, val) => {
                self.state.edit_item_editor.update_field(idx, &field, val);
                Task::none()
            }
            Message::EditItemOpSave => {
                if self.state.shared_game_path.is_empty() {
                    self.state.edit_item_editor.status_msg =
                        "Please select game path first.".into();
                    return Task::none();
                }
                self.state.edit_item_editor.is_loading = true;
                let result = self
                    .state
                    .edit_item_editor
                    .save(&self.state.shared_game_path, "CharacterInGame/EditItem.db");
                self.state.edit_item_editor.is_loading = false;
                match result {
                    Ok(_) => {
                        self.state.edit_item_editor.status_msg =
                            "Edit items saved successfully.".into()
                    }
                    Err(e) => {
                        self.state.edit_item_editor.status_msg =
                            format!("Error saving edit items: {}", e)
                    }
                }
                Task::none()
            }
            Message::EventItemOpBrowseGamePath => browse_folder("shared_game_path"),
            Message::EventItemOpLoadCatalog | Message::EventItemOpScanItems => {
                if self.state.shared_game_path.is_empty() {
                    self.state.event_item_editor.status_msg =
                        "Please select game path first.".into();
                    return Task::none();
                }
                self.state.event_item_editor.is_loading = true;
                let path = PathBuf::from(&self.state.shared_game_path);
                Task::perform(
                    async move {
                        EventItem::read_file(&path.join("CharacterInGame").join("EventItem.db"))
                            .map_err(|e: std::io::Error| e.to_string())
                    },
                    |res| Message::EventItemCatalogLoaded(res),
                )
            }
            Message::EventItemCatalogLoaded(res) => {
                self.state.event_item_editor.is_loading = false;
                match res {
                    Ok(catalog) => {
                        self.state.event_item_editor.catalog = Some(catalog.clone());
                        self.state.event_item_editor.status_msg =
                            format!("Event item catalog loaded: {} items", catalog.len()).into();
                        self.state.event_item_editor.refresh();
                    }
                    Err(e) => {
                        self.state.event_item_editor.status_msg =
                            format!("Error loading event item catalog: {}", e)
                    }
                }
                Task::none()
            }
            Message::EventItemOpSelectItem(idx) => {
                self.state.event_item_editor.select(idx);
                Task::none()
            }
            Message::EventItemOpFieldChanged(idx, field, val) => {
                self.state.event_item_editor.update_field(idx, &field, val);
                Task::none()
            }
            Message::EventItemOpSave => {
                if self.state.shared_game_path.is_empty() {
                    self.state.event_item_editor.status_msg =
                        "Please select game path first.".into();
                    return Task::none();
                }
                self.state.event_item_editor.is_loading = true;
                let result = self
                    .state
                    .event_item_editor
                    .save(&self.state.shared_game_path, "CharacterInGame/EventItem.db");
                self.state.event_item_editor.is_loading = false;
                match result {
                    Ok(_) => {
                        self.state.event_item_editor.status_msg =
                            "Event items saved successfully.".into()
                    }
                    Err(e) => {
                        self.state.event_item_editor.status_msg =
                            format!("Error saving event items: {}", e)
                    }
                }
                Task::none()
            }
            Message::MonsterOpBrowseGamePath => browse_folder("shared_game_path"),
            Message::MonsterOpLoadCatalog | Message::MonsterOpScanMonsters => {
                if self.state.shared_game_path.is_empty() {
                    self.state.monster_editor.status_msg = "Please select game path first.".into();
                    return Task::none();
                }
                self.state.monster_editor.is_loading = true;
                let path = PathBuf::from(&self.state.shared_game_path);
                Task::perform(
                    async move {
                        Monster::read_file(&path.join("MonsterInGame").join("Monster.db"))
                            .map_err(|e: std::io::Error| e.to_string())
                    },
                    |res| Message::MonsterCatalogLoaded(res),
                )
            }
            Message::MonsterCatalogLoaded(res) => {
                self.state.monster_editor.is_loading = false;
                match res {
                    Ok(catalog) => {
                        self.state.monster_editor.catalog = Some(catalog.clone());
                        self.state.monster_editor.status_msg =
                            format!("Monster catalog loaded: {} monsters", catalog.len()).into();
                        self.state.monster_editor.refresh_monsters();
                    }
                    Err(e) => {
                        self.state.monster_editor.status_msg =
                            format!("Error loading monster catalog: {}", e)
                    }
                }
                Task::none()
            }
            Message::MonsterOpSelectMonster(idx) => {
                self.state.monster_editor.select_monster(idx);
                Task::none()
            }
            Message::MonsterOpFieldChanged(idx, field, val) => {
                self.state.monster_editor.update_field(idx, &field, val);
                Task::none()
            }
            Message::MonsterOpSave => {
                if self.state.shared_game_path.is_empty() {
                    self.state.monster_editor.status_msg = "Please select game path first.".into();
                    return Task::none();
                }
                self.state.monster_editor.is_loading = true;
                let result = self
                    .state
                    .monster_editor
                    .save_monsters(&self.state.shared_game_path);
                self.state.monster_editor.is_loading = false;
                match result {
                    Ok(_) => {
                        self.state.monster_editor.status_msg = "Monsters saved successfully.".into()
                    }
                    Err(e) => {
                        self.state.monster_editor.status_msg =
                            format!("Error saving monsters: {}", e)
                    }
                }
                Task::none()
            }
            Message::NpcIniOpBrowseGamePath => browse_folder("shared_game_path"),
            Message::NpcIniOpLoadCatalog | Message::NpcIniOpScanNpcs => {
                if self.state.shared_game_path.is_empty() {
                    self.state.npc_ini_editor.status_msg = "Please select game path first.".into();
                    return Task::none();
                }
                self.state.npc_ini_editor.is_loading = true;
                let path = PathBuf::from(&self.state.shared_game_path);
                Task::perform(
                    async move {
                        NpcIni::read_file(&path.join("Npc.ini"))
                            .map_err(|e: std::io::Error| e.to_string())
                    },
                    |res| Message::NpcIniCatalogLoaded(res),
                )
            }
            Message::NpcIniCatalogLoaded(res) => {
                self.state.npc_ini_editor.is_loading = false;
                match res {
                    Ok(catalog) => {
                        self.state.npc_ini_editor.catalog = Some(catalog.clone());
                        self.state.npc_ini_editor.status_msg =
                            format!("NPC catalog loaded: {} npcs", catalog.len()).into();
                        self.state.npc_ini_editor.refresh_npcs();
                    }
                    Err(e) => {
                        self.state.npc_ini_editor.status_msg =
                            format!("Error loading NPC catalog: {}", e)
                    }
                }
                Task::none()
            }
            Message::NpcIniOpSelectNpc(idx) => {
                self.state.npc_ini_editor.select_npc(idx);
                Task::none()
            }
            Message::NpcIniOpFieldChanged(idx, field, val) => {
                self.state.npc_ini_editor.update_field(idx, &field, val);
                Task::none()
            }
            Message::NpcIniOpSave => {
                if self.state.shared_game_path.is_empty() {
                    self.state.npc_ini_editor.status_msg = "Please select game path first.".into();
                    return Task::none();
                }
                self.state.npc_ini_editor.is_loading = true;
                let result = self
                    .state
                    .npc_ini_editor
                    .save_npcs(&self.state.shared_game_path);
                self.state.npc_ini_editor.is_loading = false;
                match result {
                    Ok(_) => {
                        self.state.npc_ini_editor.status_msg = "NPCs saved successfully.".into()
                    }
                    Err(e) => {
                        self.state.npc_ini_editor.status_msg = format!("Error saving NPCs: {}", e)
                    }
                }
                Task::none()
            }
            Message::MagicOpBrowseGamePath => browse_folder("shared_game_path"),
            Message::MagicOpLoadCatalog | Message::MagicOpScanSpells => {
                if self.state.shared_game_path.is_empty() {
                    self.state.magic_editor.status_msg = "Please select game path first.".into();
                    return Task::none();
                }
                self.state.magic_editor.is_loading = true;
                let path = PathBuf::from(&self.state.shared_game_path);
                Task::perform(
                    async move {
                        MagicSpell::read_file(&path.join("MagicInGame").join("Magic.db"))
                            .map_err(|e: std::io::Error| e.to_string())
                    },
                    |res| Message::MagicCatalogLoaded(res),
                )
            }
            Message::MagicCatalogLoaded(res) => {
                self.state.magic_editor.is_loading = false;
                match res {
                    Ok(catalog) => {
                        self.state.magic_editor.catalog = Some(catalog.clone());
                        self.state.magic_editor.status_msg =
                            format!("Magic catalog loaded: {} spells", catalog.len()).into();
                        self.state.magic_editor.refresh();
                    }
                    Err(e) => {
                        self.state.magic_editor.status_msg =
                            format!("Error loading magic catalog: {}", e)
                    }
                }
                Task::none()
            }
            Message::MagicOpSelectSpell(idx) => {
                self.state.magic_editor.select(idx);
                Task::none()
            }
            Message::MagicOpFieldChanged(idx, field, val) => {
                self.state.magic_editor.update_field(idx, &field, val);
                Task::none()
            }
            Message::MagicOpSave => {
                if self.state.shared_game_path.is_empty() {
                    self.state.magic_editor.status_msg = "Please select game path first.".into();
                    return Task::none();
                }
                self.state.magic_editor.is_loading = true;
                let result = self
                    .state
                    .magic_editor
                    .save(&self.state.shared_game_path, "MagicInGame/Magic.db");
                self.state.magic_editor.is_loading = false;
                match result {
                    Ok(_) => {
                        self.state.magic_editor.status_msg = "Spells saved successfully.".into()
                    }
                    Err(e) => {
                        self.state.magic_editor.status_msg = format!("Error saving spells: {}", e)
                    }
                }
                Task::none()
            }
            Message::StoreOpBrowseGamePath => browse_folder("shared_game_path"),
            Message::StoreOpLoadCatalog | Message::StoreOpScanStores => {
                if self.state.shared_game_path.is_empty() {
                    self.state.store_editor.status_msg = "Please select game path first.".into();
                    return Task::none();
                }
                self.state.store_editor.is_loading = true;
                self.state.store_editor.status_msg = "Loading item catalogs...".into();
                let path = PathBuf::from(&self.state.shared_game_path);
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
                        self.state.weapon_editor.catalog = weapons.clone();
                        self.state.weapon_editor.refresh();
                        self.state.heal_item_editor.catalog = heals.clone();
                        self.state.heal_item_editor.refresh();
                        self.state.misc_item_editor.catalog = misc.clone();
                        self.state.misc_item_editor.refresh();
                        self.state.edit_item_editor.catalog = edit.clone();
                        self.state.edit_item_editor.refresh();
                        self.state.store_editor.catalog = Some(stores.clone());
                        let weapons_count = weapons.as_ref().map(|w| w.len()).unwrap_or(0);
                        let heals_count = heals.as_ref().map(|h| h.len()).unwrap_or(0);
                        let misc_count = misc.as_ref().map(|m| m.len()).unwrap_or(0);
                        let edit_count = edit.as_ref().map(|e| e.len()).unwrap_or(0);
                        self.state.store_editor.status_msg = format!(
                            "Loaded: {} stores, {} weapons, {} heals, {} misc, {} edit items",
                            stores.len(),
                            weapons_count,
                            heals_count,
                            misc_count,
                            edit_count
                        )
                        .into();
                        self.state.store_editor.refresh_stores();
                    }
                    Err(e) => {
                        self.state.store_editor.status_msg =
                            format!("Error loading store catalog: {}", e)
                    }
                }
                self.state.store_editor.is_loading = false;
                Task::none()
            }
            Message::StoreCatalogLoaded(res) => {
                self.state.store_editor.is_loading = false;
                match res {
                    Ok(catalog) => {
                        self.state.store_editor.catalog = Some(catalog.clone());
                        self.state.store_editor.status_msg =
                            format!("Store catalog loaded: {} stores", catalog.len()).into();
                        self.state.store_editor.refresh_stores();
                    }
                    Err(e) => {
                        self.state.store_editor.status_msg =
                            format!("Error loading store catalog: {}", e)
                    }
                }
                Task::none()
            }
            Message::StoreOpSelectStore(idx) => {
                self.state.store_editor.select_store(idx);
                Task::none()
            }
            Message::StoreOpFieldChanged(idx, field, val) => {
                self.state.store_editor.update_field(idx, &field, val);
                Task::none()
            }
            Message::StoreOpSelectProduct(idx) => {
                self.state.store_editor.select_product(idx);
                Task::none()
            }
            Message::StoreOpAddProduct => {
                self.state.store_editor.add_product();
                Task::none()
            }
            Message::StoreOpRemoveProduct(idx) => {
                self.state.store_editor.remove_product(idx);
                Task::none()
            }
            Message::StoreOpProductFieldChanged(prod_idx, field, val) => {
                self.state
                    .store_editor
                    .update_product(prod_idx, &field, val);
                Task::none()
            }
            Message::StoreOpSave => {
                if self.state.shared_game_path.is_empty() {
                    self.state.store_editor.status_msg = "Please select game path first.".into();
                    return Task::none();
                }
                self.state.store_editor.is_loading = true;
                let result = self
                    .state
                    .store_editor
                    .save_stores(&self.state.shared_game_path);
                self.state.store_editor.is_loading = false;
                match result {
                    Ok(_) => {
                        self.state.store_editor.status_msg = "Stores saved successfully.".into()
                    }
                    Err(e) => {
                        self.state.store_editor.status_msg = format!("Error saving stores: {}", e)
                    }
                }
                Task::none()
            }
            Message::PartyRefOpBrowseGamePath => browse_folder("shared_game_path"),
            Message::PartyRefOpLoadCatalog | Message::PartyRefOpScanParty => {
                if self.state.shared_game_path.is_empty() {
                    self.state.party_ref_editor.status_msg =
                        "Please select game path first.".into();
                    return Task::none();
                }
                self.state.party_ref_editor.is_loading = true;
                let path = PathBuf::from(&self.state.shared_game_path);
                Task::perform(
                    async move {
                        PartyRef::read_file(&path.join("Ref").join("PartyRef.ref"))
                            .map_err(|e: std::io::Error| e.to_string())
                    },
                    |res| Message::PartyRefCatalogLoaded(res),
                )
            }
            Message::PartyRefCatalogLoaded(res) => {
                self.state.party_ref_editor.is_loading = false;
                match res {
                    Ok(catalog) => {
                        self.state.party_ref_editor.catalog = Some(catalog.clone());
                        self.state.party_ref_editor.status_msg =
                            format!("Party catalog loaded: {} members", catalog.len()).into();
                        self.state.party_ref_editor.refresh();
                    }
                    Err(e) => {
                        self.state.party_ref_editor.status_msg =
                            format!("Error loading party catalog: {}", e)
                    }
                }
                Task::none()
            }
            Message::PartyRefOpSelectMember(idx) => {
                self.state.party_ref_editor.select(idx);
                Task::none()
            }
            Message::PartyRefOpFieldChanged(idx, field, val) => {
                self.state.party_ref_editor.update_field(idx, &field, val);
                Task::none()
            }
            Message::PartyRefOpSave => {
                if self.state.shared_game_path.is_empty() {
                    self.state.party_ref_editor.status_msg =
                        "Please select game path first.".into();
                    return Task::none();
                }
                self.state.party_ref_editor.is_loading = true;
                let result = self
                    .state
                    .party_ref_editor
                    .save(&self.state.shared_game_path, "PartyRef.ref");
                self.state.party_ref_editor.is_loading = false;
                match result {
                    Ok(_) => {
                        self.state.party_ref_editor.status_msg = "Party saved successfully.".into()
                    }
                    Err(e) => {
                        self.state.party_ref_editor.status_msg =
                            format!("Error saving party: {}", e)
                    }
                }
                Task::none()
            }
            Message::PartyIniOpBrowseGamePath => browse_folder("shared_game_path"),
            Message::PartyIniOpLoadCatalog | Message::PartyIniOpScanNpcs => {
                if self.state.shared_game_path.is_empty() {
                    self.state.party_ini_editor.status_msg =
                        "Please select game path first.".into();
                    return Task::none();
                }
                self.state.party_ini_editor.is_loading = true;
                let path = PathBuf::from(&self.state.shared_game_path);
                Task::perform(
                    async move {
                        PartyIniNpc::read_file(&path.join("NpcInGame").join("PrtIni.db"))
                            .map_err(|e: std::io::Error| e.to_string())
                    },
                    |res| Message::PartyIniCatalogLoaded(res),
                )
            }
            Message::PartyIniCatalogLoaded(res) => {
                self.state.party_ini_editor.is_loading = false;
                match res {
                    Ok(catalog) => {
                        self.state.party_ini_editor.catalog = Some(catalog.clone());
                        self.state.party_ini_editor.status_msg =
                            format!("Party ini catalog loaded: {} npcs", catalog.len()).into();
                        self.state.party_ini_editor.refresh();
                    }
                    Err(e) => {
                        self.state.party_ini_editor.status_msg =
                            format!("Error loading party ini catalog: {}", e)
                    }
                }
                Task::none()
            }
            Message::PartyIniOpSelectNpc(idx) => {
                self.state.party_ini_editor.select(idx);
                Task::none()
            }
            Message::PartyIniOpFieldChanged(idx, field, val) => {
                self.state.party_ini_editor.update_field(idx, &field, val);
                Task::none()
            }
            Message::PartyIniOpSave => {
                if self.state.shared_game_path.is_empty() {
                    self.state.party_ini_editor.status_msg =
                        "Please select game path first.".into();
                    return Task::none();
                }
                self.state.party_ini_editor.is_loading = true;
                let result = self
                    .state
                    .party_ini_editor
                    .save_npcs(&self.state.shared_game_path);
                self.state.party_ini_editor.is_loading = false;
                match result {
                    Ok(_) => {
                        self.state.party_ini_editor.status_msg =
                            "Party ini saved successfully.".into()
                    }
                    Err(e) => {
                        self.state.party_ini_editor.status_msg =
                            format!("Error saving party ini: {}", e)
                    }
                }
                Task::none()
            }

            // ─── DB Viewer messages ─────────────────────────────────
            Message::ViewerDbPathChanged(v) => {
                self.state.viewer.db_path = v;
                Task::none()
            }
            Message::ViewerBrowseDb => crate::utils::browse_file("viewer_db"),
            Message::ViewerConnect => {
                self.state.viewer.is_loading = true;
                self.state.viewer.status_msg = "Connecting…".into();
                let path = self.state.viewer.db_path.clone();
                Task::perform(
                    async move { db::list_tables(&path) },
                    Message::ViewerTablesLoaded,
                )
            }
            Message::ViewerTablesLoaded(result) => {
                self.state.viewer.is_loading = false;
                match result {
                    Ok(tables) => {
                        self.state.viewer.status_msg =
                            format!("Connected – {} tables", tables.len());
                        self.state.viewer.tables = tables;
                        self.state.viewer.active_table = None;
                        self.state.viewer.rows.clear();
                        self.state.viewer.columns.clear();
                    }
                    Err(e) => {
                        self.state.viewer.status_msg = format!("✖ {}", e);
                    }
                }
                Task::none()
            }
            Message::ViewerSelectTable(t) => {
                self.state.viewer.active_table = Some(t.clone());
                self.state.viewer.page = 0;
                self.state.viewer.search.clear();
                self.state.viewer.sort_col = None;
                self.state.viewer.pending_edits.clear();
                self.state.viewer.editing_cell = None;
                self.state.viewer.sql_mode = false;
                self.state.viewer.sql_query = format!("SELECT * FROM \"{}\"", t);
                self.fetch_viewer_data()
            }
            Message::ViewerDataLoaded(result) => {
                self.state.viewer.is_loading = false;
                match result {
                    Ok(qr) => {
                        self.state.viewer.columns = qr.columns;
                        self.state.viewer.rows = qr.rows;
                        self.state.viewer.total_rows = qr.total_rows;
                        let page_start = self.state.viewer.page * PAGE_SIZE + 1;
                        let page_end =
                            (page_start - 1 + self.state.viewer.rows.len()).max(page_start - 1);
                        self.state.viewer.status_msg = format!(
                            "Showing {}-{} of {} rows",
                            page_start, page_end, self.state.viewer.total_rows
                        );
                    }
                    Err(e) => {
                        self.state.viewer.status_msg = format!("✖ Query error: {}", e);
                    }
                }
                Task::none()
            }
            Message::ViewerSearch(v) => {
                self.state.viewer.search = v;
                self.state.viewer.page = 0;
                self.fetch_viewer_data()
            }
            Message::ViewerSortColumn(idx) => {
                if self.state.viewer.sort_col == Some(idx) {
                    self.state.viewer.sort_dir = self.state.viewer.sort_dir.toggle();
                } else {
                    self.state.viewer.sort_col = Some(idx);
                    self.state.viewer.sort_dir = db::SortDir::Asc;
                }
                self.state.viewer.page = 0;
                self.fetch_viewer_data()
            }
            Message::ViewerNextPage => {
                let max_page = self.state.viewer.total_rows.saturating_sub(1) / PAGE_SIZE;
                if self.state.viewer.page < max_page {
                    self.state.viewer.page += 1;
                    return self.fetch_viewer_data();
                }
                Task::none()
            }
            Message::ViewerPrevPage => {
                if self.state.viewer.page > 0 {
                    self.state.viewer.page -= 1;
                    return self.fetch_viewer_data();
                }
                Task::none()
            }
            Message::ViewerCellClick(r, c) => {
                // Confirm previous edit if any
                if let Some((pr, pc)) = self.state.viewer.editing_cell {
                    if !self.state.viewer.edit_buffer.is_empty()
                        || self
                            .state
                            .viewer
                            .rows
                            .get(pr)
                            .and_then(|row| row.get(pc).map(|v| v.as_str()))
                            != Some(&self.state.viewer.edit_buffer)
                    {
                        let original = self
                            .state
                            .viewer
                            .rows
                            .get(pr)
                            .and_then(|row| row.get(pc))
                            .cloned()
                            .unwrap_or_default();
                        if self.state.viewer.edit_buffer != original {
                            self.state
                                .viewer
                                .pending_edits
                                .insert((pr, pc), self.state.viewer.edit_buffer.clone());
                        }
                    }
                }
                let val = self
                    .state
                    .viewer
                    .rows
                    .get(r)
                    .and_then(|row| row.get(c))
                    .cloned()
                    .unwrap_or_default();
                self.state.viewer.editing_cell = Some((r, c));
                self.state.viewer.edit_buffer = val;
                Task::none()
            }
            Message::ViewerCellEdit(v) => {
                self.state.viewer.edit_buffer = v;
                Task::none()
            }
            Message::ViewerCellConfirm => {
                if let Some((r, c)) = self.state.viewer.editing_cell {
                    let original = self
                        .state
                        .viewer
                        .rows
                        .get(r)
                        .and_then(|row| row.get(c))
                        .cloned()
                        .unwrap_or_default();
                    if self.state.viewer.edit_buffer != original {
                        self.state
                            .viewer
                            .pending_edits
                            .insert((r, c), self.state.viewer.edit_buffer.clone());
                    }
                }
                self.state.viewer.editing_cell = None;
                Task::none()
            }
            Message::ViewerCellCancel => {
                self.state.viewer.editing_cell = None;
                Task::none()
            }
            Message::ViewerCommit => {
                if self.state.viewer.pending_edits.is_empty() {
                    self.state.viewer.status_msg = "Nothing to commit.".into();
                    return Task::none();
                }
                let path = self.state.viewer.db_path.clone();
                let table = self.state.viewer.active_table.clone().unwrap_or_default();
                let cols = self.state.viewer.columns.clone();
                let rows = self.state.viewer.rows.clone();
                let edits = self.state.viewer.pending_edits.clone();
                self.state.viewer.is_loading = true;
                Task::perform(
                    async move { db::commit_edits(&path, &table, &cols, &rows, &edits) },
                    Message::ViewerCommitDone,
                )
            }
            Message::ViewerCommitDone(result) => {
                self.state.viewer.is_loading = false;
                match result {
                    Ok(n) => {
                        // Apply edits to local rows
                        for ((r, c), val) in &self.state.viewer.pending_edits {
                            if let Some(row) = self.state.viewer.rows.get_mut(*r) {
                                if let Some(cell) = row.get_mut(*c) {
                                    *cell = val.clone();
                                }
                            }
                        }
                        self.state.viewer.pending_edits.clear();
                        self.state.viewer.status_msg = format!("✔ Committed {} row(s)", n);
                    }
                    Err(e) => {
                        self.state.viewer.status_msg = format!("✖ Commit failed: {}", e);
                    }
                }
                Task::none()
            }
            Message::ViewerToggleSql => {
                self.state.viewer.sql_mode = !self.state.viewer.sql_mode;
                Task::none()
            }
            Message::ViewerSqlChanged(v) => {
                self.state.viewer.sql_query = v;
                Task::none()
            }
            Message::ViewerRunSql => {
                self.state.viewer.page = 0;
                self.state.viewer.pending_edits.clear();
                self.state.viewer.editing_cell = None;
                self.fetch_viewer_data_sql()
            }
            Message::ViewerExportCsv => {
                let cols = self.state.viewer.columns.clone();
                let rows = self.state.viewer.rows.clone();
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
                    Ok(p) => self.state.viewer.status_msg = format!("✔ Exported to {}", p),
                    Err(e) => self.state.viewer.status_msg = format!("✖ Export: {}", e),
                }
                Task::none()
            }
            Message::ViewerRevertEdits => {
                self.state.viewer.pending_edits.clear();
                self.state.viewer.editing_cell = None;
                self.state.viewer.status_msg = "Reverted all pending edits.".into();
                Task::none()
            }
            // Sprite Browser
            Message::SpriteBrowserOpBrowsePath => browse_folder("shared_game_path"),
            Message::SpriteBrowserOpScan => {
                if self.state.shared_game_path.is_empty() {
                    self.state.sprite_browser.status_msg = "Please select game path first.".into();
                    return Task::none();
                }
                self.state.sprite_browser.is_loading = true;
                self.state.sprite_browser.status_msg = "Scanning for sprites...".into();
                let path = PathBuf::from(&self.state.shared_game_path);
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
                self.state.sprite_browser.is_loading = false;
                match res {
                    Ok(entries) => {
                        self.state.sprite_browser.sprites = entries;
                        self.state.sprite_browser.filter_sprites();
                        self.state.sprite_browser.status_msg = format!(
                            "Found {} sprite files",
                            self.state.sprite_browser.sprites.len()
                        )
                        .into();
                    }
                    Err(e) => {
                        self.state.sprite_browser.status_msg =
                            format!("Error scanning sprites: {}", e).into();
                    }
                }
                Task::none()
            }
            Message::SpriteBrowserOpSearch(query) => {
                self.state.sprite_browser.search_query = query;
                self.state.sprite_browser.filter_sprites();
                Task::none()
            }
            Message::SpriteBrowserOpSelectSprite(filtered_idx) => {
                self.state
                    .sprite_browser
                    .select_sprite_filtered(filtered_idx);
                Task::none()
            }
            Message::SpriteBrowserOpSelectSequence(seq_idx) => {
                self.state.sprite_browser.select_sequence(seq_idx);
                Task::none()
            }
            Message::SpriteBrowserOpSelectFrame(frame_idx) => {
                self.state.sprite_browser.select_frame(frame_idx);
                Task::none()
            }
            // Monster Ref Editor
            Message::MonsterRefOpBrowseFile => crate::utils::browse_file("monster_ref_file"),
            Message::MonsterRefOpSelectFile(path) => {
                self.state.monster_ref_editor.select_file(path);
                Task::none()
            }
            Message::MonsterRefOpScanFiles => {
                if self.state.shared_game_path.is_empty() {
                    self.state.monster_ref_editor.editor.status_msg =
                        "Please select game path first.".into();
                    return Task::none();
                }
                let path = PathBuf::from(&self.state.shared_game_path).join("MonsterInGame");
                self.state.monster_ref_editor.scan_files(&path, "Mon*.ref");
                self.state.monster_ref_editor.editor.status_msg = format!(
                    "Found {} monster ref files",
                    self.state.monster_ref_editor.file_list.len()
                );
                // Load monster names for the lookup dropdown if not already loaded
                if !self.state.lookups.contains_key("monster_names") {
                    return Task::perform(
                        async { Message::MonsterRefOpLoadMonsterNames },
                        std::convert::identity,
                    );
                }
                Task::none()
            }
            Message::MonsterRefOpSelectEntry(idx) => {
                self.state.monster_ref_editor.select(idx);
                Task::none()
            }
            Message::MonsterRefOpAddEntry => {
                self.state.monster_ref_editor.add_record();
                Task::none()
            }
            Message::MonsterRefOpRemoveEntry(idx) => {
                self.state.monster_ref_editor.remove_record(idx);
                Task::none()
            }
            Message::MonsterRefOpFieldChanged(idx, field, val) => {
                self.state.monster_ref_editor.update_field(idx, &field, val);
                Task::none()
            }
            Message::MonsterRefOpSave => {
                self.state.monster_ref_editor.editor.is_loading = true;
                let result = self.state.monster_ref_editor.save();
                self.state.monster_ref_editor.editor.is_loading = false;
                match result {
                    Ok(_) => {
                        self.state.monster_ref_editor.editor.status_msg =
                            "Monster ref saved successfully.".into()
                    }
                    Err(e) => {
                        self.state.monster_ref_editor.editor.status_msg =
                            format!("Error saving monster ref: {}", e)
                    }
                }
                Task::none()
            }
            Message::MonsterRefCatalogLoaded(res) => {
                self.state.monster_ref_editor.editor.is_loading = false;
                match res {
                    Ok(catalog) => {
                        self.state.monster_ref_editor.editor.catalog = Some(catalog.clone());
                        self.state.monster_ref_editor.editor.status_msg =
                            format!("Monster ref loaded: {} entries", catalog.len()).into();
                        self.state.monster_ref_editor.editor.refresh();
                    }
                    Err(e) => {
                        self.state.monster_ref_editor.editor.status_msg =
                            format!("Error loading monster ref: {}", e)
                    }
                }
                Task::none()
            }
            Message::MonsterRefOpLoadMonsterNames => {
                if self.state.shared_game_path.is_empty() {
                    return Task::none();
                }
                let path = PathBuf::from(&self.state.shared_game_path).join("Monster.ini");
                Task::perform(
                    async move {
                        MonsterIni::read_file(&path)
                            .map(|monsters| {
                                monsters
                                    .iter()
                                    .map(|m| (m.id.to_string(), m.name.clone().unwrap_or_default()))
                                    .collect()
                            })
                            .map_err(|e| e.to_string())
                    },
                    Message::MonsterNamesLoaded,
                )
            }
            Message::MonsterNamesLoaded(res) => {
                match res {
                    Ok(names) => {
                        self.state
                            .lookups
                            .insert("monster_names".to_string(), names);
                    }
                    Err(e) => {
                        eprintln!("Failed to load monster names: {}", e);
                    }
                }
                Task::none()
            }
            // All Map Ini Editor
            Message::AllMapIniOpBrowseGamePath => browse_folder("shared_game_path"),
            Message::AllMapIniOpLoadCatalog => {
                if self.state.shared_game_path.is_empty() {
                    self.state.all_map_ini_editor.status_msg =
                        "Please select game path first.".into();
                    return Task::none();
                }
                self.state.all_map_ini_editor.is_loading = true;
                let path = PathBuf::from(&self.state.shared_game_path).join("AllMap.ini");
                Task::perform(
                    async move { Map::read_file(&path).map_err(|e: std::io::Error| e.to_string()) },
                    |res| Message::AllMapIniCatalogLoaded(res),
                )
            }
            Message::AllMapIniCatalogLoaded(res) => {
                self.state.all_map_ini_editor.is_loading = false;
                match res {
                    Ok(catalog) => {
                        self.state.all_map_ini_editor.catalog = Some(catalog.clone());
                        self.state.all_map_ini_editor.status_msg =
                            format!("Map catalog loaded: {} maps", catalog.len()).into();
                        self.state.all_map_ini_editor.refresh_maps();
                    }
                    Err(e) => {
                        self.state.all_map_ini_editor.status_msg =
                            format!("Error loading map catalog: {}", e)
                    }
                }
                Task::none()
            }
            Message::AllMapIniOpSelectMap(idx) => {
                self.state.all_map_ini_editor.select_map(idx);
                Task::none()
            }
            Message::AllMapIniOpFieldChanged(idx, field, val) => {
                self.state.all_map_ini_editor.update_field(idx, &field, val);
                Task::none()
            }
            Message::AllMapIniOpSave => {
                if self.state.shared_game_path.is_empty() {
                    self.state.all_map_ini_editor.status_msg =
                        "Please select game path first.".into();
                    return Task::none();
                }
                self.state.all_map_ini_editor.is_loading = true;
                let result = self
                    .state
                    .all_map_ini_editor
                    .save_maps(&self.state.shared_game_path);
                self.state.all_map_ini_editor.is_loading = false;
                match result {
                    Ok(_) => {
                        self.state.all_map_ini_editor.status_msg = "Maps saved successfully.".into()
                    }
                    Err(e) => {
                        self.state.all_map_ini_editor.status_msg =
                            format!("Error saving maps: {}", e)
                    }
                }
                Task::none()
            }
            // Dialog Editor
            Message::DialogOpBrowseGamePath => browse_folder("shared_game_path"),
            Message::DialogOpBrowseFile => crate::utils::browse_file("dialog_file"),
            Message::DialogOpSelectFile(path) => {
                self.state.dialog_editor.current_file = path.to_string_lossy().to_string();
                self.state.dialog_editor.is_loading = true;
                Task::perform(
                    async move { Dialog::read_file(&path).map_err(|e: std::io::Error| e.to_string()) },
                    |res| Message::DialogCatalogLoaded(res),
                )
            }
            Message::DialogOpScanFiles => {
                if self.state.shared_game_path.is_empty() {
                    self.state.dialog_editor.status_msg = "Please select game path first.".into();
                    return Task::none();
                }
                let path = PathBuf::from(&self.state.shared_game_path).join("NpcInGame");
                Task::perform(
                    async move {
                        let mut files = vec![];
                        if let Ok(entries) = std::fs::read_dir(&path) {
                            for entry in entries.flatten() {
                                let p = entry.path();
                                if p.is_file() && p.extension().map(|e| e == "dlg").unwrap_or(false)
                                {
                                    files.push(p);
                                }
                            }
                        }
                        files.sort();
                        files
                    },
                    Message::DialogOpFilesScanned,
                )
            }
            Message::DialogOpFilesScanned(files) => {
                self.state.dialog_editor.dialog_files = files;
                self.state.dialog_editor.status_msg = format!(
                    "Found {} dialog files",
                    self.state.dialog_editor.dialog_files.len()
                );
                Task::none()
            }
            Message::DialogOpLoadCatalog => {
                if self.state.dialog_editor.current_file.is_empty() {
                    self.state.dialog_editor.status_msg =
                        "Please select a dialog file first.".into();
                    return Task::none();
                }
                self.state.dialog_editor.is_loading = true;
                let path = PathBuf::from(&self.state.dialog_editor.current_file);
                Task::perform(
                    async move { Dialog::read_file(&path).map_err(|e: std::io::Error| e.to_string()) },
                    |res| Message::DialogCatalogLoaded(res),
                )
            }
            Message::DialogCatalogLoaded(res) => {
                self.state.dialog_editor.is_loading = false;
                match res {
                    Ok(catalog) => {
                        self.state.dialog_editor.catalog = Some(catalog.clone());
                        self.state.dialog_editor.status_msg =
                            format!("Dialog catalog loaded: {} entries", catalog.len()).into();
                        self.state.dialog_editor.refresh_dialogs();
                    }
                    Err(e) => {
                        self.state.dialog_editor.status_msg =
                            format!("Error loading dialog catalog: {}", e)
                    }
                }
                Task::none()
            }
            Message::DialogOpSelectDialog(idx) => {
                self.state.dialog_editor.select_dialog(idx);
                Task::none()
            }
            Message::DialogOpFieldChanged(idx, field, val) => {
                self.state.dialog_editor.update_field(idx, &field, val);
                Task::none()
            }
            Message::DialogOpSave => {
                if self.state.shared_game_path.is_empty()
                    || self.state.dialog_editor.current_file.is_empty()
                {
                    self.state.dialog_editor.status_msg =
                        "Please select game path and file first.".into();
                    return Task::none();
                }
                self.state.dialog_editor.is_loading = true;
                let filename = std::path::Path::new(&self.state.dialog_editor.current_file)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "dialog.dlg".to_string());
                let result = self
                    .state
                    .dialog_editor
                    .save_dialogs(&self.state.shared_game_path, &filename);
                self.state.dialog_editor.is_loading = false;
                match result {
                    Ok(_) => {
                        self.state.dialog_editor.status_msg = "Dialogs saved successfully.".into()
                    }
                    Err(e) => {
                        self.state.dialog_editor.status_msg = format!("Error saving dialogs: {}", e)
                    }
                }
                Task::none()
            }
            // Dialogue Text Editor
            Message::DialogueTextOpBrowseGamePath => browse_folder("shared_game_path"),
            Message::DialogueTextOpBrowseFile => crate::utils::browse_file("dialogue_text_file"),
            Message::DialogueTextOpSelectFile(path) => {
                self.state.dialogue_text_editor.current_file = path.to_string_lossy().to_string();
                self.state.dialogue_text_editor.is_loading = true;
                Task::perform(
                    async move {
                        DialogueText::read_file(&path).map_err(|e: std::io::Error| e.to_string())
                    },
                    |res| Message::DialogueTextCatalogLoaded(res),
                )
            }
            Message::DialogueTextOpScanFiles => {
                if self.state.shared_game_path.is_empty() {
                    self.state.dialogue_text_editor.status_msg =
                        "Please select game path first.".into();
                    return Task::none();
                }
                let path = PathBuf::from(&self.state.shared_game_path).join("NpcInGame");
                Task::perform(
                    async move {
                        let mut files = vec![];
                        if let Ok(entries) = std::fs::read_dir(&path) {
                            for entry in entries.flatten() {
                                let p = entry.path();
                                if p.is_file() && p.extension().map(|e| e == "pgp").unwrap_or(false)
                                {
                                    files.push(p);
                                }
                            }
                        }
                        files.sort();
                        files
                    },
                    Message::DialogueTextOpFilesScanned,
                )
            }
            Message::DialogueTextOpFilesScanned(files) => {
                self.state.dialogue_text_editor.text_files = files;
                self.state.dialogue_text_editor.status_msg = format!(
                    "Found {} text files",
                    self.state.dialogue_text_editor.text_files.len()
                );
                Task::none()
            }
            Message::DialogueTextOpLoadCatalog => {
                if self.state.dialogue_text_editor.current_file.is_empty() {
                    self.state.dialogue_text_editor.status_msg =
                        "Please select a file first.".into();
                    return Task::none();
                }
                self.state.dialogue_text_editor.is_loading = true;
                let path = PathBuf::from(&self.state.dialogue_text_editor.current_file);
                Task::perform(
                    async move {
                        DialogueText::read_file(&path).map_err(|e: std::io::Error| e.to_string())
                    },
                    |res| Message::DialogueTextCatalogLoaded(res),
                )
            }
            Message::DialogueTextCatalogLoaded(res) => {
                self.state.dialogue_text_editor.is_loading = false;
                match res {
                    Ok(catalog) => {
                        self.state.dialogue_text_editor.catalog = Some(catalog.clone());
                        self.state.dialogue_text_editor.status_msg =
                            format!("Text catalog loaded: {} entries", catalog.len()).into();
                        self.state.dialogue_text_editor.refresh_texts();
                    }
                    Err(e) => {
                        self.state.dialogue_text_editor.status_msg =
                            format!("Error loading text catalog: {}", e)
                    }
                }
                Task::none()
            }
            Message::DialogueTextOpSelectText(idx) => {
                self.state.dialogue_text_editor.select_text(idx);
                Task::none()
            }
            Message::DialogueTextOpFieldChanged(idx, field, val) => {
                self.state
                    .dialogue_text_editor
                    .update_field(idx, &field, val);
                Task::none()
            }
            Message::DialogueTextOpTextAction(idx, action) => {
                use iced::widget::text_editor;
                if let text_editor::Action::Edit(_) = action {
                    self.state.dialogue_text_editor.update_text_content(idx);
                }
                Task::none()
            }
            Message::DialogueTextOpCommentAction(idx, action) => {
                use iced::widget::text_editor;
                if let text_editor::Action::Edit(_) = action {
                    self.state.dialogue_text_editor.update_comment_content(idx);
                }
                Task::none()
            }
            Message::DialogueTextOpSave => {
                if self.state.shared_game_path.is_empty()
                    || self.state.dialogue_text_editor.current_file.is_empty()
                {
                    self.state.dialogue_text_editor.status_msg =
                        "Please select game path and file first.".into();
                    return Task::none();
                }
                self.state.dialogue_text_editor.is_loading = true;
                let filename = std::path::Path::new(&self.state.dialogue_text_editor.current_file)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "text.pgp".to_string());
                let result = self
                    .state
                    .dialogue_text_editor
                    .save_texts(&self.state.shared_game_path, &filename);
                self.state.dialogue_text_editor.is_loading = false;
                match result {
                    Ok(_) => {
                        self.state.dialogue_text_editor.status_msg =
                            "Texts saved successfully.".into()
                    }
                    Err(e) => {
                        self.state.dialogue_text_editor.status_msg =
                            format!("Error saving texts: {}", e)
                    }
                }
                Task::none()
            }
            // Draw Item Editor
            Message::DrawItemOpBrowseGamePath => browse_folder("shared_game_path"),
            Message::DrawItemOpLoadCatalog => {
                if self.state.shared_game_path.is_empty() {
                    self.state.draw_item_editor.status_msg =
                        "Please select game path first.".into();
                    return Task::none();
                }
                self.state.draw_item_editor.is_loading = true;
                let path = PathBuf::from(&self.state.shared_game_path)
                    .join("Ref")
                    .join("DRAWITEM.ref");
                Task::perform(
                    async move { DrawItem::read_file(&path).map_err(|e: std::io::Error| e.to_string()) },
                    |res| Message::DrawItemCatalogLoaded(res),
                )
            }
            Message::DrawItemCatalogLoaded(res) => {
                self.state.draw_item_editor.is_loading = false;
                match res {
                    Ok(catalog) => {
                        self.state.draw_item_editor.catalog = Some(catalog.clone());
                        self.state.draw_item_editor.status_msg =
                            format!("Draw item catalog loaded: {} entries", catalog.len()).into();
                        self.state.draw_item_editor.refresh_items();
                    }
                    Err(e) => {
                        self.state.draw_item_editor.status_msg =
                            format!("Error loading draw item catalog: {}", e)
                    }
                }
                Task::none()
            }
            Message::DrawItemOpSelectItem(idx) => {
                self.state.draw_item_editor.select(idx);
                Task::none()
            }
            Message::DrawItemOpFieldChanged(idx, field, val) => {
                self.state.draw_item_editor.update_field(idx, &field, val);
                Task::none()
            }
            Message::DrawItemOpSave => {
                if self.state.shared_game_path.is_empty() {
                    self.state.draw_item_editor.status_msg =
                        "Please select game path first.".into();
                    return Task::none();
                }
                self.state.draw_item_editor.is_loading = true;
                let result = self
                    .state
                    .draw_item_editor
                    .save_items(&self.state.shared_game_path);
                self.state.draw_item_editor.is_loading = false;
                match result {
                    Ok(_) => {
                        self.state.draw_item_editor.status_msg =
                            "Draw items saved successfully.".into()
                    }
                    Err(e) => {
                        self.state.draw_item_editor.status_msg =
                            format!("Error saving draw items: {}", e)
                    }
                }
                Task::none()
            }
            // Event Ini Editor
            Message::EventIniOpBrowseGamePath => browse_folder("shared_game_path"),
            Message::EventIniOpLoadCatalog => {
                if self.state.shared_game_path.is_empty() {
                    self.state.event_ini_editor.status_msg =
                        "Please select game path first.".into();
                    return Task::none();
                }
                self.state.event_ini_editor.is_loading = true;
                let path = PathBuf::from(&self.state.shared_game_path).join("Event.ini");
                Task::perform(
                    async move { Event::read_file(&path).map_err(|e: std::io::Error| e.to_string()) },
                    |res| Message::EventIniCatalogLoaded(res),
                )
            }
            Message::EventIniCatalogLoaded(res) => {
                self.state.event_ini_editor.is_loading = false;
                match res {
                    Ok(catalog) => {
                        self.state.event_ini_editor.catalog = Some(catalog.clone());
                        self.state.event_ini_editor.status_msg =
                            format!("Event catalog loaded: {} entries", catalog.len()).into();
                        self.state.event_ini_editor.refresh_events();
                    }
                    Err(e) => {
                        self.state.event_ini_editor.status_msg =
                            format!("Error loading event catalog: {}", e)
                    }
                }
                Task::none()
            }
            Message::EventIniOpSelectEvent(idx) => {
                self.state.event_ini_editor.select(idx);
                Task::none()
            }
            Message::EventIniOpFieldChanged(idx, field, val) => {
                self.state.event_ini_editor.update_field(idx, &field, val);
                Task::none()
            }
            Message::EventIniOpSave => {
                if self.state.shared_game_path.is_empty() {
                    self.state.event_ini_editor.status_msg =
                        "Please select game path first.".into();
                    return Task::none();
                }
                self.state.event_ini_editor.is_loading = true;
                let result = self
                    .state
                    .event_ini_editor
                    .save_events(&self.state.shared_game_path);
                self.state.event_ini_editor.is_loading = false;
                match result {
                    Ok(_) => {
                        self.state.event_ini_editor.status_msg = "Events saved successfully.".into()
                    }
                    Err(e) => {
                        self.state.event_ini_editor.status_msg =
                            format!("Error saving events: {}", e)
                    }
                }
                Task::none()
            }
            // Event Npc Ref Editor
            Message::EventNpcRefOpBrowseGamePath => browse_folder("shared_game_path"),
            Message::EventNpcRefOpLoadCatalog => {
                if self.state.shared_game_path.is_empty() {
                    self.state.event_npc_ref_editor.status_msg =
                        "Please select game path first.".into();
                    return Task::none();
                }
                self.state.event_npc_ref_editor.is_loading = true;
                let path = PathBuf::from(&self.state.shared_game_path)
                    .join("NpcInGame")
                    .join("Eventnpc.ref");
                Task::perform(
                    async move {
                        EventNpcRef::read_file(&path).map_err(|e: std::io::Error| e.to_string())
                    },
                    |res| Message::EventNpcRefCatalogLoaded(res),
                )
            }
            Message::EventNpcRefCatalogLoaded(res) => {
                self.state.event_npc_ref_editor.is_loading = false;
                match res {
                    Ok(catalog) => {
                        self.state.event_npc_ref_editor.catalog = Some(catalog.clone());
                        self.state.event_npc_ref_editor.status_msg =
                            format!("Event NPC catalog loaded: {} entries", catalog.len()).into();
                        self.state.event_npc_ref_editor.refresh_npcs();
                    }
                    Err(e) => {
                        self.state.event_npc_ref_editor.status_msg =
                            format!("Error loading event NPC catalog: {}", e)
                    }
                }
                Task::none()
            }
            Message::EventNpcRefOpSelectNpc(idx) => {
                self.state.event_npc_ref_editor.select(idx);
                Task::none()
            }
            Message::EventNpcRefOpFieldChanged(idx, field, val) => {
                self.state
                    .event_npc_ref_editor
                    .update_field(idx, &field, val);
                Task::none()
            }
            Message::EventNpcRefOpSave => {
                if self.state.shared_game_path.is_empty() {
                    self.state.event_npc_ref_editor.status_msg =
                        "Please select game path first.".into();
                    return Task::none();
                }
                self.state.event_npc_ref_editor.is_loading = true;
                let result = self
                    .state
                    .event_npc_ref_editor
                    .save_npcs(&self.state.shared_game_path);
                self.state.event_npc_ref_editor.is_loading = false;
                match result {
                    Ok(_) => {
                        self.state.event_npc_ref_editor.status_msg =
                            "Event NPCs saved successfully.".into()
                    }
                    Err(e) => {
                        self.state.event_npc_ref_editor.status_msg =
                            format!("Error saving event NPCs: {}", e)
                    }
                }
                Task::none()
            }
            // Extra Ini Editor
            Message::ExtraIniOpBrowseGamePath => browse_folder("shared_game_path"),
            Message::ExtraIniOpLoadCatalog => {
                if self.state.shared_game_path.is_empty() {
                    self.state.extra_ini_editor.status_msg =
                        "Please select game path first.".into();
                    return Task::none();
                }
                self.state.extra_ini_editor.is_loading = true;
                let path = PathBuf::from(&self.state.shared_game_path).join("Extra.ini");
                Task::perform(
                    async move { Extra::read_file(&path).map_err(|e: std::io::Error| e.to_string()) },
                    |res| Message::ExtraIniCatalogLoaded(res),
                )
            }
            Message::ExtraIniCatalogLoaded(res) => {
                self.state.extra_ini_editor.is_loading = false;
                match res {
                    Ok(catalog) => {
                        self.state.extra_ini_editor.catalog = Some(catalog.clone());
                        self.state.extra_ini_editor.status_msg =
                            format!("Extra catalog loaded: {} entries", catalog.len()).into();
                        self.state.extra_ini_editor.refresh_extras();
                    }
                    Err(e) => {
                        self.state.extra_ini_editor.status_msg =
                            format!("Error loading extra catalog: {}", e)
                    }
                }
                Task::none()
            }
            Message::ExtraIniOpSelectExtra(idx) => {
                self.state.extra_ini_editor.select(idx);
                Task::none()
            }
            Message::ExtraIniOpFieldChanged(idx, field, val) => {
                self.state.extra_ini_editor.update_field(idx, &field, val);
                Task::none()
            }
            Message::ExtraIniOpSave => {
                if self.state.shared_game_path.is_empty() {
                    self.state.extra_ini_editor.status_msg =
                        "Please select game path first.".into();
                    return Task::none();
                }
                self.state.extra_ini_editor.is_loading = true;
                let result = self
                    .state
                    .extra_ini_editor
                    .save_extras(&self.state.shared_game_path);
                self.state.extra_ini_editor.is_loading = false;
                match result {
                    Ok(_) => {
                        self.state.extra_ini_editor.status_msg = "Extras saved successfully.".into()
                    }
                    Err(e) => {
                        self.state.extra_ini_editor.status_msg =
                            format!("Error saving extras: {}", e)
                    }
                }
                Task::none()
            }
            // Extra Ref Editor
            Message::ExtraRefOpBrowseGamePath => browse_folder("shared_game_path"),
            Message::ExtraRefOpBrowseMapFile => crate::utils::browse_file("extra_ref_map_file"),
            Message::ExtraRefOpSelectMapFile(path) => {
                self.state.extra_ref_editor.current_map_file = path.to_string_lossy().to_string();
                self.state.extra_ref_editor.is_loading = true;
                Task::perform(
                    async move { ExtraRef::read_file(&path).map_err(|e: std::io::Error| e.to_string()) },
                    |res| Message::ExtraRefCatalogLoaded(res),
                )
            }
            Message::ExtraRefOpLoadCatalog => {
                if self.state.extra_ref_editor.current_map_file.is_empty() {
                    self.state.extra_ref_editor.status_msg =
                        "Please select a map file first.".into();
                    return Task::none();
                }
                self.state.extra_ref_editor.is_loading = true;
                let path = PathBuf::from(&self.state.extra_ref_editor.current_map_file);
                Task::perform(
                    async move { ExtraRef::read_file(&path).map_err(|e: std::io::Error| e.to_string()) },
                    |res| Message::ExtraRefCatalogLoaded(res),
                )
            }
            Message::ExtraRefCatalogLoaded(res) => {
                self.state.extra_ref_editor.is_loading = false;
                match res {
                    Ok(catalog) => {
                        self.state.extra_ref_editor.catalog = Some(catalog.clone());
                        self.state.extra_ref_editor.status_msg =
                            format!("Extra ref catalog loaded: {} entries", catalog.len()).into();
                        self.state.extra_ref_editor.refresh_items();
                    }
                    Err(e) => {
                        self.state.extra_ref_editor.status_msg =
                            format!("Error loading extra ref catalog: {}", e)
                    }
                }
                Task::none()
            }
            Message::ExtraRefOpSelectItem(idx) => {
                self.state.extra_ref_editor.select_item(idx);
                Task::none()
            }
            Message::ExtraRefOpFieldChanged(idx, field, val) => {
                self.state.extra_ref_editor.update_field(idx, &field, val);
                Task::none()
            }
            Message::ExtraRefOpSave => {
                self.state.extra_ref_editor.is_loading = true;
                let result = self.state.extra_ref_editor.save_items();
                self.state.extra_ref_editor.is_loading = false;
                match result {
                    Ok(_) => {
                        self.state.extra_ref_editor.status_msg =
                            "Extra refs saved successfully.".into()
                    }
                    Err(e) => {
                        self.state.extra_ref_editor.status_msg =
                            format!("Error saving extra refs: {}", e)
                    }
                }
                Task::none()
            }
            // Map Ini Editor
            Message::MapIniOpBrowseGamePath => browse_folder("shared_game_path"),
            Message::MapIniOpLoadCatalog => {
                if self.state.shared_game_path.is_empty() {
                    self.state.map_ini_editor.status_msg = "Please select game path first.".into();
                    return Task::none();
                }
                self.state.map_ini_editor.is_loading = true;
                let path = PathBuf::from(&self.state.shared_game_path)
                    .join("Ref")
                    .join("Map.ini");
                Task::perform(
                    async move { MapIni::read_file(&path).map_err(|e: std::io::Error| e.to_string()) },
                    |res| Message::MapIniCatalogLoaded(res),
                )
            }
            Message::MapIniCatalogLoaded(res) => {
                self.state.map_ini_editor.is_loading = false;
                match res {
                    Ok(catalog) => {
                        self.state.map_ini_editor.catalog = Some(catalog.clone());
                        self.state.map_ini_editor.status_msg =
                            format!("Map ini catalog loaded: {} entries", catalog.len()).into();
                        self.state.map_ini_editor.refresh_maps();
                    }
                    Err(e) => {
                        self.state.map_ini_editor.status_msg =
                            format!("Error loading map ini catalog: {}", e)
                    }
                }
                Task::none()
            }
            Message::MapIniOpSelectMap(idx) => {
                self.state.map_ini_editor.select(idx);
                Task::none()
            }
            Message::MapIniOpFieldChanged(idx, field, val) => {
                self.state.map_ini_editor.update_field(idx, &field, val);
                Task::none()
            }
            Message::MapIniOpSave => {
                if self.state.shared_game_path.is_empty() {
                    self.state.map_ini_editor.status_msg = "Please select game path first.".into();
                    return Task::none();
                }
                self.state.map_ini_editor.is_loading = true;
                let result = self
                    .state
                    .map_ini_editor
                    .save_maps(&self.state.shared_game_path);
                self.state.map_ini_editor.is_loading = false;
                match result {
                    Ok(_) => {
                        self.state.map_ini_editor.status_msg = "Map ini saved successfully.".into()
                    }
                    Err(e) => {
                        self.state.map_ini_editor.status_msg =
                            format!("Error saving map ini: {}", e)
                    }
                }
                Task::none()
            }
            // Message Scr Editor
            Message::MessageScrOpBrowseGamePath => browse_folder("shared_game_path"),
            Message::MessageScrOpLoadCatalog => {
                if self.state.shared_game_path.is_empty() {
                    self.state.message_scr_editor.status_msg =
                        "Please select game path first.".into();
                    return Task::none();
                }
                self.state.message_scr_editor.is_loading = true;
                let path = PathBuf::from(&self.state.shared_game_path)
                    .join("ExtraInGame")
                    .join("Message.scr");
                Task::perform(
                    async move {
                        ScrMessage::read_file(&path).map_err(|e: std::io::Error| e.to_string())
                    },
                    |res| Message::MessageScrCatalogLoaded(res),
                )
            }
            Message::MessageScrCatalogLoaded(res) => {
                self.state.message_scr_editor.is_loading = false;
                match res {
                    Ok(catalog) => {
                        self.state.message_scr_editor.catalog = Some(catalog.clone());
                        self.state.message_scr_editor.status_msg =
                            format!("Message catalog loaded: {} entries", catalog.len()).into();
                        self.state.message_scr_editor.refresh_messages();
                    }
                    Err(e) => {
                        self.state.message_scr_editor.status_msg =
                            format!("Error loading message catalog: {}", e)
                    }
                }
                Task::none()
            }
            Message::MessageScrOpSelectMessage(idx) => {
                self.state.message_scr_editor.select(idx);
                Task::none()
            }
            Message::MessageScrOpFieldChanged(idx, field, val) => {
                self.state.message_scr_editor.update_field(idx, &field, val);
                Task::none()
            }
            Message::MessageScrOpSave => {
                if self.state.shared_game_path.is_empty() {
                    self.state.message_scr_editor.status_msg =
                        "Please select game path first.".into();
                    return Task::none();
                }
                self.state.message_scr_editor.is_loading = true;
                let result = self
                    .state
                    .message_scr_editor
                    .save_messages(&self.state.shared_game_path);
                self.state.message_scr_editor.is_loading = false;
                match result {
                    Ok(_) => {
                        self.state.message_scr_editor.status_msg =
                            "Messages saved successfully.".into()
                    }
                    Err(e) => {
                        self.state.message_scr_editor.status_msg =
                            format!("Error saving messages: {}", e)
                    }
                }
                Task::none()
            }
            // Npc Ref Editor
            Message::NpcRefOpBrowseGamePath => browse_folder("shared_game_path"),
            Message::NpcRefOpBrowseMapFile => crate::utils::browse_file("npc_ref_map_file"),
            Message::NpcRefOpSelectMapFile(path) => {
                self.state.npc_ref_editor.select_file(path);
                Task::none()
            }
            Message::NpcRefOpScanFiles => {
                if self.state.shared_game_path.is_empty() {
                    self.state.npc_ref_editor.editor.status_msg = "Please select game path first.".into();
                    return Task::none();
                }
                let path = PathBuf::from(&self.state.shared_game_path).join("NpcInGame");
                self.state.npc_ref_editor.scan_files(&path, "Npc*.ref");
                self.state.npc_ref_editor.editor.status_msg = format!(
                    "Found {} NPC ref files",
                    self.state.npc_ref_editor.file_list.len()
                );
                Task::none()
            }
            Message::NpcRefOpFilesScanned(files) => {
                self.state.npc_ref_editor.file_list = files;
                self.state.npc_ref_editor.editor.status_msg = format!(
                    "Found {} NPC ref files",
                    self.state.npc_ref_editor.file_list.len()
                );
                Task::none()
            }
            Message::NpcRefOpLoadCatalog => {
                let Some(ref current_file) = self.state.npc_ref_editor.current_file else {
                    self.state.npc_ref_editor.editor.status_msg = "Please select a map file first.".into();
                    return Task::none();
                };
                self.state.npc_ref_editor.editor.is_loading = true;
                let path = current_file.clone();
                Task::perform(
                    async move { NPC::read_file(&path).map_err(|e: std::io::Error| e.to_string()) },
                    |res| Message::NpcRefCatalogLoaded(res),
                )
            }
            Message::NpcRefCatalogLoaded(res) => {
                self.state.npc_ref_editor.editor.is_loading = false;
                match res {
                    Ok(catalog) => {
                        self.state.npc_ref_editor.editor.catalog = Some(catalog.clone());
                        self.state.npc_ref_editor.editor.status_msg =
                            format!("NPC ref catalog loaded: {} entries", catalog.len()).into();
                        self.state.npc_ref_editor.refresh_npcs();
                    }
                    Err(e) => {
                        self.state.npc_ref_editor.editor.status_msg =
                            format!("Error loading NPC ref catalog: {}", e)
                    }
                }
                Task::none()
            }
            Message::NpcRefOpSelectNpc(idx) => {
                self.state.npc_ref_editor.select_npc(idx);
                Task::none()
            }
            Message::NpcRefOpFieldChanged(idx, field, val) => {
                self.state.npc_ref_editor.update_field(idx, &field, val);
                Task::none()
            }
            Message::NpcRefOpSave => {
                self.state.npc_ref_editor.editor.is_loading = true;
                let result = self.state.npc_ref_editor.save_npcs();
                self.state.npc_ref_editor.editor.is_loading = false;
                match result {
                    Ok(_) => {
                        self.state.npc_ref_editor.editor.status_msg = "NPC refs saved successfully.".into()
                    }
                    Err(e) => {
                        self.state.npc_ref_editor.editor.status_msg =
                            format!("Error saving NPC refs: {}", e)
                    }
                }
                Task::none()
            }
            Message::NpcRefOpAddEntry => {
                self.state.npc_ref_editor.add_record();
                Task::none()
            }
            Message::NpcRefOpRemoveEntry(idx) => {
                self.state.npc_ref_editor.remove_record(idx);
                Task::none()
            }
            // Party Level Db Editor
            Message::PartyLevelDbOpBrowseGamePath => browse_folder("shared_game_path"),
            Message::PartyLevelDbOpLoadCatalog => {
                if self.state.shared_game_path.is_empty() {
                    self.state.party_level_db_editor.status_msg =
                        "Please select game path first.".into();
                    return Task::none();
                }
                self.state.party_level_db_editor.is_loading = true;
                let level_path = PathBuf::from(&self.state.shared_game_path)
                    .join("NpcInGame")
                    .join("PrtLevel.db");
                Task::perform(
                    async move {
                        PartyLevelNpc::read_file(&level_path)
                            .map_err(|e: std::io::Error| e.to_string())
                    },
                    |res| Message::PartyLevelDbCatalogLoaded(res),
                )
            }
            Message::PartyLevelDbCatalogLoaded(res) => {
                self.state.party_level_db_editor.is_loading = false;
                match res {
                    Ok(levels) => {
                        self.state.party_level_db_editor.catalog = Some(levels.clone());
                        self.state.party_level_db_editor.status_msg =
                            format!("Party level catalog loaded: {} NPCs", levels.len()).into();
                    }
                    Err(e) => {
                        self.state.party_level_db_editor.status_msg =
                            format!("Error loading party level catalog: {}", e)
                    }
                }
                Task::none()
            }
            Message::PartyLevelDbOpSelectRecord(idx) => {
                self.state.party_level_db_editor.select(idx);
                Task::none()
            }
            Message::PartyLevelDbOpFieldChanged(idx, field, val) => {
                self.state.party_level_db_editor.update_field(idx, &field, val);
                Task::none()
            }
            Message::PartyLevelDbOpSave => {
                if self.state.shared_game_path.is_empty() {
                    self.state.party_level_db_editor.status_msg =
                        "Please select game path first.".into();
                    return Task::none();
                }
                self.state.party_level_db_editor.is_loading = true;
                let result = self
                    .state
                    .party_level_db_editor
                    .save_levels(&self.state.shared_game_path);
                self.state.party_level_db_editor.is_loading = false;
                match result {
                    Ok(_) => {
                        self.state.party_level_db_editor.status_msg =
                            "Party levels saved successfully.".into()
                    }
                    Err(e) => {
                        self.state.party_level_db_editor.status_msg =
                            format!("Error saving party levels: {}", e)
                    }
                }
                Task::none()
            }
            // Quest Scr Editor
            Message::QuestScrOpBrowseGamePath => browse_folder("shared_game_path"),
            Message::QuestScrOpLoadCatalog => {
                if self.state.shared_game_path.is_empty() {
                    self.state.quest_scr_editor.status_msg =
                        "Please select game path first.".into();
                    return Task::none();
                }
                self.state.quest_scr_editor.is_loading = true;
                let path = PathBuf::from(&self.state.shared_game_path)
                    .join("ExtraInGame")
                    .join("Quest.scr");
                Task::perform(
                    async move { Quest::read_file(&path).map_err(|e: std::io::Error| e.to_string()) },
                    |res| Message::QuestScrCatalogLoaded(res),
                )
            }
            Message::QuestScrCatalogLoaded(res) => {
                self.state.quest_scr_editor.is_loading = false;
                match res {
                    Ok(catalog) => {
                        self.state.quest_scr_editor.catalog = Some(catalog.clone());
                        self.state.quest_scr_editor.status_msg =
                            format!("Quest catalog loaded: {} entries", catalog.len()).into();
                        self.state.quest_scr_editor.refresh_quests();
                    }
                    Err(e) => {
                        self.state.quest_scr_editor.status_msg =
                            format!("Error loading quest catalog: {}", e)
                    }
                }
                Task::none()
            }
            Message::QuestScrOpSelectQuest(idx) => {
                self.state.quest_scr_editor.select(idx);
                Task::none()
            }
            Message::QuestScrOpFieldChanged(idx, field, val) => {
                self.state.quest_scr_editor.update_field(idx, &field, val);
                Task::none()
            }
            Message::QuestScrOpDescriptionAction(_idx, _action) => {
                Task::none()
            }
            Message::QuestScrOpSave => {
                if self.state.shared_game_path.is_empty() {
                    self.state.quest_scr_editor.status_msg =
                        "Please select game path first.".into();
                    return Task::none();
                }
                self.state.quest_scr_editor.is_loading = true;
                let result = self
                    .state
                    .quest_scr_editor
                    .save_quests(&self.state.shared_game_path);
                self.state.quest_scr_editor.is_loading = false;
                match result {
                    Ok(_) => {
                        self.state.quest_scr_editor.status_msg = "Quests saved successfully.".into()
                    }
                    Err(e) => {
                        self.state.quest_scr_editor.status_msg =
                            format!("Error saving quests: {}", e)
                    }
                }
                Task::none()
            }
            // Wave Ini Editor
            Message::WaveIniOpBrowseGamePath => browse_folder("shared_game_path"),
            Message::WaveIniOpLoadCatalog => {
                if self.state.shared_game_path.is_empty() {
                    self.state.wave_ini_editor.status_msg = "Please select game path first.".into();
                    return Task::none();
                }
                self.state.wave_ini_editor.is_loading = true;
                let path = PathBuf::from(&self.state.shared_game_path).join("Wave.ini");
                Task::perform(
                    async move { WaveIni::read_file(&path).map_err(|e: std::io::Error| e.to_string()) },
                    |res| Message::WaveIniCatalogLoaded(res),
                )
            }
            Message::WaveIniCatalogLoaded(res) => {
                self.state.wave_ini_editor.is_loading = false;
                match res {
                    Ok(catalog) => {
                        self.state.wave_ini_editor.catalog = Some(catalog.clone());
                        self.state.wave_ini_editor.status_msg =
                            format!("Wave catalog loaded: {} entries", catalog.len()).into();
                        self.state.wave_ini_editor.refresh_waves();
                    }
                    Err(e) => {
                        self.state.wave_ini_editor.status_msg =
                            format!("Error loading wave catalog: {}", e)
                    }
                }
                Task::none()
            }
            Message::WaveIniOpSelectWave(idx) => {
                self.state.wave_ini_editor.select(idx);
                Task::none()
            }
            Message::WaveIniOpFieldChanged(idx, field, val) => {
                self.state.wave_ini_editor.update_field(idx, &field, val);
                Task::none()
            }
            Message::WaveIniOpSave => {
                if self.state.shared_game_path.is_empty() {
                    self.state.wave_ini_editor.status_msg = "Please select game path first.".into();
                    return Task::none();
                }
                self.state.wave_ini_editor.is_loading = true;
                let result = self
                    .state
                    .wave_ini_editor
                    .save_waves(&self.state.shared_game_path);
                self.state.wave_ini_editor.is_loading = false;
                match result {
                    Ok(_) => {
                        self.state.wave_ini_editor.status_msg =
                            "Wave ini saved successfully.".into()
                    }
                    Err(e) => {
                        self.state.wave_ini_editor.status_msg =
                            format!("Error saving wave ini: {}", e)
                    }
                }
                Task::none()
            }
            Message::WaveIniOpExportWav(idx) => {
                if self.state.shared_game_path.is_empty() {
                    self.state.wave_ini_editor.status_msg =
                        "Please select game path first.".into();
                    return Task::none();
                }
                if let Some((_, wave)) = self.state.wave_ini_editor.filtered.get(idx) {
                    let snf_filename = match &wave.snf_filename {
                        Some(f) => f.clone(),
                        None => {
                            self.state.wave_ini_editor.status_msg =
                                "No SNF filename for this entry.".into();
                            return Task::none();
                        }
                    };
                    let stem = std::path::Path::new(&snf_filename)
                        .file_stem()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_else(|| format!("wave_{}", wave.id));
                    let game_path = self.state.shared_game_path.clone();
                    self.state.wave_ini_editor.is_loading = true;
                    return Task::perform(
                        async move {
                            let handle = rfd::AsyncFileDialog::new()
                                .set_file_name(&format!("{}.wav", stem))
                                .add_filter("WAV Audio", &["wav"])
                                .save_file()
                                .await;
                            match handle {
                                Some(h) => {
                                    let output_path = h.path().to_path_buf();
                                    if let Some(parent) = output_path.parent() {
                                        let _ = std::fs::create_dir_all(parent);
                                    }
                                    let snf_path = App::find_snf_file(&game_path, &snf_filename);
                                    dispel_core::snf::extract(&snf_path, &output_path)
                                        .map(|_| output_path.to_string_lossy().to_string())
                                        .map_err(|e| e.to_string())
                                }
                                None => Err("Export cancelled".into()),
                            }
                        },
                        Message::WaveIniWavExported,
                    );
                }
                Task::none()
            }
            Message::WaveIniWavExported(res) => {
                self.state.wave_ini_editor.is_loading = false;
                match res {
                    Ok(p) => {
                        self.state.wave_ini_editor.status_msg = format!("Exported to {}", p)
                    }
                    Err(e) => {
                        self.state.wave_ini_editor.status_msg = format!("Export failed: {}", e)
                    }
                }
                Task::none()
            }
            // ChData Editor
            Message::ChDataOpBrowseGamePath => browse_folder("shared_game_path"),
            Message::ChDataOpLoadCatalog => {
                if self.state.shared_game_path.is_empty() {
                    self.state.chdata_editor.status_msg = "Please select game path first.".into();
                    return Task::none();
                }
                self.state.chdata_editor.is_loading = true;
                let path = PathBuf::from(&self.state.shared_game_path)
                    .join("CharacterInGame")
                    .join("ChData.db");
                Task::perform(
                    async move { ChData::read_file(&path).map_err(|e: std::io::Error| e.to_string()) },
                    |res| Message::ChDataCatalogLoaded(res),
                )
            }
            Message::ChDataCatalogLoaded(res) => {
                self.state.chdata_editor.is_loading = false;
                match res {
                    Ok(catalog) => {
                        self.state.chdata_editor.catalog = Some(catalog.clone());
                        self.state.chdata_editor.status_msg =
                            format!("Loaded {} ChData records.", catalog.len());
                        if !catalog.is_empty() {
                            self.state.chdata_editor.select(0);
                        }
                    }
                    Err(e) => {
                        self.state.chdata_editor.status_msg = format!("Error loading ChData: {}", e)
                    }
                }
                Task::none()
            }
            Message::ChDataOpFieldChanged(idx, field, val) => {
                self.state.chdata_editor.update_field(idx, &field, val);
                Task::none()
            }
            Message::ChDataOpSelectData(idx) => {
                self.state.chdata_editor.select(idx);
                Task::none()
            }
            Message::ChDataOpSave => {
                if self.state.shared_game_path.is_empty() {
                    self.state.chdata_editor.status_msg = "Please select game path first.".into();
                    return Task::none();
                }
                self.state.chdata_editor.is_loading = true;
                let result = self
                    .state
                    .chdata_editor
                    .save_data(&self.state.shared_game_path);
                self.state.chdata_editor.is_loading = false;
                match result {
                    Ok(_) => {
                        self.state.chdata_editor.status_msg = "ChData saved successfully.".into()
                    }
                    Err(e) => {
                        self.state.chdata_editor.status_msg = format!("Error saving ChData: {}", e)
                    }
                }
                Task::none()
            }

            // ─── Workspace messages ──────────────────────────────
            Message::Workspace(ws_msg) => self.update_workspace(ws_msg),

            // ─── Undo/Redo ────────────────────────────────────────
            Message::Undo => {
                use crate::generic_editor::UndoRedo;
                let result = match self.state.active_tab {
                    Tab::HealItemEditor => self.state.heal_item_editor.undo(),
                    Tab::MiscItemEditor => self.state.misc_item_editor.undo(),
                    Tab::EditItemEditor => self.state.edit_item_editor.undo(),
                    Tab::EventItemEditor => self.state.event_item_editor.undo(),
                    Tab::MagicEditor => self.state.magic_editor.undo(),
                    Tab::WeaponEditor => self.state.weapon_editor.undo(),
                    Tab::MonsterRefEditor => self.state.monster_ref_editor.undo(),
                    Tab::ExtraRefEditor => self.state.extra_ref_editor.undo(),
                    Tab::NpcRefEditor => self.state.npc_ref_editor.undo(),
                    Tab::DialogEditor => self.state.dialog_editor.undo(),
                    Tab::DialogueTextEditor => self.state.dialogue_text_editor.undo(),
                    Tab::DrawItemEditor => self.state.draw_item_editor.undo(),
                    Tab::EventIniEditor => self.state.event_ini_editor.undo(),
                    Tab::EventNpcRefEditor => self.state.event_npc_ref_editor.undo(),
                    Tab::ExtraIniEditor => self.state.extra_ini_editor.undo(),
                    Tab::MapIniEditor => self.state.map_ini_editor.undo(),
                    Tab::MessageScrEditor => self.state.message_scr_editor.undo(),
                    Tab::PartyLevelDbEditor => self.state.party_level_db_editor.undo(),
                    Tab::QuestScrEditor => self.state.quest_scr_editor.undo(),
                    Tab::WaveIniEditor => self.state.wave_ini_editor.undo(),
                    Tab::AllMapIniEditor => self.state.all_map_ini_editor.undo(),
                    Tab::ChDataEditor => self.state.chdata_editor.undo(),
                    Tab::PartyRefEditor => self.state.party_ref_editor.undo(),
                    Tab::PartyIniEditor => self.state.party_ini_editor.undo(),
                    Tab::NpcIniEditor => self.state.npc_ini_editor.undo(),
                    Tab::StoreEditor => self.state.store_editor.undo(),
                    _ => None,
                };
                self.state.status_msg = result.unwrap_or_else(|| "Nothing to undo".to_string());
                return Task::none();
            }
            Message::Redo => {
                use crate::generic_editor::UndoRedo;
                let result = match self.state.active_tab {
                    Tab::HealItemEditor => self.state.heal_item_editor.redo(),
                    Tab::MiscItemEditor => self.state.misc_item_editor.redo(),
                    Tab::EditItemEditor => self.state.edit_item_editor.redo(),
                    Tab::EventItemEditor => self.state.event_item_editor.redo(),
                    Tab::MagicEditor => self.state.magic_editor.redo(),
                    Tab::WeaponEditor => self.state.weapon_editor.redo(),
                    Tab::MonsterRefEditor => self.state.monster_ref_editor.redo(),
                    Tab::ExtraRefEditor => self.state.extra_ref_editor.redo(),
                    Tab::NpcRefEditor => self.state.npc_ref_editor.redo(),
                    Tab::DialogEditor => self.state.dialog_editor.redo(),
                    Tab::DialogueTextEditor => self.state.dialogue_text_editor.redo(),
                    Tab::DrawItemEditor => self.state.draw_item_editor.redo(),
                    Tab::EventIniEditor => self.state.event_ini_editor.redo(),
                    Tab::EventNpcRefEditor => self.state.event_npc_ref_editor.redo(),
                    Tab::ExtraIniEditor => self.state.extra_ini_editor.redo(),
                    Tab::MapIniEditor => self.state.map_ini_editor.redo(),
                    Tab::MessageScrEditor => self.state.message_scr_editor.redo(),
                    Tab::PartyLevelDbEditor => self.state.party_level_db_editor.redo(),
                    Tab::QuestScrEditor => self.state.quest_scr_editor.redo(),
                    Tab::WaveIniEditor => self.state.wave_ini_editor.redo(),
                    Tab::AllMapIniEditor => self.state.all_map_ini_editor.redo(),
                    Tab::ChDataEditor => self.state.chdata_editor.redo(),
                    Tab::PartyRefEditor => self.state.party_ref_editor.redo(),
                    Tab::PartyIniEditor => self.state.party_ini_editor.redo(),
                    Tab::NpcIniEditor => self.state.npc_ini_editor.redo(),
                    Tab::StoreEditor => self.state.store_editor.redo(),
                    _ => None,
                };
                self.state.status_msg = result.unwrap_or_else(|| "Nothing to redo".to_string());
                return Task::none();
            }
            Message::ToggleHistoryPanel => {
                self.history_panel_visible = !self.history_panel_visible;
                return Task::none();
            }
            Message::ToggleCommandPalette => {
                if self.command_palette.is_some() {
                    self.command_palette = None;
                } else {
                    self.command_palette = Some(CommandPalette::new());
                }
                return Task::none();
            }
            Message::CommandPaletteInput(input) => {
                if let Some(palette) = &mut self.command_palette {
                    palette.update_input(input);
                }
                return Task::none();
            }
            Message::CommandPaletteSelect(idx) => {
                if let Some(palette) = &self.command_palette {
                    if let Some(cmd) = palette.filtered_commands.get(idx) {
                        let action_msg = (cmd.action)();
                        self.command_palette = None;
                        return self.update(action_msg);
                    }
                }
                return Task::none();
            }
            Message::CommandPaletteClose => {
                self.command_palette = None;
                return Task::none();
            }
            Message::CommandPaletteArrowUp => {
                if let Some(palette) = &mut self.command_palette {
                    palette.select_previous();
                }
                return Task::none();
            }
            Message::CommandPaletteArrowDown => {
                if let Some(palette) = &mut self.command_palette {
                    palette.select_next();
                }
                return Task::none();
            }
            Message::CommandPaletteConfirm => {
                if let Some(palette) = &self.command_palette {
                    if let Some(cmd) = palette.selected_command() {
                        let action_msg = (cmd.action)();
                        self.command_palette = None;
                        return self.update(action_msg);
                    }
                }
                return Task::none();
            }

            // ─── Global Search ─────────────────────────────────────
            Message::ToggleGlobalSearch => {
                self.global_search.toggle();
                if self.global_search.is_visible {
                    self.command_palette = None;
                }
                return Task::none();
            }
            Message::GlobalSearchInput(input) => {
                self.global_search.query = input;
                self.global_search.search(&self.state);
                return Task::none();
            }
            Message::GlobalSearchSelect(idx) => {
                if let Some(result) = self.global_search.results.get(idx) {
                    let tab = match result.catalog_type.as_str() {
                        "Weapon" => Tab::WeaponEditor,
                        "HealItem" => Tab::HealItemEditor,
                        "MiscItem" => Tab::MiscItemEditor,
                        "EditItem" => Tab::EditItemEditor,
                        "EventItem" => Tab::EventItemEditor,
                        "Monster" => Tab::MonsterEditor,
                        "NpcIni" => Tab::NpcIniEditor,
                        "MagicSpell" => Tab::MagicEditor,
                        "Dialog" => Tab::DialogEditor,
                        "DialogueText" => Tab::DialogueTextEditor,
                        "Store" => Tab::StoreEditor,
                        "PartyRef" => Tab::PartyRefEditor,
                        "PartyIni" => Tab::PartyIniEditor,
                        _ => Tab::WeaponEditor,
                    };
                    self.state.active_tab = tab;
                    self.global_search.is_visible = false;
                    self.global_search.query.clear();
                    self.state.status_msg = format!("Navigated to {} via search", result.display_text);
                }
                return Task::none();
            }

            // ─── App close ────────────────────────────────────────
            Message::CloseRequested => {
                use rfd::MessageDialog;
                use rfd::MessageDialogResult;
                use rfd::MessageButtons;
                let dialog = MessageDialog::new()
                    .set_title("Save workspace?")
                    .set_description("Do you want to save your workspace before closing?")
                    .set_buttons(MessageButtons::YesNoCancel);
                let result = dialog.show();
                match result {
                    MessageDialogResult::Yes => {
                        self.save_workspace();
                        return iced::window::close(self.window_id);
                    }
                    MessageDialogResult::No => {
                        return iced::window::close(self.window_id);
                    }
                    _ => Task::none(),
                }
            }
        }
    }

    fn find_sprites_recursive(dir: &Path, results: &mut Vec<SpriteEntry>) {
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
                        let (seq_count, frame_counts) = Self::analyze_sprite_file(&path);
                        results.push(SpriteEntry {
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
        match dispel_core::sprite::get_sprite_metadata(path) {
            Ok(frame_counts) => (frame_counts.len(), frame_counts),
            Err(_) => (0, Vec::new()),
        }
    }

    fn find_snf_file(game_path: &str, snf_filename: &str) -> PathBuf {
        let direct = PathBuf::from(game_path).join(snf_filename);
        if direct.exists() {
            return direct;
        }

        let candidate = PathBuf::from(game_path).join("Sound").join(snf_filename);
        if candidate.exists() {
            return candidate;
        }

        if let Ok(entries) = std::fs::read_dir(game_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let candidate = path.join(snf_filename);
                    if candidate.exists() {
                        return candidate;
                    }
                }
            }
        }
        direct
    }

    /// Fetch data using the built table query (filters + sorting).
    pub fn fetch_viewer_data(&mut self) -> Task<Message> {
        let table = match &self.state.viewer.active_table {
            Some(t) => t.clone(),
            None => return Task::none(),
        };
        self.state.viewer.is_loading = true;

        // First get column info, then build query
        let path = self.state.viewer.db_path.clone();
        let search = self.state.viewer.search.clone();
        let sort_col = self.state.viewer.sort_col;
        let sort_dir = self.state.viewer.sort_dir;
        let page = self.state.viewer.page;

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
        self.state.viewer.is_loading = true;
        let path = self.state.viewer.db_path.clone();
        let sql = self.state.viewer.sql_query.clone();
        let page = self.state.viewer.page;

        Task::perform(
            async move { db::execute_query(&path, &sql, PAGE_SIZE, page * PAGE_SIZE) },
            Message::ViewerDataLoaded,
        )
    }

    pub fn build_internal_command(&self) -> Option<Box<dyn Command>> {
        let factory = CommandFactory::new();
        match self.state.active_tab {
            Tab::Map => {
                let op = self.state.map_op?;
                let subcommand = match op {
                    MapOp::Tiles => commands::map::MapSubcommand::Tiles {
                        input: self.state.map_input.clone(),
                        output: if self.state.map_output.is_empty() {
                            "out".to_string()
                        } else {
                            self.state.map_output.clone()
                        },
                    },
                    MapOp::Atlas => commands::map::MapSubcommand::Atlas {
                        input: self.state.map_input.clone(),
                        output: self.state.map_output.clone(),
                    },
                    MapOp::Render => commands::map::MapSubcommand::Render {
                        map: self.state.map_map_path.clone(),
                        btl: self.state.map_btl_path.clone(),
                        gtl: self.state.map_gtl_path.clone(),
                        output: self.state.map_output.clone(),
                        save_sprites: self.state.map_save_sprites,
                    },
                    MapOp::FromDb => commands::map::MapSubcommand::FromDb {
                        database: self.state.map_database.clone(),
                        map_id: self.state.map_map_id.clone(),
                        gtl_atlas: self.state.map_gtl_atlas.clone(),
                        btl_atlas: self.state.map_btl_atlas.clone(),
                        atlas_columns: self.state.map_atlas_columns.parse().unwrap_or(48),
                        output: self.state.map_output.clone(),
                        game_path: if self.state.map_game_path.is_empty() {
                            None
                        } else {
                            Some(self.state.map_game_path.clone())
                        },
                    },
                    MapOp::ToDb => commands::map::MapSubcommand::ToDb {
                        database: self.state.map_database.clone(),
                        map: self.state.map_map_path.clone(),
                    },
                    MapOp::Sprites => commands::map::MapSubcommand::Sprites {
                        input: self.state.map_input.clone(),
                        output: if self.state.map_output.is_empty() {
                            "out".to_string()
                        } else {
                            self.state.map_output.clone()
                        },
                    },
                };
                Some(Box::new(factory.create_map_command(subcommand)))
            }
            Tab::Ref => {
                let op = self.state.ref_op?;
                let input = self.state.ref_input.clone();
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
                    RefOp::DialogTexts => {
                        commands::ref_command::RefSubcommand::DialogTexts { input }
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
                // let op = self.state.db_op?;
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
            | Tab::SpriteBrowser
            | Tab::MonsterRefEditor
            | Tab::AllMapIniEditor
            | Tab::DialogEditor
            | Tab::DialogueTextEditor
            | Tab::DrawItemEditor
            | Tab::EventIniEditor
            | Tab::EventNpcRefEditor
            | Tab::ExtraIniEditor
            | Tab::ExtraRefEditor
            | Tab::MapIniEditor
            | Tab::MessageScrEditor
            | Tab::NpcRefEditor
            | Tab::PartyLevelDbEditor
            | Tab::QuestScrEditor
            | Tab::WaveIniEditor
            | Tab::ChDataEditor => None,
        }
    }
}

pub fn view(app: &App) -> Element<'_, Message> {
    if app.workspace_mode {
        view_workspace(app)
    } else {
        app.view()
    }
}

fn view_workspace(app: &App) -> Element<'_, Message> {
    use crate::message::WorkspaceMessage;
    use crate::tab_bar::view_tab_bar;

    let game_path = app.state.workspace.game_path.clone();
    if game_path.is_none() {
        return container(
            column![
                text("Welcome to Workspace Mode").size(20),
                horizontal_space(),
                text("Set a game path to browse files").size(14).style(style::subtle_text),
                horizontal_space(),
                button(text("Select Game Folder"))
                    .on_press(Message::BrowseSharedGamePath),
            ]
            .spacing(20),
        )
        .width(Fill)
        .height(Fill)
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center)
        .into();
    }

    let sidebar = container(
        column![
            row![
                text("Explorer").size(14),
                horizontal_space(),
                button(text("Legacy").size(11))
                    .on_press(Message::Workspace(WorkspaceMessage::ToggleWorkspaceMode))
                    .style(style::chip),
            ]
            .padding([8, 12]),
            horizontal_rule(1),
            app.file_tree
                .view()
                .map(|m| Message::Workspace(WorkspaceMessage::FileTree(m))),
        ]
        .spacing(0)
        .height(Fill),
    )
    .width(220)
    .style(style::sidebar_container);

    let tab_bar =
        view_tab_bar(&app.state.workspace).map(|m| Message::Workspace(WorkspaceMessage::TabBar(m)));

    let content = match app.state.workspace.active() {
        Some(_tab) => app.view_tab_content(),
        None => container(
            column![
                text("Open a file from the explorer").size(16),
                text("or set a game path to browse")
                    .size(12)
                    .style(style::subtle_text),
            ]
            .spacing(8),
        )
        .width(Fill)
        .height(Fill)
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center)
        .into(),
    };

    let main = column![tab_bar, horizontal_rule(1), content,]
        .spacing(0)
        .height(Fill);

    let layout = if app.history_panel_visible {
        use crate::view::history_panel::view_history_panel;
        let history = app.get_active_edit_history();
        row![sidebar, main, view_history_panel(history)].height(Fill).width(Fill)
    } else {
        row![sidebar, main].height(Fill).width(Fill)
    };

    let main_container = container(layout)
        .width(Fill)
        .height(Fill)
        .style(style::root_container);

    if let Some(ref palette) = app.command_palette {
        let palette_view = palette.view();

        let backdrop = container(main_container)
            .width(Fill)
            .height(Fill)
            .style(|_theme| iced::widget::container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgb(
                    0.0, 0.0, 0.0,
                ))),
                ..Default::default()
            });

        let overlay = container(palette_view)
            .center_x(Length::Fill)
            .center_y(Length::Fill);

        return column![backdrop, overlay].width(Fill).height(Fill).into();
    }

    main_container.into()
}
