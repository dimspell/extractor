use crate::app::App;
use crate::message::Message;
use crate::style;
use crate::utils::{
    horizontal_rule, horizontal_space, labeled_input, labeled_select, truncate_path, vertical_space,
};
use dispel_core::{ExtraObjectType, ItemTypeId, VisibilityType};
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Element, Fill, Font};

impl App {
    pub fn view_extra_ref_editor_tab(&self) -> Element<'_, Message> {
        let editor = &self.state.extra_ref_editor;

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
                .on_press(Message::ExtraRefOpBrowseMapFile)
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
                    .on_press(Message::ExtraRefOpSave)
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
                        .on_press(Message::ExtraRefOpSelectMapFile(path.clone()));
                if is_selected {
                    btn.style(style::active_tab_button).into()
                } else {
                    btn.style(style::tab_button).into()
                }
            })
            .collect();

        let item_list: Vec<Element<Message>> = editor
            .filtered_items
            .iter()
            .enumerate()
            .map(|(idx, (_, item))| {
                let is_selected = editor.selected_idx == Some(idx);
                let label = format!(
                    "[{}] {} type:{} x:{} y:{}",
                    item.id,
                    item.name,
                    format!("{:?}", item.object_type),
                    item.x_pos,
                    item.y_pos
                );

                let btn = button(text(label).size(11).font(Font::MONOSPACE))
                    .width(Fill)
                    .on_press(Message::ExtraRefOpSelectItem(idx));

                if is_selected {
                    btn.style(style::active_chip).into()
                } else {
                    btn.style(style::chip).into()
                }
            })
            .collect();

        let mut detail_content: Vec<Element<Message>> = vec![
            text("Extra Object Details")
                .size(16)
                .font(Font::MONOSPACE)
                .into(),
            vertical_space().height(10).into(),
        ];

        if let Some(idx) = editor.selected_idx {
            if let Some((orig_idx, _item)) = editor.filtered_items.get(idx) {
                let orig = *orig_idx;

                detail_content.push(labeled_input("ID:", &editor.edit_id, move |v| {
                    Message::ExtraRefOpFieldChanged(orig, "id".into(), v)
                }));
                detail_content.push(labeled_input("Name:", &editor.edit_name, move |v| {
                    Message::ExtraRefOpFieldChanged(orig, "name".into(), v)
                }));
                detail_content.push(labeled_input("Ext ID:", &editor.edit_ext_id, move |v| {
                    Message::ExtraRefOpFieldChanged(orig, "ext_id".into(), v)
                }));

                let object_type_options = vec![
                    ExtraObjectType::Chest,
                    ExtraObjectType::Door,
                    ExtraObjectType::Sign,
                    ExtraObjectType::Altar,
                    ExtraObjectType::Interactive,
                    ExtraObjectType::Magic,
                    ExtraObjectType::Unknown,
                ];
                let object_type_value = if editor.edit_object_type.contains("Chest") {
                    ExtraObjectType::Chest
                } else if editor.edit_object_type.contains("Door") {
                    ExtraObjectType::Door
                } else if editor.edit_object_type.contains("Sign") {
                    ExtraObjectType::Sign
                } else if editor.edit_object_type.contains("Altar") {
                    ExtraObjectType::Altar
                } else if editor.edit_object_type.contains("Interactive") {
                    ExtraObjectType::Interactive
                } else if editor.edit_object_type.contains("Magic") {
                    ExtraObjectType::Magic
                } else {
                    ExtraObjectType::Unknown
                };
                detail_content.push(labeled_select(
                    "Object Type:",
                    object_type_value,
                    object_type_options,
                    move |v| {
                        Message::ExtraRefOpFieldChanged(
                            orig,
                            "object_type".into(),
                            format!("{:?}", v),
                        )
                    },
                ));

                detail_content.push(labeled_input("X Pos:", &editor.edit_x_pos, move |v| {
                    Message::ExtraRefOpFieldChanged(orig, "x_pos".into(), v)
                }));
                detail_content.push(labeled_input("Y Pos:", &editor.edit_y_pos, move |v| {
                    Message::ExtraRefOpFieldChanged(orig, "y_pos".into(), v)
                }));
                detail_content.push(labeled_input(
                    "Rotation:",
                    &editor.edit_rotation,
                    move |v| Message::ExtraRefOpFieldChanged(orig, "rotation".into(), v),
                ));
                detail_content.push(labeled_input("Closed:", &editor.edit_closed, move |v| {
                    Message::ExtraRefOpFieldChanged(orig, "closed".into(), v)
                }));
                detail_content.push(labeled_input(
                    "Required Item ID:",
                    &editor.edit_required_item_id,
                    move |v| Message::ExtraRefOpFieldChanged(orig, "required_item_id".into(), v),
                ));

                let item_type_options = vec![
                    ItemTypeId::Weapon,
                    ItemTypeId::Healing,
                    ItemTypeId::Edit,
                    ItemTypeId::Misc,
                    ItemTypeId::Event,
                    ItemTypeId::Other,
                ];
                let req_type1 = ItemTypeId::from_name(&editor.edit_required_item_type_id)
                    .unwrap_or(ItemTypeId::Other);
                detail_content.push(labeled_select(
                    "Req Item Type 1:",
                    req_type1,
                    item_type_options.clone(),
                    move |v| {
                        Message::ExtraRefOpFieldChanged(
                            orig,
                            "required_item_type_id".into(),
                            v.to_string(),
                        )
                    },
                ));

                detail_content.push(labeled_input(
                    "Required Item ID 2:",
                    &editor.edit_required_item_id2,
                    move |v| Message::ExtraRefOpFieldChanged(orig, "required_item_id2".into(), v),
                ));
                let req_type2 = ItemTypeId::from_name(&editor.edit_required_item_type_id2)
                    .unwrap_or(ItemTypeId::Other);
                detail_content.push(labeled_select(
                    "Req Item Type 2:",
                    req_type2,
                    item_type_options.clone(),
                    move |v| {
                        Message::ExtraRefOpFieldChanged(
                            orig,
                            "required_item_type_id2".into(),
                            v.to_string(),
                        )
                    },
                ));

                detail_content.push(labeled_input(
                    "Gold Amount:",
                    &editor.edit_gold_amount,
                    move |v| Message::ExtraRefOpFieldChanged(orig, "gold_amount".into(), v),
                ));
                detail_content.push(labeled_input("Item ID:", &editor.edit_item_id, move |v| {
                    Message::ExtraRefOpFieldChanged(orig, "item_id".into(), v)
                }));
                let item_type =
                    ItemTypeId::from_name(&editor.edit_item_type_id).unwrap_or(ItemTypeId::Other);
                detail_content.push(labeled_select(
                    "Item Type:",
                    item_type,
                    item_type_options,
                    move |v| {
                        Message::ExtraRefOpFieldChanged(orig, "item_type_id".into(), v.to_string())
                    },
                ));

                detail_content.push(labeled_input(
                    "Item Count:",
                    &editor.edit_item_count,
                    move |v| Message::ExtraRefOpFieldChanged(orig, "item_count".into(), v),
                ));
                detail_content.push(labeled_input(
                    "Event ID:",
                    &editor.edit_event_id,
                    move |v| Message::ExtraRefOpFieldChanged(orig, "event_id".into(), v),
                ));
                detail_content.push(labeled_input(
                    "Message ID:",
                    &editor.edit_message_id,
                    move |v| Message::ExtraRefOpFieldChanged(orig, "message_id".into(), v),
                ));

                let visibility_options = vec![
                    VisibilityType::Visible0,
                    VisibilityType::Visible10,
                    VisibilityType::Unknown,
                ];
                let visibility_value = if editor.edit_visibility.contains("Visible10") {
                    VisibilityType::Visible10
                } else if editor.edit_visibility.contains("Visible0") {
                    VisibilityType::Visible0
                } else {
                    VisibilityType::Unknown
                };
                detail_content.push(labeled_select(
                    "Visibility:",
                    visibility_value,
                    visibility_options,
                    move |v| {
                        Message::ExtraRefOpFieldChanged(
                            orig,
                            "visibility".into(),
                            format!("{:?}", v),
                        )
                    },
                ));

                detail_content.push(labeled_input(
                    "Interactive Element Type:",
                    &editor.edit_interactive_element_type,
                    move |v| {
                        Message::ExtraRefOpFieldChanged(orig, "interactive_element_type".into(), v)
                    },
                ));
                detail_content.push(labeled_input(
                    "Is Quest Element:",
                    &editor.edit_is_quest_element,
                    move |v| Message::ExtraRefOpFieldChanged(orig, "is_quest_element".into(), v),
                ));
            }
        } else {
            detail_content.push(
                text("No extra object selected")
                    .size(13)
                    .style(style::subtle_text)
                    .into(),
            );
        }

        let detail_scroll = scrollable(column(detail_content).spacing(8)).height(Fill);
        let detail_panel = container(detail_scroll)
            .padding(16)
            .width(Fill)
            .style(style::info_card);

        let list_header = row![
            text("Extra Objects").size(14),
            horizontal_space(),
            text(format!("{} found", editor.filtered_items.len()))
                .size(12)
                .style(style::subtle_text),
        ]
        .padding(10)
        .align_y(iced::Alignment::Center);

        let main_content = row![
            column![
                container(
                    row![
                        text("Map Files").size(14),
                        horizontal_space(),
                        button(text("Scan"))
                            .on_press(Message::ExtraRefOpLoadCatalog)
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
