use crate::app::App;
use crate::message::Message;
use crate::style;
use crate::utils::{
    horizontal_rule, horizontal_space, labeled_input, labeled_select, vertical_space,
};
use iced::widget::{button, column, container, row, scrollable, text, text_editor};
use iced::{Element, Fill, Font, Length};

impl App {
    pub fn view_quest_scr_editor_tab(&self) -> Element<'_, Message> {
        let editor = &self.quest_scr_editor;

        let item_list: Vec<Element<Message>> = editor
            .filtered_quests
            .iter()
            .enumerate()
            .map(|(idx, (_, quest))| {
                let is_selected = editor.selected_idx == Some(idx);
                let type_label = match quest.type_id {
                    0 => "Main",
                    1 => "Side",
                    2 => "Traders",
                    _ => "Other",
                };
                let title = quest.title.as_deref().unwrap_or("(no title)");
                let label = format!("[{}] {} - {}", quest.id, type_label, title);

                let btn = button(text(label).size(11).font(Font::MONOSPACE))
                    .width(Fill)
                    .on_press(Message::QuestScrOpSelectQuest(idx));

                if is_selected {
                    btn.style(style::active_chip).into()
                } else {
                    btn.style(style::chip).into()
                }
            })
            .collect();

        let item_scroll = scrollable(column(item_list).spacing(4)).height(Length::Fill);

        let mut detail_content: Vec<Element<Message>> = vec![
            text("Quest Details").size(16).font(Font::MONOSPACE).into(),
            vertical_space().height(10).into(),
        ];

        if let Some(idx) = editor.selected_idx {
            if let Some((orig_idx, _quest)) = editor.filtered_quests.get(idx) {
                let orig = *orig_idx;

                detail_content.push(labeled_input("ID:", &editor.edit_id, move |v| {
                    Message::QuestScrOpFieldChanged(orig, "id".into(), v)
                }));

                let type_options: Vec<String> =
                    vec!["0 - Main".into(), "1 - Side".into(), "2 - Traders".into()];
                let type_value = match editor.edit_type_id.parse::<i32>().unwrap_or(0) {
                    0 => "0 - Main",
                    1 => "1 - Side",
                    2 => "2 - Traders",
                    _ => "0 - Main",
                }
                .to_string();
                detail_content.push(labeled_select(
                    "Type:",
                    type_value,
                    type_options,
                    move |v: String| {
                        let type_id = v.chars().next().unwrap_or('0').to_string();
                        Message::QuestScrOpFieldChanged(orig, "type_id".into(), type_id)
                    },
                ));

                detail_content.push(labeled_input("Title:", &editor.edit_title, move |v| {
                    Message::QuestScrOpFieldChanged(orig, "title".into(), v)
                }));

                detail_content.push(text("Description:").size(13).into());
                let te = text_editor(&editor.edit_description_content)
                    .on_action(move |action| Message::QuestScrOpDescriptionAction(orig, action))
                    .padding(8)
                    .height(120);
                detail_content.push(container(te).width(Fill).style(style::info_card).into());
            }
        } else {
            detail_content.push(
                text("No quest selected")
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
            text("Quests").size(14),
            horizontal_space(),
            button(text("Scan"))
                .on_press(Message::QuestScrOpLoadCatalog)
                .padding([5, 10])
                .style(style::run_button),
        ]
        .padding(10)
        .align_y(iced::Alignment::Center);

        let left_panel = column![
            container(item_list_header).style(style::grid_header_cell),
            item_scroll,
        ];

        let main_content = row![left_panel, detail_panel]
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
                    button(text("Save Quests"))
                        .on_press(Message::QuestScrOpSave)
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
