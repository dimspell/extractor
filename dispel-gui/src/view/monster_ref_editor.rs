use crate::app::App;
use crate::message::Message;
use crate::style;
use crate::utils::{
    horizontal_rule, horizontal_space, labeled_input, labeled_select, truncate_path, vertical_space,
};
use dispel_core::ItemTypeId;
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Element, Fill, Font};

impl App {
    pub fn view_monster_ref_editor_tab(&self) -> Element<'_, Message> {
        let editor = &self.monster_ref_editor;

        let file_path_row = row![
            text("File:").size(12).width(60).style(style::subtle_text),
            container(
                text(truncate_path(&editor.current_map_file, 60))
                    .size(11)
                    .font(Font::MONOSPACE)
            )
            .padding([4, 10])
            .width(Fill)
            .style(style::sql_editor_container),
            button(text("Browse").size(11))
                .on_press(Message::MonsterRefOpBrowseFile)
                .padding([5, 10])
                .style(style::browse_button),
        ]
        .spacing(10)
        .padding([0, 8])
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
                button(text("Save File"))
                    .on_press(Message::MonsterRefOpSave)
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
                        .on_press(Message::MonsterRefOpSelectFile(path.clone()));
                if is_selected {
                    btn.style(style::active_tab_button).into()
                } else {
                    btn.style(style::tab_button).into()
                }
            })
            .collect();

        let item_list: Vec<Element<Message>> = editor
            .filtered_entries
            .iter()
            .map(|(i, entry)| {
                let is_selected = editor.selected_idx == Some(*i);
                let label = format!(
                    "[{:03}] mon:{} x:{} y:{} l1:{} l2:{} l3:{}",
                    i,
                    entry.mon_id,
                    entry.pos_x,
                    entry.pos_y,
                    entry.loot1_item_id,
                    entry.loot2_item_id,
                    entry.loot3_item_id,
                );

                let btn = button(text(label).size(11).font(Font::MONOSPACE))
                    .width(Fill)
                    .on_press(Message::MonsterRefOpSelectEntry(*i));

                if is_selected {
                    btn.style(style::active_chip).into()
                } else {
                    btn.style(style::chip).into()
                }
            })
            .collect();

        let mut detail_content: Vec<Element<Message>> = vec![
            text("Monster Placement Details")
                .size(16)
                .font(Font::MONOSPACE)
                .into(),
            vertical_space().height(10).into(),
        ];

        if let Some(idx) = editor.selected_idx {
            if let Some((orig_idx, _entry)) = editor.filtered_entries.get(idx) {
                let orig = *orig_idx;

                detail_content.push(labeled_input(
                    "File ID:",
                    &editor.entry_editor.edit_file_id,
                    move |v| Message::MonsterRefOpFieldChanged(orig, "file_id".into(), v),
                ));
                detail_content.push(labeled_input(
                    "Monster ID:",
                    &editor.entry_editor.edit_mon_id,
                    move |v| Message::MonsterRefOpFieldChanged(orig, "mon_id".into(), v),
                ));
                detail_content.push(labeled_input(
                    "Position X:",
                    &editor.entry_editor.edit_pos_x,
                    move |v| Message::MonsterRefOpFieldChanged(orig, "pos_x".into(), v),
                ));
                detail_content.push(labeled_input(
                    "Position Y:",
                    &editor.entry_editor.edit_pos_y,
                    move |v| Message::MonsterRefOpFieldChanged(orig, "pos_y".into(), v),
                ));

                detail_content.push(horizontal_rule(1).into());
                detail_content.push(
                    text("Loot Slot 1")
                        .size(13)
                        .style(style::subtle_text)
                        .into(),
                );
                detail_content.push(labeled_input(
                    "Item ID:",
                    &editor.entry_editor.edit_loot1_item_id,
                    move |v| Message::MonsterRefOpFieldChanged(orig, "loot1_item_id".into(), v),
                ));
                let item_type_options = vec![
                    ItemTypeId::Weapon,
                    ItemTypeId::Healing,
                    ItemTypeId::Edit,
                    ItemTypeId::Misc,
                    ItemTypeId::Event,
                    ItemTypeId::Other,
                ];
                let loot1_type = ItemTypeId::from_name(&editor.entry_editor.edit_loot1_item_type)
                    .unwrap_or(ItemTypeId::Other);
                detail_content.push(labeled_select(
                    "Item Type:",
                    loot1_type,
                    item_type_options.clone(),
                    move |v| {
                        Message::MonsterRefOpFieldChanged(
                            orig,
                            "loot1_item_type".into(),
                            v.to_string(),
                        )
                    },
                ));

                detail_content.push(horizontal_rule(1).into());
                detail_content.push(
                    text("Loot Slot 2")
                        .size(13)
                        .style(style::subtle_text)
                        .into(),
                );
                detail_content.push(labeled_input(
                    "Item ID:",
                    &editor.entry_editor.edit_loot2_item_id,
                    move |v| Message::MonsterRefOpFieldChanged(orig, "loot2_item_id".into(), v),
                ));
                let loot2_type = ItemTypeId::from_name(&editor.entry_editor.edit_loot2_item_type)
                    .unwrap_or(ItemTypeId::Other);
                detail_content.push(labeled_select(
                    "Item Type:",
                    loot2_type,
                    item_type_options.clone(),
                    move |v| {
                        Message::MonsterRefOpFieldChanged(
                            orig,
                            "loot2_item_type".into(),
                            v.to_string(),
                        )
                    },
                ));

                detail_content.push(horizontal_rule(1).into());
                detail_content.push(
                    text("Loot Slot 3")
                        .size(13)
                        .style(style::subtle_text)
                        .into(),
                );
                detail_content.push(labeled_input(
                    "Item ID:",
                    &editor.entry_editor.edit_loot3_item_id,
                    move |v| Message::MonsterRefOpFieldChanged(orig, "loot3_item_id".into(), v),
                ));
                let loot3_type = ItemTypeId::from_name(&editor.entry_editor.edit_loot3_item_type)
                    .unwrap_or(ItemTypeId::Other);
                detail_content.push(labeled_select(
                    "Item Type:",
                    loot3_type,
                    item_type_options,
                    move |v| {
                        Message::MonsterRefOpFieldChanged(
                            orig,
                            "loot3_item_type".into(),
                            v.to_string(),
                        )
                    },
                ));
            }
        } else {
            detail_content.push(
                text("No entry selected")
                    .size(13)
                    .style(style::subtle_text)
                    .into(),
            );
        }

        let detail_panel = container(scrollable(column(detail_content).spacing(8)).height(Fill))
            .padding(16)
            .width(Fill)
            .style(style::info_card);

        let list_header = row![
            text("Placements").size(14),
            horizontal_space(),
            text(format!("{} found", editor.filtered_entries.len()))
                .size(12)
                .style(style::subtle_text),
        ]
        .padding(10)
        .align_y(iced::Alignment::Center);

        let main_content = row![
            column![
                container(
                    row![
                        text("Files").size(14),
                        horizontal_space(),
                        button(text("Scan"))
                            .on_press(Message::MonsterRefOpScanFiles)
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
                scrollable(column(item_list)).height(Fill),
            ]
            .width(280),
            detail_panel,
        ]
        .spacing(0)
        .height(Fill);

        column![
            container(column![file_path_row].padding(10).spacing(8))
                .style(style::toolbar_container),
            horizontal_rule(1),
            main_content,
            status_row,
        ]
        .spacing(0)
        .into()
    }
}
