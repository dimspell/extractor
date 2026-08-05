use crate::components::command_palette::CommandPalette;
use crate::components::edit_history::EditHistory;
use crate::components::file_tree::FileTree;
use crate::editors::snf_editor::SnfEditorState;
use crate::editors::sprite_editor::SpriteViewerState;
use crate::editors::tileset::TilesetEditorState;
use crate::message::Message;
use crate::message::MessageExt;
use crate::message::SystemMessage;
use crate::state::AppState;
use crate::workspace::EditorType;
use dispel_core::Extractor;
use hexedit::HexEditorState;
use iced::Task;
use std::path::Path;

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
                    if idx.game_path.as_deref() == Some(gp.to_string_lossy().as_ref()) {
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
                search_index,
                app_mode,
                start_page_input,
                is_indexing,
                error_dialog: None,
            },
            final_init_task,
        )
    }

    pub(crate) fn set_title(&self) -> String {
        let default_title: String = "Dispel Extractor".to_owned();

        match self.state.shared_game_path.is_empty() {
            true => default_title,
            false => default_title + " - " + self.state.shared_game_path.as_str(),
        }
    }

    #[cfg(test)]
    pub fn test_new(workspace: crate::workspace::Workspace) -> Self {
        use crate::state::AppState;

        let state = AppState {
            workspace,
            ..Default::default()
        };

        Self {
            state,
            file_tree: crate::components::file_tree::FileTree::default(),
            window_id: iced::window::Id::unique(),
            history_panel_visible: false,
            sidebar_visible: true,
            empty_edit_history: EditHistory::default(),
            command_palette: None,
            global_search: crate::components::global_search::GlobalSearch::new(),
            search_index: crate::indexation::search_index::SearchIndex::new(),
            app_mode: AppMode::EditorMode,
            start_page_input: String::new(),
            is_indexing: false,
            error_dialog: None,
        }
    }

    pub fn get_active_edit_history(&self) -> &EditHistory {
        if let Some(tab) = self.state.workspace.active() {
            self.state
                .editors
                .get_active_edit_history(tab.editor_type, tab.id)
                .unwrap_or(&self.empty_edit_history)
        } else {
            &self.empty_edit_history
        }
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        crate::subscriptions::subscription(self)
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
                self.state.editors.dialogue_script_editor.editors.insert(
                    tab_id,
                    crate::components::generic_editor::MultiFileEditorState {
                        current_file: Some(path_buf.clone()),
                        ..Default::default()
                    },
                );
                self.state
                    .editors
                    .dialogue_script_editor
                    .spreadsheets
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
                self.state.editors.dialogue_paragraph_editor.editors.insert(
                    tab_id,
                    crate::components::generic_editor::MultiFileEditorState {
                        current_file: Some(path_buf.clone()),
                        ..Default::default()
                    },
                );
                self.state
                    .editors
                    .dialogue_paragraph_editor
                    .spreadsheets
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
                        .editors
                        .tileset_editors
                        .entry(tab_id)
                        .or_insert_with(|| TilesetEditorState::load(path));
                }
                Task::none()
            }
            EditorType::SpriteViewer => {
                if let Some(tab_id) = self.active_tab_id() {
                    self.state
                        .editors
                        .sprite_viewers
                        .entry(tab_id)
                        .or_insert_with(|| SpriteViewerState::load_from_path(path));
                }
                Task::none()
            }
            EditorType::SnfEditor => {
                if let Some(tab_id) = self.active_tab_id() {
                    self.state
                        .editors
                        .snf_editors
                        .entry(tab_id)
                        .or_insert_with(|| SnfEditorState::load_from_path(path));
                }
                Task::none()
            }
            EditorType::HexEditor => {
                let scripts_dir = self
                    .state
                    .workspace
                    .game_path
                    .as_ref()
                    .map(|gp| gp.join("hexedit_scripts"));
                if let Some(tab_id) = self.active_tab_id() {
                    let state = self
                        .state
                        .editors
                        .hex_editors
                        .entry(tab_id)
                        .or_insert_with(|| HexEditorState::load_from_path(path));
                    if let Some(ref dir) = scripts_dir {
                        let errors = state.load_lua_scripts(dir);
                        for e in &errors {
                            log::warn!("hexedit script: {e}");
                        }
                    }
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
                self.state.editors.event_scr_editor.file_path = Some(path_buf.clone());
                Task::done(Message::Editor(
                    crate::message::editor::EditorMessage::EventScr(
                        crate::editors::event_scr::message::EventScrEditorMessage::LoadScript(
                            path_buf,
                        ),
                    ),
                ))
            }
            EditorType::SaveFileViewer => {
                let Some(tab_id) = self.active_tab_id() else {
                    return Task::none();
                };
                use crate::editors::save_file_viewer::SaveFileViewerState;
                self.state
                    .editors
                    .save_file_viewers
                    .entry(tab_id)
                    .or_insert_with(|| SaveFileViewerState {
                        loading: true,
                        ..Default::default()
                    });
                let path_buf = path.to_path_buf();
                let game_path = self.state.workspace.game_path.clone();
                Task::perform(
                    async move {
                        let data = std::fs::read(&path_buf).map_err(|e| e.to_string())?;
                        let save_file = dispel_core::references::save_file::SaveFile::parse(&data)
                            .map_err(|e| e.to_string())?;

                        // Load AllMap.ini to get display names for map IDs
                        let map_names: std::collections::HashMap<u32, String> = game_path
                            .as_ref()
                            .and_then(|gp| {
                                let all_map_path = gp.join("AllMap.ini");
                                if all_map_path.exists() {
                                    match dispel_core::references::all_map_ini::read_all_map_ini(
                                        &all_map_path,
                                    ) {
                                        Ok(maps) => Some(
                                            maps.into_iter()
                                                .map(|m| (m.id as u32, m.map_name))
                                                .collect(),
                                        ),
                                        Err(e) => {
                                            eprintln!("Failed to read AllMap.ini: {e}");
                                            None
                                        }
                                    }
                                } else {
                                    None
                                }
                            })
                            .unwrap_or_default();

                        let hex_editors =
                            crate::editors::save_file_viewer::state::get_hex_editors(&save_file);

                        Ok(crate::editors::save_file_viewer::message::SaveFileLoaded {
                            save_file,
                            hex_editors,
                            map_names,
                        })
                    },
                    move |result| {
                        crate::message::Message::save_file_viewer(
                            crate::editors::save_file_viewer::SaveFileViewerMessage::Loaded(result),
                        )
                    },
                )
            }
            et => crate::dispatch_table::load_catalog_task(et).unwrap_or(Task::none()),
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

        let scripts_dir = self
            .state
            .workspace
            .game_path
            .as_ref()
            .map(|gp| gp.join("hexedit_scripts"));
        if let Some(tab_id) = self.active_tab_id() {
            let state = self
                .state
                .editors
                .hex_editors
                .entry(tab_id)
                .or_insert_with(|| HexEditorState::load_from_path(path));
            if let Some(ref dir) = scripts_dir {
                let errors = state.load_lua_scripts(dir);
                for e in &errors {
                    log::warn!("hexedit script: {e}");
                }
            }
        }
        Task::none()
    }

    fn active_tab_id(&self) -> Option<usize> {
        let idx = self.state.workspace.active_tab?;
        self.state.workspace.tabs.get(idx).map(|t| t.id)
    }
}
