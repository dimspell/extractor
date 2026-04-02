use crate::app::App;
use crate::message::Message;
use crate::style;
use crate::utils::{horizontal_rule, horizontal_space, labeled_input, vertical_space};
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Element, Fill, Font, Length};

impl App {
    pub fn view_map_ini_editor_tab(&self) -> Element<'_, Message> {
        let editor = &self.map_ini_editor;

        let item_list: Vec<Element<Message>> = editor
            .filtered_maps
            .iter()
            .enumerate()
            .map(|(idx, (_, map))| {
                let is_selected = editor.selected_idx == Some(idx);
                let label = format!(
                    "[{}] map:{} start:{},{} music:{}",
                    map.id, map.map_id, map.start_pos_x, map.start_pos_y, map.cd_music_track_number
                );

                let btn = button(text(label).size(11).font(Font::MONOSPACE))
                    .width(Fill)
                    .on_press(Message::MapIniOpSelectMap(idx));

                if is_selected {
                    btn.style(style::active_chip).into()
                } else {
                    btn.style(style::chip).into()
                }
            })
            .collect();

        let item_scroll = scrollable(column(item_list).spacing(4)).height(Length::Fill);

        let mut detail_content: Vec<Element<Message>> = vec![
            text("Map INI Details")
                .size(16)
                .font(Font::MONOSPACE)
                .into(),
            vertical_space().height(10).into(),
        ];

        if let Some(idx) = editor.selected_idx {
            if let Some((orig_idx, _map)) = editor.filtered_maps.get(idx) {
                let orig = *orig_idx;

                detail_content.push(labeled_input("ID:", &editor.edit_id, move |v| {
                    Message::MapIniOpFieldChanged(orig, "id".into(), v)
                }));
                detail_content.push(labeled_input(
                    "Event ID on Camera Move:",
                    &editor.edit_event_id_on_camera_move,
                    move |v| {
                        Message::MapIniOpFieldChanged(orig, "event_id_on_camera_move".into(), v)
                    },
                ));
                detail_content.push(labeled_input(
                    "Start Pos X:",
                    &editor.edit_start_pos_x,
                    move |v| Message::MapIniOpFieldChanged(orig, "start_pos_x".into(), v),
                ));
                detail_content.push(labeled_input(
                    "Start Pos Y:",
                    &editor.edit_start_pos_y,
                    move |v| Message::MapIniOpFieldChanged(orig, "start_pos_y".into(), v),
                ));
                detail_content.push(labeled_input("Map ID:", &editor.edit_map_id, move |v| {
                    Message::MapIniOpFieldChanged(orig, "map_id".into(), v)
                }));
                detail_content.push(labeled_input(
                    "Monsters Filename:",
                    &editor.edit_monsters_filename,
                    move |v| Message::MapIniOpFieldChanged(orig, "monsters_filename".into(), v),
                ));
                detail_content.push(labeled_input(
                    "NPC Filename:",
                    &editor.edit_npc_filename,
                    move |v| Message::MapIniOpFieldChanged(orig, "npc_filename".into(), v),
                ));
                detail_content.push(labeled_input(
                    "Extra Filename:",
                    &editor.edit_extra_filename,
                    move |v| Message::MapIniOpFieldChanged(orig, "extra_filename".into(), v),
                ));
                detail_content.push(labeled_input(
                    "CD Music Track Number:",
                    &editor.edit_cd_music_track_number,
                    move |v| Message::MapIniOpFieldChanged(orig, "cd_music_track_number".into(), v),
                ));
            }
        } else {
            detail_content.push(
                text("No map INI selected")
                    .size(13)
                    .style(style::subtle_text)
                    .into(),
            );
        }

        let detail_scroll = scrollable(column(detail_content).spacing(8)).height(Length::Fill);

        let detail_panel = container(detail_scroll)
            .padding(16)
            .width(380)
            .style(style::info_card);

        let item_list_header = row![
            text("Map INI").size(14),
            horizontal_space(),
            button(text("Scan"))
                .on_press(Message::MapIniOpLoadCatalog)
                .padding([5, 10])
                .style(style::run_button),
        ]
        .padding(10)
        .align_y(iced::Alignment::Center);

        let left_panel = column![
            container(item_list_header).style(style::grid_header_cell),
            item_scroll,
        ];

        let main_content = row![left_panel, detail_panel.width(Length::FillPortion(2)),]
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
                    button(text("Save Map INI"))
                        .on_press(Message::MapIniOpSave)
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
