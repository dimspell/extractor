use crate::app::{App, AppMode};
use crate::components::modal::modal;
use crate::components::tab_bar::view as tab_bar;
use crate::components::utils::{truncate_path, vertical_space};
use crate::message::{
    startpage::StartPageMessage, FileTreeMessage, Message, MessageExt, SystemMessage,
    WorkspaceMessage,
};
use crate::state::PaneContent;
use crate::style;
use crate::view::history_panel::view_history_panel;
use crate::workspace::EditorType;
use iced::widget::pane_grid;
use iced::widget::{button, column, container, progress_bar, row, stack, text};
use iced::{Element, Fill, Font, Length};

pub mod editor;
pub mod history_panel;
pub mod start_page;

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
                            Some(EditorType::ChestEditor) => crate::editors::chest::view(self),
                            Some(EditorType::WeaponEditor) => crate::editors::weapon::view(self),
                            Some(EditorType::SpriteViewer) => {
                                crate::editors::sprite_browser::view(self)
                            }
                            Some(EditorType::HealItemEditor) => {
                                crate::editors::heal_item::view(self)
                            }
                            Some(EditorType::MiscItemEditor) => {
                                crate::editors::misc_item::view(self)
                            }
                            Some(EditorType::EditItemEditor) => {
                                crate::editors::edit_item::view(self)
                            }
                            Some(EditorType::EventItemEditor) => {
                                crate::editors::event_item::view(self)
                            }
                            Some(EditorType::MonsterEditor) => crate::editors::monster::view(self),
                            Some(EditorType::MonsterIniEditor) => {
                                crate::editors::monster_ini::view(self)
                            }
                            Some(EditorType::NpcIniEditor) => crate::editors::npc_ini::view(self),
                            Some(EditorType::MagicEditor) => crate::editors::magic::view(self),
                            Some(EditorType::StoreEditor) => crate::editors::store::view(self),
                            Some(EditorType::PartyRefEditor) => {
                                crate::editors::party_ref::view(self)
                            }
                            Some(EditorType::PartyIniEditor) => {
                                crate::editors::party_ini::view(self)
                            }
                            Some(EditorType::MonsterRefEditor) => {
                                crate::editors::monster_ref::view(self)
                            }
                            Some(EditorType::AllMapIniEditor) => {
                                crate::editors::all_map_ini::view(self)
                            }
                            Some(EditorType::DialogueScriptEditor) => {
                                crate::editors::dialogue_script::view(self)
                            }
                            Some(EditorType::DialogueTextEditor) => {
                                crate::editors::dialogue_paragraph::view(self)
                            }
                            Some(EditorType::DrawItemEditor) => {
                                crate::editors::draw_item::view(self)
                            }
                            Some(EditorType::EventIniEditor) => {
                                crate::editors::event_ini::view(self)
                            }
                            Some(EditorType::EventNpcRefEditor) => {
                                crate::editors::event_npc_ref::view(self)
                            }
                            Some(EditorType::ExtraIniEditor) => {
                                crate::editors::extra_ini::view(self)
                            }
                            Some(EditorType::ExtraRefEditor) => {
                                crate::editors::extra_ref::view(self)
                            }
                            Some(EditorType::MapIniEditor) => crate::editors::map_ini::view(self),
                            Some(EditorType::MessageScrEditor) => {
                                crate::editors::message_scr::view(self)
                            }
                            Some(EditorType::NpcRefEditor) => crate::editors::npc_ref::view(self),
                            Some(EditorType::PartyLevelDbEditor) => {
                                crate::editors::party_level_db::view(self)
                            }
                            Some(EditorType::QuestScrEditor) => {
                                crate::editors::quest_scr::view(self)
                            }
                            Some(EditorType::WaveIniEditor) => crate::editors::wave_ini::view(self),
                            Some(EditorType::ChDataEditor) => crate::editors::chdata::view(self),
                            Some(EditorType::TilesetEditor) => crate::editors::tileset::view(self),
                            Some(EditorType::MapEditor) => crate::editors::map_editor::view(self),
                            Some(EditorType::SnfEditor) => crate::editors::snf_editor::view(self),
                            Some(EditorType::ModPackager) => {
                                crate::editors::mod_packager::view(self)
                            }
                            Some(EditorType::LocalizationManager) => {
                                crate::editors::localization_manager::view(self)
                            }
                            Some(EditorType::Unknown) | None => {
                                let content: Element<'_, Message> =
                                    if self.state.recent_files.is_empty() {
                                        column![text("Select a file to edit")
                                            .size(16)
                                            .style(style::subtle_text),]
                                        .align_x(iced::Alignment::Center)
                                        .into()
                                    } else {
                                        column![
                                            text("Select a file to edit")
                                                .size(16)
                                                .style(style::subtle_text),
                                            vertical_space().height(20),
                                            container(
                                                column![
                                                    text("Recent Files")
                                                        .size(14)
                                                        .style(style::subtle_text),
                                                    vertical_space().height(10),
                                                    self.view_recent_files(),
                                                ]
                                                .spacing(4)
                                            )
                                            .max_width(400),
                                        ]
                                        .align_x(iced::Alignment::Center)
                                        .into()
                                    };

                                container(content)
                                    .center_x(Fill)
                                    .center_y(Fill)
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
            return container(vertical_space())
                .width(Length::Fixed(0.0))
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
            tool_btn("Mod Manager", EditorType::ModPackager),
            tool_btn("Localization Packager", EditorType::LocalizationManager),
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
