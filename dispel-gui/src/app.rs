use crate::components::command_palette::CommandPalette;
use crate::components::edit_history::EditHistory;
use crate::components::file_tree::FileTree;
use crate::components::tab_bar::TabBarMessage;
use crate::editors::chest::ChestEditorMessage;
use crate::editors::db_viewer::db;
use crate::editors::db_viewer::PAGE_SIZE;
use crate::editors::hex_editor::HexEditorState;
use crate::editors::snf_editor::SnfEditorState;
use crate::editors::sprite_browser::SpriteViewerState;
use crate::editors::tileset::TilesetEditorState;
use crate::message::{Message, MessageExt, SystemMessage, ViewerMessage, WorkspaceMessage};
use crate::state::AppState;
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
    pub global_search: crate::components::global_search::GlobalSearch,
    pub draft_manager: crate::auto_save::DraftManager,
    pub search_index: crate::indexation::search_index::SearchIndex,
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
        let index_path = crate::indexation::search_index::SearchIndex::index_path();
        let search_index = if let Some(ref gp) = game_path {
            match crate::indexation::search_index::SearchIndex::load(&index_path) {
                Ok(idx) => {
                    if idx.game_path.as_deref() == Some(gp.to_str().unwrap_or("")) {
                        idx
                    } else {
                        let mut fresh = crate::indexation::search_index::SearchIndex::new();
                        fresh.game_path = Some(gp.to_string_lossy().to_string());
                        fresh
                    }
                }
                Err(_) => {
                    let mut fresh = crate::indexation::search_index::SearchIndex::new();
                    fresh.game_path = game_path.as_ref().map(|p| p.to_string_lossy().to_string());
                    fresh
                }
            }
        } else {
            crate::indexation::search_index::SearchIndex::new()
        };

        let init_task: Option<Task<Message>> =
            if game_path.is_some() && search_index.file_mappings.is_empty() {
                game_path.map(|gp| {
                    Task::perform(
                        async move { crate::indexation::search_index::build_index(&gp).await },
                        |index| Message::System(SystemMessage::IndexLoaded(Ok(index))),
                    )
                })
            } else {
                None
            };

        // Also start file indexation for cache
        let indexation_task = state.start_file_indexation_if_needed();

        // Combine tasks if they exist
        let final_init_task = match (init_task, indexation_task) {
            (None, None) => Task::none(),
            (a, b) => match (a, b) {
                (None, None) => Task::none(),
                (None, Some(t)) | (Some(t), None) => t,
                (Some(a), Some(b)) => Task::batch([a, b]),
            },
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
                global_search: crate::components::global_search::GlobalSearch::new(),
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
        use crate::state::AppState;

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
            global_search: crate::components::global_search::GlobalSearch::new(),
            draft_manager: crate::auto_save::DraftManager::load(),
            search_index: crate::indexation::search_index::SearchIndex::new(),
            app_mode: AppMode::EditorMode,
            start_page_input: String::new(),
            is_indexing: false,
            error_dialog: None,
        }
    }

    pub fn get_active_edit_history(&self) -> &EditHistory {
        use crate::components::generic_editor::UndoRedo;
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
                EditorType::DialogueScriptEditor => {
                    let tab_id = tab.id;
                    self.state
                        .dialogue_script_editors
                        .get(&tab_id)
                        .map(|ed| ed.edit_history())
                        .unwrap_or(&self.empty_edit_history)
                }
                EditorType::DialogueTextEditor => {
                    let tab_id = tab.id;
                    self.state
                        .dialogue_paragraphs_editors
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
                EditorType::EventScrEditor => &self.empty_edit_history,
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
        use crate::editors::*;
        use crate::workspace::EditorType::*;
        let et = self.state.workspace.active()?.editor_type;
        Some(match et {
            WeaponEditor => Message::weapon(weapon::WeaponEditorMessage::Spreadsheet(sm)),
            MonsterEditor => Message::monster(monster::MonsterEditorMessage::Spreadsheet(sm)),
            MonsterIniEditor => {
                Message::monster_ini(monster_ini::MonsterIniEditorMessage::Spreadsheet(sm))
            }
            HealItemEditor => Message::heal_item(heal_item::HealItemEditorMessage::Spreadsheet(sm)),
            MiscItemEditor => Message::misc_item(misc_item::MiscItemEditorMessage::Spreadsheet(sm)),
            EditItemEditor => Message::edit_item(edit_item::EditItemEditorMessage::Spreadsheet(sm)),
            EventItemEditor => {
                Message::event_item(event_item::EventItemEditorMessage::Spreadsheet(sm))
            }
            MagicEditor => Message::magic(magic::MagicEditorMessage::Spreadsheet(sm)),
            StoreEditor => return None, // Store editor has a custom layout, no generic spreadsheet
            NpcIniEditor => Message::npc_ini(npc_ini::NpcIniEditorMessage::Spreadsheet(sm)),
            NpcRefEditor => Message::npc_ref(npc_ref::NpcRefEditorMessage::Spreadsheet(sm)),
            MonsterRefEditor => {
                Message::monster_ref(monster_ref::MonsterRefEditorMessage::Spreadsheet(sm))
            }
            PartyRefEditor => Message::party_ref(party_ref::PartyRefEditorMessage::Spreadsheet(sm)),
            PartyIniEditor => Message::party_ini(party_ini::PartyIniEditorMessage::Spreadsheet(sm)),
            AllMapIniEditor => {
                Message::all_map_ini(all_map_ini::AllMapIniEditorMessage::Spreadsheet(sm))
            }
            MapIniEditor => Message::map_ini(map_ini::MapIniEditorMessage::Spreadsheet(sm)),
            ExtraIniEditor => Message::extra_ini(extra_ini::ExtraIniEditorMessage::Spreadsheet(sm)),
            ExtraRefEditor => Message::extra_ref(extra_ref::ExtraRefEditorMessage::Spreadsheet(sm)),
            EventIniEditor => Message::event_ini(event_ini::EventIniEditorMessage::Spreadsheet(sm)),
            EventNpcRefEditor => {
                Message::event_npc_ref(event_npc_ref::EventNpcRefEditorMessage::Spreadsheet(sm))
            }
            WaveIniEditor => Message::wave_ini(wave_ini::WaveIniEditorMessage::Spreadsheet(sm)),
            DrawItemEditor => Message::draw_item(draw_item::DrawItemEditorMessage::Spreadsheet(sm)),
            MessageScrEditor => {
                Message::message_scr(message_scr::MessageScrEditorMessage::Spreadsheet(sm))
            }
            QuestScrEditor => Message::quest_scr(quest_scr::QuestScrEditorMessage::Spreadsheet(sm)),
            DialogueScriptEditor => Message::dialogue_script(
                dialogue_script::DialogueScriptEditorMessage::Spreadsheet(sm),
            ),
            DialogueTextEditor => Message::dialogue_paragraph(
                dialogue_paragraph::DialogueParagraphEditorMessage::Spreadsheet(sm),
            ),
            ChDataEditor => Message::chdata(chdata::ChDataEditorMessage::Spreadsheet(sm)),
            PartyLevelDbEditor => {
                Message::party_level_db(party_level_db::PartyLevelDbEditorMessage::Spreadsheet(sm))
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
                        // Shift+X → reopen active file in hex editor
                        if modifiers.shift() && ch == 'x' {
                            return Some(Message::Workspace(
                                WorkspaceMessage::ReopenActiveTabAsHex,
                            ));
                        }
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
            use crate::editors::sprite_browser::SpriteViewerMessage;
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
            use crate::editors::snf_editor::SnfEditorMessage;
            let snf_tick = iced::time::every(std::time::Duration::from_millis(250))
                .map(|_| Message::snf_editor(SnfEditorMessage::Tick));
            subscriptions.push(snf_tick);
        }

        // Poll for event script indexing progress.
        if matches!(
            self.state.event_scr_editor.index_state,
            crate::editors::event_scr::FunctionIndexState::Indexing { .. }
        ) {
            let index_tick = iced::time::every(std::time::Duration::from_millis(100)).map(|_| {
                Message::event_scr(crate::editors::event_scr::EventScrEditorMessage::IndexTick)
            });
            subscriptions.push(index_tick);
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

        // Event Script Editor keyboard shortcuts.
        if !palette_open
            && !search_open
            && active_et == Some(crate::workspace::EditorType::EventScrEditor)
        {
            use crate::editors::event_scr::{EventScrEditorMessage, KeyboardShortcut};
            let esc_sub = keyboard::listen().filter_map(|event| {
                if let keyboard::Event::KeyPressed { key, modifiers, .. } = event {
                    if modifiers.control() || modifiers.command() {
                        return match key.as_ref() {
                            Key::Character("enter") => {
                                Some(Message::event_scr(EventScrEditorMessage::KeyboardShortcut(
                                    KeyboardShortcut::InsertActionBelow,
                                )))
                            }
                            Key::Character(" ") => {
                                Some(Message::event_scr(EventScrEditorMessage::KeyboardShortcut(
                                    KeyboardShortcut::TogglePicker,
                                )))
                            }
                            _ => None,
                        };
                    }
                    if let Key::Named(named) = key.as_ref() {
                        match named {
                            Named::ArrowUp => {
                                return Some(Message::event_scr(
                                    EventScrEditorMessage::KeyboardShortcut(
                                        KeyboardShortcut::MoveActionUp,
                                    ),
                                ))
                            }
                            Named::ArrowDown => {
                                return Some(Message::event_scr(
                                    EventScrEditorMessage::KeyboardShortcut(
                                        KeyboardShortcut::MoveActionDown,
                                    ),
                                ))
                            }
                            _ => {}
                        }
                    }
                }
                None
            });
            subscriptions.push(esc_sub);
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
        self.track_recent_file(path);
        self.save_workspace();

        match EditorType::from_path(path) {
            EditorType::DialogueScriptEditor => {
                let Some(tab_id) = self.active_tab_id() else {
                    return Task::none();
                };
                let path_buf = path.to_path_buf();
                self.state.dialogue_script_editors.insert(
                    tab_id,
                    crate::components::generic_editor::MultiFileEditorState {
                        current_file: Some(path_buf.clone()),
                        ..Default::default()
                    },
                );
                self.state
                    .dialogue_script_spreadsheets
                    .insert(tab_id, Default::default());
                Task::perform(
                    async move {
                        dispel_core::DialogueScript::read_file(&path_buf)
                            .map_err(|e: std::io::Error| e.to_string())
                    },
                    move |result| {
                        crate::message::Message::dialogue_script(
                            crate::editors::dialogue_script::DialogueScriptEditorMessage::CatalogLoaded(result),
                        )
                    },
                )
            }
            EditorType::DialogueTextEditor => {
                let Some(tab_id) = self.active_tab_id() else {
                    return Task::none();
                };
                let path_buf = path.to_path_buf();
                self.state.dialogue_paragraphs_editors.insert(
                    tab_id,
                    crate::components::generic_editor::MultiFileEditorState {
                        current_file: Some(path_buf.clone()),
                        ..Default::default()
                    },
                );
                self.state
                    .dialogue_paragraph_spreadsheets
                    .insert(tab_id, Default::default());
                Task::perform(
                    async move {
                        dispel_core::DialogueParagraph::read_file(&path_buf)
                            .map_err(|e: std::io::Error| e.to_string())
                    },
                    move |result| {
                        crate::message::Message::dialogue_paragraph(
                            crate::editors::dialogue_paragraph::DialogueParagraphEditorMessage::CatalogLoaded(tab_id, result),
                        )
                    },
                )
            }
            EditorType::NpcRefEditor => Task::done(Message::npc_ref(
                crate::editors::npc_ref::NpcRefEditorMessage::LoadCatalog(path.to_path_buf()),
            )),
            EditorType::MonsterRefEditor => Task::done(Message::monster_ref(
                crate::editors::monster_ref::MonsterRefEditorMessage::LoadCatalog(
                    path.to_path_buf(),
                ),
            )),
            EditorType::ExtraRefEditor => Task::done(Message::extra_ref(
                crate::editors::extra_ref::ExtraRefEditorMessage::LoadCatalog(path.to_path_buf()),
            )),
            EditorType::TilesetEditor => {
                if let Some(tab_id) = self.active_tab_id() {
                    self.state
                        .tileset_editors
                        .entry(tab_id)
                        .or_insert_with(|| TilesetEditorState::load(path));
                }
                Task::none()
            }
            EditorType::SpriteViewer => {
                if let Some(tab_id) = self.active_tab_id() {
                    self.state
                        .sprite_viewers
                        .entry(tab_id)
                        .or_insert_with(|| SpriteViewerState::load_from_path(path));
                }
                Task::none()
            }
            EditorType::SnfEditor => {
                if let Some(tab_id) = self.active_tab_id() {
                    self.state
                        .snf_editors
                        .entry(tab_id)
                        .or_insert_with(|| SnfEditorState::load_from_path(path));
                }
                Task::none()
            }
            EditorType::HexEditor => {
                if let Some(tab_id) = self.active_tab_id() {
                    self.state
                        .hex_editors
                        .entry(tab_id)
                        .or_insert_with(|| HexEditorState::load_from_path(path));
                }
                Task::none()
            }
            EditorType::MapEditor => {
                let Some(tab_id) = self.active_tab_id() else {
                    return Task::none();
                };
                Task::done(Message::map_editor(
                    crate::editors::map_editor::MapEditorMessage::Open(tab_id, path.to_path_buf()),
                ))
            }
            EditorType::EventScrEditor => {
                let path_buf = path.to_path_buf();
                self.state.event_scr_editor.file_path = Some(path_buf.clone());
                Task::done(Message::Editor(
                    crate::message::editor::EditorMessage::EventScr(
                        crate::editors::event_scr::message::EventScrEditorMessage::LoadScript(
                            path_buf,
                        ),
                    ),
                ))
            }
            et => load_catalog_task(et).unwrap_or(Task::none()),
        }
    }

    /// Open a file in the hex editor, bypassing the auto-detected editor type.
    pub fn open_file_in_workspace_as_hex(&mut self, path: &Path) -> Task<Message> {
        let label = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        self.state.workspace.open_with_editor_type(
            label,
            Some(path.to_path_buf()),
            EditorType::HexEditor,
        );
        self.track_recent_file(path);
        self.save_workspace();

        if let Some(tab_id) = self.active_tab_id() {
            self.state
                .hex_editors
                .entry(tab_id)
                .or_insert_with(|| HexEditorState::load_from_path(path));
        }
        Task::none()
    }

    fn active_tab_id(&self) -> Option<usize> {
        let idx = self.state.workspace.active_tab?;
        self.state.workspace.tabs.get(idx).map(|t| t.id)
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
        self.state.chest_editor.loading_state =
            crate::components::loading_state::LoadingState::Loading;
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
        self.state.viewer.loading_state = crate::components::loading_state::LoadingState::Loading;

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
        self.state.viewer.loading_state = crate::components::loading_state::LoadingState::Loading;
        let path = self.state.viewer.db_path.clone();
        let sql = self.state.viewer.sql_query.clone();
        let page = self.state.viewer.page;

        Task::perform(
            async move { db::execute_query(&path, &sql, PAGE_SIZE, page * PAGE_SIZE) },
            |result| Message::Viewer(ViewerMessage::DataLoaded(result)),
        )
    }
}

/// Return a `LoadCatalog` task for editors that load from the configured game
/// path.  Returns `None` for editors that are opened by explicit file path
/// (dialogue, tileset, map, ref files) or that have no load-on-start behaviour.
fn load_catalog_task(et: EditorType) -> Option<Task<Message>> {
    use crate::components::standard::message::StandardEditorMessage;
    use crate::message::MessageExt;

    macro_rules! load {
        ($wrap:expr) => {
            Some(Task::done($wrap(StandardEditorMessage::LoadCatalog)))
        };
    }

    match et {
        EditorType::WeaponEditor => load!(Message::weapon),
        EditorType::HealItemEditor => load!(Message::heal_item),
        EditorType::MiscItemEditor => load!(Message::misc_item),
        EditorType::EditItemEditor => load!(Message::edit_item),
        EditorType::EventItemEditor => load!(Message::event_item),
        EditorType::MonsterEditor => load!(Message::monster),
        EditorType::MonsterIniEditor => load!(Message::monster_ini),
        EditorType::NpcIniEditor => load!(Message::npc_ini),
        EditorType::MagicEditor => load!(Message::magic),
        EditorType::PartyRefEditor => load!(Message::party_ref),
        EditorType::PartyIniEditor => load!(Message::party_ini),
        EditorType::AllMapIniEditor => load!(Message::all_map_ini),
        EditorType::MapIniEditor => load!(Message::map_ini),
        EditorType::ExtraIniEditor => load!(Message::extra_ini),
        EditorType::EventIniEditor => load!(Message::event_ini),
        EditorType::WaveIniEditor => Some(Task::done(Message::wave_ini(
            crate::editors::wave_ini::WaveIniEditorMessage::LoadCatalog,
        ))),
        EditorType::DrawItemEditor => load!(Message::draw_item),
        EditorType::EventNpcRefEditor => load!(Message::event_npc_ref),
        EditorType::QuestScrEditor => load!(Message::quest_scr),
        EditorType::MessageScrEditor => load!(Message::message_scr),
        EditorType::ChDataEditor => load!(Message::chdata),
        EditorType::StoreEditor => Some(Task::done(Message::store(
            crate::editors::store::StoreEditorMessage::LoadCatalog,
        ))),
        EditorType::PartyLevelDbEditor => Some(Task::done(Message::party_level_db(
            crate::editors::party_level_db::PartyLevelDbEditorMessage::LoadCatalog,
        ))),
        _ => None,
    }
}

/// Map `(EditorType, SpreadsheetMessage)` to the correct `Message` variant.
/// Returns `None` for editor types that have no spreadsheet (map editor, sprite
/// viewer, etc.) so callers can use this as a capability check.
fn build_spreadsheet_nav_msg(
    et: crate::workspace::EditorType,
    sm: crate::view::editor::SpreadsheetMessage,
) -> Option<crate::message::Message> {
    use crate::editors::*;
    use crate::message::Message;
    use crate::message::MessageExt as _;
    use crate::workspace::EditorType::*;
    Some(match et {
        WeaponEditor => Message::weapon(weapon::WeaponEditorMessage::Spreadsheet(sm)),
        MonsterEditor => Message::monster(monster::MonsterEditorMessage::Spreadsheet(sm)),
        MonsterIniEditor => {
            Message::monster_ini(monster_ini::MonsterIniEditorMessage::Spreadsheet(sm))
        }
        HealItemEditor => Message::heal_item(heal_item::HealItemEditorMessage::Spreadsheet(sm)),
        MiscItemEditor => Message::misc_item(misc_item::MiscItemEditorMessage::Spreadsheet(sm)),
        EditItemEditor => Message::edit_item(edit_item::EditItemEditorMessage::Spreadsheet(sm)),
        EventItemEditor => Message::event_item(event_item::EventItemEditorMessage::Spreadsheet(sm)),
        MagicEditor => Message::magic(magic::MagicEditorMessage::Spreadsheet(sm)),
        StoreEditor => return None, // Store editor has a custom layout, no generic spreadsheet
        NpcIniEditor => Message::npc_ini(npc_ini::NpcIniEditorMessage::Spreadsheet(sm)),
        NpcRefEditor => Message::npc_ref(npc_ref::NpcRefEditorMessage::Spreadsheet(sm)),
        MonsterRefEditor => {
            Message::monster_ref(monster_ref::MonsterRefEditorMessage::Spreadsheet(sm))
        }
        PartyRefEditor => Message::party_ref(party_ref::PartyRefEditorMessage::Spreadsheet(sm)),
        PartyIniEditor => Message::party_ini(party_ini::PartyIniEditorMessage::Spreadsheet(sm)),
        AllMapIniEditor => {
            Message::all_map_ini(all_map_ini::AllMapIniEditorMessage::Spreadsheet(sm))
        }
        MapIniEditor => Message::map_ini(map_ini::MapIniEditorMessage::Spreadsheet(sm)),
        ExtraIniEditor => Message::extra_ini(extra_ini::ExtraIniEditorMessage::Spreadsheet(sm)),
        ExtraRefEditor => Message::extra_ref(extra_ref::ExtraRefEditorMessage::Spreadsheet(sm)),
        EventIniEditor => Message::event_ini(event_ini::EventIniEditorMessage::Spreadsheet(sm)),
        EventNpcRefEditor => {
            Message::event_npc_ref(event_npc_ref::EventNpcRefEditorMessage::Spreadsheet(sm))
        }
        WaveIniEditor => Message::wave_ini(wave_ini::WaveIniEditorMessage::Spreadsheet(sm)),
        DrawItemEditor => Message::draw_item(draw_item::DrawItemEditorMessage::Spreadsheet(sm)),
        MessageScrEditor => {
            Message::message_scr(message_scr::MessageScrEditorMessage::Spreadsheet(sm))
        }
        QuestScrEditor => Message::quest_scr(quest_scr::QuestScrEditorMessage::Spreadsheet(sm)),
        DialogueScriptEditor => Message::dialogue_script(
            dialogue_script::DialogueScriptEditorMessage::Spreadsheet(sm),
        ),
        DialogueTextEditor => Message::dialogue_paragraph(
            dialogue_paragraph::DialogueParagraphEditorMessage::Spreadsheet(sm),
        ),
        ChDataEditor => Message::chdata(chdata::ChDataEditorMessage::Spreadsheet(sm)),
        PartyLevelDbEditor => {
            Message::party_level_db(party_level_db::PartyLevelDbEditorMessage::Spreadsheet(sm))
        }
        _ => return None,
    })
}

#[cfg(all(test, feature = "iced_test"))]
mod tests {
    use super::*;
    use crate::workspace::{EditorType, Workspace, WorkspaceTab};

    #[test]
    fn test_empty_state_displayed_when_no_tabs_open() {
        let workspace = Workspace::default();
        let app = App::test_new(workspace);

        assert_eq!(app.app_mode, AppMode::EditorMode);
        assert!(app.state.workspace.active().is_none());

        let view = app.view();

        let mut ui = iced_test::simulator(view);

        ui.find("Select a file to edit")
            .expect("Empty state should display 'Select a file to edit'");
    }

    #[test]
    fn test_empty_state_not_shown_when_tab_is_open() {
        let mut workspace = Workspace::default();
        workspace.tabs.push(WorkspaceTab {
            id: 1,
            label: "Test".into(),
            path: None,
            editor_type: EditorType::LocalizationManager,
            modified: false,
            pinned: false,
        });
        workspace.active_tab = Some(0);

        let app = App::test_new(workspace);

        let view = app.view();

        let mut ui = iced_test::simulator(view);

        ui.find("Select a file to edit")
            .expect_err("Empty state should NOT be shown when a tab is open");
    }
}
