use crate::app::App;
use crate::message::Message;
use crate::style;
use crate::utils::{
    horizontal_rule, horizontal_space, labeled_input, labeled_select, truncate_path, vertical_space,
};
use dispel_core::{DialogOwner, DialogType};
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Element, Fill, Font, Length};

impl App {
    pub fn view_dialog_editor_tab(&self) -> Element<'_, Message> {
        let editor = &self.state.dialog_editor;

        let file_list: Vec<Element<Message>> = editor
            .dialog_files
            .iter()
            .enumerate()
            .map(|(_idx, path)| {
                let is_selected = editor.current_file == path.to_string_lossy();
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                let label = truncate_path(&name, 30);

                let btn = button(text(label).size(11).font(Font::MONOSPACE))
                    .width(Fill)
                    .on_press(Message::DialogOpSelectFile(path.clone()));

                if is_selected {
                    btn.style(style::active_chip).into()
                } else {
                    btn.style(style::chip).into()
                }
            })
            .collect();

        let file_scroll = scrollable(column(file_list).spacing(4)).height(Length::Fill);

        let item_list: Vec<Element<Message>> = editor
            .filtered_dialogs
            .iter()
            .enumerate()
            .map(|(idx, (_, dialog))| {
                let is_selected = editor.selected_idx == Some(idx);
                let label = format!(
                    "[{}] type:{} owner:{} dlg_id:{} evt_id:{}",
                    dialog.id,
                    dialog
                        .dialog_type
                        .map(|t| format!("{:?}", t))
                        .unwrap_or_else(|| "null".into()),
                    dialog
                        .dialog_owner
                        .map(|o| format!("{:?}", o))
                        .unwrap_or_else(|| "null".into()),
                    dialog
                        .dialog_id
                        .map(|d| d.to_string())
                        .unwrap_or_else(|| "null".into()),
                    dialog
                        .event_id
                        .map(|e| e.to_string())
                        .unwrap_or_else(|| "null".into()),
                );

                let btn = button(text(label).size(11).font(Font::MONOSPACE))
                    .width(Fill)
                    .on_press(Message::DialogOpSelectDialog(idx));

                if is_selected {
                    btn.style(style::active_chip).into()
                } else {
                    btn.style(style::chip).into()
                }
            })
            .collect();

        let item_scroll = scrollable(column(item_list).spacing(4)).height(Length::Fill);

        let mut detail_content: Vec<Element<Message>> = vec![
            text("Dialog Details").size(16).font(Font::MONOSPACE).into(),
            vertical_space().height(10).into(),
        ];

        if let Some(idx) = editor.selected_idx {
            if let Some((orig_idx, _dialog)) = editor.filtered_dialogs.get(idx) {
                let orig = *orig_idx;

                detail_content.push(labeled_input("ID:", &editor.edit_id, move |v| {
                    Message::DialogOpFieldChanged(orig, "id".into(), v)
                }));
                detail_content.push(labeled_input(
                    "Previous Event ID:",
                    &editor.edit_previous_event_id,
                    move |v| Message::DialogOpFieldChanged(orig, "previous_event_id".into(), v),
                ));
                detail_content.push(labeled_input(
                    "Next Dialog to Check:",
                    &editor.edit_next_dialog_to_check,
                    move |v| Message::DialogOpFieldChanged(orig, "next_dialog_to_check".into(), v),
                ));

                let dialog_type_options = vec![DialogType::Normal, DialogType::Choice];
                let dialog_type_value = if editor.edit_dialog_type.contains("Choice") {
                    DialogType::Choice
                } else if editor.edit_dialog_type.contains("Normal") {
                    DialogType::Normal
                } else {
                    DialogType::Normal
                };
                detail_content.push(labeled_select(
                    "Dialog Type:",
                    dialog_type_value,
                    dialog_type_options,
                    move |v| {
                        Message::DialogOpFieldChanged(
                            orig,
                            "dialog_type".into(),
                            format!("{:?}", v),
                        )
                    },
                ));

                let dialog_owner_options = vec![DialogOwner::Player, DialogOwner::Npc];
                let dialog_owner_value = if editor.edit_dialog_owner.contains("Npc") {
                    DialogOwner::Npc
                } else {
                    DialogOwner::Player
                };
                detail_content.push(labeled_select(
                    "Dialog Owner:",
                    dialog_owner_value,
                    dialog_owner_options,
                    move |v| {
                        Message::DialogOpFieldChanged(
                            orig,
                            "dialog_owner".into(),
                            format!("{:?}", v),
                        )
                    },
                ));

                detail_content.push(labeled_input(
                    "Dialog ID:",
                    &editor.edit_dialog_id,
                    move |v| Message::DialogOpFieldChanged(orig, "dialog_id".into(), v),
                ));
                detail_content.push(labeled_input(
                    "Event ID:",
                    &editor.edit_event_id,
                    move |v| Message::DialogOpFieldChanged(orig, "event_id".into(), v),
                ));
            }
        } else {
            detail_content.push(
                text("No dialog selected")
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

        let file_header = row![
            text("Dialog Files").size(14),
            horizontal_space(),
            button(text("Scan"))
                .on_press(Message::DialogOpScanFiles)
                .padding([5, 10])
                .style(style::run_button),
            button(text("Browse"))
                .on_press(Message::DialogOpBrowseFile)
                .padding([5, 10])
                .style(style::browse_button),
        ]
        .padding(10)
        .align_y(iced::Alignment::Center);

        let item_list_header = row![text("Entries").size(14), horizontal_space()]
            .padding(10)
            .align_y(iced::Alignment::Center);

        let file_panel = column![
            container(file_header).style(style::grid_header_cell),
            file_scroll,
        ];

        let item_panel = column![
            container(item_list_header).style(style::grid_header_cell),
            item_scroll,
        ];

        let main_content = row![
            file_panel.width(Length::FillPortion(1)),
            item_panel.width(Length::FillPortion(1)),
            detail_panel,
        ]
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
                    button(text("Save Dialogs"))
                        .on_press(Message::DialogOpSave)
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
