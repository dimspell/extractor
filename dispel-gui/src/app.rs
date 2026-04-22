use crate::components::command_palette::CommandPalette;
use crate::components::file_tree::FileTree;
use crate::components::tab_bar::TabBarMessage;
use crate::db;
use crate::edit_history::EditHistory;
use crate::message::{
    editor::chest::ChestEditorMessage, Message, MessageExt, SystemMessage, ViewerMessage,
    WorkspaceMessage,
};
use crate::state::db_viewer_state::PAGE_SIZE;
use crate::state::state::AppState;
use crate::workspace::EditorType;
use dispel_core::Extractor;
use iced::{Subscription, Task};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    StartPage,
    EditorMode,
}

pub struct App {
    pub state: AppState,
    pub file_tree: FileTree,
    pub window_id: iced::window::Id,
    pub history_panel_visible: bool,
    pub sidebar_visible: bool,
    pub empty_edit_history: EditHistory,
    pub command_palette: Option<CommandPalette>,
    pub global_search: crate::global_search::GlobalSearch,
    pub draft_manager: crate::auto_save::DraftManager,
    pub search_index: crate::search_index::SearchIndex,
    pub app_mode: AppMode,
    pub start_page_input: String,
    pub is_indexing: bool,
    pub error_dialog: Option<String>,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let mut state = AppState::default();
        if let Err(e) = state.load_workspace() {
            eprintln!("Failed to load workspace: {}", e);
        }
        let game_path = state.workspace.game_path.clone();

        // Initialize cache manager if not already done
        state.initialize_cache_manager();

        let file_tree = if let Some(ref path) = game_path {
            // Use cache-aware file tree scanning
            let cache_manager = state.file_index_cache_manager.clone();
            FileTree::scan_with_cache(path, &cache_manager)
        } else {
            FileTree::default()
        };

        // Try to load existing search index
        let index_path = crate::search_index::SearchIndex::index_path();
        let search_index = if let Some(ref gp) = game_path {
            match crate::search_index::SearchIndex::load(&index_path) {
                Ok(idx) => {
                    if idx.game_path.as_deref() == Some(gp.to_str().unwrap_or("")) {
                        idx
                    } else {
                        let mut fresh = crate::search_index::SearchIndex::new();
                        fresh.game_path = Some(gp.to_string_lossy().to_string());
                        fresh
                    }
                }
                Err(_) => {
                    let mut fresh = crate::search_index::SearchIndex::new();
                    fresh.game_path = game_path.as_ref().map(|p| p.to_string_lossy().to_string());
                    fresh
                }
            }
        } else {
            crate::search_index::SearchIndex::new()
        };

        let init_task: Option<Task<Message>> =
            if game_path.is_some() && search_index.file_mappings.is_empty() {
                game_path.map(|gp| {
                    Task::perform(
                        async move { crate::search_index::build_index(&gp).await },
                        |index| Message::System(SystemMessage::IndexLoaded(Ok(index))),
                    )
                })
            } else {
                None
            };

        // Also start file indexation for cache
        let indexation_task = state.start_file_indexation_if_needed();

        // Trigger initial load for restored active tab if it exists
        let restore_tab_load_task = if let Some(active_tab) = state.workspace.active() {
            // Generate load task based on editor type
            match active_tab.editor_type {
                EditorType::WeaponEditor => Some(Task::done(Message::weapon(
                    crate::message::editor::weapon::WeaponEditorMessage::ScanWeapons,
                ))),
                EditorType::HealItemEditor => Some(Task::done(Message::heal_item(
                    crate::message::editor::healitem::HealItemEditorMessage::ScanItems,
                ))),
                EditorType::MiscItemEditor => Some(Task::done(Message::misc_item(
                    crate::message::editor::miscitem::MiscItemEditorMessage::ScanItems,
                ))),
                EditorType::EditItemEditor => Some(Task::done(Message::edit_item(
                    crate::message::editor::edititem::EditItemEditorMessage::ScanItems,
                ))),
                EditorType::EventItemEditor => Some(Task::done(Message::event_item(
                    crate::message::editor::eventitem::EventItemEditorMessage::ScanItems,
                ))),
                EditorType::MonsterEditor => Some(Task::done(Message::monster(
                    crate::message::editor::monster::MonsterEditorMessage::ScanMonsters,
                ))),
                EditorType::MonsterIniEditor => Some(Task::done(Message::monster_ini(
                    crate::message::editor::monsterini::MonsterIniEditorMessage::LoadCatalog,
                ))),
                EditorType::NpcIniEditor => Some(Task::done(Message::npc_ini(
                    crate::message::editor::npcini::NpcIniEditorMessage::LoadCatalog,
                ))),
                EditorType::MagicEditor => Some(Task::done(Message::magic(
                    crate::message::editor::magic::MagicEditorMessage::LoadCatalog,
                ))),
                EditorType::StoreEditor => Some(Task::done(Message::store(
                    crate::message::editor::store::StoreEditorMessage::ScanStores,
                ))),
                EditorType::PartyRefEditor => Some(Task::done(Message::party_ref(
                    crate::message::editor::partyref::PartyRefEditorMessage::LoadCatalog,
                ))),
                EditorType::PartyIniEditor => Some(Task::done(Message::party_ini(
                    crate::message::editor::partyini::PartyIniEditorMessage::LoadCatalog,
                ))),
                EditorType::AllMapIniEditor => Some(Task::done(Message::all_map_ini(
                    crate::message::editor::allmapini::AllMapIniEditorMessage::LoadCatalog,
                ))),
                EditorType::MapIniEditor => Some(Task::done(Message::map_ini(
                    crate::message::editor::mapini::MapIniEditorMessage::LoadCatalog,
                ))),
                EditorType::ExtraIniEditor => Some(Task::done(Message::extra_ini(
                    crate::message::editor::extraini::ExtraIniEditorMessage::LoadCatalog,
                ))),
                EditorType::EventIniEditor => Some(Task::done(Message::event_ini(
                    crate::message::editor::eventini::EventIniEditorMessage::LoadCatalog,
                ))),
                EditorType::WaveIniEditor => Some(Task::done(Message::wave_ini(
                    crate::message::editor::waveini::WaveIniEditorMessage::LoadCatalog,
                ))),
                EditorType::DrawItemEditor => Some(Task::done(Message::draw_item(
                    crate::message::editor::drawitem::DrawItemEditorMessage::LoadCatalog,
                ))),
                EditorType::EventNpcRefEditor => Some(Task::done(Message::event_npc_ref(
                    crate::message::editor::eventnpcref::EventNpcRefEditorMessage::LoadCatalog,
                ))),
                EditorType::QuestScrEditor => Some(Task::done(Message::quest_scr(
                    crate::message::editor::questscr::QuestScrEditorMessage::LoadCatalog,
                ))),
                EditorType::MessageScrEditor => Some(Task::done(Message::message_scr(
                    crate::message::editor::messagescr::MessageScrEditorMessage::LoadCatalog,
                ))),
                EditorType::PartyLevelDbEditor => Some(Task::done(Message::party_level_db(
                    crate::message::editor::partyleveldb::PartyLevelDbEditorMessage::LoadCatalog,
                ))),
                EditorType::ChDataEditor => Some(Task::done(Message::ch_data(
                    crate::message::editor::chdata::ChDataEditorMessage::LoadCatalog,
                ))),
                _ => None,
            }
        } else {
            None
        };

        // Combine tasks if they exist
        let final_init_task = match (init_task, indexation_task, restore_tab_load_task) {
            (None, None, None) => Task::none(),
            (a, b, None) => match (a, b) {
                (None, None) => Task::none(),
                (None, Some(t)) | (Some(t), None) => t,
                (Some(a), Some(b)) => Task::batch([a, b]),
            },
            (a, b, Some(c)) => {
                let mut tasks: Vec<Task<Message>> = vec![c];
                if let Some(t) = a {
                    tasks.push(t);
                }
                if let Some(t) = b {
                    tasks.push(t);
                }
                Task::batch(tasks)
            }
        };

        let app_mode = if state.workspace.game_path.is_some() {
            AppMode::EditorMode
        } else {
            AppMode::StartPage
        };
        let start_page_input = state
            .workspace
            .game_path
            .as_deref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        let is_indexing = app_mode == AppMode::EditorMode;

        (
            Self {
                state,
                file_tree,
                window_id: iced::window::Id::unique(),
                history_panel_visible: false,
                sidebar_visible: true,
                empty_edit_history: EditHistory::default(),
                command_palette: None,
                global_search: crate::global_search::GlobalSearch::new(),
                draft_manager: crate::auto_save::DraftManager::load(),
                search_index,
                app_mode,
                start_page_input,
                is_indexing,
                error_dialog: None,
            },
            final_init_task,
        )
    }

    #[cfg(test)]
    pub fn test_new(workspace: crate::workspace::Workspace) -> Self {
        use crate::state::state::AppState;

        let mut state = AppState::default();
        state.workspace = workspace;

        Self {
            state,
            file_tree: crate::components::file_tree::FileTree::default(),
            window_id: iced::window::Id::unique(),
            history_panel_visible: false,
            sidebar_visible: true,
            empty_edit_history: EditHistory::default(),
            command_palette: None,
            global_search: crate::global_search::GlobalSearch::new(),
            draft_manager: crate::auto_save::DraftManager::load(),
            search_index: crate::search_index::SearchIndex::new(),
            app_mode: AppMode::EditorMode,
            start_page_input: String::new(),
            is_indexing: false,
            error_dialog: None,
        }
    }

    pub fn get_active_edit_history(&self) -> &EditHistory {
        use crate::generic_editor::UndoRedo;
        use crate::workspace::EditorType;

        if let Some(tab) = self.state.workspace.active() {
            return match tab.editor_type {
                EditorType::HealItemEditor => self.state.heal_item_editor.edit_history(),
                EditorType::MiscItemEditor => self.state.misc_item_editor.edit_history(),
                EditorType::EditItemEditor => self.state.edit_item_editor.edit_history(),
                EditorType::EventItemEditor => self.state.event_item_editor.edit_history(),
                EditorType::MagicEditor => self.state.magic_editor.edit_history(),
                EditorType::WeaponEditor => self.state.weapon_editor.edit_history(),
                EditorType::MonsterRefEditor => {
                    let tab_id = tab.id;
                    self.state
                        .monster_ref_editors
                        .get(&tab_id)
                        .map(|ed| ed.edit_history())
                        .unwrap_or(&self.empty_edit_history)
                }
                EditorType::ExtraRefEditor => {
                    let tab_id = tab.id;
                    self.state
                        .extra_ref_editors
                        .get(&tab_id)
                        .map(|ed| ed.edit_history())
                        .unwrap_or(&self.empty_edit_history)
                }
                EditorType::NpcRefEditor => {
                    let tab_id = tab.id;
                    self.state
                        .npc_ref_editors
                        .get(&tab_id)
                        .map(|ed| ed.edit_history())
                        .unwrap_or(&self.empty_edit_history)
                }
                EditorType::DialogEditor => {
                    let tab_id = tab.id;
                    self.state
                        .dialog_editors
                        .get(&tab_id)
                        .map(|ed| ed.edit_history())
                        .unwrap_or(&self.empty_edit_history)
                }
                EditorType::DialogueTextEditor => {
                    let tab_id = tab.id;
                    self.state
                        .dialogue_text_editors
                        .get(&tab_id)
                        .map(|ed| ed.edit_history())
                        .unwrap_or(&self.empty_edit_history)
                }
                EditorType::DrawItemEditor => self.state.draw_item_editor.edit_history(),
                EditorType::EventIniEditor => self.state.event_ini_editor.edit_history(),
                EditorType::EventNpcRefEditor => self.state.event_npc_ref_editor.edit_history(),
                EditorType::ExtraIniEditor => self.state.extra_ini_editor.edit_history(),
                EditorType::MapIniEditor => self.state.map_ini_editor.edit_history(),
                EditorType::MessageScrEditor => self.state.message_scr_editor.edit_history(),
                EditorType::PartyLevelDbEditor => self.state.party_level_db_editor.edit_history(),
                EditorType::QuestScrEditor => self.state.quest_scr_editor.edit_history(),
                EditorType::WaveIniEditor => self.state.wave_ini_editor.edit_history(),
                EditorType::AllMapIniEditor => self.state.all_map_ini_editor.edit_history(),
                EditorType::ChDataEditor => self.state.chdata_editor.edit_history(),
                EditorType::PartyRefEditor => self.state.party_ref_editor.edit_history(),
                EditorType::PartyIniEditor => self.state.party_ini_editor.edit_history(),
                EditorType::StoreEditor => self.state.store_editor.edit_history(),
                _ => &self.empty_edit_history,
            };
        }
        &self.empty_edit_history
    }

    /// Build a `Message` that delivers `sm` to the spreadsheet of whichever
    /// editor is currently active in the workspace.  Returns `None` when the
    /// active tab has no associated spreadsheet (e.g. map editor, DB viewer).
    pub fn spreadsheet_nav_msg(
        &self,
        sm: crate::view::editor::SpreadsheetMessage,
    ) -> Option<Message> {
        use crate::message::editor::*;
        use crate::workspace::EditorType::*;
        let et = self.state.workspace.active()?.editor_type;
        Some(match et {
            WeaponEditor => Message::weapon(weapon::WeaponEditorMessage::Spreadsheet(sm)),
            MonsterEditor => Message::monster(monster::MonsterEditorMessage::Spreadsheet(sm)),
            MonsterIniEditor => {
                Message::monster_ini(monsterini::MonsterIniEditorMessage::Spreadsheet(sm))
            }
            HealItemEditor => Message::heal_item(healitem::HealItemEditorMessage::Spreadsheet(sm)),
            MiscItemEditor => Message::misc_item(miscitem::MiscItemEditorMessage::Spreadsheet(sm)),
            EditItemEditor => Message::edit_item(edititem::EditItemEditorMessage::Spreadsheet(sm)),
            EventItemEditor => {
                Message::event_item(eventitem::EventItemEditorMessage::Spreadsheet(sm))
            }
            MagicEditor => Message::magic(magic::MagicEditorMessage::Spreadsheet(sm)),
            StoreEditor => return None, // Store editor has a custom layout, no generic spreadsheet
            NpcIniEditor => Message::npc_ini(npcini::NpcIniEditorMessage::Spreadsheet(sm)),
            NpcRefEditor => Message::npc_ref(npcref::NpcRefEditorMessage::Spreadsheet(sm)),
            MonsterRefEditor => {
                Message::monster_ref(monsterref::MonsterRefEditorMessage::Spreadsheet(sm))
            }
            PartyRefEditor => Message::party_ref(partyref::PartyRefEditorMessage::Spreadsheet(sm)),
            PartyIniEditor => Message::party_ini(partyini::PartyIniEditorMessage::Spreadsheet(sm)),
            AllMapIniEditor => {
                Message::all_map_ini(allmapini::AllMapIniEditorMessage::Spreadsheet(sm))
            }
            MapIniEditor => Message::map_ini(mapini::MapIniEditorMessage::Spreadsheet(sm)),
            ExtraIniEditor => Message::extra_ini(extraini::ExtraIniEditorMessage::Spreadsheet(sm)),
            ExtraRefEditor => Message::extra_ref(extraref::ExtraRefEditorMessage::Spreadsheet(sm)),
            EventIniEditor => Message::event_ini(eventini::EventIniEditorMessage::Spreadsheet(sm)),
            EventNpcRefEditor => {
                Message::event_npc_ref(eventnpcref::EventNpcRefEditorMessage::Spreadsheet(sm))
            }
            WaveIniEditor => Message::wave_ini(waveini::WaveIniEditorMessage::Spreadsheet(sm)),
            DrawItemEditor => Message::draw_item(drawitem::DrawItemEditorMessage::Spreadsheet(sm)),
            MessageScrEditor => {
                Message::message_scr(messagescr::MessageScrEditorMessage::Spreadsheet(sm))
            }
            QuestScrEditor => Message::quest_scr(questscr::QuestScrEditorMessage::Spreadsheet(sm)),
            DialogEditor => Message::dialog(dialog::DialogEditorMessage::Spreadsheet(sm)),
            DialogueTextEditor => {
                Message::dialogue_text(dialoguetext::DialogueTextEditorMessage::Spreadsheet(sm))
            }
            ChDataEditor => Message::ch_data(chdata::ChDataEditorMessage::Spreadsheet(sm)),
            PartyLevelDbEditor => {
                Message::party_level_db(partyleveldb::PartyLevelDbEditorMessage::Spreadsheet(sm))
            }
            _ => return None,
        })
    }

    pub fn subscription(&self) -> Subscription<Message> {
        use iced::keyboard::{self, key::Named, Key};
        use iced::window;

        let close =
            window::close_requests().map(|_| Message::System(SystemMessage::CloseRequested));

        let keyboard_sub = keyboard::listen().filter_map(|event| {
            if let keyboard::Event::KeyPressed { key, modifiers, .. } = event {
                if modifiers.control() || modifiers.command() {
                    if let Key::Character(c) = key.as_ref() {
                        let ch = c.chars().next()?;
                        return match ch {
                            'z' => Some(Message::System(SystemMessage::Undo)),
                            'y' => Some(Message::System(SystemMessage::Redo)),
                            's' => Some(Message::System(SystemMessage::Save)),
                            'h' => Some(Message::Workspace(WorkspaceMessage::ToggleHistoryPanel)),
                            'p' => Some(Message::Workspace(WorkspaceMessage::ToggleCommandPalette)),
                            'f' => Some(Message::Workspace(WorkspaceMessage::ToggleGlobalSearch)),
                            'w' => Some(Message::tab_bar(TabBarMessage::CloseActiveTab)),
                            _ => None,
                        };
                    }
                }
                if let Key::Named(named) = key.as_ref() {
                    match named {
                        Named::Escape => {
                            Some(Message::Workspace(WorkspaceMessage::CommandPaletteClose))
                        }
                        Named::Enter => {
                            Some(Message::Workspace(WorkspaceMessage::CommandPaletteConfirm))
                        }
                        Named::ArrowUp => {
                            Some(Message::Workspace(WorkspaceMessage::CommandPaletteArrowUp))
                        }
                        Named::ArrowDown => Some(Message::Workspace(
                            WorkspaceMessage::CommandPaletteArrowDown,
                        )),
                        _ => None,
                    }
                } else {
                    None
                }
            } else {
                None
            }
        });

        // Global search keyboard handling (only when active)
        let global_search_keyboard_sub = keyboard::listen().filter_map(move |event| {
            if let keyboard::Event::KeyPressed { key, .. } = event {
                if let Key::Named(named) = key.as_ref() {
                    match named {
                        Named::Escape => {
                            Some(Message::Workspace(WorkspaceMessage::ToggleGlobalSearch))
                        }
                        Named::Enter => {
                            Some(Message::Workspace(WorkspaceMessage::GlobalSearchConfirm))
                        }
                        Named::ArrowUp => {
                            Some(Message::Workspace(WorkspaceMessage::GlobalSearchArrowUp))
                        }
                        Named::ArrowDown => {
                            Some(Message::Workspace(WorkspaceMessage::GlobalSearchArrowDown))
                        }
                        _ => None,
                    }
                } else {
                    None
                }
            } else {
                None
            }
        });

        // Only include global search keyboard handling when it's active
        let mut subscriptions: Vec<Subscription<Message>> = vec![close, keyboard_sub];
        if self.global_search.is_visible {
            subscriptions.push(global_search_keyboard_sub);
        }

        // Drive animation playback when any sprite viewer is playing.
        if self.state.sprite_viewers.values().any(|v| v.is_playing) {
            use crate::message::editor::spritebrowser::SpriteViewerMessage;
            let anim = iced::time::every(std::time::Duration::from_millis(16))
                .map(|_| Message::sprite_viewer(SpriteViewerMessage::Tick));
            subscriptions.push(anim);
        }

        // Poll for SNF playback completion so the Play/Pause button stays in sync.
        if self
            .state
            .snf_editors
            .values()
            .any(|e| e.playback.is_some())
        {
            use crate::message::editor::snf::SnfEditorMessage;
            let snf_tick = iced::time::every(std::time::Duration::from_millis(250))
                .map(|_| Message::snf_editor(SnfEditorMessage::Tick));
            subscriptions.push(snf_tick);
        }

        // Spreadsheet row navigation (Arrow / Home / End keys).
        // Only active when a spreadsheet editor is in the foreground and no
        // overlay is consuming keys.
        let active_et = self.state.workspace.active().map(|t| t.editor_type);
        let palette_open = self.command_palette.is_some();
        let search_open = self.global_search.is_visible;

        if !palette_open && !search_open {
            if let Some(et) = active_et {
                use crate::view::editor::SpreadsheetMessage as SM;
                // Probe whether this editor type has a spreadsheet.
                if build_spreadsheet_nav_msg(et, SM::NavigateUp).is_some() {
                    // Pass `et` via `.with()` so the closure itself is zero-sized
                    // (iced 0.14 requires filter_map closures to be non-capturing).
                    let ss_sub = keyboard::listen().with(et).filter_map(|(et, event)| {
                        if let keyboard::Event::KeyPressed { key, modifiers, .. } = event {
                            if modifiers.control() || modifiers.command() || modifiers.shift() {
                                return None;
                            }
                            if let Key::Named(named) = key.as_ref() {
                                use crate::view::editor::SpreadsheetMessage as SM;
                                let sm = match named {
                                    Named::ArrowUp => SM::NavigateUp,
                                    Named::ArrowDown => SM::NavigateDown,
                                    Named::Home => SM::NavigateTop,
                                    Named::End => SM::NavigateBottom,
                                    Named::Escape => SM::CancelEdit,
                                    _ => return None,
                                };
                                return build_spreadsheet_nav_msg(et, sm);
                            }
                        }
                        None
                    });
                    subscriptions.push(ss_sub);
                }
            }
        }

        Subscription::batch(subscriptions)
    }

    pub fn save_workspace(&self) {
        if let Err(e) = self.state.save_workspace() {
            eprintln!("Failed to save workspace: {}", e);
        }
    }

    /// Track a file in the recent files list
    pub fn track_recent_file(&mut self, path: &Path) {
        // Add file to beginning of recent files list
        self.state.recent_files.retain(|p| p != path); // Remove if already exists
        self.state.recent_files.insert(0, path.to_path_buf());

        // Limit to 10 most recent files (LRU eviction)
        if self.state.recent_files.len() > 10 {
            self.state.recent_files.truncate(10);
        }
    }

    pub fn open_file_in_workspace(&mut self, path: &Path) -> Task<Message> {
        let label = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        self.state.workspace.open(label, Some(path.to_path_buf()));

        // Track recent file (add to beginning of list)
        self.track_recent_file(path);

        self.save_workspace();

        // Auto-load the file based on extension
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            let stem = path
                .file_stem()
                .map(|s| s.to_string_lossy().to_lowercase())
                .unwrap_or_default();
            match ext {
                "db" => match stem.as_str() {
                    "weaponitem" => return self.load_editor_auto("weapons"),
                    "monster" => return self.load_editor_auto("monster_db"),
                    "healitem" => return self.load_editor_auto("heal_items"),
                    "miscitem" => return self.load_editor_auto("misc_items"),
                    "edititem" => return self.load_editor_auto("edit_items"),
                    "eventitem" => return self.load_editor_auto("event_items"),
                    "store" => return self.load_editor_auto("stores"),
                    "magic" => return self.load_editor_auto("magic"),
                    "chdata" => return self.load_editor_auto("chdata"),
                    "prtlevel" => return self.load_editor_auto("party_levels"),
                    "prtini" => return self.load_editor_auto("party_ini"),
                    _ => {}
                },
                "ini" => match stem.as_str() {
                    "allmap" => return self.load_editor_auto("all_maps"),
                    "map" => return self.load_editor_auto("map_ini"),
                    "extra" => return self.load_editor_auto("extra_ini"),
                    "event" => return self.load_editor_auto("event_ini"),
                    "monster" => return self.load_editor_auto("monster_ini"),
                    "npc" => return self.load_editor_auto("npc_ini"),
                    "wave" => return self.load_editor_auto("wave_ini"),
                    _ => {}
                },
                "ref" => match stem.as_str() {
                    "partyref" => return self.load_editor_auto("party_ref"),
                    "drawitem" => return self.load_editor_auto("draw_items"),
                    "eventnpc" => return self.load_editor_auto("event_npc_ref"),
                    _ => {
                        if stem.starts_with("npc") {
                            if let Some(tab_idx) = self.state.workspace.active_tab {
                                if let Some(tab) = self.state.workspace.tabs.get(tab_idx) {
                                    let tab_id = tab.id;
                                    let mut editor_state =
                                        crate::state::npc_ref_editor::NpcRefEditorState::default();
                                    editor_state.select_file(path.to_path_buf());

                                    // Initialize spreadsheet state
                                    let mut ss = crate::view::editor::SpreadsheetState::new();
                                    if let Some(catalog) = editor_state.editor.catalog.as_ref() {
                                        ss.apply_filter(catalog);
                                        ss.init_pane_state();
                                    }

                                    self.state.npc_ref_editors.insert(tab_id, editor_state);
                                    self.state.npc_ref_spreadsheets.insert(tab_id, ss);
                                }
                            }
                        } else if stem.starts_with("mon") {
                            if let Some(tab_idx) = self.state.workspace.active_tab {
                                if let Some(tab) = self.state.workspace.tabs.get(tab_idx) {
                                    let tab_id = tab.id;
                                    let mut editor_state =
                                        crate::state::monster_ref_editor::MonsterRefEditorState::default();
                                    editor_state.select_file(path.to_path_buf());

                                    // Initialize spreadsheet state
                                    let mut ss = crate::view::editor::SpreadsheetState::new();
                                    if let Some(catalog) = editor_state.editor.catalog.as_ref() {
                                        ss.apply_filter(catalog);
                                        ss.init_pane_state();
                                    }

                                    self.state.monster_ref_editors.insert(tab_id, editor_state);
                                    self.state.monster_ref_spreadsheets.insert(tab_id, ss);
                                    if !self.state.lookups.contains_key("monster_names") {
                                        return Task::done(crate::message::Message::monster_ref(
                                            crate::message::editor::monsterref::MonsterRefEditorMessage::LoadMonsterNames,
                                        ));
                                    }
                                }
                            }
                        } else if stem.starts_with("ext") {
                            if let Some(tab_idx) = self.state.workspace.active_tab {
                                if let Some(tab) = self.state.workspace.tabs.get(tab_idx) {
                                    let tab_id = tab.id;
                                    let mut editor_state =
                                        crate::state::extra_ref_editor::ExtraRefEditorState::default();
                                    editor_state.select_file(path.to_path_buf());

                                    // Initialize spreadsheet state
                                    let mut ss = crate::view::editor::SpreadsheetState::new();
                                    if let Some(catalog) = editor_state.editor.catalog.as_ref() {
                                        ss.apply_filter(catalog);
                                        ss.init_pane_state();
                                    }

                                    self.state.extra_ref_editors.insert(tab_id, editor_state);
                                    self.state.extra_ref_spreadsheets.insert(tab_id, ss);
                                    let path_buf = path.to_path_buf();
                                    return Task::perform(
                                        async move {
                                            dispel_core::ExtraRef::read_file(&path_buf)
                                                .map_err(|e: std::io::Error| e.to_string())
                                        },
                                        move |result| {
                                            crate::message::Message::Editor(
                                                crate::message::editor::EditorMessage::ExtraRef(
                                                    crate::message::editor::extraref::ExtraRefEditorMessage::CatalogLoaded(tab_id, result),
                                                ),
                                            )
                                        },
                                    );
                                }
                            }
                        }
                    }
                },
                "scr" => match stem.as_str() {
                    "quest" => return self.load_editor_auto("quests"),
                    "message" => return self.load_editor_auto("messages"),
                    _ => {}
                },
                "dlg" => {
                    if let Some(tab_idx) = self.state.workspace.active_tab {
                        if let Some(tab) = self.state.workspace.tabs.get(tab_idx) {
                            let tab_id = tab.id;
                            let path_str = path.to_string_lossy().to_string();
                            let editor_state = crate::state::dialog_editor::DialogEditorState {
                                current_file: path_str,
                                editor: Default::default(),
                            };
                            self.state.dialog_editors.insert(tab_id, editor_state);
                            self.state
                                .dialog_spreadsheets
                                .insert(tab_id, Default::default());
                            let path_buf = path.to_path_buf();
                            return Task::perform(
                                async move {
                                    dispel_core::Dialog::read_file(&path_buf)
                                        .map_err(|e: std::io::Error| e.to_string())
                                },
                                move |result| {
                                    crate::message::Message::Editor(
                                        crate::message::editor::EditorMessage::Dialog(
                                            crate::message::editor::dialog::DialogEditorMessage::Scanned(result),
                                        ),
                                    )
                                },
                            );
                        }
                    }
                }
                "pgp" => {
                    if let Some(tab_idx) = self.state.workspace.active_tab {
                        if let Some(tab) = self.state.workspace.tabs.get(tab_idx) {
                            let tab_id = tab.id;
                            let path_str = path.to_string_lossy().to_string();
                            let editor_state =
                                crate::state::dialogue_text_editor::DialogueTextEditorState {
                                    editor: crate::generic_editor::GenericEditorState::default(),
                                    current_file: path_str,
                                };
                            self.state
                                .dialogue_text_editors
                                .insert(tab_id, editor_state);
                            self.state
                                .dialogue_text_spreadsheets
                                .insert(tab_id, Default::default());
                            let path_buf = path.to_path_buf();
                            return Task::perform(
                                async move {
                                    dispel_core::DialogueText::read_file(&path_buf)
                                        .map_err(|e: std::io::Error| e.to_string())
                                },
                                move |result| {
                                    crate::message::Message::Editor(
                                        crate::message::editor::EditorMessage::DialogueText(
                                            crate::message::editor::dialoguetext::DialogueTextEditorMessage::CatalogLoaded(tab_id, result),
                                        ),
                                    )
                                },
                            );
                        }
                    }
                }
                ext if ext.eq_ignore_ascii_case("btl") || ext.eq_ignore_ascii_case("gtl") => {
                    if let Some(tab_idx) = self.state.workspace.active_tab {
                        if let Some(tab) = self.state.workspace.tabs.get(tab_idx) {
                            let tab_id = tab.id;
                            self.state.tileset_editors.entry(tab_id).or_insert_with(|| {
                                crate::state::tileset_editor::TilesetEditorState::load(path)
                            });
                        }
                    }
                }
                ext if ext.eq_ignore_ascii_case("spr") => {
                    if let Some(tab_idx) = self.state.workspace.active_tab {
                        if let Some(tab) = self.state.workspace.tabs.get(tab_idx) {
                            let tab_id = tab.id;
                            self.state.sprite_viewers.entry(tab_id).or_insert_with(|| {
                                crate::state::sprite_viewer::SpriteViewerState::load_from_path(path)
                            });
                        }
                    }
                }
                ext if ext.eq_ignore_ascii_case("snf") => {
                    if let Some(tab_idx) = self.state.workspace.active_tab {
                        if let Some(tab) = self.state.workspace.tabs.get(tab_idx) {
                            let tab_id = tab.id;
                            self.state.snf_editors.entry(tab_id).or_insert_with(|| {
                                crate::state::snf_editor::SnfEditorState::load_from_path(path)
                            });
                        }
                    }
                }
                ext if ext.eq_ignore_ascii_case("map") => {
                    if let Some(tab_idx) = self.state.workspace.active_tab {
                        if let Some(tab) = self.state.workspace.tabs.get(tab_idx) {
                            let tab_id = tab.id;
                            let path_buf = path.to_path_buf();
                            return Task::done(Message::map_editor(
                                crate::message::editor::map_editor::MapEditorMessage::Open(
                                    tab_id, path_buf,
                                ),
                            ));
                        }
                    }
                }
                _ => {}
            }
        }

        Task::none()
    }

    /// Dispatch the appropriate scan/load message for a file opened from the file tree.
    pub fn load_editor_auto(&self, type_name: &str) -> Task<Message> {
        use crate::message::MessageExt;
        match type_name {
            "weapons" => Task::done(Message::weapon(
                crate::message::editor::weapon::WeaponEditorMessage::ScanWeapons,
            )),
            "monster_db" => Task::done(Message::monster(
                crate::message::editor::monster::MonsterEditorMessage::ScanMonsters,
            )),
            "heal_items" => Task::done(Message::heal_item(
                crate::message::editor::healitem::HealItemEditorMessage::ScanItems,
            )),
            "misc_items" => Task::done(Message::misc_item(
                crate::message::editor::miscitem::MiscItemEditorMessage::ScanItems,
            )),
            "edit_items" => Task::done(Message::edit_item(
                crate::message::editor::edititem::EditItemEditorMessage::ScanItems,
            )),
            "event_items" => Task::done(Message::event_item(
                crate::message::editor::eventitem::EventItemEditorMessage::ScanItems,
            )),
            "stores" => Task::done(Message::store(
                crate::message::editor::store::StoreEditorMessage::ScanStores,
            )),
            "magic" => Task::done(Message::magic(
                crate::message::editor::magic::MagicEditorMessage::LoadCatalog,
            )),
            "chdata" => Task::done(Message::ch_data(
                crate::message::editor::chdata::ChDataEditorMessage::LoadCatalog,
            )),
            "party_levels" => Task::done(Message::party_level_db(
                crate::message::editor::partyleveldb::PartyLevelDbEditorMessage::LoadCatalog,
            )),
            "party_ini" => Task::done(Message::party_ini(
                crate::message::editor::partyini::PartyIniEditorMessage::LoadCatalog,
            )),
            "all_maps" => Task::done(Message::all_map_ini(
                crate::message::editor::allmapini::AllMapIniEditorMessage::LoadCatalog,
            )),
            "map_ini" => Task::done(Message::map_ini(
                crate::message::editor::mapini::MapIniEditorMessage::LoadCatalog,
            )),
            "extra_ini" => Task::done(Message::extra_ini(
                crate::message::editor::extraini::ExtraIniEditorMessage::LoadCatalog,
            )),
            "event_ini" => Task::done(Message::event_ini(
                crate::message::editor::eventini::EventIniEditorMessage::LoadCatalog,
            )),
            "monster_ini" => Task::done(Message::monster_ini(
                crate::message::editor::monsterini::MonsterIniEditorMessage::LoadCatalog,
            )),
            "npc_ini" => Task::done(Message::npc_ini(
                crate::message::editor::npcini::NpcIniEditorMessage::LoadCatalog,
            )),
            "wave_ini" => Task::done(Message::wave_ini(
                crate::message::editor::waveini::WaveIniEditorMessage::LoadCatalog,
            )),
            "quests" => Task::done(Message::quest_scr(
                crate::message::editor::questscr::QuestScrEditorMessage::LoadCatalog,
            )),
            "messages" => Task::done(Message::message_scr(
                crate::message::editor::messagescr::MessageScrEditorMessage::LoadCatalog,
            )),
            "party_ref" => Task::done(Message::party_ref(
                crate::message::editor::partyref::PartyRefEditorMessage::LoadCatalog,
            )),
            "draw_items" => Task::done(Message::draw_item(
                crate::message::editor::drawitem::DrawItemEditorMessage::LoadCatalog,
            )),
            "event_npc_ref" => Task::done(Message::event_npc_ref(
                crate::message::editor::eventnpcref::EventNpcRefEditorMessage::LoadCatalog,
            )),
            _ => Task::none(),
        }
    }

    // fn load_single_file<R: Extractor + 'static>(&mut self, editor: &mut GenericEditorState<R>, path: &Path) {
    //     use dispel_core::Extractor;
    //     match R::read_file(path) {
    //         Ok(catalog) => {
    //             editor.catalog = Some(catalog);
    //             editor.refresh();
    //         }
    //         Err(e) => {
    //             editor.status_msg = format!("Error loading {}: {}", path.display(), e);
    //         }
    //     }
    // }

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
        self.state.chest_editor.loading_state = crate::loading_state::LoadingState::Loading;
        Task::perform(
            async move { dispel_core::ExtraRef::read_file(&path) },
            |res: Result<Vec<dispel_core::ExtraRef>, std::io::Error>| {
                let indexed = res.map(|vec| vec.into_iter().enumerate().collect());
                Message::chest(ChestEditorMessage::MapLoaded(
                    indexed.map_err(|e| e.to_string()),
                ))
            },
        )
    }

    pub fn find_snf_file(game_path: &str, snf_filename: &str) -> PathBuf {
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
        self.state.viewer.loading_state = crate::loading_state::LoadingState::Loading;

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
            |result| Message::Viewer(ViewerMessage::DataLoaded(result)),
        )
    }

    /// Fetch data using the custom SQL query.
    pub fn fetch_viewer_data_sql(&mut self) -> Task<Message> {
        self.state.viewer.loading_state = crate::loading_state::LoadingState::Loading;
        let path = self.state.viewer.db_path.clone();
        let sql = self.state.viewer.sql_query.clone();
        let page = self.state.viewer.page;

        Task::perform(
            async move { db::execute_query(&path, &sql, PAGE_SIZE, page * PAGE_SIZE) },
            |result| Message::Viewer(ViewerMessage::DataLoaded(result)),
        )
    }
}

/// Determine which editor key should be used for a given file path.
/// This function extracts the extension and stem (lowercase filename without extension)
/// and returns the editor key that should be opened.
///
/// Returns Some(editor_key) if a known editor should open the file,
/// or None if the file type is not recognized.
pub fn get_editor_key_for_file(path: &Path) -> Option<&'static str> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase());
    let ext_str = ext.as_deref()?;
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    match ext_str {
        "db" => match stem.as_str() {
            "weaponitem" => Some("weapons"),
            "monster" => Some("monster_db"),
            "healitem" => Some("heal_items"),
            "miscitem" => Some("misc_items"),
            "edititem" => Some("edit_items"),
            "eventitem" => Some("event_items"),
            "store" => Some("stores"),
            "magic" => Some("magic"),
            "chdata" => Some("chdata"),
            "prtlevel" => Some("party_levels"),
            "prtini" => Some("party_ini"),
            _ => None,
        },
        "ini" => match stem.as_str() {
            "allmap" => Some("all_maps"),
            "map" => Some("map_ini"),
            "extra" => Some("extra_ini"),
            "event" => Some("event_ini"),
            "monster" => Some("monster_ini"),
            "npc" => Some("npc_ini"),
            "wave" => Some("wave_ini"),
            _ => None,
        },
        "scr" => match stem.as_str() {
            "quest" => Some("quests"),
            "message" => Some("messages"),
            _ => None,
        },
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monster_db_opens_monster_db_editor() {
        let path = Path::new("MonsterInGame/Monster.db");
        assert_eq!(get_editor_key_for_file(path), Some("monster_db"));
    }

    #[test]
    fn test_monster_ini_opens_monster_ini_editor() {
        let path = Path::new("Monster.ini");
        assert_eq!(get_editor_key_for_file(path), Some("monster_ini"));
    }

    #[test]
    fn test_monster_ini_case_insensitive() {
        // Test with different case variations
        let path1 = Path::new("MONSTER.INI");
        let path2 = Path::new("Monster.ini");
        let path3 = Path::new("monster.INI");

        assert_eq!(get_editor_key_for_file(path1), Some("monster_ini"));
        assert_eq!(get_editor_key_for_file(path2), Some("monster_ini"));
        assert_eq!(get_editor_key_for_file(path3), Some("monster_ini"));
    }

    #[test]
    fn test_monster_db_case_insensitive() {
        // Test with different case variations
        let path1 = Path::new("MONSTER.DB");
        let path2 = Path::new("Monster.db");
        let path3 = Path::new("monster.DB");

        assert_eq!(get_editor_key_for_file(path1), Some("monster_db"));
        assert_eq!(get_editor_key_for_file(path2), Some("monster_db"));
        assert_eq!(get_editor_key_for_file(path3), Some("monster_db"));
    }

    #[test]
    fn test_other_db_files_open_correct_editors() {
        assert_eq!(
            get_editor_key_for_file(Path::new("weaponItem.db")),
            Some("weapons")
        );
        assert_eq!(
            get_editor_key_for_file(Path::new("HealItem.db")),
            Some("heal_items")
        );
        assert_eq!(
            get_editor_key_for_file(Path::new("MiscItem.db")),
            Some("misc_items")
        );
        assert_eq!(
            get_editor_key_for_file(Path::new("Magic.db")),
            Some("magic")
        );
    }

    #[test]
    fn test_other_ini_files_open_correct_editors() {
        assert_eq!(
            get_editor_key_for_file(Path::new("Npc.ini")),
            Some("npc_ini")
        );
        assert_eq!(
            get_editor_key_for_file(Path::new("Wave.ini")),
            Some("wave_ini")
        );
        assert_eq!(
            get_editor_key_for_file(Path::new("Map.ini")),
            Some("map_ini")
        );
        assert_eq!(
            get_editor_key_for_file(Path::new("Extra.ini")),
            Some("extra_ini")
        );
        assert_eq!(
            get_editor_key_for_file(Path::new("Event.ini")),
            Some("event_ini")
        );
        assert_eq!(
            get_editor_key_for_file(Path::new("AllMap.ini")),
            Some("all_maps")
        );
    }

    #[test]
    fn test_script_files_open_correct_editors() {
        assert_eq!(
            get_editor_key_for_file(Path::new("Quest.scr")),
            Some("quests")
        );
        assert_eq!(
            get_editor_key_for_file(Path::new("Message.scr")),
            Some("messages")
        );
    }

    #[test]
    fn test_unknown_file_types_return_none() {
        assert_eq!(get_editor_key_for_file(Path::new("Unknown.xyz")), None);
        assert_eq!(get_editor_key_for_file(Path::new("RandomFile.txt")), None);
        assert_eq!(get_editor_key_for_file(Path::new("NoExtension")), None);
    }

    #[test]
    fn test_unknown_db_files_return_none() {
        assert_eq!(get_editor_key_for_file(Path::new("Unknown.db")), None);
        assert_eq!(get_editor_key_for_file(Path::new("RandomFile.db")), None);
    }

    #[test]
    fn test_unknown_ini_files_return_none() {
        assert_eq!(get_editor_key_for_file(Path::new("Unknown.ini")), None);
        assert_eq!(get_editor_key_for_file(Path::new("RandomConfig.ini")), None);
    }

    #[test]
    fn test_paths_with_directories() {
        assert_eq!(
            get_editor_key_for_file(Path::new("CharacterInGame/weaponItem.db")),
            Some("weapons")
        );
        assert_eq!(
            get_editor_key_for_file(Path::new("MonsterInGame/Monster.db")),
            Some("monster_db")
        );
        assert_eq!(
            get_editor_key_for_file(Path::new("Dispel/Monster.ini")),
            Some("monster_ini")
        );
    }
}

/// Map `(EditorType, SpreadsheetMessage)` to the correct `Message` variant.
/// Returns `None` for editor types that have no spreadsheet (map editor, sprite
/// viewer, etc.) so callers can use this as a capability check.
fn build_spreadsheet_nav_msg(
    et: crate::workspace::EditorType,
    sm: crate::view::editor::SpreadsheetMessage,
) -> Option<crate::message::Message> {
    use crate::message::editor::*;
    use crate::message::Message;
    use crate::message::MessageExt as _;
    use crate::workspace::EditorType::*;
    Some(match et {
        WeaponEditor => Message::weapon(weapon::WeaponEditorMessage::Spreadsheet(sm)),
        MonsterEditor => Message::monster(monster::MonsterEditorMessage::Spreadsheet(sm)),
        MonsterIniEditor => {
            Message::monster_ini(monsterini::MonsterIniEditorMessage::Spreadsheet(sm))
        }
        HealItemEditor => Message::heal_item(healitem::HealItemEditorMessage::Spreadsheet(sm)),
        MiscItemEditor => Message::misc_item(miscitem::MiscItemEditorMessage::Spreadsheet(sm)),
        EditItemEditor => Message::edit_item(edititem::EditItemEditorMessage::Spreadsheet(sm)),
        EventItemEditor => Message::event_item(eventitem::EventItemEditorMessage::Spreadsheet(sm)),
        MagicEditor => Message::magic(magic::MagicEditorMessage::Spreadsheet(sm)),
        StoreEditor => return None, // Store editor has a custom layout, no generic spreadsheet
        NpcIniEditor => Message::npc_ini(npcini::NpcIniEditorMessage::Spreadsheet(sm)),
        NpcRefEditor => Message::npc_ref(npcref::NpcRefEditorMessage::Spreadsheet(sm)),
        MonsterRefEditor => {
            Message::monster_ref(monsterref::MonsterRefEditorMessage::Spreadsheet(sm))
        }
        PartyRefEditor => Message::party_ref(partyref::PartyRefEditorMessage::Spreadsheet(sm)),
        PartyIniEditor => Message::party_ini(partyini::PartyIniEditorMessage::Spreadsheet(sm)),
        AllMapIniEditor => Message::all_map_ini(allmapini::AllMapIniEditorMessage::Spreadsheet(sm)),
        MapIniEditor => Message::map_ini(mapini::MapIniEditorMessage::Spreadsheet(sm)),
        ExtraIniEditor => Message::extra_ini(extraini::ExtraIniEditorMessage::Spreadsheet(sm)),
        ExtraRefEditor => Message::extra_ref(extraref::ExtraRefEditorMessage::Spreadsheet(sm)),
        EventIniEditor => Message::event_ini(eventini::EventIniEditorMessage::Spreadsheet(sm)),
        EventNpcRefEditor => {
            Message::event_npc_ref(eventnpcref::EventNpcRefEditorMessage::Spreadsheet(sm))
        }
        WaveIniEditor => Message::wave_ini(waveini::WaveIniEditorMessage::Spreadsheet(sm)),
        DrawItemEditor => Message::draw_item(drawitem::DrawItemEditorMessage::Spreadsheet(sm)),
        MessageScrEditor => {
            Message::message_scr(messagescr::MessageScrEditorMessage::Spreadsheet(sm))
        }
        QuestScrEditor => Message::quest_scr(questscr::QuestScrEditorMessage::Spreadsheet(sm)),
        DialogEditor => Message::dialog(dialog::DialogEditorMessage::Spreadsheet(sm)),
        DialogueTextEditor => {
            Message::dialogue_text(dialoguetext::DialogueTextEditorMessage::Spreadsheet(sm))
        }
        ChDataEditor => Message::ch_data(chdata::ChDataEditorMessage::Spreadsheet(sm)),
        PartyLevelDbEditor => {
            Message::party_level_db(partyleveldb::PartyLevelDbEditorMessage::Spreadsheet(sm))
        }
        _ => return None,
    })
}
