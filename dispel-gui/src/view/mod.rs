use crate::app::{App, AppMode};
use crate::components::modal::modal;
use crate::components::tab_bar::view as tab_bar;
use crate::message::{
    startpage::StartPageMessage, FileTreeMessage, Message, MessageExt, SystemMessage,
    WorkspaceMessage,
};
use crate::state::state::PaneContent;
use crate::style;
use crate::utils::{truncate_path, vertical_space};
use crate::view::history_panel::view_history_panel;
use crate::workspace::EditorType;
use iced::widget::pane_grid;
use iced::widget::{button, column, container, progress_bar, row, stack, text};
use iced::{Element, Fill, Font, Length};

pub mod all_map_ini_editor;
pub mod chdata_editor;
pub mod chest_editor;
pub mod db_viewer;
pub mod dialogue_script_editor;
pub mod dialogue_text_editor;
pub mod draw_item_editor;
pub mod edit_item_editor;
pub mod editor;
pub mod event_ini_editor;
pub mod event_item_editor;
pub mod event_npc_ref_editor;
pub mod extra_ini_editor;
pub mod extra_ref_editor;
pub mod heal_item_editor;
pub mod history_panel;
pub mod localization_manager;
pub mod magic_editor;
pub mod map_editor;
pub mod map_ini_editor;
pub mod message_scr_editor;
pub mod misc_item_editor;
pub mod mod_packager;
pub mod monster_editor;
pub mod monster_ini_editor;
pub mod monster_ref_editor;
pub mod npc_ini_editor;
pub mod npc_ref_editor;
pub mod party_ini_editor;
pub mod party_level_db_editor;
pub mod party_ref_editor;
pub mod quest_scr_editor;
pub mod snf_editor;
pub mod sprite_browser;
pub mod start_page;
pub mod store_editor;
pub mod tileset_editor;
pub mod wave_ini_editor;
pub mod weapon_editor;

impl App {
    /// Main view entry point called by the Iced framework.
    ///
    /// This function creates the pane grid layout that divides the UI into resizable panels:
    /// - Sidebar: Contains file tree and navigation
    /// - MainContent: Contains editor area and tab bar
    ///
    /// The pane grid system allows users to resize panels and provides a flexible workspace layout
    /// similar to modern code editors like VS Code and Sublime Text.
    pub fn view(&self) -> Element<'_, Message> {
        if self.app_mode == AppMode::StartPage {
            return self.view_start_page();
        }
        self.view_editor()
    }

    fn view_editor(&self) -> Element<'_, Message> {
        let pane_grid =
            pane_grid::PaneGrid::new(&self.state.pane_state.state, |_id, pane, _maximized| {
                let pane_content = match pane {
                    PaneContent::Sidebar => self.view_sidebar(),
                    PaneContent::MainContent => {
                        let content = match self.state.workspace.active().map(|t| t.editor_type) {
                            Some(EditorType::DbViewer) => self.view_db_viewer(),
                            Some(EditorType::ChestEditor) => self.view_chest_editor_tab(),
                            Some(EditorType::WeaponEditor) => self.view_weapon_editor_tab(),
                            Some(EditorType::SpriteViewer) => self.view_sprite_viewer_tab(),
                            Some(EditorType::HealItemEditor) => self.view_heal_item_editor_tab(),
                            Some(EditorType::MiscItemEditor) => self.view_misc_item_editor_tab(),
                            Some(EditorType::EditItemEditor) => self.view_edit_item_editor_tab(),
                            Some(EditorType::EventItemEditor) => self.view_event_item_editor_tab(),
                            Some(EditorType::MonsterEditor) => self.view_monster_editor_tab(),
                            Some(EditorType::MonsterIniEditor) => {
                                self.view_monster_ini_editor_tab()
                            }
                            Some(EditorType::NpcIniEditor) => self.view_npc_ini_editor_tab(),
                            Some(EditorType::MagicEditor) => self.view_magic_editor_tab(),
                            Some(EditorType::StoreEditor) => self.view_store_editor_tab(),
                            Some(EditorType::PartyRefEditor) => self.view_party_ref_tab(),
                            Some(EditorType::PartyIniEditor) => self.view_party_ini_tab(),
                            Some(EditorType::MonsterRefEditor) => {
                                self.view_monster_ref_editor_tab()
                            }
                            Some(EditorType::AllMapIniEditor) => self.view_all_map_ini_editor_tab(),
                            Some(EditorType::DialogueScriptEditor) => {
                                self.view_dialogue_script_editor_tab()
                            }
                            Some(EditorType::DialogueTextEditor) => {
                                self.view_dialogue_paragraph_editor_tab()
                            }
                            Some(EditorType::DrawItemEditor) => self.view_draw_item_tab(),
                            Some(EditorType::EventIniEditor) => self.view_event_ini_tab(),
                            Some(EditorType::EventNpcRefEditor) => self.view_event_npc_ref_tab(),
                            Some(EditorType::ExtraIniEditor) => self.view_extra_ini_tab(),
                            Some(EditorType::ExtraRefEditor) => self.view_extra_ref_editor_tab(),
                            Some(EditorType::MapIniEditor) => self.view_map_ini_tab(),
                            Some(EditorType::MessageScrEditor) => self.view_message_scr_tab(),
                            Some(EditorType::NpcRefEditor) => self.view_npc_ref_tab(),
                            Some(EditorType::PartyLevelDbEditor) => self.view_party_level_db_tab(),
                            Some(EditorType::QuestScrEditor) => self.view_quest_scr_tab(),
                            Some(EditorType::WaveIniEditor) => self.view_wave_ini_tab(),
                            Some(EditorType::ChDataEditor) => self.view_chdata_tab(),
                            Some(EditorType::TilesetEditor) => self.view_tileset_editor_tab(),
                            Some(EditorType::MapEditor) => self.view_map_editor_tab(),
                            Some(EditorType::SnfEditor) => self.view_snf_editor_tab(),
                            Some(EditorType::ModPackager) => self.view_mod_packager_tab(),
                            Some(EditorType::LocalizationManager) => {
                                self.view_localization_manager_tab()
                            }
                            Some(EditorType::Unknown) | None => {
                                let placeholder_text = text("Select a file to edit")
                                    .size(16)
                                    .align_x(iced::Alignment::Center)
                                    .align_y(iced::Alignment::Center)
                                    .height(Length::Fill)
                                    .width(Length::Fill)
                                    .style(style::subtle_text);

                                column![placeholder_text]
                                    .padding(8)
                                    .height(Length::Fill)
                                    .width(Length::Fill)
                                    .into()
                            }
                        };
                        let tab_bar =
                            tab_bar::view_tab_bar(&self.state.workspace).map(Message::tab_bar);
                        column![self.view_shared_game_path_toolbar(), tab_bar, content]
                            .spacing(0)
                            .height(Fill)
                            .into()
                    }
                    PaneContent::HistoryPanel => {
                        if self.history_panel_visible {
                            view_history_panel(self.get_active_edit_history())
                        } else {
                            container(
                                text("History panel hidden")
                                    .size(13)
                                    .style(style::subtle_text),
                            )
                            .width(Fill)
                            .height(Fill)
                            .into()
                        }
                    }
                };
                pane_grid::Content::new(pane_content)
            })
            .on_click(|pane| Message::Workspace(WorkspaceMessage::PaneClicked(pane)))
            .on_drag(|event| Message::Workspace(WorkspaceMessage::PaneDragged(event)))
            .on_resize(10, |event| {
                Message::Workspace(WorkspaceMessage::PaneResized(event))
            });

        let main_container = container(pane_grid)
            .width(Fill)
            .height(Fill)
            .style(style::root_container);

        if let Some(ref palette) = self.command_palette {
            let palette_view = palette.view();

            let backdrop = container(main_container).width(Fill).height(Fill);

            let overlay = container(palette_view)
                .width(Fill)
                .height(Fill)
                .center_x(Fill)
                .center_y(Fill)
                .style(|_theme| iced::widget::container::Style {
                    background: Some(iced::Background::Color(iced::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 0.65,
                    })),
                    ..Default::default()
                });

            return stack![backdrop, overlay].width(Fill).height(Fill).into();
        }

        if self.global_search.is_visible {
            let search_view = self.global_search.view();

            let backdrop = container(column![].width(Fill).height(Fill))
                .width(Fill)
                .height(Fill)
                .style(|_theme| iced::widget::container::Style {
                    background: Some(iced::Background::Color(iced::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 0.5,
                    })),
                    ..Default::default()
                });

            let overlay = container(search_view).center_x(Fill).center_y(Fill);

            return stack![
                container(main_container).width(Fill).height(Fill),
                backdrop,
                overlay
            ]
            .width(Fill)
            .height(Fill)
            .into();
        }

        if let Some(ref err_msg) = self.error_dialog {
            let dialog = container(
                column![
                    text("Error")
                        .size(14)
                        .color(iced::Color::from_rgb(0.8, 0.2, 0.2)),
                    text(err_msg.as_str()).size(12),
                    button(text("Dismiss").size(11))
                        .on_press(Message::System(SystemMessage::DismissError))
                        .padding([4, 12])
                        .style(style::browse_button),
                ]
                .spacing(12)
                .padding(20),
            )
            .style(style::modal_container)
            .max_width(480);

            return modal(
                main_container,
                dialog,
                || Message::System(SystemMessage::DismissError),
                0.5,
            );
        }

        main_container.into()
    }

    fn view_shared_game_path_toolbar(&self) -> Element<'_, Message> {
        let path_display = if self.state.shared_game_path.is_empty() {
            "No game path set"
        } else {
            &self.state.shared_game_path
        };

        let path_text = container(
            text(truncate_path(path_display, 80))
                .size(12)
                .font(Font::MONOSPACE),
        )
        .padding([4, 12])
        .width(Fill)
        .style(style::sql_editor_container);

        container(
            row![
                text("Game Path:")
                    .size(12)
                    .width(80)
                    .style(style::subtle_text),
                path_text,
                button(text("Change Path").size(12))
                    .on_press(Message::StartPage(StartPageMessage::BackToStart))
                    .padding([4, 12])
                    .style(style::browse_button),
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center)
            .padding([8, 16]),
        )
        .width(Fill)
        .style(style::toolbar_container)
        .into()
    }

    /// Renders the sidebar pane content.
    ///
    /// The sidebar contains:
    /// - Application title and branding
    /// - File tree for game file navigation (Sublime-inspired)
    /// - Tools section with quick access to utility editors
    /// - Recent files list for quick access
    ///
    /// This is the primary navigation hub following the Sublime Text pattern of
    /// having a persistent file browser alongside the editing area.
    fn view_sidebar(&self) -> Element<'_, Message> {
        if !self.sidebar_visible {
            return container(vertical_space().height(Fill))
                .width(Fill)
                .height(Fill)
                .into();
        }

        let title = text("Dispel Extractor").size(18).font(Font::MONOSPACE);

        // File tree component - core of the Sublime-inspired navigation
        // Maps FileTreeMessage to WorkspaceMessage for proper routing
        let file_tree_view = self.file_tree.view().map(Message::file_tree);

        // Tools section — always-accessible tool views not tied to a file
        let tool_btn = |label: &'static str, editor_type: EditorType| {
            button(text(label).size(12))
                .width(Fill)
                .padding([5, 16])
                .on_press(Message::Workspace(WorkspaceMessage::OpenToolTab(
                    editor_type,
                )))
                .style(style::tab_button)
        };
        let tools_section = column![
            container(text("Tools").size(11).style(style::subtle_text)).padding([4, 16]),
            tool_btn("DB Viewer", EditorType::DbViewer),
            tool_btn("Chest Editor", EditorType::ChestEditor),
            tool_btn("Store Editor", EditorType::StoreEditor),
            tool_btn("Mod Packager", EditorType::ModPackager),
            tool_btn("Localization Packager", EditorType::LocalizationManager),
        ]
        .spacing(1);

        let _recent_section = column![
            container(text("Recent").size(11).style(style::subtle_text)).padding([4, 16]),
            self.view_recent_files(),
        ]
        .spacing(1);

        let tree_is_empty = self.file_tree.data.root.is_none();
        let file_tree_area: Element<'_, Message> = if self.is_indexing && tree_is_empty {
            container(
                column![
                    text("Indexing files…").size(11).style(style::subtle_text),
                    progress_bar(0.0..=1.0, 0.5).style(style::primary_progress_bar),
                ]
                .spacing(8)
                .align_x(iced::Alignment::Center),
            )
            .width(Fill)
            .height(Fill)
            .center_x(Fill)
            .center_y(Fill)
            .padding([0, 16])
            .into()
        } else {
            file_tree_view
        };

        let sidebar_content = column![
            vertical_space().height(12),
            container(title).padding([0, 16]),
            vertical_space().height(16),
            file_tree_area,
            // recent_section,
            vertical_space().height(8),
            tools_section,
            vertical_space().height(8),
        ]
        .spacing(0)
        .width(Fill);
        container(sidebar_content)
            .height(Fill)
            .style(style::sidebar_container)
            .into()
    }

    fn view_recent_files(&self) -> Element<'_, Message> {
        if self.state.recent_files.is_empty() {
            return container(text("No recent files").size(11).style(style::subtle_text))
                .padding([2, 16])
                .into();
        }

        let items = self.state.recent_files.iter().take(10).enumerate().fold(
            column![].spacing(0),
            |col, (idx, file_path)| {
                let file_name = file_path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "Unknown".to_string());

                col.push(
                    button(
                        row![
                            text(format!("{}. ", idx + 1))
                                .size(10)
                                .style(style::subtle_text),
                            text(file_name).size(11),
                        ]
                        .spacing(4)
                        .align_y(iced::Alignment::Center),
                    )
                    .on_press(Message::file_tree(FileTreeMessage::OpenFile(
                        file_path.clone(),
                    )))
                    .width(Fill)
                    .style(style::tab_button)
                    .padding([4, 12]),
                )
            },
        );

        items.into()
    }
}
