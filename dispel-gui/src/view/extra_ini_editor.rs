use crate::app::App;
use crate::message::Message;
use crate::style;
use crate::utils::{horizontal_rule, horizontal_space, labeled_input, vertical_space};
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Element, Fill, Font, Length};

impl App {
    pub fn view_extra_ini_editor_tab(&self) -> Element<'_, Message> {
        let editor = &self.state.extra_ini_editor;

        let item_list: Vec<Element<Message>> = editor
            .filtered_extras
            .iter()
            .enumerate()
            .map(|(idx, (_, extra))| {
                let is_selected = editor.selected_idx == Some(idx);
                let sprite = extra
                    .sprite_filename
                    .clone()
                    .unwrap_or_else(|| "(none)".into());
                let label = format!("[{}] {} - {}", extra.id, sprite, extra.id);

                let btn = button(text(label).size(11).font(Font::MONOSPACE))
                    .width(Fill)
                    .on_press(Message::ExtraIniOpSelectExtra(idx));

                if is_selected {
                    btn.style(style::active_chip).into()
                } else {
                    btn.style(style::chip).into()
                }
            })
            .collect();

        let item_scroll = scrollable(column(item_list).spacing(4)).height(Length::Fill);

        let mut detail_content: Vec<Element<Message>> = vec![
            text("Extra Details").size(16).font(Font::MONOSPACE).into(),
            vertical_space().height(10).into(),
        ];

        if let Some(idx) = editor.selected_idx {
            if let Some((orig_idx, _extra)) = editor.filtered_extras.get(idx) {
                let orig = *orig_idx;

                detail_content.push(labeled_input("ID:", &editor.edit_id, move |v| {
                    Message::ExtraIniOpFieldChanged(orig, "id".into(), v)
                }));
                detail_content.push(labeled_input(
                    "Sprite Filename:",
                    &editor.edit_sprite_filename,
                    move |v| Message::ExtraIniOpFieldChanged(orig, "sprite_filename".into(), v),
                ));
                detail_content.push(labeled_input("Unknown:", &editor.edit_unknown, move |v| {
                    Message::ExtraIniOpFieldChanged(orig, "unknown".into(), v)
                }));
                detail_content.push(labeled_input(
                    "Description:",
                    &editor.edit_description,
                    move |v| Message::ExtraIniOpFieldChanged(orig, "description".into(), v),
                ));
            }
        } else {
            detail_content.push(
                text("No extra selected")
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
            text("Extras").size(14),
            horizontal_space(),
            button(text("Scan"))
                .on_press(Message::ExtraIniOpLoadCatalog)
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
                    button(text("Save Extras"))
                        .on_press(Message::ExtraIniOpSave)
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
