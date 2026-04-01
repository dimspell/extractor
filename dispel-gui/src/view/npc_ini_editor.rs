use crate::app::App;
use crate::message::Message;
use crate::style;
use crate::utils::labeled_file_row;
use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Element, Fill, Length};

impl App {
    pub fn view_npc_ini_editor_tab(&self) -> Element<'_, Message> {
        let editor = &self.npc_ini_editor;
        let header = row![
            text("NPC Visual Editor").size(20),
            text(format!(
                " - {} NPCs",
                editor.catalog.as_ref().map_or(0, |c| c.len())
            ))
            .size(14)
            .style(style::subtle_text),
        ];
        let controls = row![
            labeled_file_row(
                "Game Path:",
                &editor.game_path,
                |_| Message::FileSelected {
                    field: "npc_ini_game_path".into(),
                    path: None
                },
                Message::NpcIniOpBrowseGamePath,
            ),
            button(text("Load Catalog").size(13))
                .padding([8, 16])
                .on_press(Message::NpcIniOpLoadCatalog)
                .style(style::chip),
        ]
        .spacing(12)
        .align_y(iced::Alignment::Center);
        let status = text(&editor.status_msg).size(12).style(style::subtle_text);

        let item_list: Vec<Element<'_, Message>> = editor
            .filtered_npcs
            .iter()
            .map(|(i, npc)| {
                let is_selected = editor.selected_idx == Some(*i);
                let sprite = npc.sprite_filename.as_deref().unwrap_or("none");
                let btn = button(
                    row![
                        text(format!("{:03}", npc.id)).size(12).width(40),
                        text(&npc.description).size(12),
                        text(sprite).size(11).style(style::subtle_text),
                    ]
                    .spacing(8)
                    .align_y(iced::Alignment::Center),
                )
                .width(Fill)
                .padding([6, 12])
                .on_press(Message::NpcIniOpSelectNpc(*i));
                if is_selected {
                    btn.style(style::active_tab_button).into()
                } else {
                    btn.style(style::tab_button).into()
                }
            })
            .collect();

        let editor_panel: Element<'_, Message> = if let Some(idx) = editor.selected_idx {
            column![
                text("Edit NPC").size(16),
                row![
                    text("Sprite:").size(13).width(80),
                    text_input("", &editor.edit_sprite_filename)
                        .on_input(move |v| Message::NpcIniOpFieldChanged(
                            idx,
                            "sprite_filename".into(),
                            v
                        ))
                        .padding(6)
                        .size(13)
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
                row![
                    text("Description:").size(13).width(80),
                    text_input("", &editor.edit_description)
                        .on_input(move |v| Message::NpcIniOpFieldChanged(
                            idx,
                            "description".into(),
                            v
                        ))
                        .padding(6)
                        .size(13)
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
            ]
            .spacing(12)
            .into()
        } else {
            text("Select an NPC to edit")
                .size(13)
                .style(style::subtle_text)
                .into()
        };

        let save_btn = button(text("Save to File").size(14))
            .padding([10, 24])
            .on_press(Message::NpcIniOpSave)
            .style(style::run_button);

        let detail_panel = container(scrollable(editor_panel).height(Fill))
            .padding(16)
            .width(350)
            .style(style::info_card);

        let main_content = row![
            column![
                container(
                    row![
                        text("NPCs").size(14),
                        text(format!("{} loaded", editor.filtered_npcs.len())).size(12),
                    ]
                    .padding(10)
                    .align_y(iced::Alignment::Center)
                )
                .style(style::grid_header_cell),
                scrollable(column(item_list).spacing(2)).height(Fill),
            ]
            .width(Length::Fill),
            detail_panel,
        ]
        .height(Fill);

        column![header, controls, status, main_content, save_btn]
            .spacing(12)
            .padding(16)
            .into()
    }
}
