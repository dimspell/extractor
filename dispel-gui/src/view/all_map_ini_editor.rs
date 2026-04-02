use crate::app::App;
use crate::message::Message;
use crate::style;
use crate::utils::{
    horizontal_rule, horizontal_space, labeled_input, labeled_select, vertical_space,
};
use dispel_core::MapLighting;
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Element, Fill, Font, Length};

impl App {
    pub fn view_all_map_ini_editor_tab(&self) -> Element<'_, Message> {
        let editor = &self.all_map_ini_editor;

        let item_list: Vec<Element<Message>> = editor
            .filtered_maps
            .iter()
            .enumerate()
            .map(|(idx, (_, map))| {
                let is_selected = editor.selected_idx == Some(idx);
                let label = format!("[{}] {} ({})", map.id, map.map_name, map.map_filename);

                let btn = button(text(label).size(11).font(Font::MONOSPACE))
                    .width(Fill)
                    .on_press(Message::AllMapIniOpSelectMap(idx));

                if is_selected {
                    btn.style(style::active_chip).into()
                } else {
                    btn.style(style::chip).into()
                }
            })
            .collect();

        let item_scroll = scrollable(column(item_list).spacing(4)).height(Length::Fill);

        let mut detail_content: Vec<Element<Message>> = vec![
            text("Map Details").size(16).font(Font::MONOSPACE).into(),
            vertical_space().height(10).into(),
        ];

        if let Some(idx) = editor.selected_idx {
            if let Some((orig_idx, _map)) = editor.filtered_maps.get(idx) {
                let orig = *orig_idx;

                detail_content.push(labeled_input("ID:", &editor.edit_id, move |v| {
                    Message::AllMapIniOpFieldChanged(orig, "id".into(), v)
                }));
                detail_content.push(labeled_input(
                    "Map Filename:",
                    &editor.edit_map_filename,
                    move |v| Message::AllMapIniOpFieldChanged(orig, "map_filename".into(), v),
                ));
                detail_content.push(labeled_input(
                    "Map Name:",
                    &editor.edit_map_name,
                    move |v| Message::AllMapIniOpFieldChanged(orig, "map_name".into(), v),
                ));
                detail_content.push(labeled_input(
                    "PGP Filename:",
                    &editor.edit_pgp_filename,
                    move |v| Message::AllMapIniOpFieldChanged(orig, "pgp_filename".into(), v),
                ));
                detail_content.push(labeled_input(
                    "DLG Filename:",
                    &editor.edit_dlg_filename,
                    move |v| Message::AllMapIniOpFieldChanged(orig, "dlg_filename".into(), v),
                ));

                let lighting_options = vec![MapLighting::Dark, MapLighting::Light];
                let lighting_value = if editor.edit_lighting.contains("Light") {
                    MapLighting::Light
                } else {
                    MapLighting::Dark
                };
                detail_content.push(labeled_select(
                    "Lighting:",
                    lighting_value,
                    lighting_options,
                    move |v| {
                        Message::AllMapIniOpFieldChanged(
                            orig,
                            "lighting".into(),
                            format!("{:?}", v),
                        )
                    },
                ));
            }
        } else {
            detail_content.push(
                text("No map selected")
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
            text("All Maps").size(14),
            horizontal_space(),
            button(text("Scan"))
                .on_press(Message::AllMapIniOpLoadCatalog)
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
                    button(text("Save All Maps"))
                        .on_press(Message::AllMapIniOpSave)
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
