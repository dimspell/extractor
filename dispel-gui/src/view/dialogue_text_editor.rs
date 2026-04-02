use crate::app::App;
use crate::message::Message;
use crate::style;
use crate::utils::{
    horizontal_rule, horizontal_space, labeled_input, truncate_path, vertical_space,
};
use iced::widget::{button, column, container, row, scrollable, text, text_editor};
use iced::{Element, Fill, Font, Length};

impl App {
    pub fn view_dialogue_text_editor_tab(&self) -> Element<'_, Message> {
        let editor = &self.dialogue_text_editor;

        let file_list: Vec<Element<Message>> = editor
            .text_files
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
                    .on_press(Message::DialogueTextOpSelectFile(path.clone()));

                if is_selected {
                    btn.style(style::active_chip).into()
                } else {
                    btn.style(style::chip).into()
                }
            })
            .collect();

        let file_scroll = scrollable(column(file_list).spacing(4)).height(Length::Fill);

        let item_list: Vec<Element<Message>> = editor
            .filtered_texts
            .iter()
            .enumerate()
            .map(|(idx, (_, text_rec))| {
                let is_selected = editor.selected_idx == Some(idx);
                let preview = if text_rec.text.len() > 40 {
                    format!("{}...", &text_rec.text[..40])
                } else {
                    text_rec.text.clone()
                };
                let label = format!("[{}] {}", text_rec.id, preview);

                let btn = button(text(label).size(11).font(Font::MONOSPACE))
                    .width(Fill)
                    .on_press(Message::DialogueTextOpSelectText(idx));

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
                    Message::DialogueTextOpFieldChanged(orig, "id".into(), v)
                }));

                detail_content.push(text("Text:").size(13).into());
                let te = text_editor(&editor.edit_text_content)
                    .on_action(move |action| Message::DialogueTextOpTextAction(orig, action))
                    .padding(8)
                    .height(100);
                detail_content.push(container(te).width(Fill).style(style::info_card).into());

                detail_content.push(text("Comment:").size(13).into());
                let te = text_editor(&editor.edit_comment_content)
                    .on_action(move |action| Message::DialogueTextOpCommentAction(orig, action))
                    .padding(8)
                    .height(80);
                detail_content.push(container(te).width(Fill).style(style::info_card).into());

                detail_content.push(labeled_input("Param 1:", &editor.edit_param1, move |v| {
                    Message::DialogueTextOpFieldChanged(orig, "param1".into(), v)
                }));
                detail_content.push(labeled_input(
                    "Wave INI Entry ID:",
                    &editor.edit_wave_ini_entry_id,
                    move |v| {
                        Message::DialogueTextOpFieldChanged(orig, "wave_ini_entry_id".into(), v)
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

        let file_header = row![
            text("Text Files").size(14),
            horizontal_space(),
            button(text("Scan"))
                .on_press(Message::DialogueTextOpScanFiles)
                .padding([5, 10])
                .style(style::run_button),
            button(text("Browse"))
                .on_press(Message::DialogueTextOpBrowseFile)
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
                    button(text("Save Dialogue Texts"))
                        .on_press(Message::DialogueTextOpSave)
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
