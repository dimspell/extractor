use crate::app::App;
use crate::message::{editor::dialoguetext::DialogueTextEditorMessage, Message, MessageExt};
use crate::style;
use crate::utils::{
    horizontal_rule, horizontal_space, labeled_input, truncate_path, vertical_space,
};
use iced::widget::{button, column, container, row, scrollable, text, text_editor};
use iced::{Element, Fill, Font, Length};

impl App {
    pub fn view_dialogue_text_editor_tab(&self) -> Element<'_, Message> {
        let tab_id = self
            .state
            .workspace
            .active()
            .map(|t| t.id)
            .unwrap_or(usize::MAX);

        let Some(editor) = self.state.dialogue_text_editors.get(&tab_id) else {
            return container(
                text("Dialogue text file not loaded")
                    .size(14)
                    .style(style::subtle_text),
            )
            .width(Fill)
            .height(Fill)
            .padding(16)
            .into();
        };

        let file_path_row = row![
            text("File:").size(12).width(40).style(style::subtle_text),
            container(
                text(truncate_path(&editor.current_file, 80))
                    .size(11)
                    .font(Font::MONOSPACE)
            )
            .padding([4, 10])
            .width(Fill)
            .style(style::sql_editor_container),
        ]
        .spacing(10)
        .padding([0, 8])
        .align_y(iced::Alignment::Center);

        let item_list: Vec<Element<Message>> = editor
            .filtered_texts
            .iter()
            .enumerate()
            .map(|(idx, (_, text_rec))| {
                let is_selected = editor.selected_idx == Some(idx);
                let preview = if text_rec.text.chars().count() > 40 {
                    format!("{}...", text_rec.text.chars().take(40).collect::<String>())
                } else {
                    text_rec.text.clone()
                };
                let label = format!("[{}] {}", text_rec.id, preview);

                let btn = button(text(label).size(11).font(Font::MONOSPACE))
                    .width(Fill)
                    .on_press(Message::dialogue_text(
                        DialogueTextEditorMessage::SelectText(idx),
                    ));

                if is_selected {
                    btn.style(style::active_chip).into()
                } else {
                    btn.style(style::chip).into()
                }
            })
            .collect();

        let item_scroll = scrollable(column(item_list).spacing(4)).height(Length::Fill);

        let mut detail_content: Vec<Element<Message>> = vec![
            text("Dialogue Text Details")
                .size(16)
                .font(Font::MONOSPACE)
                .into(),
            vertical_space().height(10).into(),
        ];

        if let Some(idx) = editor.selected_idx {
            if let Some((orig_idx, _text_rec)) = editor.filtered_texts.get(idx) {
                let orig = *orig_idx;

                detail_content.push(labeled_input("ID:", &editor.edit_id, move |v| {
                    Message::dialogue_text(DialogueTextEditorMessage::FieldChanged(
                        orig,
                        "id".into(),
                        v,
                    ))
                }));

                detail_content.push(text("Text:").size(13).into());
                let te = text_editor(&editor.edit_text_content)
                    .on_action(move |action| {
                        Message::dialogue_text(DialogueTextEditorMessage::TextAction(orig, action))
                    })
                    .padding(8)
                    .height(100);
                detail_content.push(container(te).width(Fill).style(style::info_card).into());

                detail_content.push(text("Comment:").size(13).into());
                let te = text_editor(&editor.edit_comment_content)
                    .on_action(move |action| {
                        Message::dialogue_text(DialogueTextEditorMessage::CommentAction(
                            orig, action,
                        ))
                    })
                    .padding(8)
                    .height(80);
                detail_content.push(container(te).width(Fill).style(style::info_card).into());

                detail_content.push(labeled_input("Param 1:", &editor.edit_param1, move |v| {
                    Message::dialogue_text(DialogueTextEditorMessage::FieldChanged(
                        orig,
                        "param1".into(),
                        v,
                    ))
                }));
                detail_content.push(labeled_input(
                    "Wave INI Entry ID:",
                    &editor.edit_wave_ini_entry_id,
                    move |v| {
                        Message::dialogue_text(DialogueTextEditorMessage::FieldChanged(
                            orig,
                            "wave_ini_entry_id".into(),
                            v,
                        ))
                    },
                ));
            }
        } else {
            detail_content.push(
                text("No dialogue text selected")
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

        let item_list_header = row![
            text("Entries").size(14),
            horizontal_space(),
            text(format!("{} found", editor.filtered_texts.len()))
                .size(12)
                .style(style::subtle_text),
        ]
        .padding(10)
        .align_y(iced::Alignment::Center);

        let item_panel = column![
            container(item_list_header).style(style::grid_header_cell),
            item_scroll,
        ];

        let main_content = row![item_panel.width(Length::FillPortion(1)), detail_panel,]
            .spacing(0)
            .height(Length::Fill);

        column![
            container(column![file_path_row].padding(10).spacing(8))
                .style(style::toolbar_container),
            horizontal_rule(1),
            main_content,
            container(
                row![
                    text(&editor.status_msg).size(13).style(style::subtle_text),
                    horizontal_space(),
                    if editor.loading_state.is_loading() {
                        Element::from(text("Loading...").size(13))
                    } else {
                        Element::from(text(""))
                    },
                    horizontal_space().width(20),
                    button(text("Save Dialogue Texts"))
                        .on_press(Message::dialogue_text(DialogueTextEditorMessage::Save))
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
