use crate::app::App;
use crate::message::Message;
use crate::style;
use crate::utils::{labeled_input, truncate_path};
use iced::widget::{
    button, column, container, horizontal_rule, horizontal_space, row, scrollable, text,
    vertical_space,
};
use iced::{Element, Fill, Font};

impl App {
    pub fn view_chest_editor_tab(&self) -> Element<'_, Message> {
        let editor = &self.chest_editor;

        let game_path_row = row![
            text("Game:").size(12).width(60).style(style::subtle_text),
            container(
                text(truncate_path(&editor.game_path, 60))
                    .size(11)
                    .font(Font::MONOSPACE)
            )
            .padding([4, 10])
            .width(Fill)
            .style(style::sql_editor_container),
            button(text("Browse").size(11))
                .on_press(Message::ChestOpBrowseGamePath)
                .padding([5, 10])
                .style(style::browse_button),
            button(text("Load Catalog").size(11))
                .on_press(Message::ChestOpLoadCatalog)
                .padding([5, 10])
                .style(style::run_button),
        ]
        .spacing(10)
        .align_y(iced::Alignment::Center);

        let map_file_row = row![
            text("Map:").size(12).width(60).style(style::subtle_text),
            container(
                text(truncate_path(&editor.current_map_file, 60))
                    .size(11)
                    .font(Font::MONOSPACE)
            )
            .padding([4, 10])
            .width(Fill)
            .style(style::sql_editor_container),
            button(text("Browse").size(11))
                .on_press(Message::ChestOpBrowseMapFile)
                .padding([5, 10])
                .style(style::browse_button),
            button(text("Load Map").size(11))
                .on_press(Message::ChestOpSelectMap)
                .padding([5, 10])
                .style(style::run_button),
        ]
        .spacing(10)
        .align_y(iced::Alignment::Center);

        let status_row = container(
            row![
                text(&editor.status_msg).size(13).style(style::subtle_text),
                horizontal_space(),
                if editor.is_loading {
                    Element::from(text("Loading...").size(13))
                } else {
                    Element::from(text(""))
                },
                horizontal_space().width(20),
                button(text("Save Map Changes"))
                    .on_press(Message::ChestOpSave)
                    .style(style::commit_button),
            ]
            .padding([10, 20])
            .align_y(iced::Alignment::Center),
        )
        .width(Fill)
        .style(style::status_bar);

        let map_list: Vec<Element<Message>> = editor
            .map_files
            .iter()
            .map(|path| {
                let is_selected = editor.current_map_file == path.to_string_lossy();
                let btn =
                    button(text(path.file_name().unwrap_or_default().to_string_lossy()).size(12))
                        .width(Fill)
                        .on_press(Message::ChestOpSelectMapFromFile(path.clone()));
                if is_selected {
                    btn.style(style::active_tab_button).into()
                } else {
                    btn.style(style::tab_button).into()
                }
            })
            .collect();

        let chest_list: Vec<Element<Message>> = editor
            .filtered_chests
            .iter()
            .enumerate()
            .map(|(idx, (_, record))| {
                let is_selected = editor.selected_idx == Some(idx);
                let item_name = editor
                    .catalog
                    .as_ref()
                    .and_then(|c| c.get_item_name(record.item_type_id, record.item_id))
                    .unwrap_or_else(|| format!("{:?}_{}", record.item_type_id, record.item_id));

                let label = format!(
                    "Chest [{}] x:{} y:{}\n  {} (x{})\n  {} gold",
                    record.id,
                    record.x_pos,
                    record.y_pos,
                    item_name,
                    record.item_count,
                    record.gold_amount
                );

                let btn = button(text(label).size(11).font(Font::MONOSPACE))
                    .width(Fill)
                    .on_press(Message::ChestOpSelectChest(idx));

                if is_selected {
                    btn.style(style::active_chip).into()
                } else {
                    btn.style(style::chip).into()
                }
            })
            .collect();

        let mut detail_content: Vec<Element<Message>> = vec![
            text("Chest Details").size(16).font(Font::MONOSPACE).into(),
            vertical_space().height(10).into(),
        ];

        if let Some(idx) = editor.selected_idx {
            if let Some((orig_idx, record)) = editor.filtered_chests.get(idx) {
                let orig = *orig_idx;

                detail_content.push(labeled_input("Name:", &editor.edit_name, move |v| {
                    Message::ChestOpFieldChanged(orig, "name".into(), v)
                }));
                detail_content.push(labeled_input("X Pos:", &editor.edit_x, move |v| {
                    Message::ChestOpFieldChanged(orig, "x".into(), v)
                }));
                detail_content.push(labeled_input("Y Pos:", &editor.edit_y, move |v| {
                    Message::ChestOpFieldChanged(orig, "y".into(), v)
                }));
                detail_content.push(labeled_input("Gold:", &editor.edit_gold, move |v| {
                    Message::ChestOpFieldChanged(orig, "gold".into(), v)
                }));
                detail_content.push(labeled_input(
                    "Item Count:",
                    &editor.edit_item_count,
                    move |v| Message::ChestOpFieldChanged(orig, "item_count".into(), v),
                ));
                detail_content.push(labeled_input("Item ID:", &editor.edit_item_id, move |v| {
                    Message::ChestOpFieldChanged(orig, "item_id".into(), v)
                }));
                detail_content.push(labeled_input(
                    "Item Type:",
                    &editor.edit_item_type,
                    move |v| Message::ChestOpFieldChanged(orig, "item_type".into(), v),
                ));
                detail_content.push(labeled_input(
                    "Closed (0=open, 1=closed):",
                    &editor.edit_closed,
                    move |v| Message::ChestOpFieldChanged(orig, "closed".into(), v),
                ));

                let item_name = editor
                    .catalog
                    .as_ref()
                    .and_then(|c| c.get_item_name(record.item_type_id, record.item_id))
                    .unwrap_or_default();
                if !item_name.is_empty() {
                    detail_content.push(
                        text(format!("Resolved Item: {}", item_name))
                            .size(12)
                            .style(style::subtle_text)
                            .into(),
                    );
                }
            }
        } else {
            detail_content.push(
                text("No chest selected")
                    .size(13)
                    .style(style::subtle_text)
                    .into(),
            );
        }

        let detail_panel = container(scrollable(column(detail_content).spacing(8)).height(Fill))
            .padding(16)
            .width(250)
            .style(style::info_card);

        let list_header = row![
            text("Chests").size(14),
            horizontal_space(),
            text(format!("{} found", editor.filtered_chests.len()))
                .size(12)
                .style(style::subtle_text)
        ]
        .padding(10)
        .align_y(iced::Alignment::Center);

        let main_content = row![
            column![
                container(
                    row![
                        text("Maps").size(14),
                        horizontal_space(),
                        button(text("Scan"))
                            .on_press(Message::ChestOpScanMaps)
                            .style(style::chip)
                    ]
                    .padding(10)
                    .align_y(iced::Alignment::Center)
                )
                .style(style::grid_header_cell),
                scrollable(column(map_list)).height(Fill),
            ]
            .width(180),
            column![
                container(list_header).style(style::grid_header_cell),
                scrollable(column(chest_list)).height(Fill),
            ]
            .width(Fill),
            detail_panel,
        ]
        .spacing(0)
        .height(Fill);

        column![
            container(
                column![game_path_row, map_file_row]
                    .padding(10)
                    .spacing(8)
            )
            .style(style::toolbar_container),
            horizontal_rule(1),
            main_content,
            status_row,
        ]
        .spacing(0)
        .into()
    }
}
