use crate::app::App;
use crate::message::Message;
use crate::style;
use crate::utils::{
    horizontal_rule, horizontal_space, labeled_input, labeled_select, vertical_space,
};
use dispel_core::NpcLookingDirection;
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Element, Fill, Font, Length};

impl App {
    pub fn view_npc_ref_editor_tab(&self) -> Element<'_, Message> {
        let editor = &self.npc_ref_editor;

        let file_bar = row![
            text("Map File:").size(13).width(100),
            text(&editor.current_map_file)
                .size(12)
                .font(Font::MONOSPACE)
                .width(Fill),
            button(text("Browse").size(12))
                .on_press(Message::NpcRefOpBrowseMapFile)
                .padding([5, 10])
                .style(style::browse_button),
            button(text("Load").size(12))
                .on_press(Message::NpcRefOpLoadCatalog)
                .padding([5, 10])
                .style(style::run_button),
        ]
        .spacing(8)
        .padding(10)
        .align_y(iced::Alignment::Center);

        let npc_list: Vec<Element<Message>> = editor
            .filtered_npcs
            .iter()
            .enumerate()
            .map(|(idx, (_, npc))| {
                let is_selected = editor.selected_idx == Some(idx);
                let label = format!("[{}] {} (NPC:{})", npc.id, npc.name, npc.npc_id);

                let btn = button(text(label).size(11).font(Font::MONOSPACE))
                    .width(Fill)
                    .on_press(Message::NpcRefOpSelectNpc(idx));

                if is_selected {
                    btn.style(style::active_chip).into()
                } else {
                    btn.style(style::chip).into()
                }
            })
            .collect();

        let npc_scroll = scrollable(column(npc_list).spacing(4)).height(Length::Fill);

        let mut detail_content: Vec<Element<Message>> = vec![
            text("NPC Details").size(16).font(Font::MONOSPACE).into(),
            vertical_space().height(10).into(),
        ];

        if let Some(idx) = editor.selected_idx {
            if let Some((orig_idx, _npc)) = editor.filtered_npcs.get(idx) {
                let orig = *orig_idx;

                detail_content.push(labeled_input("ID:", &editor.edit_id, move |v| {
                    Message::NpcRefOpFieldChanged(orig, "id".into(), v)
                }));
                detail_content.push(labeled_input("NPC ID:", &editor.edit_npc_id, move |v| {
                    Message::NpcRefOpFieldChanged(orig, "npc_id".into(), v)
                }));
                detail_content.push(labeled_input("Name:", &editor.edit_name, move |v| {
                    Message::NpcRefOpFieldChanged(orig, "name".into(), v)
                }));
                detail_content.push(labeled_input(
                    "Party Script ID:",
                    &editor.edit_party_script_id,
                    move |v| Message::NpcRefOpFieldChanged(orig, "party_script_id".into(), v),
                ));
                detail_content.push(labeled_input(
                    "Show On Event:",
                    &editor.edit_show_on_event,
                    move |v| Message::NpcRefOpFieldChanged(orig, "show_on_event".into(), v),
                ));

                let dirs = vec![
                    NpcLookingDirection::Up,
                    NpcLookingDirection::UpRight,
                    NpcLookingDirection::Right,
                    NpcLookingDirection::DownRight,
                    NpcLookingDirection::Down,
                    NpcLookingDirection::DownLeft,
                    NpcLookingDirection::Left,
                    NpcLookingDirection::UpLeft,
                ];
                let dir_value = dirs
                    .iter()
                    .find(|d| format!("{:?}", d) == editor.edit_looking_direction)
                    .copied()
                    .unwrap_or(NpcLookingDirection::Up);
                detail_content.push(labeled_select(
                    "Looking Direction:",
                    dir_value,
                    dirs.clone(),
                    move |v| {
                        Message::NpcRefOpFieldChanged(
                            orig,
                            "looking_direction".into(),
                            format!("{:?}", v),
                        )
                    },
                ));

                detail_content.push(labeled_input(
                    "Dialog ID:",
                    &editor.edit_dialog_id,
                    move |v| Message::NpcRefOpFieldChanged(orig, "dialog_id".into(), v),
                ));

                detail_content.push(text("Waypoints:").size(14).font(Font::MONOSPACE).into());

                detail_content.push(labeled_input(
                    "WP1 Active:",
                    &editor.edit_goto1_filled,
                    move |v| Message::NpcRefOpFieldChanged(orig, "goto1_filled".into(), v),
                ));
                detail_content.push(labeled_input("WP1 X:", &editor.edit_goto1_x, move |v| {
                    Message::NpcRefOpFieldChanged(orig, "goto1_x".into(), v)
                }));
                detail_content.push(labeled_input("WP1 Y:", &editor.edit_goto1_y, move |v| {
                    Message::NpcRefOpFieldChanged(orig, "goto1_y".into(), v)
                }));
                detail_content.push(labeled_input(
                    "WP2 Active:",
                    &editor.edit_goto2_filled,
                    move |v| Message::NpcRefOpFieldChanged(orig, "goto2_filled".into(), v),
                ));
                detail_content.push(labeled_input("WP2 X:", &editor.edit_goto2_x, move |v| {
                    Message::NpcRefOpFieldChanged(orig, "goto2_x".into(), v)
                }));
                detail_content.push(labeled_input("WP2 Y:", &editor.edit_goto2_y, move |v| {
                    Message::NpcRefOpFieldChanged(orig, "goto2_y".into(), v)
                }));
                detail_content.push(labeled_input(
                    "WP3 Active:",
                    &editor.edit_goto3_filled,
                    move |v| Message::NpcRefOpFieldChanged(orig, "goto3_filled".into(), v),
                ));
                detail_content.push(labeled_input("WP3 X:", &editor.edit_goto3_x, move |v| {
                    Message::NpcRefOpFieldChanged(orig, "goto3_x".into(), v)
                }));
                detail_content.push(labeled_input("WP3 Y:", &editor.edit_goto3_y, move |v| {
                    Message::NpcRefOpFieldChanged(orig, "goto3_y".into(), v)
                }));
                detail_content.push(labeled_input(
                    "WP4 Active:",
                    &editor.edit_goto4_filled,
                    move |v| Message::NpcRefOpFieldChanged(orig, "goto4_filled".into(), v),
                ));
                detail_content.push(labeled_input("WP4 X:", &editor.edit_goto4_x, move |v| {
                    Message::NpcRefOpFieldChanged(orig, "goto4_x".into(), v)
                }));
                detail_content.push(labeled_input("WP4 Y:", &editor.edit_goto4_y, move |v| {
                    Message::NpcRefOpFieldChanged(orig, "goto4_y".into(), v)
                }));
            }
        } else {
            detail_content.push(
                text("No NPC selected")
                    .size(13)
                    .style(style::subtle_text)
                    .into(),
            );
        }

        let detail_scroll = scrollable(column(detail_content).spacing(8)).height(Length::Fill);

        let detail_panel = container(detail_scroll)
            .padding(16)
            .width(Length::FillPortion(2))
            .style(style::info_card);

        let npc_list_header = row![text("NPCs").size(14), horizontal_space()]
            .padding(10)
            .align_y(iced::Alignment::Center);

        let left_panel = column![
            container(file_bar).style(style::toolbar_container),
            container(npc_list_header).style(style::grid_header_cell),
            npc_scroll,
        ];

        let main_content = row![left_panel.width(Length::FillPortion(1)), detail_panel]
            .spacing(0)
            .height(Length::Fill);

        column![
            horizontal_rule(1),
            main_content,
            container(
                row![
                    text(&editor.status_msg).size(13).style(style::subtle_text),
                    horizontal_space(),
                    if editor.is_loading {
                        Element::from(text("Loading...").size(13))
                    } else {
                        Element::from(text(""))
                    },
                    horizontal_space().width(20),
                    button(text("Save NPCs"))
                        .on_press(Message::NpcRefOpSave)
                        .style(style::commit_button),
                ]
                .padding([10, 20])
                .align_y(iced::Alignment::Center),
            )
            .width(Fill)
            .style(style::status_bar),
        ]
        .spacing(0)
        .height(Length::Fill)
        .into()
    }
}
